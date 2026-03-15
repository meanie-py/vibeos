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
use ntkrnl_sys::CONTAINING_RECORD;

use ntkrnl_sys::ddk::{
    _DEVICE_OBJECT, _FILE_OBJECT, _EPROCESS, _ETHREAD, _VPB, _IRP, _IO_STACK_LOCATION,
    _OBJECT_ATTRIBUTES, _IO_STATUS_BLOCK, _SECURITY_QUALITY_OF_SERVICE,
    _IO_SECURITY_CONTEXT, _ACCESS_STATE, _PRIVILEGE_SET, _REPARSE_DATA_BUFFER,
    _HANDLE_TABLE_ENTRY, _OBJECT_HANDLE_COUNT_ENTRY, _FILE_BASIC_INFORMATION,
    _FILE_NETWORK_OPEN_INFORMATION, _FILE_NAME_INFORMATION, _OBJECT_NAME_INFORMATION,
    _NAMED_PIPE_CREATE_PARAMETERS, _MAILSLOT_CREATE_PARAMETERS, _OPEN_PACKET,
    _DUMMY_FILE_OBJECT, _FILE_OBJECT_EXTENSION, _SHARE_ACCESS, _ERESOURCE,
    _GENERIC_MAPPING, _SECURITY_DESCRIPTOR, _OBJECT_TYPE, _UNICODE_STRING, _KEVENT,
    _LUID,
    PDEVICE_OBJECT, PFILE_OBJECT, PEPROCESS, PETHREAD, PVPB, PIRP, PIO_STACK_LOCATION,
    POBJECT_ATTRIBUTES, PIO_STATUS_BLOCK, PSECURITY_QUALITY_OF_SERVICE,
    PIO_SECURITY_CONTEXT, PACCESS_STATE, PPRIVILEGE_SET, PREPARSE_DATA_BUFFER,
    PHANDLE_TABLE_ENTRY, POBJECT_HANDLE_COUNT_ENTRY, PFILE_BASIC_INFORMATION,
    PFILE_NETWORK_OPEN_INFORMATION, PFILE_NAME_INFORMATION, POBJECT_NAME_INFORMATION,
    PNAMED_PIPE_CREATE_PARAMETERS, PMAILSLOT_CREATE_PARAMETERS, POPEN_PACKET,
    PDUMMY_FILE_OBJECT, PFILE_OBJECT_EXTENSION, PSHARE_ACCESS, PERESOURCE,
    PGENERIC_MAPPING, PSECURITY_DESCRIPTOR, PUNICODE_STRING,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE, OB_OPEN_REASON,
    SECURITY_OPERATION_CODE, FILE_INFORMATION_CLASS, CREATE_FILE_TYPE,
    ExAcquireResourceExclusiveLite, ExReleaseResourceLite, ExAcquireResourceSharedLite,
    ExGetPreviousMode, IoAllocateIrp, IoFreeIrp, IoGetAttachedDevice,
    IoGetRelatedDeviceObject, IoGetNextIrpStackLocation,
    KeWaitForSingleObject, KeInitializeEvent, KeClearEvent, KeSetEvent,
    KeAcquireQueuedSpinLock, KeReleaseQueuedSpinLock,
    ObCreateObject, ObInsertObject, ObOpenObjectByName, ObCloseHandle,
    ObReferenceSecurityDescriptor, ObDereferenceSecurityDescriptor,
    ObLogSecurityDescriptor, ObQueryNameString, ObFreeObjectCreateInfoBuffer,
    SeAccessCheck, SeAppendPrivileges, SeFreePrivileges, SePrivilegeCheck,
    SeLockSubjectContext, SeUnlockSubjectContext, SeOpenObjectAuditAlarm,
    SeFastTraverseCheck, SeSetSecurityDescriptorInfo, SeQuerySecurityDescriptorInfo,
    SeSetWorldSecurityDescriptor, SeSetAccessStateGenericMapping,
    PsGetCurrentThread,
    IopCheckVpbMounted, IopDereferenceDeviceObject,
    IopDereferenceVpbAndFree, IopVerifyDeviceObjectOnStack,
    IopAllocateIrpMustSucceed,
    IoVolumeDeviceToDosName, IoQueryFileInformation,
    _EVENT_TYPE_SynchronizationEvent, _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive, _KWAIT_REASON_UserRequest,
    _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _SECURITY_OPERATION_CODE_SetSecurityDescriptor,
    _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor,
    _SECURITY_OPERATION_CODE_AssignSecurityDescriptor,
    ObReferenceObjectByHandle,
};

extern "system" {
    static mut IopSecurityResource: _ERESOURCE;
    static mut SeBackupPrivilege: _LUID;
    static mut SeRestorePrivilege: _LUID;
    static mut IoFileObjectType: *mut _OBJECT_TYPE;
    static mut IopFileMapping: _GENERIC_MAPPING;
    fn ProbeForRead(Address: *const core::ffi::c_void, Length: usize, Alignment: usize);
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
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
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
    #[link_name = "IoGetDevObjExtension_shim"]
    fn IoGetDevObjExtension(DeviceObject: PDEVICE_OBJECT) -> *mut ntkrnl_sys::ddk::_DEVOBJ_EXTENSION;
    #[link_name = "IopQueueIrpToThread_shim"]
    fn IopQueueIrpToThread(Irp: PIRP);
    #[link_name = "IopUnQueueIrpFromThread_shim"]
    fn IopUnQueueIrpFromThread(Irp: PIRP);
    static mut SeDefaultObjectMethod: *const core::ffi::c_void;
}

extern "C" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
    fn ProbeForReadLargeInteger(Address: *const LARGE_INTEGER) -> LARGE_INTEGER;
    fn ProbeForWriteHandle(Address: *mut HANDLE);
    fn ProbeForWriteIoStatusBlock(IoStatusBlock: PIO_STATUS_BLOCK);
}

const TAG_IO: ULONG = 0x20204F49;      // '  OI' — generic IO pool tag
const TAG_IO_NAME: ULONG = 0x4E204F49;  // 'N OI' — IO name pool tag
const TAG_EA: ULONG = 0x20204145;        // '  AE' — EA pool tag

const IOP_MAX_REPARSE_TRAVERSAL: ULONG = 0x20;
const IopOtherTransfer: ULONG = 0;

// Bindgen ULONG_PTR differs from base::ULONG_PTR (usize vs u64/u32)
#[cfg(target_pointer_width = "64")]
type DdkUlongPtr = u64;
#[cfg(target_pointer_width = "32")]
type DdkUlongPtr = u32;

const IOP_CREATE_FILE_OBJECT_EXTENSION: ULONG = 0x1;
const IOP_USE_TOP_LEVEL_DEVICE_HINT: ULONG = 0x2;

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

const IO_TYPE_OPEN_PACKET: USHORT = 0x0E;
const IO_TYPE_FILE: USHORT = 0x05;
const IO_TYPE_DEVICE: USHORT = 0x03;

const SE_BACKUP_PRIVILEGES_CHECKED: ULONG = 0x10;
const TOKEN_HAS_BACKUP_PRIVILEGE: ULONG = 0x01;
const TOKEN_HAS_RESTORE_PRIVILEGE: ULONG = 0x02;
const TOKEN_HAS_TRAVERSE_PRIVILEGE: ULONG = 0x40;
const TOKEN_IS_RESTRICTED: ULONG = 0x10000;

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

const IRP_CREATE_OPERATION: ULONG = 0x01;
const IRP_SYNCHRONOUS_API: ULONG = 0x04;
const IRP_DEFER_IO_COMPLETION: ULONG = 0x800;
const IRP_CLOSE_OPERATION: ULONG = 0x200;
const IRP_BUFFERED_IO: ULONG = 0x10;
const IRP_DEALLOCATE_BUFFER: ULONG = 0x20;

const FO_SYNCHRONOUS_IO: ULONG = 0x00000002;
const FO_ALERTABLE_IO: ULONG = 0x00000004;
const FO_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;
const FO_WRITE_THROUGH: ULONG = 0x00000010;
const FO_SEQUENTIAL_ONLY: ULONG = 0x00000020;
const FO_RANDOM_ACCESS: ULONG = 0x00000200;
const FO_FILE_OBJECT_HAS_EXTENSION: ULONG = 0x00800000;
const FO_HANDLE_CREATED: ULONG = 0x00040000;
const FO_DIRECT_DEVICE_OPEN: ULONG = 0x00000001;
const FO_VOLUME_OPEN: ULONG = 0x00080000;
const FO_STREAM_FILE: ULONG = 0x00400000;
const FO_OPENED_CASE_SENSITIVE: ULONG = 0x01000000;
const FO_REMOTE_ORIGIN: ULONG = 0x01000000;
const FO_FILE_OPEN_CANCELLED: ULONG = 0x00100000;

const DO_DEVICE_INITIALIZING: ULONG = 0x00000080;
const DO_EXCLUSIVE: ULONG = 0x00000008;
const DO_NEVER_LAST_DEVICE: ULONG = 0x40000000;
const DO_BUS_ENUMERATED_DEVICE: ULONG = 0x00001000;

const DOE_UNLOAD_PENDING: ULONG = 0x01;
const DOE_DELETE_PENDING: ULONG = 0x02;
const DOE_REMOVE_PENDING: ULONG = 0x04;
const DOE_REMOVE_PROCESSED: ULONG = 0x08;

const SL_CASE_SENSITIVE: UCHAR = 0x80;

const IRP_MJ_CREATE: UCHAR = 0x00;
const IRP_MJ_CLOSE: UCHAR = 0x02;
const IRP_MJ_CLEANUP: UCHAR = 0x12;
const IRP_MJ_CREATE_NAMED_PIPE: UCHAR = 0x1C;
const IRP_MJ_CREATE_MAILSLOT: UCHAR = 0x1D;
const IRP_MJ_QUERY_SECURITY: UCHAR = 0x14;
const IRP_MJ_SET_SECURITY: UCHAR = 0x15;
const IRP_MJ_LOCK_CONTROL: UCHAR = 0x11;

const IRP_MN_UNLOCK_ALL: UCHAR = 0x04;

const IO_NO_PARAMETER_CHECKING: ULONG = 0x100;
const IO_CHECK_CREATE_PARAMETERS: ULONG = 0x200;
const IO_ATTACH_DEVICE: ULONG = 0x400;
const IO_ATTACH_DEVICE_API: ULONG = 0x80000000;
const IO_FORCE_ACCESS_CHECK: ULONG = 0x001;
const IO_IGNORE_SHARE_ACCESS_CHECK: ULONG = 0x800;

const FILE_DEVICE_DISK_FILE_SYSTEM: ULONG = 0x00000008;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: ULONG = 0x00000003;
const FILE_DEVICE_TAPE_FILE_SYSTEM: ULONG = 0x00000005;
const FILE_DEVICE_NETWORK_FILE_SYSTEM: ULONG = 0x00000014;
const FILE_DEVICE_DFS_FILE_SYSTEM: ULONG = 0x00000035;
const FILE_DEVICE_DISK: ULONG = 0x00000007;
const FILE_DEVICE_CD_ROM: ULONG = 0x00000002;
const FILE_DEVICE_TAPE: ULONG = 0x0000001F;
const FILE_DEVICE_VIRTUAL_DISK: ULONG = 0x00000024;

const FILE_DEVICE_SECURE_OPEN: ULONG = 0x00000100;

const CreateFileTypeNone: CREATE_FILE_TYPE = 0;
const CreateFileTypeNamedPipe: CREATE_FILE_TYPE = 1;
const CreateFileTypeMailslot: CREATE_FILE_TYPE = 2;

const FileBasicInformation: FILE_INFORMATION_CLASS = 4;
const FileNetworkOpenInformation: FILE_INFORMATION_CLASS = 34;
const FileNameInformation: FILE_INFORMATION_CLASS = 9;

const QuerySecurityDescriptor: SECURITY_OPERATION_CODE = 1;
const SetSecurityDescriptor: SECURITY_OPERATION_CODE = 2;
const DeleteSecurityDescriptor: SECURITY_OPERATION_CODE = 3;
const AssignSecurityDescriptor: SECURITY_OPERATION_CODE = 4;

const PRIVILEGE_SET_ALL_NECESSARY: ULONG = 1;

const MUTANT_INCREMENT: LONG = 1;

const LockQueueIoDatabaseLock: ULONG = 11;

const SynchronizationEvent: ULONG = 0;
const NotificationEvent: ULONG = 1;

const Executive: ULONG = 0;
const UserRequest: ULONG = 6;

unsafe fn IopCheckBackupRestorePrivilege(
    AccessState: PACCESS_STATE,
    CreateOptions: *mut ULONG,
    PreviousMode: KPROCESSOR_MODE,
    Disposition: ULONG,
) {
    let mut DesiredAccess: ACCESS_MASK;
    let ReadAccess: ACCESS_MASK;
    let WriteAccess: ACCESS_MASK;
    let mut Privileges: _PRIVILEGE_SET = core::mem::zeroed();
    let mut AccessGranted: BOOLEAN;
    let mut HaveBackupPriv = 0u8;
    let mut CheckRestore = 0u8;

    if (*AccessState).Flags & SE_BACKUP_PRIVILEGES_CHECKED != 0 {
        return;
    }

    if *CreateOptions & FILE_OPEN_FOR_BACKUP_INTENT != 0 {
        (*AccessState).Flags |= SE_BACKUP_PRIVILEGES_CHECKED;

        ReadAccess = READ_CONTROL
            | ACCESS_SYSTEM_SECURITY
            | FILE_GENERIC_READ
            | FILE_TRAVERSE;
        WriteAccess = WRITE_DAC
            | WRITE_OWNER
            | ACCESS_SYSTEM_SECURITY
            | FILE_GENERIC_WRITE
            | FILE_ADD_FILE
            | FILE_ADD_SUBDIRECTORY
            | DELETE;
        DesiredAccess = (*AccessState).RemainingDesiredAccess;

        if DesiredAccess & MAXIMUM_ALLOWED != 0 {
            DesiredAccess |= ReadAccess | WriteAccess;
        }

        if Disposition & FILE_OPEN != 0 {
            if ReadAccess & DesiredAccess != 0 {
                Privileges.PrivilegeCount = 1;
                Privileges.Control = PRIVILEGE_SET_ALL_NECESSARY;
                Privileges.Privilege[0].Luid = SeBackupPrivilege;
                Privileges.Privilege[0].Attributes = 0;
                AccessGranted = SePrivilegeCheck(
                    &mut Privileges,
                    core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
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
        } else {
            CheckRestore = 1;
        }

        if (WriteAccess & DesiredAccess != 0) || (CheckRestore != 0) {
            Privileges.PrivilegeCount = 1;
            Privileges.Control = PRIVILEGE_SET_ALL_NECESSARY;
            Privileges.Privilege[0].Luid = SeRestorePrivilege;
            Privileges.Privilege[0].Attributes = 0;
            AccessGranted = SePrivilegeCheck(
                &mut Privileges,
                core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
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
    if (ext_flags & (DOE_UNLOAD_PENDING | DOE_DELETE_PENDING | DOE_REMOVE_PENDING | DOE_REMOVE_PROCESSED) != 0)
        || ((*DeviceObject).Flags & DO_DEVICE_INITIALIZING != 0)
    {
        return STATUS_NO_SUCH_DEVICE;
    } else if ((*DeviceObject).Flags & DO_EXCLUSIVE != 0)
        && ((*DeviceObject).ReferenceCount != 0)
        && ((*OpenPacket).RelatedFileObject.is_null())
        && ((*OpenPacket).Options & IO_ATTACH_DEVICE == 0)
    {
        return STATUS_ACCESS_DENIED;
    } else {
        InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceObject).ReferenceCount) as *mut LONG);
        return STATUS_SUCCESS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopDoNameTransmogrify(
    Irp: PIRP,
    FileObject: PFILE_OBJECT,
    DataBuffer: PREPARSE_DATA_BUFFER,
) {
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut Length: USHORT = 0;
    let mut RequiredLength: USHORT = 0;
    let mut NewBuffer: PWSTR = core::ptr::null_mut();

    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_REPARSE
        || (*Irp).IoStatus.Information != IO_REPARSE_TAG_MOUNT_POINT as DdkUlongPtr
        || (*Irp).Tail.Overlay.AuxiliaryBuffer.is_null()
        || DataBuffer.is_null()
        || (*DataBuffer).ReparseTag != IO_REPARSE_TAG_MOUNT_POINT
        || (*DataBuffer).ReparseDataLength >= MAXIMUM_REPARSE_DATA_BUFFER_SIZE as u16
        || (*DataBuffer).Reserved >= MAXIMUM_REPARSE_DATA_BUFFER_SIZE as u16
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_DATA_INVALID;
        return;
    }

    if (*DataBuffer).ReparseDataLength < REPARSE_DATA_BUFFER_HEADER_SIZE as u16
        || ((*DataBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameLength
            + (*DataBuffer).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameLength
            + core::mem::size_of::<_REPARSE_DATA_BUFFER>() as u16
            - core::mem::size_of::<[u16; 1]>() as u16
            > MAXIMUM_REPARSE_DATA_BUFFER_SIZE as u16)
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_DATA_INVALID;
    }

    if (*Irp).IoStatus.__bindgen_anon_1.Status >= 0 {
        Buffer = ((*DataBuffer).__bindgen_anon_1.MountPointReparseBuffer.PathBuffer.as_ptr() as usize
            + (*DataBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameOffset as usize)
            as PWSTR;
        Length = (*DataBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength;

        if (0xFFFFu32 - (*DataBuffer).Reserved as u32) <= (Length as u32 + core::mem::size_of::<u16>() as u32) {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_DATA_INVALID;
        } else {
            RequiredLength = (*DataBuffer).Reserved + Length + core::mem::size_of::<u16>() as u16;

            if (*FileObject).FileName.MaximumLength >= RequiredLength {
                NewBuffer = (*FileObject).FileName.Buffer;
            } else {
                NewBuffer = ExAllocatePoolWithTag(PagedPool, RequiredLength as usize, TAG_IO_NAME) as PWSTR;
                if NewBuffer.is_null() {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                }
            }
        }
    }

    if (*Irp).IoStatus.__bindgen_anon_1.Status >= 0 {
        if (*DataBuffer).Reserved != 0 {
            core::ptr::copy(
                ((*FileObject).FileName.Buffer as usize + (*FileObject).FileName.Length as usize
                    - (*DataBuffer).Reserved as usize) as *const u8,
                (NewBuffer as usize + Length as usize) as *mut u8,
                (*DataBuffer).Reserved as usize,
            );
        }

        if Length != 0 {
            core::ptr::copy_nonoverlapping(
                Buffer as *const u8,
                NewBuffer as *mut u8,
                Length as usize,
            );
        }

        (*FileObject).FileName.Length = RequiredLength - core::mem::size_of::<u16>() as u16;
        if NewBuffer != (*FileObject).FileName.Buffer {
            if !(*FileObject).FileName.Buffer.is_null() {
                ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
            }

            (*FileObject).FileName.Buffer = NewBuffer;
            (*FileObject).FileName.MaximumLength = RequiredLength;
            *NewBuffer.add((RequiredLength / 2 - 1) as usize) = 0;
        }
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
    _ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: ULONG,
    CompleteName: *mut _UNICODE_STRING,
    RemainingName: *mut _UNICODE_STRING,
    Context: PVOID,
    _SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Object: *mut PVOID,
) -> NTSTATUS {
    let OpenPacket = Context as POPEN_PACKET;
    let OriginalDeviceObject = ParseObject as PDEVICE_OBJECT;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut OwnerDevice: PDEVICE_OBJECT;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Vpb: PVPB = core::ptr::null_mut();
    let mut Irp: PIRP;
    let mut StackLoc: PIO_STACK_LOCATION;
    let mut SecurityContext: _IO_SECURITY_CONTEXT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DirectOpen = 0u8;
    let mut OpenCancelled: BOOLEAN;
    let mut UseDummyFile: BOOLEAN = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
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
    let mut Privileges: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut FileString = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 8,
        Buffer: b"F\0i\0l\0e\0\0\0".as_ptr() as PWSTR,
    };

    let mut Attempt: ULONG = 0;
    while Attempt < IOP_MAX_REPARSE_TRAVERSAL {
        *Object = core::ptr::null_mut();

        if IopValidateOpenPacket(OpenPacket) == 0 {
            return STATUS_OBJECT_TYPE_MISMATCH;
        }

        if (*OpenPacket).TraversedMountPoint != 0 {
            if (*OpenPacket).Information != IO_REPARSE_TAG_MOUNT_POINT as DdkUlongPtr {
                (*OpenPacket).FinalStatus = STATUS_IO_REPARSE_DATA_INVALID;
                return STATUS_IO_REPARSE_DATA_INVALID;
            }

            if (*OriginalDeviceObject).DeviceType != FILE_DEVICE_DISK
                && (*OriginalDeviceObject).DeviceType != FILE_DEVICE_CD_ROM
                && (*OriginalDeviceObject).DeviceType != FILE_DEVICE_VIRTUAL_DISK
                && (*OriginalDeviceObject).DeviceType != FILE_DEVICE_TAPE
            {
                (*OpenPacket).FinalStatus = STATUS_IO_REPARSE_DATA_INVALID;
                return STATUS_IO_REPARSE_DATA_INVALID;
            }
        }

        let mut device_to_use = OriginalDeviceObject;
        if !(*OpenPacket).RelatedFileObject.is_null() {
            device_to_use = (*(*OpenPacket).RelatedFileObject).DeviceObject;
        }

        Status = IopCheckDeviceAndDriver(OpenPacket, device_to_use);
        if Status < 0 {
            (*OpenPacket).FinalStatus = Status;
            return Status;
        }

        RtlMapGenericMask(
            core::ptr::addr_of_mut!((*AccessState).RemainingDesiredAccess),
            core::ptr::addr_of_mut!((*IoFileObjectType).TypeInfo.GenericMapping),
        );
        RtlMapGenericMask(
            core::ptr::addr_of_mut!((*AccessState).OriginalDesiredAccess),
            core::ptr::addr_of_mut!((*IoFileObjectType).TypeInfo.GenericMapping),
        );
        SeSetAccessStateGenericMapping(
            AccessState,
            core::ptr::addr_of_mut!((*IoFileObjectType).TypeInfo.GenericMapping),
        );
        DesiredAccess = (*AccessState).RemainingDesiredAccess;

        CheckMode = if (AccessMode != 0i8) || ((*OpenPacket).Options & IO_FORCE_ACCESS_CHECK != 0) {
            1i8
        } else {
            0i8
        };

        IopCheckBackupRestorePrivilege(
            AccessState,
            core::ptr::addr_of_mut!((*OpenPacket).CreateOptions),
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
            && (*RemainingName).Length == 0
        {
            VolumeOpen = 1;
        }

        if ((AccessMode != 0i8) || ((*OpenPacket).Options & IO_FORCE_ACCESS_CHECK != 0))
            && ((*OpenPacket).RelatedFileObject.is_null() || VolumeOpen != 0)
            && ((*OpenPacket).Override == 0)
        {
            KeEnterCriticalRegion();
            ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(IopSecurityResource), 1u8);

            if (*RemainingName).Length == 0 {
                SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
                LockHeld = 1;

                AccessGranted = SeAccessCheck(
                    (*device_to_use).SecurityDescriptor,
                    core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
                    LockHeld,
                    DesiredAccess,
                    0,
                    core::ptr::addr_of_mut!(Privileges),
                    core::ptr::addr_of_mut!((*IoFileObjectType).TypeInfo.GenericMapping),
                    1i8,
                    core::ptr::addr_of_mut!(GrantedAccess),
                    core::ptr::addr_of_mut!(Status),
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

                SeOpenObjectAuditAlarm(
                    core::ptr::addr_of_mut!(FileString),
                    device_to_use as PVOID,
                    CompleteName,
                    (*device_to_use).SecurityDescriptor,
                    AccessState,
                    0u8,
                    AccessGranted,
                    1i8,
                    core::ptr::addr_of_mut!((*AccessState).GenerateOnClose),
                );
            } else {
                if ((*AccessState).Flags & TOKEN_HAS_TRAVERSE_PRIVILEGE == 0)
                    || (((*device_to_use).DeviceType == FILE_DEVICE_DISK)
                        || ((*device_to_use).DeviceType == FILE_DEVICE_CD_ROM))
                {
                    if (*AccessState).Flags & TOKEN_IS_RESTRICTED == 0 {
                        AccessGranted = SeFastTraverseCheck(
                            (*device_to_use).SecurityDescriptor,
                            AccessState,
                            FILE_TRAVERSE,
                            1i8,
                        );
                    } else {
                        AccessGranted = 0;
                    }

                    if AccessGranted == 0 {
                        SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
                        LockHeld = 1;

                        AccessGranted = SeAccessCheck(
                            (*device_to_use).SecurityDescriptor,
                            core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
                            LockHeld,
                            FILE_TRAVERSE,
                            0,
                            core::ptr::addr_of_mut!(Privileges),
                            core::ptr::addr_of_mut!((*IoFileObjectType).TypeInfo.GenericMapping),
                            1i8,
                            core::ptr::addr_of_mut!(GrantedAccess),
                            core::ptr::addr_of_mut!(Status),
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

            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
            KeLeaveCriticalRegion();

            if LockHeld != 0 {
                SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
            }

            if AccessGranted == 0 {
                IopDereferenceDeviceObject(device_to_use, 0u8);
                return STATUS_ACCESS_DENIED;
            }
        }

        UseDummyFile = if ((*OpenPacket).QueryOnly != 0) || ((*OpenPacket).DeleteOnly != 0) {
            1u8
        } else {
            0u8
        };

        if (*RemainingName).Length == 0
            && (*OpenPacket).RelatedFileObject.is_null()
            && ((DesiredAccess
                & !(SYNCHRONIZE
                    | FILE_READ_ATTRIBUTES
                    | READ_CONTROL
                    | ACCESS_SYSTEM_SECURITY
                    | WRITE_OWNER
                    | WRITE_DAC))
                == 0)
            && UseDummyFile == 0
        {
            DirectOpen = 1;
        }

        if !(*OpenPacket).RelatedFileObject.is_null()
            && ((*(*OpenPacket).RelatedFileObject).Flags & FO_DIRECT_DEVICE_OPEN == 0)
        {
            DeviceObject = ParseObject as PDEVICE_OBJECT;

            if !(*(*OpenPacket).RelatedFileObject).Vpb.is_null() {
                Vpb = (*(*OpenPacket).RelatedFileObject).Vpb;
                InterlockedIncrement(core::ptr::addr_of_mut!((*Vpb).ReferenceCount) as *mut LONG);

                if (*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT != 0 {
                    DeviceObject = (*Vpb).DeviceObject;
                }
            }
        } else {
            if !(*device_to_use).Vpb.is_null() && DirectOpen == 0 {
                Vpb = IopCheckVpbMounted(
                    OpenPacket,
                    device_to_use,
                    RemainingName,
                    core::ptr::addr_of_mut!(Status),
                );
                if Vpb.is_null() {
                    return Status;
                }

                DeviceObject = (*Vpb).DeviceObject;
            } else {
                DeviceObject = device_to_use;
            }

            if ((*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT == 0)
                && !(*DeviceObject).AttachedDevice.is_null()
            {
                DeviceObject = IoGetAttachedDevice(DeviceObject);
            }
        }

        if (*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT != 0 {
            Status = IopCheckTopDeviceHint(core::ptr::addr_of_mut!(DeviceObject), OpenPacket, DirectOpen);
            if Status < 0 {
                IopDereferenceDeviceObject(device_to_use, 0u8);
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
            && ((!(*OpenPacket).RelatedFileObject.is_null()) || ((*RemainingName).Length != 0))
            && (VolumeOpen == 0)
        {
            Privileges = core::ptr::null_mut();
            GrantedAccess = 0;

            KeEnterCriticalRegion();
            ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(IopSecurityResource), 1u8);

            SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

            AccessGranted = SeAccessCheck(
                (*device_to_use).SecurityDescriptor,
                core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
                1u8,
                DesiredAccess,
                0,
                core::ptr::addr_of_mut!(Privileges),
                core::ptr::addr_of_mut!((*IoFileObjectType).TypeInfo.GenericMapping),
                1i8,
                core::ptr::addr_of_mut!(GrantedAccess),
                core::ptr::addr_of_mut!(Status),
            );
            if !Privileges.is_null() {
                SeAppendPrivileges(AccessState, Privileges);
                SeFreePrivileges(Privileges);
            }

            if GrantedAccess != 0 {
                (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
                (*AccessState).RemainingDesiredAccess &= !(GrantedAccess | MAXIMUM_ALLOWED);
            }

            SeOpenObjectAuditAlarm(
                core::ptr::addr_of_mut!(FileString),
                DeviceObject as PVOID,
                CompleteName,
                (*device_to_use).SecurityDescriptor,
                AccessState,
                0u8,
                AccessGranted,
                1i8,
                core::ptr::addr_of_mut!((*AccessState).GenerateOnClose),
            );

            SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
            KeLeaveCriticalRegion();

            if AccessGranted == 0 {
                IopDereferenceDeviceObject(device_to_use, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                return STATUS_ACCESS_DENIED;
            }
        }

        Irp = IoAllocateIrp((*DeviceObject).StackSize as i8, 1u8);
        if Irp.is_null() {
            IopDereferenceDeviceObject(device_to_use, 0u8);
            if !Vpb.is_null() {
                IopDereferenceVpbAndFree(Vpb);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Irp).RequestorMode = AccessMode;
        (*Irp).Flags = IRP_CREATE_OPERATION | IRP_SYNCHRONOUS_API | IRP_DEFER_IO_COMPLETION;
        (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
        (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock);
        (*Irp).MdlAddress = core::ptr::null_mut();
        (*Irp).PendingReturned = 0u8;
        (*Irp).UserEvent = core::ptr::null_mut();
        (*Irp).Cancel = 0u8;
        (*Irp).CancelRoutine = None;
        (*Irp).Tail.Overlay.AuxiliaryBuffer = core::ptr::null_mut();

        SecurityContext.SecurityQos = _SecurityQos;
        SecurityContext.AccessState = AccessState;
        SecurityContext.DesiredAccess = (*AccessState).RemainingDesiredAccess;
        SecurityContext.FullCreateOptions = (*OpenPacket).CreateOptions;

        StackLoc = IoGetNextIrpStackLocation(Irp);
        (*StackLoc).Control = 0;

        match (*OpenPacket).CreateFileType {
            CreateFileTypeNone => {
                (*StackLoc).MajorFunction = IRP_MJ_CREATE;
                (*StackLoc).Parameters.Create.EaLength = (*OpenPacket).EaLength;

                (*StackLoc).Flags = (*OpenPacket).Options as u8;
                if Attributes & OBJ_CASE_INSENSITIVE == 0 {
                    (*StackLoc).Flags |= SL_CASE_SENSITIVE;
                }
            }
            CreateFileTypeNamedPipe => {
                (*StackLoc).MajorFunction = IRP_MJ_CREATE_NAMED_PIPE;
                (*StackLoc).Parameters.CreatePipe.Parameters = (*OpenPacket).ExtraCreateParameters as PNAMED_PIPE_CREATE_PARAMETERS;
            }
            CreateFileTypeMailslot => {
                (*StackLoc).MajorFunction = IRP_MJ_CREATE_MAILSLOT;
                (*StackLoc).Parameters.CreateMailslot.Parameters = (*OpenPacket).ExtraCreateParameters as PMAILSLOT_CREATE_PARAMETERS;
            }
            _ => {}
        }

        (*Irp).Overlay.AllocationSize = (*OpenPacket).AllocationSize;
        (*Irp).AssociatedIrp.SystemBuffer = (*OpenPacket).EaBuffer;
        (*StackLoc).Parameters.Create.Options = ((*OpenPacket).Disposition << 24) | ((*OpenPacket).CreateOptions & 0xFFFFFF);
        (*StackLoc).Parameters.Create.FileAttributes = (*OpenPacket).FileAttributes;
        (*StackLoc).Parameters.Create.ShareAccess = (*OpenPacket).ShareAccess;
        (*StackLoc).Parameters.Create.SecurityContext = core::ptr::addr_of_mut!(SecurityContext);

        if UseDummyFile == 0 {
            let mut ObjectSize = core::mem::size_of::<_FILE_OBJECT>();
            if (*OpenPacket).InternalFlags & IOP_CREATE_FILE_OBJECT_EXTENSION != 0 {
                ObjectSize += core::mem::size_of::<_FILE_OBJECT_EXTENSION>();
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(ObjectAttributes), 0, 1);
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;

            Status = ObCreateObject(
                0i8,
                IoFileObjectType,
                core::ptr::addr_of_mut!(ObjectAttributes),
                AccessMode,
                core::ptr::null_mut(),
                ObjectSize as u32,
                0,
                0,
                core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
            );
            if Status < 0 {
                IoFreeIrp(Irp);
                IopDereferenceDeviceObject(device_to_use, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                (*OpenPacket).FinalStatus = Status;
                return Status;
            }

            core::ptr::write_bytes(FileObject as *mut u8, 0, ObjectSize);

            if (*OpenPacket).CreateOptions & (FILE_SYNCHRONOUS_IO_ALERT | FILE_SYNCHRONOUS_IO_NONALERT) != 0 {
                (*FileObject).Flags |= FO_SYNCHRONOUS_IO;
                if (*OpenPacket).CreateOptions & FILE_SYNCHRONOUS_IO_ALERT != 0 {
                    (*FileObject).Flags |= FO_ALERTABLE_IO;
                }
            }

            if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
                KeInitializeEvent(
                    core::ptr::addr_of_mut!((*FileObject).Lock),
                    _EVENT_TYPE_SynchronizationEvent,
                    0u8,
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
                let FileObjectExtension: PFILE_OBJECT_EXTENSION;

                (*FileObject).Flags |= FO_FILE_OBJECT_HAS_EXTENSION;

                FileObjectExtension = (FileObject as usize + core::mem::size_of::<_FILE_OBJECT>()) as PFILE_OBJECT_EXTENSION;
                (*FileObject).FileObjectExtension = FileObjectExtension as PVOID;

                if (*OpenPacket).InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT != 0 {
                    (*FileObjectExtension).TopDeviceObjectHint = DeviceObject;
                }
            }
        } else {
            LocalFileObject = (*OpenPacket).LocalFileObject;
            core::ptr::write_bytes(LocalFileObject as *mut u8, 0, core::mem::size_of::<_DUMMY_FILE_OBJECT>());

            FileObject = core::ptr::addr_of_mut!((*LocalFileObject).ObjectHeader.Body) as PFILE_OBJECT;
            (*LocalFileObject).ObjectHeader.Type = IoFileObjectType;
            (*LocalFileObject).ObjectHeader.PointerCount = 1;
        }

        (*FileObject).Type = IO_TYPE_FILE as i16;
        (*FileObject).Size = core::mem::size_of::<_FILE_OBJECT>() as i16;
        (*FileObject).RelatedFileObject = (*OpenPacket).RelatedFileObject;
        (*FileObject).DeviceObject = device_to_use;

        if DirectOpen != 0 {
            (*FileObject).Flags |= FO_DIRECT_DEVICE_OPEN;
        }

        if Attributes & OBJ_CASE_INSENSITIVE == 0 {
            (*FileObject).Flags |= FO_OPENED_CASE_SENSITIVE;
        }

        (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
        (*StackLoc).FileObject = FileObject;

        if (*RemainingName).Length != 0 {
            (*FileObject).FileName.MaximumLength = (*RemainingName).Length + core::mem::size_of::<u16>() as u16;
            (*FileObject).FileName.Buffer = ExAllocatePoolWithTag(
                PagedPool,
                (*FileObject).FileName.MaximumLength as usize,
                TAG_IO_NAME,
            ) as PWSTR;
            if (*FileObject).FileName.Buffer.is_null() {
                IoFreeIrp(Irp);
                IopDereferenceDeviceObject(device_to_use, 0u8);
                if !Vpb.is_null() {
                    IopDereferenceVpbAndFree(Vpb);
                }
                (*FileObject).DeviceObject = core::ptr::null_mut();
                if UseDummyFile == 0 {
                    ObDereferenceObject(FileObject as PVOID);
                }
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        RtlCopyUnicodeString(core::ptr::addr_of_mut!((*FileObject).FileName), RemainingName);

        KeInitializeEvent(core::ptr::addr_of_mut!((*FileObject).Event), _EVENT_TYPE_NotificationEvent, 0u8);
        (*OpenPacket).FileObject = FileObject;

        IopQueueIrpToThread(Irp);
        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        } else {
            if Status == STATUS_REPARSE {
                if (*Irp).IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT as DdkUlongPtr {
                    let ReparseData: PREPARSE_DATA_BUFFER = (*Irp).Tail.Overlay.AuxiliaryBuffer as PREPARSE_DATA_BUFFER;
                    IopDoNameTransmogrify(Irp, FileObject, ReparseData);
                }
            }

            KeRaiseIrql(APC_LEVEL as u8, core::ptr::addr_of_mut!(OldIrql));
            IoStatusBlock = (*Irp).IoStatus;
            Status = IoStatusBlock.__bindgen_anon_1.Status;
            (*FileObject).Event.Header.SignalState = 1;
            IopUnQueueIrpFromThread(Irp);

            if ((*Irp).Flags & IRP_BUFFERED_IO != 0) && ((*Irp).Flags & IRP_DEALLOCATE_BUFFER != 0) {
                ExFreePool((*Irp).AssociatedIrp.SystemBuffer);
            }

            IoFreeIrp(Irp);
            KeLowerIrql(OldIrql);
        }

        (*OpenPacket).Information = IoStatusBlock.Information;

        if Status < 0 {
            if (*FileObject).FileName.Length != 0 {
                ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
                (*FileObject).FileName.Buffer = core::ptr::null_mut();
                (*FileObject).FileName.Length = 0;
            }

            (*FileObject).DeviceObject = core::ptr::null_mut();

            OpenCancelled = if (*FileObject).Flags & FO_FILE_OPEN_CANCELLED != 0 { 1u8 } else { 0u8 };

            (*OpenPacket).FileObject = core::ptr::null_mut();

            if UseDummyFile == 0 {
                ObDereferenceObject(FileObject as PVOID);
            }

            IopDereferenceDeviceObject(device_to_use, 0u8);

            if (OpenCancelled == 0) && !Vpb.is_null() {
                IopDereferenceVpbAndFree(Vpb);
            }

            (*OpenPacket).FinalStatus = Status;
            return Status;
        } else if Status == STATUS_REPARSE {
            if (*OpenPacket).Information == IO_REPARSE as DdkUlongPtr
                || (*OpenPacket).Information == IO_REPARSE_TAG_MOUNT_POINT as DdkUlongPtr
            {
                if (*CompleteName).MaximumLength < (*FileObject).FileName.Length {
                    let NewCompleteName = ExAllocatePoolWithTag(
                        PagedPool,
                        (*FileObject).FileName.Length as usize,
                        TAG_IO_NAME,
                    ) as PWSTR;
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

                RtlCopyUnicodeString(CompleteName, core::ptr::addr_of_mut!((*FileObject).FileName));

                if (*OpenPacket).Information == IO_REPARSE_TAG_MOUNT_POINT as DdkUlongPtr {
                    (*OpenPacket).RelatedFileObject = core::ptr::null_mut();
                }
            }

            if (*FileObject).FileName.Length != 0 {
                ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
                (*FileObject).FileName.Buffer = core::ptr::null_mut();
                (*FileObject).FileName.Length = 0;
            }

            (*FileObject).DeviceObject = core::ptr::null_mut();
            (*OpenPacket).FileObject = core::ptr::null_mut();

            if UseDummyFile == 0 {
                ObDereferenceObject(FileObject as PVOID);
            }

            IopDereferenceDeviceObject(device_to_use, 0u8);

            if !Vpb.is_null() {
                IopDereferenceVpbAndFree(Vpb);
            }

            if (*OpenPacket).Information != IO_REMOUNT as DdkUlongPtr {
                (*OpenPacket).RelatedFileObject = core::ptr::null_mut();

                if (*OpenPacket).Information == IO_REPARSE_TAG_MOUNT_POINT as DdkUlongPtr {
                    (*OpenPacket).TraversedMountPoint = 1;
                }

                if (*OpenPacket).Override != 0 {
                    KeBugCheckEx(
                        0x7E,
                        device_to_use as usize,
                        DeviceObject as usize,
                        CompleteName as usize,
                        (*OpenPacket).Information as usize,
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

    OwnerDevice = IoGetRelatedDeviceObject(FileObject);

    if OwnerDevice != DeviceObject {
        if !Vpb.is_null() {
            IopDereferenceVpbAndFree(Vpb);
        }

        Vpb = (*FileObject).Vpb;
        if !Vpb.is_null() {
            InterlockedIncrement(core::ptr::addr_of_mut!((*Vpb).ReferenceCount) as *mut LONG);
        }
    }

    if UseDummyFile == 0 {
        if (((*FileObject).RelatedFileObject.is_null())
            || ((*(*FileObject).RelatedFileObject).Flags & FO_VOLUME_OPEN != 0))
            && ((*FileObject).FileName.Length == 0)
        {
            if ((*OwnerDevice).DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM)
                || ((*OwnerDevice).DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM)
                || ((*OwnerDevice).DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM)
                || ((*OwnerDevice).DeviceType == FILE_DEVICE_DFS_FILE_SYSTEM)
            {
                (*FileObject).Flags |= FO_VOLUME_OPEN;
            }
        }

        ObReferenceObject(FileObject as PVOID);
        *Object = FileObject as PVOID;
        (*OpenPacket).FinalStatus = IoStatusBlock.__bindgen_anon_1.Status;
        (*OpenPacket).ParseCheck = 1;
        return (*OpenPacket).FinalStatus;
    } else {
        if (*OpenPacket).QueryOnly != 0 {
            if (*OpenPacket).FullAttributes == 0 {
                FileBasicInfo = ExAllocatePoolWithTag(
                    NonPagedPool,
                    core::mem::size_of::<_FILE_BASIC_INFORMATION>(),
                    TAG_IO,
                ) as PFILE_BASIC_INFORMATION;
                if !FileBasicInfo.is_null() {
                    Status = IoQueryFileInformation(
                        FileObject,
                        FileBasicInformation,
                        core::mem::size_of::<_FILE_BASIC_INFORMATION>() as u32,
                        FileBasicInfo as PVOID,
                        core::ptr::addr_of_mut!(ReturnLength),
                    );
                    if Status >= 0 {
                        core::ptr::copy_nonoverlapping(
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
                    core::mem::size_of::<_FILE_NETWORK_OPEN_INFORMATION>() as u32,
                    (*OpenPacket).NetworkInformation as PVOID,
                    core::ptr::addr_of_mut!(ReturnLength),
                );
            }
        }

        IopDeleteFile(FileObject as PVOID);

        (*OpenPacket).FileObject = core::ptr::null_mut();

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
    let FileObject = ObjectBody as PFILE_OBJECT;
    let mut Irp: PIRP;
    let mut StackPtr: PIO_STACK_LOCATION;
    let Status: NTSTATUS;
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut DereferenceDone = 0u8;
    let Vpb: PVPB;
    let mut OldIrql: KIRQL = 0;

    if !(*FileObject).DeviceObject.is_null() {
        if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
            DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
        } else {
            DeviceObject = IoGetRelatedDeviceObject(FileObject);
        }

        if ((*FileObject).Flags & FO_HANDLE_CREATED) == 0 {
            IopCloseFile(core::ptr::null_mut(), ObjectBody, 0, 1, 1);
        }

        KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
        KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0u8);

        Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);

        (*Irp).UserEvent = core::ptr::addr_of_mut!(Event);
        (*Irp).UserIosb = core::ptr::addr_of_mut!((*Irp).IoStatus);
        (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
        (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
        (*Irp).Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;

        StackPtr = IoGetNextIrpStackLocation(Irp);
        (*StackPtr).MajorFunction = IRP_MJ_CLOSE;
        (*StackPtr).FileObject = FileObject;

        IopQueueIrpToThread(Irp);

        Vpb = (*FileObject).Vpb;
        if !Vpb.is_null() && ((*FileObject).Flags & FO_DIRECT_DEVICE_OPEN == 0) {
            InterlockedDecrement(core::ptr::addr_of_mut!((*Vpb).ReferenceCount) as *mut LONG);
        }

        if (*(*FileObject).DeviceObject).Flags & DO_NEVER_LAST_DEVICE != 0 {
            InterlockedDecrement(core::ptr::addr_of_mut!((*(*FileObject).DeviceObject).ReferenceCount) as *mut LONG);
            DereferenceDone = 1;
        }

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }

        KeRaiseIrql(APC_LEVEL as u8, core::ptr::addr_of_mut!(OldIrql));
        IopUnQueueIrpFromThread(Irp);
        KeLowerIrql(OldIrql);

        IoFreeIrp(Irp);

        if !(*FileObject).FileName.Buffer.is_null() {
            ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
            (*FileObject).FileName.Buffer = core::ptr::null_mut();
        }

        if !(*FileObject).CompletionContext.is_null() {
            ObDereferenceObject((*(*FileObject).CompletionContext).Port as PVOID);
            ExFreePool((*FileObject).CompletionContext as PVOID);
        }

        if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
            FsRtlPTeardownPerFileObjectContexts(FileObject);
        }

        if DereferenceDone == 0 {
            IopDereferenceDeviceObject((*FileObject).DeviceObject, 0u8);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopGetDeviceAttachmentBase(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let mut PDO = DeviceObject;

    loop {
        let ext = (*PDO).DeviceObjectExtension as *mut ntkrnl_sys::ddk::_EXTENDED_DEVOBJ_EXTENSION;
        let attached_to = (*ext).AttachedTo;
        if attached_to.is_null() {
            break;
        }
        PDO = attached_to;
    }

    PDO
}

unsafe fn IopGetDevicePDO(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
    let mut OldIrql: KIRQL = 0;
    let mut PDO: PDEVICE_OBJECT;

    OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock as _);
    PDO = IopGetDeviceAttachmentBase(DeviceObject);
    if ((*PDO).Flags & DO_BUS_ENUMERATED_DEVICE) != DO_BUS_ENUMERATED_DEVICE {
        PDO = core::ptr::null_mut();
    } else {
        ObReferenceObject(PDO as PVOID);
    }
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock as _, OldIrql);

    PDO
}

unsafe fn IopSetDeviceSecurityDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    SecurityInformation: *mut ULONG,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut OldSecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut CachedSecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut NewSecurityDescriptor: PSECURITY_DESCRIPTOR;

    loop {
        KeEnterCriticalRegion();
        ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(IopSecurityResource), 1u8);

        OldSecurityDescriptor = (*DeviceObject).SecurityDescriptor;
        if !OldSecurityDescriptor.is_null() {
            ObReferenceSecurityDescriptor(OldSecurityDescriptor, 1);
        }

        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
        KeLeaveCriticalRegion();

        NewSecurityDescriptor = OldSecurityDescriptor;
        Status = SeSetSecurityDescriptorInfo(
            core::ptr::null_mut(),
            SecurityInformation,
            SecurityDescriptor,
            core::ptr::addr_of_mut!(NewSecurityDescriptor),
            PoolType,
            GenericMapping,
        );

        if Status < 0 {
            if !OldSecurityDescriptor.is_null() {
                ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
            }
            break;
        }

        Status = ObLogSecurityDescriptor(
            NewSecurityDescriptor,
            core::ptr::addr_of_mut!(CachedSecurityDescriptor),
            1,
        );
        ExFreePool(NewSecurityDescriptor as PVOID);
        if Status < 0 {
            ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
            break;
        }

        KeEnterCriticalRegion();
        ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopSecurityResource), 1u8);
        if (*DeviceObject).SecurityDescriptor == OldSecurityDescriptor {
            (*DeviceObject).SecurityDescriptor = CachedSecurityDescriptor;
            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
            KeLeaveCriticalRegion();

            ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 2);
            break;
        }
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
        KeLeaveCriticalRegion();

        ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
        ObDereferenceSecurityDescriptor(CachedSecurityDescriptor, 1);
    }

    Status
}

unsafe fn IopSetDeviceSecurityDescriptors(
    UpperDeviceObject: PDEVICE_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    SecurityInformation: *mut ULONG,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let mut CurrentDO = PhysicalDeviceObject;
    let mut NextDevice: PDEVICE_OBJECT;
    let mut Status = STATUS_SUCCESS;
    let mut TmpStatus: NTSTATUS;

    ObReferenceObject(CurrentDO as PVOID);

    loop {
        TmpStatus = IopSetDeviceSecurityDescriptor(
            CurrentDO,
            SecurityInformation,
            SecurityDescriptor,
            PoolType,
            GenericMapping,
        );
        if CurrentDO == UpperDeviceObject {
            Status = TmpStatus;
        }

        NextDevice = (*CurrentDO).AttachedDevice;
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
    OperationCode: SECURITY_OPERATION_CODE,
    SecurityInformation: *mut ULONG,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut ULONG,
    OldSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut StackPtr: PIO_STACK_LOCATION;
    let FileObject: PFILE_OBJECT;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Irp: PIRP;
    let mut LocalEvent = 0u8;
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut Status = STATUS_SUCCESS;

    if (*(ObjectBody as PFILE_OBJECT)).Type == IO_TYPE_DEVICE as i16 {
        DeviceObject = ObjectBody as PDEVICE_OBJECT;
        FileObject = core::ptr::null_mut();
    } else {
        FileObject = ObjectBody as PFILE_OBJECT;

        if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
            DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
        } else {
            DeviceObject = (*FileObject).DeviceObject;
        }
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
                core::ptr::addr_of_mut!((*DeviceObject).SecurityDescriptor),
            );
        } else if OperationCode == _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor {
            return STATUS_SUCCESS;
        } else if OperationCode == _SECURITY_OPERATION_CODE_AssignSecurityDescriptor {
            Status = STATUS_SUCCESS;

            if FileObject.is_null() || ((*FileObject).Flags & FO_STREAM_FILE == 0) {
                let mut CachedSecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

                Status = ObLogSecurityDescriptor(
                    SecurityDescriptor,
                    core::ptr::addr_of_mut!(CachedSecurityDescriptor),
                    1,
                );
                if Status >= 0 {
                    KeEnterCriticalRegion();
                    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopSecurityResource), 1u8);

                    (*DeviceObject).SecurityDescriptor = CachedSecurityDescriptor;

                    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
                    KeLeaveCriticalRegion();
                }
            }

            return Status;
        } else if OperationCode == _SECURITY_OPERATION_CODE_SetSecurityDescriptor {
            let PDO = IopGetDevicePDO(DeviceObject);

            if !PDO.is_null() {
                Status = IopSetDeviceSecurityDescriptors(
                    DeviceObject,
                    PDO,
                    SecurityInformation,
                    SecurityDescriptor,
                    PoolType,
                    GenericMapping,
                );
                ObDereferenceObject(PDO as PVOID);
            } else {
                Status = IopSetDeviceSecurityDescriptor(
                    DeviceObject,
                    SecurityInformation,
                    SecurityDescriptor,
                    PoolType,
                    GenericMapping,
                );
            }

            return STATUS_SUCCESS;
        }

        return STATUS_SUCCESS;
    } else if OperationCode == _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor {
        return STATUS_SUCCESS;
    }

    ObReferenceObject(FileObject as PVOID);

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, ExGetPreviousMode());
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    } else {
        KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0u8);
        LocalEvent = 1;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));

    DeviceObject = IoGetRelatedDeviceObject(FileObject);

    Irp = IoAllocateIrp((*DeviceObject).StackSize as i8, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), core::ptr::null_mut());
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).RequestorMode = ExGetPreviousMode();
    (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock);
    (*Irp).UserEvent = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(Event)
    } else {
        core::ptr::null_mut()
    };
    (*Irp).Flags = if LocalEvent != 0 { IRP_SYNCHRONOUS_API } else { 0 };
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).FileObject = FileObject;

    if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        (*StackPtr).MajorFunction = IRP_MJ_QUERY_SECURITY;
        (*StackPtr).Parameters.QuerySecurity.SecurityInformation = *SecurityInformation;
        (*StackPtr).Parameters.QuerySecurity.Length = *BufferLength;
        (*Irp).UserBuffer = SecurityDescriptor as PVOID;
    } else {
        (*StackPtr).MajorFunction = IRP_MJ_SET_SECURITY;
        (*StackPtr).Parameters.SetSecurity.SecurityInformation = *SecurityInformation;
        (*StackPtr).Parameters.SetSecurity.SecurityDescriptor = SecurityDescriptor;
    }

    IopQueueIrpToThread(Irp);

    IopUpdateOperationCount(IopOtherTransfer);

    Status = IoCallDriver(DeviceObject, Irp);

    if LocalEvent != 0 {
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
    } else {
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = (*FileObject).FinalStatus;
        }

        IopUnlockFileObject(FileObject);
    }

    if Status == STATUS_INVALID_DEVICE_REQUEST {
        if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
            Status = SeSetWorldSecurityDescriptor(
                *SecurityInformation,
                SecurityDescriptor as ntkrnl_sys::ddk::PISECURITY_DESCRIPTOR,
                BufferLength,
            );
        } else {
            Status = STATUS_SUCCESS;
        }
    } else if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        if Status == STATUS_BUFFER_OVERFLOW {
            Status = STATUS_BUFFER_TOO_SMALL;
        }

        let inner = || -> Result<NTSTATUS, NTSTATUS> {
            write_user(BufferLength, IoStatusBlock.Information as ULONG, ExGetPreviousMode())?;
            Ok(Status)
        };
        match inner() {
            Ok(s) => Status = s,
            Err(e) => Status = e,
        }
    }

    Status
}

unsafe fn IopQueryNameInternal(
    ObjectBody: PVOID,
    HasName: BOOLEAN,
    QueryDosName: BOOLEAN,
    ObjectNameInfo: POBJECT_NAME_INFORMATION,
    Length: ULONG,
    ReturnLength: *mut ULONG,
    PreviousMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let mut LocalInfo: POBJECT_NAME_INFORMATION;
    let mut LocalFileInfo: PFILE_NAME_INFORMATION = core::ptr::null_mut();
    let FileObject = ObjectBody as PFILE_OBJECT;
    let mut LocalReturnLength: ULONG = 0;
    let mut FileLength: ULONG = 0;
    let mut LengthMismatch = 0u8;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut p: PWSTR;
    let DeviceObject: PDEVICE_OBJECT;
    let mut NoObCall: BOOLEAN;

    if Length < core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG {
        *ReturnLength = core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG;
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    LocalInfo = ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_IO) as POBJECT_NAME_INFORMATION;
    if LocalInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    NoObCall = 0u8;
    if QueryDosName != 0 {
        DeviceObject = (*FileObject).DeviceObject;

        if (*DeviceObject).DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM {
            LocalReturnLength = core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG + 2 * core::mem::size_of::<u16>() as ULONG;
            if Length < LocalReturnLength {
                Status = STATUS_BUFFER_OVERFLOW;
            } else {
                (*LocalInfo).Name.Length = core::mem::size_of::<u16>() as u16;
                (*LocalInfo).Name.MaximumLength = core::mem::size_of::<u16>() as u16;
                (*LocalInfo).Name.Buffer = (LocalInfo as usize + core::mem::size_of::<_OBJECT_NAME_INFORMATION>()) as PWSTR;
                *(*LocalInfo).Name.Buffer = b'\\' as u16;
                Status = STATUS_SUCCESS;
            }
        } else {
            Status = IoVolumeDeviceToDosName(DeviceObject as PVOID, core::ptr::addr_of_mut!((*LocalInfo).Name));
            LocalReturnLength = (*LocalInfo).Name.Length as ULONG + core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG + core::mem::size_of::<u16>() as ULONG;
        }
    }

    if (QueryDosName == 0) || (Status < 0) {
        Status = ObQueryNameString(
            (*FileObject).DeviceObject as PVOID,
            LocalInfo,
            Length,
            core::ptr::addr_of_mut!(LocalReturnLength),
        );
    } else {
        NoObCall = 1;
    }

    if (Status < 0) && (Status != STATUS_INFO_LENGTH_MISMATCH) {
        ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
        return Status;
    }

    p = (ObjectNameInfo as usize + core::mem::size_of::<_OBJECT_NAME_INFORMATION>()) as PWSTR;

    let mut inner = || -> Result<NTSTATUS, NTSTATUS> {
        if (QueryDosName != 0) && (NoObCall != 0) {
            let copy_len = if Length >= LocalReturnLength {
                core::mem::size_of::<_OBJECT_NAME_INFORMATION>()
            } else {
                Length as usize
            };
            core::ptr::copy_nonoverlapping(LocalInfo as *const u8, ObjectNameInfo as *mut u8, copy_len);

            let name_copy_len = if Length >= LocalReturnLength {
                (*LocalInfo).Name.Length as usize
            } else {
                (Length as usize).saturating_sub(core::mem::size_of::<_OBJECT_NAME_INFORMATION>())
            };
            core::ptr::copy_nonoverlapping((*LocalInfo).Name.Buffer as *const u8, p as *mut u8, name_copy_len);

            if (*(*FileObject).DeviceObject).DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM {
                ExFreePool((*LocalInfo).Name.Buffer as PVOID);
            }
        } else {
            let copy_len = if LocalReturnLength > Length {
                Length as usize
            } else {
                LocalReturnLength as usize
            };
            core::ptr::copy_nonoverlapping(LocalInfo as *const u8, ObjectNameInfo as *mut u8, copy_len);
        }

        (*ObjectNameInfo).Name.Buffer = p;

        p = p.add(((*LocalInfo).Name.Length / 2) as usize);

        if LocalReturnLength > Length {
            LengthMismatch = 1;
            *ReturnLength = LocalReturnLength;
        }

        LocalFileInfo = LocalInfo as PFILE_NAME_INFORMATION;
        FileLength = Length
            .saturating_sub(LocalReturnLength)
            .saturating_add(core::mem::size_of::<u32>() as ULONG);

        if (PreviousMode == 0i8) && ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0) {
            Status = IopGetFileInformation(
                FileObject,
                if LengthMismatch != 0 { Length } else { FileLength },
                FileNameInformation,
                LocalFileInfo as PVOID,
                core::ptr::addr_of_mut!(LocalReturnLength),
            );
        } else {
            Status = IoQueryFileInformation(
                FileObject,
                FileNameInformation,
                if LengthMismatch != 0 { Length } else { FileLength },
                LocalFileInfo as PVOID,
                core::ptr::addr_of_mut!(LocalReturnLength),
            );
        }

        if Status < 0 {
            if Status != STATUS_INVALID_PARAMETER
                && Status != STATUS_INVALID_DEVICE_REQUEST
                && Status != STATUS_NOT_IMPLEMENTED
                && Status != STATUS_INVALID_INFO_CLASS
            {
                return Err(Status);
            }

            LocalReturnLength = core::mem::size_of::<u32>() as ULONG;
            (*LocalFileInfo).FileNameLength = 0;
            (*LocalFileInfo).FileName[0] = b'\\' as u16;
            Status = STATUS_SUCCESS;
        } else {
            if LocalReturnLength < core::mem::size_of::<u32>() as ULONG {
                LocalReturnLength = core::mem::size_of::<u32>() as ULONG;
            }
        }

        if LengthMismatch != 0 {
            *ReturnLength = (*ReturnLength).saturating_add((*LocalFileInfo).FileNameLength);
            return Err(STATUS_BUFFER_OVERFLOW);
        }

        FileLength = LocalReturnLength.saturating_sub(core::mem::size_of::<u32>() as ULONG);
        LocalReturnLength = (p as usize - ObjectNameInfo as usize) as ULONG + (*LocalFileInfo).FileNameLength;

        if (*LocalFileInfo).FileName[0] != b'\\' as u16 {
            return Err(STATUS_OBJECT_PATH_INVALID);
        }

        core::ptr::copy_nonoverlapping((*LocalFileInfo).FileName.as_ptr() as *const u8, p as *mut u8, FileLength as usize);
        p = p.add((FileLength / core::mem::size_of::<u16>() as ULONG) as usize);
        *p = 0;
        LocalReturnLength += core::mem::size_of::<u16>() as ULONG;

        *ReturnLength = LocalReturnLength;

        FileLength = (p as usize - ObjectNameInfo as usize) as ULONG;
        (*ObjectNameInfo).Name.Length = (FileLength - core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as ULONG) as u16;
        (*ObjectNameInfo).Name.MaximumLength = (*ObjectNameInfo).Name.Length + core::mem::size_of::<u16>() as u16;

        Ok(STATUS_SUCCESS)
    };

    match inner() {
        Ok(s) => Status = s,
        Err(s) => Status = s,
    }

    ExFreePoolWithTag(LocalInfo as PVOID, TAG_IO);
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
    IopQueryNameInternal(ObjectBody, HasName, 0u8, ObjectNameInfo, Length, ReturnLength, PreviousMode)
}

#[no_mangle]
pub unsafe extern "system" fn IopCloseFile(
    Process: PEPROCESS,
    ObjectBody: PVOID,
    GrantedAccess: ACCESS_MASK,
    HandleCount: ULONG,
    SystemHandleCount: ULONG,
) {
    let FileObject = ObjectBody as PFILE_OBJECT;
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut OldIrql: KIRQL = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if HandleCount != 1 {
        return;
    }

    if ((*FileObject).LockOperation != 0) && (SystemHandleCount != 1) {
        if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
            DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
        } else {
            DeviceObject = IoGetRelatedDeviceObject(FileObject);
        }

        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            IopLockFileObject(FileObject, 0i8);
        }

        if (*DeviceObject).DriverObject.is_null()
            || (*(*DeviceObject).DriverObject).FastIoDispatch.is_null()
            || (*(*(*DeviceObject).DriverObject).FastIoDispatch).FastIoUnlockAll.is_none()
            || ((*(*(*DeviceObject).DriverObject).FastIoDispatch).FastIoUnlockAll.unwrap())(
                FileObject,
                PsGetCurrentProcess() as *mut _EPROCESS,
                core::ptr::addr_of_mut!(IoStatusBlock),
                DeviceObject,
            ) == 0
        {
            KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
            KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0u8);

            Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);

            (*Irp).UserEvent = core::ptr::addr_of_mut!(Event);
            (*Irp).UserIosb = core::ptr::addr_of_mut!((*Irp).IoStatus);
            (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
            (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
            (*Irp).RequestorMode = 0i8;
            (*Irp).Flags = IRP_SYNCHRONOUS_API;
            (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
            ObReferenceObject(FileObject as PVOID);

            StackPtr = IoGetNextIrpStackLocation(Irp);
            (*StackPtr).MajorFunction = IRP_MJ_LOCK_CONTROL;
            (*StackPtr).MinorFunction = IRP_MN_UNLOCK_ALL;
            (*StackPtr).FileObject = FileObject;

            IopQueueIrpToThread(Irp);

            Status = IoCallDriver(DeviceObject, Irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    _KWAIT_REASON_UserRequest,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
            }
        }

        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            IopUnlockFileObject(FileObject);
        }
    }

    if SystemHandleCount != 1 {
        return;
    }

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
    } else {
        DeviceObject = IoGetRelatedDeviceObject(FileObject);
    }

    (*FileObject).Flags |= FO_HANDLE_CREATED;

    if !Process.is_null() && ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0) {
        IopLockFileObject(FileObject, 0i8);
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0u8);

    Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);

    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event);
    (*Irp).UserIosb = core::ptr::addr_of_mut!((*Irp).IoStatus);
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_CLEANUP;
    (*StackPtr).FileObject = FileObject;

    IopQueueIrpToThread(Irp);

    IopUpdateOperationCount(IopOtherTransfer);

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_UserRequest,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
    }

    KeRaiseIrql(APC_LEVEL as u8, core::ptr::addr_of_mut!(OldIrql));
    IopUnQueueIrpFromThread(Irp);
    KeLowerIrql(OldIrql);

    IoFreeIrp(Irp);

    if !Process.is_null() && ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0) {
        IopUnlockFileObject(FileObject);
    }
}

unsafe fn IopQueryAttributesFile(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    FileInformationClass: FILE_INFORMATION_CLASS,
    FileInformationSize: ULONG,
    FileInformation: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let AccessMode = ExGetPreviousMode();
    let mut LocalFileObject: _DUMMY_FILE_OBJECT = core::mem::zeroed();
    let mut NetworkOpenInfo: _FILE_NETWORK_OPEN_INFORMATION = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut OpenPacket: _OPEN_PACKET = core::mem::zeroed();
    let IsBasic: BOOLEAN;

    if AccessMode != 0i8 {
        let inner = || -> Result<(), NTSTATUS> {
            probe_user_buffer(
                FileInformation,
                FileInformationSize as usize,
                core::mem::align_of::<ULONG>(),
                AccessMode,
            )?;
            Ok(())
        };
        if let Err(e) = inner() {
            return e;
        }
    }

    IsBasic = if FileInformationSize == core::mem::size_of::<_FILE_BASIC_INFORMATION>() as ULONG {
        1u8
    } else {
        0u8
    };

    core::ptr::write_bytes(core::ptr::addr_of_mut!(OpenPacket), 0, 1);
    OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
    OpenPacket.Size = core::mem::size_of::<_OPEN_PACKET>() as i16;
    OpenPacket.CreateOptions = FILE_OPEN_REPARSE_POINT;
    OpenPacket.ShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u16;
    OpenPacket.Disposition = FILE_OPEN;
    OpenPacket.BasicInformation = if IsBasic != 0 {
        FileInformation as PFILE_BASIC_INFORMATION
    } else {
        core::ptr::null_mut()
    };
    OpenPacket.NetworkInformation = if IsBasic != 0 {
        core::ptr::addr_of_mut!(NetworkOpenInfo)
    } else if AccessMode != 0i8 {
        core::ptr::addr_of_mut!(NetworkOpenInfo)
    } else {
        FileInformation as PFILE_NETWORK_OPEN_INFORMATION
    };
    OpenPacket.QueryOnly = 1;
    OpenPacket.FullAttributes = if IsBasic != 0 { 0u8 } else { 1u8 };
    OpenPacket.LocalFileObject = core::ptr::addr_of_mut!(LocalFileObject);

    IopUpdateOperationCount(IopOtherTransfer);

    Status = ObOpenObjectByName(
        ObjectAttributes,
        core::ptr::null_mut(),
        AccessMode,
        core::ptr::null_mut(),
        FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(OpenPacket) as PVOID,
        core::ptr::addr_of_mut!(Handle),
    );
    if OpenPacket.ParseCheck == 0 {
        return Status;
    } else {
        Status = OpenPacket.FinalStatus;
    }

    if (Status >= 0) && (AccessMode != 0i8) && (IsBasic == 0) {
        let inner = || -> Result<(), NTSTATUS> {
            safe_copy_to_user(
                FileInformation,
                core::ptr::addr_of!(NetworkOpenInfo) as *const core::ffi::c_void,
                FileInformationSize as usize,
                1,
                AccessMode,
            )?;
            Ok(())
        };
        if let Err(e) = inner() {
            Status = e;
        }
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
    let mut Status: NTSTATUS;

    InterlockedIncrement(core::ptr::addr_of_mut!((*FileObject).Waiters) as *mut LONG);

    Status = STATUS_SUCCESS;
    loop {
        if InterlockedExchange(core::ptr::addr_of_mut!((*FileObject).Busy) as *mut LONG, 1) == 0 {
            break;
        }
        Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*FileObject).Lock) as PVOID,
            _KWAIT_REASON_Executive,
            WaitMode,
            Alertable,
            core::ptr::null_mut(),
        );
        if Status != STATUS_SUCCESS {
            break;
        }
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*FileObject).Waiters) as *mut LONG);
    if Status == STATUS_SUCCESS {
        ObReferenceObject(FileObject as PVOID);
        *LockFailed = 0u8;
    } else {
        if ((*FileObject).Busy == 0) && ((*FileObject).Waiters != 0) {
            KeSetEvent(core::ptr::addr_of_mut!((*FileObject).Lock), 0i32, 0u8);
        }
        *LockFailed = 1u8;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoGetFileObjectFilterContext(FileObject: PFILE_OBJECT) -> PVOID {
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        let FileObjectExtension: PFILE_OBJECT_EXTENSION = (*FileObject).FileObjectExtension as PFILE_OBJECT_EXTENSION;
        return (*FileObjectExtension).FilterContext;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn IoChangeFileObjectFilterContext(
    FileObject: PFILE_OBJECT,
    FilterContext: PVOID,
    Define: BOOLEAN,
) -> NTSTATUS {
    let Success: usize;
    let FileObjectExtension: PFILE_OBJECT_EXTENSION;

    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    FileObjectExtension = (*FileObject).FileObjectExtension as PFILE_OBJECT_EXTENSION;
    if Define != 0 {
        Success = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*FileObjectExtension).FilterContext),
            FilterContext,
            core::ptr::null_mut(),
        ) as usize;
    } else {
        Success = (InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*FileObjectExtension).FilterContext),
            core::ptr::null_mut(),
            FilterContext,
        ) as usize)
            .wrapping_sub(FilterContext as usize);
    }

    if Success != 0 {
        return STATUS_ALREADY_COMMITTED;
    }

    STATUS_SUCCESS
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
    CreateFileType: CREATE_FILE_TYPE,
    ExtraCreateParameters: PVOID,
    mut Options: ULONG,
    Flags: ULONG,
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut AccessMode: KPROCESSOR_MODE;
    let mut LocalHandle: HANDLE = core::ptr::null_mut();
    let mut SafeAllocationSize: ntkrnl_sys::ddk::_LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut Status = STATUS_SUCCESS;
    let NamedPipeCreateParameters: PNAMED_PIPE_CREATE_PARAMETERS;
    let OpenPacket: POPEN_PACKET;
    let mut EaErrorOffset: ULONG = 0;

    if Options & IO_NO_PARAMETER_CHECKING != 0 {
        AccessMode = 0i8;
    } else {
        AccessMode = ExGetPreviousMode();
    }

    if (AccessMode != 0i8) || (Options & IO_CHECK_CREATE_PARAMETERS != 0) {
        if FileAttributes & FILE_ATTRIBUTE_VALID_FLAGS != FileAttributes {
            return STATUS_INVALID_PARAMETER;
        }

        if ShareAccess & FILE_SHARE_VALID_FLAGS != ShareAccess {
            return STATUS_INVALID_PARAMETER;
        }

        if Disposition > FILE_MAXIMUM_DISPOSITION {
            return STATUS_INVALID_PARAMETER;
        }

        if CreateOptions & FILE_VALID_OPTION_FLAGS != CreateOptions {
            return STATUS_INVALID_PARAMETER;
        }

        if ((CreateOptions & (FILE_SYNCHRONOUS_IO_ALERT | FILE_SYNCHRONOUS_IO_NONALERT)) != 0)
            && ((DesiredAccess & SYNCHRONIZE) == 0)
        {
            return STATUS_INVALID_PARAMETER;
        }

        if ((CreateOptions & FILE_DELETE_ON_CLOSE) != 0) && ((DesiredAccess & DELETE) == 0) {
            return STATUS_INVALID_PARAMETER;
        }

        if (CreateOptions & (FILE_SYNCHRONOUS_IO_NONALERT | FILE_SYNCHRONOUS_IO_ALERT))
            == (FILE_SYNCHRONOUS_IO_NONALERT | FILE_SYNCHRONOUS_IO_ALERT)
        {
            return STATUS_INVALID_PARAMETER;
        }

        if ((CreateOptions & FILE_DIRECTORY_FILE) != 0)
            && ((CreateOptions & FILE_NON_DIRECTORY_FILE) == 0)
            && ((CreateOptions
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
                    | FILE_OPEN_REPARSE_POINT))
                != 0)
        {
            return STATUS_INVALID_PARAMETER;
        }

        if ((CreateOptions & FILE_DIRECTORY_FILE) != 0)
            && ((CreateOptions & FILE_NON_DIRECTORY_FILE) == 0)
            && (Disposition != FILE_CREATE)
            && (Disposition != FILE_OPEN)
            && (Disposition != FILE_OPEN_IF)
        {
            return STATUS_INVALID_PARAMETER;
        }

        if ((CreateOptions & FILE_COMPLETE_IF_OPLOCKED) != 0) && ((CreateOptions & FILE_RESERVE_OPFILTER) != 0) {
            return STATUS_INVALID_PARAMETER;
        }

        if ((CreateOptions & FILE_NO_INTERMEDIATE_BUFFERING) != 0) && ((DesiredAccess & FILE_APPEND_DATA) != 0) {
            return STATUS_INVALID_PARAMETER;
        }

        if CreateFileType == CreateFileTypeNamedPipe {
            if ExtraCreateParameters.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            NamedPipeCreateParameters = ExtraCreateParameters as PNAMED_PIPE_CREATE_PARAMETERS;
            if ((*NamedPipeCreateParameters).NamedPipeType > FILE_PIPE_MESSAGE_TYPE)
                || ((*NamedPipeCreateParameters).ReadMode > FILE_PIPE_MESSAGE_MODE)
                || ((*NamedPipeCreateParameters).CompletionMode > FILE_PIPE_COMPLETE_OPERATION)
                || ((ShareAccess & FILE_SHARE_DELETE) != 0)
                || ((Disposition < FILE_OPEN) || (Disposition > FILE_OPEN_IF))
                || ((CreateOptions & !FILE_VALID_PIPE_OPTION_FLAGS) != 0)
            {
                return STATUS_INVALID_PARAMETER;
            }
        } else if CreateFileType == CreateFileTypeMailslot {
            if ExtraCreateParameters.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            if ((ShareAccess & FILE_SHARE_DELETE) != 0)
                || ((ShareAccess & !FILE_SHARE_WRITE) == 0)
                || (Disposition != FILE_CREATE)
                || ((CreateOptions & !FILE_VALID_MAILSLOT_OPTION_FLAGS) != 0)
            {
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    OpenPacket = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_OPEN_PACKET>(), b"pOoI".as_ptr() as ULONG) as POPEN_PACKET;
    if OpenPacket.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(OpenPacket, 0, 1);

    if AccessMode != 0i8 {
        let mut inner = || -> Result<(), NTSTATUS> {
            probe_user_buffer(
                FileHandle as *const core::ffi::c_void,
                core::mem::size_of::<HANDLE>(),
                core::mem::align_of::<HANDLE>(),
                AccessMode,
            )?;
            probe_user_buffer(
                IoStatusBlock as *const core::ffi::c_void,
                core::mem::size_of::<_IO_STATUS_BLOCK>(),
                core::mem::align_of::<_IO_STATUS_BLOCK>(),
                AccessMode,
            )?;

            if !AllocationSize.is_null() {
                SafeAllocationSize = core::mem::transmute(probe_read_user::<LARGE_INTEGER>(AllocationSize, AccessMode)?);
            } else {
                SafeAllocationSize = core::mem::zeroed();
            }

            if SafeAllocationSize.QuadPart < 0 {
                return Err(STATUS_INVALID_PARAMETER);
            }

            if (!EaBuffer.is_null()) && (EaLength != 0) {
                probe_user_buffer(EaBuffer, EaLength as usize, core::mem::align_of::<ULONG>(), AccessMode)?;

                (*OpenPacket).EaBuffer = ExAllocatePoolWithTag(NonPagedPool, EaLength as usize, TAG_EA);
                (*OpenPacket).EaLength = EaLength;
                core::ptr::copy_nonoverlapping(EaBuffer as *const u8, (*OpenPacket).EaBuffer as *mut u8, EaLength as usize);

                Status = IoCheckEaBufferValidity(
                    (*OpenPacket).EaBuffer,
                    EaLength,
                    core::ptr::addr_of_mut!(EaErrorOffset),
                );
                if Status < 0 {
                    write_user(
                        core::ptr::addr_of_mut!((*IoStatusBlock).__bindgen_anon_1.Status),
                        Status,
                        AccessMode,
                    )?;
                    write_user(
                        core::ptr::addr_of_mut!((*IoStatusBlock).Information),
                        EaErrorOffset as DdkUlongPtr,
                        AccessMode,
                    )?;
                    return Err(Status);
                }
            }
            Ok(())
        };

        if let Err(e) = inner() {
            if !(*OpenPacket).EaBuffer.is_null() {
                ExFreePool((*OpenPacket).EaBuffer);
            }
            ExFreePool(OpenPacket as PVOID);
            return e;
        }
    } else {
        if CreateOptions & IO_ATTACH_DEVICE_API != 0 {
            Options |= IO_ATTACH_DEVICE;
            CreateOptions &= !IO_ATTACH_DEVICE_API;
        }

        if !AllocationSize.is_null() {
            SafeAllocationSize = core::mem::transmute(*AllocationSize);
        } else {
            SafeAllocationSize = core::mem::zeroed();
        }

        if (!EaBuffer.is_null()) && (EaLength != 0) {
            (*OpenPacket).EaBuffer = ExAllocatePoolWithTag(NonPagedPool, EaLength as usize, TAG_EA);
            if (*OpenPacket).EaBuffer.is_null() {
                ExFreePool(OpenPacket as PVOID);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*OpenPacket).EaLength = EaLength;
            core::ptr::copy_nonoverlapping(EaBuffer as *const u8, (*OpenPacket).EaBuffer as *mut u8, EaLength as usize);

            Status = IoCheckEaBufferValidity(
                (*OpenPacket).EaBuffer,
                EaLength,
                core::ptr::addr_of_mut!(EaErrorOffset),
            );
            if Status < 0 {
                ExFreePool((*OpenPacket).EaBuffer);
                (*IoStatusBlock).__bindgen_anon_1.Status = Status;
                (*IoStatusBlock).Information = EaErrorOffset as DdkUlongPtr;
                ExFreePool(OpenPacket as PVOID);
                return Status;
            }
        }
    }

    (*OpenPacket).Type = IO_TYPE_OPEN_PACKET as i16;
    (*OpenPacket).Size = core::mem::size_of::<_OPEN_PACKET>() as i16;
    (*OpenPacket).AllocationSize = SafeAllocationSize;
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

    Status = ObOpenObjectByName(
        ObjectAttributes,
        core::ptr::null_mut(),
        AccessMode,
        core::ptr::null_mut(),
        DesiredAccess,
        OpenPacket as PVOID,
        core::ptr::addr_of_mut!(LocalHandle),
    );

    if !(*OpenPacket).EaBuffer.is_null() {
        ExFreePool((*OpenPacket).EaBuffer);
    }

    if (Status < 0) || ((*OpenPacket).ParseCheck == 0) {
        if Status >= 0 {
            ZwClose(LocalHandle);
            Status = STATUS_OBJECT_TYPE_MISMATCH;
        }

        if (*OpenPacket).FinalStatus < 0 {
            Status = (*OpenPacket).FinalStatus;

            if (Status as u32 & 0x80000000) == 0x80000000 && (Status as u32 & 0x40000000) == 0 {
                let inner = || -> Result<(), NTSTATUS> {
                    write_user(
                        core::ptr::addr_of_mut!((*IoStatusBlock).Information),
                        (*OpenPacket).Information,
                        AccessMode,
                    )?;
                    write_user(
                        core::ptr::addr_of_mut!((*IoStatusBlock).__bindgen_anon_1.Status),
                        (*OpenPacket).FinalStatus,
                        AccessMode,
                    )?;
                    Ok(())
                };
                if let Err(e) = inner() {
                    Status = e;
                }
            }
        } else if !(*OpenPacket).FileObject.is_null() && ((*OpenPacket).ParseCheck == 0) {
            if (*(*OpenPacket).FileObject).FileName.Length != 0 {
                ExFreePoolWithTag((*(*OpenPacket).FileObject).FileName.Buffer as PVOID, TAG_IO_NAME);
            }

            (*(*OpenPacket).FileObject).DeviceObject = core::ptr::null_mut();
            ObDereferenceObject((*OpenPacket).FileObject as PVOID);
        }
    } else {
        (*(*OpenPacket).FileObject).Flags |= FO_HANDLE_CREATED;

        let inner = || -> Result<(), NTSTATUS> {
            write_user(FileHandle, LocalHandle, AccessMode)?;
            write_user(
                core::ptr::addr_of_mut!((*IoStatusBlock).Information),
                (*OpenPacket).Information,
                AccessMode,
            )?;
            write_user(
                core::ptr::addr_of_mut!((*IoStatusBlock).__bindgen_anon_1.Status),
                (*OpenPacket).FinalStatus,
                AccessMode,
            )?;
            Ok(())
        };
        match inner() {
            Ok(_) => Status = (*OpenPacket).FinalStatus,
            Err(e) => Status = e,
        }
    }

    if ((*OpenPacket).ParseCheck != 0) && !(*OpenPacket).FileObject.is_null() {
        ObDereferenceObject((*OpenPacket).FileObject as PVOID);
    }

    ExFreePool(OpenPacket as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckQuerySetFileInformation(
    FileInformationClass: FILE_INFORMATION_CLASS,
    Length: ULONG,
    SetOperation: BOOLEAN,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoCheckQuotaBufferValidity(
    QuotaBuffer: *mut core::ffi::c_void,
    QuotaLength: ULONG,
    ErrorOffset: *mut ULONG,
) -> NTSTATUS {
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
    CreateFileType: CREATE_FILE_TYPE,
    ExtraCreateParameters: PVOID,
    Options: ULONG,
) -> NTSTATUS {
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
        core::ptr::null_mut(),
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
    CreateFileType: CREATE_FILE_TYPE,
    ExtraCreateParameters: PVOID,
    Options: ULONG,
    DeviceObject: PVOID,
) -> NTSTATUS {
    let mut Flags = 0;

    if !DeviceObject.is_null() {
        Flags = IOP_CREATE_FILE_OBJECT_EXTENSION | IOP_USE_TOP_LEVEL_DEVICE_HINT;
    }

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
    let mut CreatedFileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    let device_to_use = if !FileObject.is_null() {
        (*FileObject).DeviceObject
    } else {
        DeviceObject
    };

    InterlockedIncrement(core::ptr::addr_of_mut!((*device_to_use).ReferenceCount) as *mut LONG);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;

    Status = ObCreateObject(
        0i8,
        IoFileObjectType,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0i8,
        core::ptr::null_mut(),
        core::mem::size_of::<_FILE_OBJECT>() as u32,
        core::mem::size_of::<_FILE_OBJECT>() as u32,
        0,
        core::ptr::addr_of_mut!(CreatedFileObject) as *mut PVOID,
    );
    if Status < 0 {
        IopDereferenceDeviceObject(device_to_use, 0u8);
        ExRaiseStatus(Status);
    }

    core::ptr::write_bytes(CreatedFileObject as *mut u8, 0, core::mem::size_of::<_FILE_OBJECT>());
    (*CreatedFileObject).DeviceObject = device_to_use;
    (*CreatedFileObject).Type = IO_TYPE_FILE as i16;
    (*CreatedFileObject).Size = core::mem::size_of::<_FILE_OBJECT>() as i16;
    (*CreatedFileObject).Flags = FO_STREAM_FILE;

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*CreatedFileObject).Event),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    Status = ObInsertObject(
        CreatedFileObject as PVOID,
        core::ptr::null_mut(),
        FILE_READ_DATA,
        1,
        core::ptr::addr_of_mut!(CreatedFileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(FileHandle),
    );
    if Status < 0 {
        ExRaiseStatus(Status);
    }

    (*CreatedFileObject).Flags |= FO_HANDLE_CREATED;

    if !(*device_to_use).Vpb.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*device_to_use).Vpb).ReferenceCount) as *mut LONG);
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
    IoCreateStreamFileObjectEx(FileObject, DeviceObject, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateStreamFileObjectLite(
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
) -> PFILE_OBJECT {
    let mut CreatedFileObject: PFILE_OBJECT = core::ptr::null_mut();
    let device_to_use = if !FileObject.is_null() { (*FileObject).DeviceObject } else { DeviceObject };
    InterlockedIncrement(core::ptr::addr_of_mut!((*device_to_use).ReferenceCount) as *mut LONG);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    let Status = ObCreateObject(0i8, IoFileObjectType, core::ptr::addr_of_mut!(ObjectAttributes), 0i8,
        core::ptr::null_mut(), core::mem::size_of::<_FILE_OBJECT>() as u32, core::mem::size_of::<_FILE_OBJECT>() as u32,
        0, core::ptr::addr_of_mut!(CreatedFileObject) as *mut PVOID);
    if Status < 0 { IopDereferenceDeviceObject(device_to_use, 0u8); ExRaiseStatus(Status); }
    core::ptr::write_bytes(CreatedFileObject as *mut u8, 0, core::mem::size_of::<_FILE_OBJECT>());
    (*CreatedFileObject).DeviceObject = device_to_use;
    (*CreatedFileObject).Type = IO_TYPE_FILE as i16;
    (*CreatedFileObject).Size = core::mem::size_of::<_FILE_OBJECT>() as i16;
    (*CreatedFileObject).Flags = FO_STREAM_FILE;
    KeInitializeEvent(core::ptr::addr_of_mut!((*CreatedFileObject).Event), _EVENT_TYPE_SynchronizationEvent, 0u8);
    let obj_header = OBJECT_TO_OBJECT_HEADER(CreatedFileObject as PVOID);
    ObFreeObjectCreateInfoBuffer((*obj_header).__bindgen_anon_2.ObjectCreateInfo);
    (*obj_header).__bindgen_anon_2.ObjectCreateInfo = core::ptr::null_mut();
    (*CreatedFileObject).Flags |= FO_HANDLE_CREATED;
    if !(*device_to_use).Vpb.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*device_to_use).Vpb).ReferenceCount) as *mut LONG);
    }
    CreatedFileObject
}

#[no_mangle]
pub unsafe extern "system" fn IoGetFileObjectGenericMapping() -> PGENERIC_MAPPING {
    core::ptr::addr_of_mut!(IopFileMapping)
}

#[no_mangle]
pub unsafe extern "system" fn IoIsFileOriginRemote(FileObject: PFILE_OBJECT) -> BOOLEAN {
    if (*FileObject).Flags & FO_REMOTE_ORIGIN != 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn IoSetFileOrigin(FileObject: PFILE_OBJECT, Remote: BOOLEAN) -> NTSTATUS {
    let FlagSet = ((*FileObject).Flags & FO_REMOTE_ORIGIN) != 0;
    if Remote != 0 && !FlagSet { (*FileObject).Flags |= FO_REMOTE_ORIGIN; }
    else if Remote == 0 && FlagSet { (*FileObject).Flags &= !FO_REMOTE_ORIGIN; }
    else { return STATUS_INVALID_PARAMETER_MIX; }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoFastQueryNetworkAttributes(
    ObjectAttributes: POBJECT_ATTRIBUTES, DesiredAccess: ACCESS_MASK, OpenOptions: ULONG,
    IoStatus: PIO_STATUS_BLOCK, Buffer: PFILE_NETWORK_OPEN_INFORMATION,
) -> BOOLEAN {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut OpenPacket: _OPEN_PACKET = core::mem::zeroed();
    let mut LocalFileObject: _DUMMY_FILE_OBJECT = core::mem::zeroed();
    OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
    OpenPacket.Size = core::mem::size_of::<_OPEN_PACKET>() as i16;
    OpenPacket.CreateOptions = OpenOptions | FILE_OPEN_REPARSE_POINT;
    OpenPacket.ShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u16;
    OpenPacket.Options = IO_FORCE_ACCESS_CHECK;
    OpenPacket.Disposition = FILE_OPEN;
    OpenPacket.NetworkInformation = Buffer;
    OpenPacket.QueryOnly = 1u8;
    OpenPacket.FullAttributes = 1u8;
    OpenPacket.LocalFileObject = core::ptr::addr_of_mut!(LocalFileObject);
    let Status = ObOpenObjectByName(ObjectAttributes, core::ptr::null_mut(), 0i8, core::ptr::null_mut(),
        DesiredAccess, core::ptr::addr_of_mut!(OpenPacket) as PVOID, core::ptr::addr_of_mut!(Handle));
    if OpenPacket.ParseCheck == 0 { (*IoStatus).__bindgen_anon_1.Status = Status; }
    else { (*IoStatus).__bindgen_anon_1.Status = OpenPacket.FinalStatus; (*IoStatus).Information = OpenPacket.Information as DdkUlongPtr; }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn IoUpdateShareAccess(FileObject: PFILE_OBJECT, ShareAccess: PSHARE_ACCESS) {
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 { return; }
    if (*FileObject).ReadAccess != 0 || (*FileObject).WriteAccess != 0 || (*FileObject).DeleteAccess != 0 {
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
    DesiredAccess: ACCESS_MASK, DesiredShareAccess: ULONG, FileObject: PFILE_OBJECT,
    ShareAccess: PSHARE_ACCESS, Update: BOOLEAN,
) -> NTSTATUS {
    let ReadAccess: BOOLEAN = if (DesiredAccess & (FILE_READ_DATA | FILE_EXECUTE)) != 0 { 1 } else { 0 };
    let WriteAccess: BOOLEAN = if (DesiredAccess & (FILE_WRITE_DATA | FILE_APPEND_DATA)) != 0 { 1 } else { 0 };
    let DeleteAccess: BOOLEAN = if (DesiredAccess & DELETE) != 0 { 1 } else { 0 };
    (*FileObject).ReadAccess = ReadAccess; (*FileObject).WriteAccess = WriteAccess; (*FileObject).DeleteAccess = DeleteAccess;
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 { return STATUS_SUCCESS; }
    if ReadAccess != 0 || WriteAccess != 0 || DeleteAccess != 0 {
        let SharedRead: BOOLEAN = if (DesiredShareAccess & FILE_SHARE_READ) != 0 { 1 } else { 0 };
        let SharedWrite: BOOLEAN = if (DesiredShareAccess & FILE_SHARE_WRITE) != 0 { 1 } else { 0 };
        let SharedDelete: BOOLEAN = if (DesiredShareAccess & FILE_SHARE_DELETE) != 0 { 1 } else { 0 };
        if (ReadAccess != 0 && (*ShareAccess).SharedRead < (*ShareAccess).OpenCount) ||
           (WriteAccess != 0 && (*ShareAccess).SharedWrite < (*ShareAccess).OpenCount) ||
           (DeleteAccess != 0 && (*ShareAccess).SharedDelete < (*ShareAccess).OpenCount) ||
           ((*ShareAccess).Readers != 0 && SharedRead == 0) ||
           ((*ShareAccess).Writers != 0 && SharedWrite == 0) ||
           ((*ShareAccess).Deleters != 0 && SharedDelete == 0) { return STATUS_SHARING_VIOLATION; }
        (*FileObject).SharedRead = SharedRead; (*FileObject).SharedWrite = SharedWrite; (*FileObject).SharedDelete = SharedDelete;
        if Update != 0 {
            (*ShareAccess).OpenCount += 1;
            (*ShareAccess).Readers += ReadAccess as u32; (*ShareAccess).Writers += WriteAccess as u32; (*ShareAccess).Deleters += DeleteAccess as u32;
            (*ShareAccess).SharedRead += SharedRead as u32; (*ShareAccess).SharedWrite += SharedWrite as u32; (*ShareAccess).SharedDelete += SharedDelete as u32;
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoRemoveShareAccess(FileObject: PFILE_OBJECT, ShareAccess: PSHARE_ACCESS) {
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 { return; }
    if (*FileObject).ReadAccess != 0 || (*FileObject).WriteAccess != 0 || (*FileObject).DeleteAccess != 0 {
        (*ShareAccess).OpenCount -= 1;
        (*ShareAccess).Readers -= (*FileObject).ReadAccess as u32; (*ShareAccess).Writers -= (*FileObject).WriteAccess as u32;
        (*ShareAccess).Deleters -= (*FileObject).DeleteAccess as u32; (*ShareAccess).SharedRead -= (*FileObject).SharedRead as u32;
        (*ShareAccess).SharedWrite -= (*FileObject).SharedWrite as u32; (*ShareAccess).SharedDelete -= (*FileObject).SharedDelete as u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoSetShareAccess(
    DesiredAccess: ACCESS_MASK, DesiredShareAccess: ULONG, FileObject: PFILE_OBJECT, ShareAccess: PSHARE_ACCESS,
) {
    let ReadAccess: BOOLEAN = if (DesiredAccess & (FILE_READ_DATA | FILE_EXECUTE)) != 0 { 1 } else { 0 };
    let WriteAccess: BOOLEAN = if (DesiredAccess & (FILE_WRITE_DATA | FILE_APPEND_DATA)) != 0 { 1 } else { 0 };
    let DeleteAccess: BOOLEAN = if (DesiredAccess & DELETE) != 0 { 1 } else { 0 };
    let mut Update = true;
    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 { Update = false; }
    (*FileObject).ReadAccess = ReadAccess; (*FileObject).WriteAccess = WriteAccess; (*FileObject).DeleteAccess = DeleteAccess;
    if ReadAccess == 0 && WriteAccess == 0 && DeleteAccess == 0 {
        if !Update { return; }
        (*ShareAccess).OpenCount = 0; (*ShareAccess).Readers = 0; (*ShareAccess).Writers = 0; (*ShareAccess).Deleters = 0;
        (*ShareAccess).SharedRead = 0; (*ShareAccess).SharedWrite = 0; (*ShareAccess).SharedDelete = 0;
    } else {
        let SharedRead: BOOLEAN = if (DesiredShareAccess & FILE_SHARE_READ) != 0 { 1 } else { 0 };
        let SharedWrite: BOOLEAN = if (DesiredShareAccess & FILE_SHARE_WRITE) != 0 { 1 } else { 0 };
        let SharedDelete: BOOLEAN = if (DesiredShareAccess & FILE_SHARE_DELETE) != 0 { 1 } else { 0 };
        (*FileObject).SharedRead = SharedRead; (*FileObject).SharedWrite = SharedWrite; (*FileObject).SharedDelete = SharedDelete;
        if !Update { return; }
        (*ShareAccess).OpenCount = 1; (*ShareAccess).Readers = ReadAccess as u32; (*ShareAccess).Writers = WriteAccess as u32;
        (*ShareAccess).Deleters = DeleteAccess as u32; (*ShareAccess).SharedRead = SharedRead as u32;
        (*ShareAccess).SharedWrite = SharedWrite as u32; (*ShareAccess).SharedDelete = SharedDelete as u32;
    }
}

const INVALID_CANCEL_OF_FILE_OPEN: ULONG = 0xE8;
#[no_mangle]
pub unsafe extern "system" fn IoCancelFileOpen(DeviceObject: PDEVICE_OBJECT, FileObject: PFILE_OBJECT) {
    if (*FileObject).Flags & FO_HANDLE_CREATED != 0 {
        KeBugCheckEx(INVALID_CANCEL_OF_FILE_OPEN, FileObject as usize, DeviceObject as usize, 0, 0);
    }
    let mut Event: _KEVENT = core::mem::zeroed();
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0u8);
    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    let Irp = IopAllocateIrpMustSucceed((*DeviceObject).StackSize as i8);
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event);
    (*Irp).UserIosb = core::ptr::addr_of_mut!((*Irp).IoStatus);
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).RequestorMode = 0i8;
    (*Irp).Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;
    let Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = IRP_MJ_CLEANUP; (*Stack).FileObject = FileObject;
    IopQueueIrpToThread(Irp);
    let Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING { KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, _KWAIT_REASON_UserRequest, 0i8, 0u8, core::ptr::null_mut()); }
    let mut OldIrql: KIRQL = 0;
    KeRaiseIrql(APC_LEVEL as u8, core::ptr::addr_of_mut!(OldIrql));
    IopUnQueueIrpFromThread(Irp);
    KeLowerIrql(OldIrql);
    IoFreeIrp(Irp);
    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    (*FileObject).Flags |= FO_FILE_OPEN_CANCELLED;
}

#[no_mangle]
pub unsafe extern "system" fn IoQueryFileDosDeviceName(
    FileObject: PFILE_OBJECT, ObjectNameInformation: *mut POBJECT_NAME_INFORMATION,
) -> NTSTATUS {
    let mut ReturnLength: ULONG = 200;
    let mut Length: ULONG = 200;
    loop {
        let LocalInfo = ExAllocatePoolWithTag(PagedPool, Length as usize, 0x6E446F49) as POBJECT_NAME_INFORMATION;
        if LocalInfo.is_null() { return STATUS_INSUFFICIENT_RESOURCES; }
        let Status = IopQueryNameInternal(FileObject as PVOID, 1u8, 1u8, LocalInfo, Length, core::ptr::addr_of_mut!(ReturnLength), 0i8);
        if Status == STATUS_SUCCESS { *ObjectNameInformation = LocalInfo; return STATUS_SUCCESS; }
        ExFreePoolWithTag(LocalInfo as PVOID, 0x6E446F49);
        if Status != STATUS_BUFFER_OVERFLOW { return Status; }
        Length = ReturnLength;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateFile(
    FileHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK, AllocateSize: *mut LARGE_INTEGER, FileAttributes: ULONG,
    ShareAccess: ULONG, CreateDisposition: ULONG, CreateOptions: ULONG, EaBuffer: PVOID, EaLength: ULONG,
) -> NTSTATUS {
    IoCreateFile(FileHandle, DesiredAccess, ObjectAttributes, IoStatusBlock, AllocateSize,
        FileAttributes, ShareAccess, CreateDisposition, CreateOptions, EaBuffer, EaLength,
        CreateFileTypeNone, core::ptr::null_mut(), 0)
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenFile(
    FileHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK, ShareAccess: ULONG, OpenOptions: ULONG,
) -> NTSTATUS {
    IoCreateFile(FileHandle, DesiredAccess, ObjectAttributes, IoStatusBlock, core::ptr::null_mut(),
        0, ShareAccess, FILE_OPEN, OpenOptions, core::ptr::null_mut(), 0,
        CreateFileTypeNone, core::ptr::null_mut(), 0)
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateMailslotFile(
    FileHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK, CreateOptions: ULONG, MailslotQuota: ULONG,
    MaxMessageSize: ULONG, TimeOut: *mut LARGE_INTEGER,
) -> NTSTATUS {
    let mut Buffer: _MAILSLOT_CREATE_PARAMETERS = core::mem::zeroed();
    if !TimeOut.is_null() {
        if ExGetPreviousMode() != 0i8 {
            let probed = ProbeForReadLargeInteger(TimeOut); Buffer.ReadTimeout.QuadPart = probed.QuadPart;
        } else { Buffer.ReadTimeout.QuadPart = (*TimeOut).QuadPart; }
        Buffer.TimeoutSpecified = 1u8;
    } else { Buffer.TimeoutSpecified = 0u8; }
    Buffer.MailslotQuota = MailslotQuota; Buffer.MaximumMessageSize = MaxMessageSize;
    IoCreateFile(FileHandle, DesiredAccess, ObjectAttributes, IoStatusBlock, core::ptr::null_mut(), 0,
        FILE_SHARE_READ | FILE_SHARE_WRITE, FILE_CREATE, CreateOptions, core::ptr::null_mut(), 0,
        CreateFileTypeMailslot, core::ptr::addr_of_mut!(Buffer) as PVOID, 0)
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateNamedPipeFile(
    FileHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK, ShareAccess: ULONG, CreateDisposition: ULONG,
    CreateOptions: ULONG, NamedPipeType: ULONG, ReadMode: ULONG, CompletionMode: ULONG,
    MaximumInstances: ULONG, InboundQuota: ULONG, OutboundQuota: ULONG, DefaultTimeout: *mut LARGE_INTEGER,
) -> NTSTATUS {
    let mut Buffer: _NAMED_PIPE_CREATE_PARAMETERS = core::mem::zeroed();
    if !DefaultTimeout.is_null() {
        if ExGetPreviousMode() != 0i8 {
            let probed = ProbeForReadLargeInteger(DefaultTimeout); Buffer.DefaultTimeout.QuadPart = probed.QuadPart;
        } else { Buffer.DefaultTimeout.QuadPart = (*DefaultTimeout).QuadPart; }
        Buffer.TimeoutSpecified = 1u8;
    } else { Buffer.TimeoutSpecified = 0u8; }
    Buffer.NamedPipeType = NamedPipeType; Buffer.ReadMode = ReadMode; Buffer.CompletionMode = CompletionMode;
    Buffer.MaximumInstances = MaximumInstances; Buffer.InboundQuota = InboundQuota; Buffer.OutboundQuota = OutboundQuota;
    IoCreateFile(FileHandle, DesiredAccess, ObjectAttributes, IoStatusBlock, core::ptr::null_mut(), 0,
        ShareAccess, CreateDisposition, CreateOptions, core::ptr::null_mut(), 0,
        CreateFileTypeNamedPipe, core::ptr::addr_of_mut!(Buffer) as PVOID, 0)
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushWriteBuffer() -> NTSTATUS { KeFlushWriteBuffer(); STATUS_SUCCESS }

#[no_mangle]
pub unsafe extern "system" fn NtQueryAttributesFile(
    ObjectAttributes: POBJECT_ATTRIBUTES, FileInformation: PFILE_BASIC_INFORMATION,
) -> NTSTATUS {
    IopQueryAttributesFile(ObjectAttributes, FileBasicInformation, core::mem::size_of::<_FILE_BASIC_INFORMATION>() as ULONG, FileInformation as PVOID)
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryFullAttributesFile(
    ObjectAttributes: POBJECT_ATTRIBUTES, FileInformation: PFILE_NETWORK_OPEN_INFORMATION,
) -> NTSTATUS {
    IopQueryAttributesFile(ObjectAttributes, FileNetworkOpenInformation, core::mem::size_of::<_FILE_NETWORK_OPEN_INFORMATION>() as ULONG, FileInformation as PVOID)
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteFile(ObjectAttributes: POBJECT_ATTRIBUTES) -> NTSTATUS {
    let mut LocalFileObject: _DUMMY_FILE_OBJECT = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let AccessMode = ExGetPreviousMode();
    let mut OpenPacket: _OPEN_PACKET = core::mem::zeroed();
    OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
    OpenPacket.Size = core::mem::size_of::<_OPEN_PACKET>() as i16;
    OpenPacket.CreateOptions = FILE_DELETE_ON_CLOSE;
    OpenPacket.ShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u16;
    OpenPacket.Disposition = FILE_OPEN;
    OpenPacket.DeleteOnly = 1u8;
    OpenPacket.LocalFileObject = core::ptr::addr_of_mut!(LocalFileObject);
    IopUpdateOperationCount(IopOtherTransfer);
    let Status = ObOpenObjectByName(ObjectAttributes, core::ptr::null_mut(), AccessMode, core::ptr::null_mut(),
        DELETE, core::ptr::addr_of_mut!(OpenPacket) as PVOID, core::ptr::addr_of_mut!(Handle));
    if OpenPacket.ParseCheck == 0 { return Status; }
    OpenPacket.FinalStatus
}

#[no_mangle]
pub unsafe extern "system" fn NtCancelIoFile(FileHandle: HANDLE, IoStatusBlock: PIO_STATUS_BLOCK) -> NTSTATUS {
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut OurIrpsInList: BOOLEAN = 0u8;
    let PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0i8 { ProbeForWriteIoStatusBlock(IoStatusBlock); }
    let Status = ObReferenceObjectByHandle(FileHandle, 0, IoFileObjectType, PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID, core::ptr::null_mut());
    if Status < 0 { return Status; }
    KeRaiseIrql(APC_LEVEL as u8, core::ptr::addr_of_mut!(OldIrql));
    let Thread = PsGetCurrentThread();
    IopUpdateOperationCount(IopOtherTransfer);
    let ListHead = core::ptr::addr_of_mut!((*Thread).IrpList);
    let mut NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        let Irp = CONTAINING_RECORD!(NextEntry, _IRP, ThreadListEntry) as PIRP;
        if (*Irp).Tail.Overlay.OriginalFileObject == FileObject { IoCancelIrp(Irp); OurIrpsInList = 1u8; }
        NextEntry = (*NextEntry).Flink;
    }
    KeLowerIrql(OldIrql);
    if OurIrpsInList != 0 {
        let mut Interval: LARGE_INTEGER = core::mem::zeroed();
        Interval.QuadPart = -100000i64;
        while OurIrpsInList != 0 {
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(Interval));
            OurIrpsInList = 0u8;
            KeRaiseIrql(APC_LEVEL as u8, core::ptr::addr_of_mut!(OldIrql));
            let mut NextEntry2 = (*ListHead).Flink;
            while NextEntry2 != ListHead {
                let Irp = CONTAINING_RECORD!(NextEntry2, _IRP, ThreadListEntry) as PIRP;
                if (*Irp).Tail.Overlay.OriginalFileObject == FileObject { OurIrpsInList = 1u8; break; }
                NextEntry2 = (*NextEntry2).Flink;
            }
            KeLowerIrql(OldIrql);
        }
    }
    (*IoStatusBlock).__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*IoStatusBlock).Information = 0;
    ObDereferenceObject(FileObject as PVOID);
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/file.c
//  * PURPOSE:         Functions that deal with managing the FILE_OBJECT itself.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Gunnar Dalsnes
//  *                  Eric Kohl
//  *                  Filip Navara (navaraf@reactos.org)
//  *                  Pierre Schweitzer
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern ERESOURCE IopSecurityResource;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// IopCheckBackupRestorePrivilege(IN PACCESS_STATE AccessState,
//                                IN OUT PULONG CreateOptions,
//                                IN KPROCESSOR_MODE PreviousMode,
//                                IN ULONG Disposition)
// {
//     ACCESS_MASK DesiredAccess, ReadAccess, WriteAccess;
//     PRIVILEGE_SET Privileges;
//     BOOLEAN AccessGranted, HaveBackupPriv = FALSE, CheckRestore = FALSE;
//     PAGED_CODE();
//
//     /* Don't do anything if privileges were checked already */
//     if (AccessState->Flags & SE_BACKUP_PRIVILEGES_CHECKED) return;
//
//     /* Check if the file was actually opened for backup purposes */
//     if (*CreateOptions & FILE_OPEN_FOR_BACKUP_INTENT)
//     {
//         /* Set the check flag since were doing it now */
//         AccessState->Flags |= SE_BACKUP_PRIVILEGES_CHECKED;
//
//         /* Set the access masks required */
//         ReadAccess = READ_CONTROL |
//                      ACCESS_SYSTEM_SECURITY |
//                      FILE_GENERIC_READ |
//                      FILE_TRAVERSE;
//         WriteAccess = WRITE_DAC |
//                       WRITE_OWNER |
//                       ACCESS_SYSTEM_SECURITY |
//                       FILE_GENERIC_WRITE |
//                       FILE_ADD_FILE |
//                       FILE_ADD_SUBDIRECTORY |
//                       DELETE;
//         DesiredAccess = AccessState->RemainingDesiredAccess;
//
//         /* Check if desired access was the maximum */
//         if (DesiredAccess & MAXIMUM_ALLOWED)
//         {
//             /* Then add all the access masks required */
//             DesiredAccess |= (ReadAccess | WriteAccess);
//         }
//
//         /* Check if the file already exists */
//         if (Disposition & FILE_OPEN)
//         {
//             /* Check if desired access has the read mask */
//             if (ReadAccess & DesiredAccess)
//             {
//                 /* Setup the privilege check lookup */
//                 Privileges.PrivilegeCount = 1;
//                 Privileges.Control = PRIVILEGE_SET_ALL_NECESSARY;
//                 Privileges.Privilege[0].Luid = SeBackupPrivilege;
//                 Privileges.Privilege[0].Attributes = 0;
//                 AccessGranted = SePrivilegeCheck(&Privileges,
//                                                  &AccessState->
//                                                  SubjectSecurityContext,
//                                                  PreviousMode);
//                 if (AccessGranted)
//                 {
//                     /* Remember that backup was allowed */
//                     HaveBackupPriv = TRUE;
//
//                     /* Append the privileges and update the access state */
//                     SeAppendPrivileges(AccessState, &Privileges);
//                     AccessState->PreviouslyGrantedAccess |= (DesiredAccess & ReadAccess);
//                     AccessState->RemainingDesiredAccess &= ~ReadAccess;
//                     DesiredAccess &= ~ReadAccess;
//
//                     /* Set backup privilege for the token */
//                     AccessState->Flags |= TOKEN_HAS_BACKUP_PRIVILEGE;
//                 }
//             }
//         }
//         else
//         {
//             /* Caller is creating the file, check restore privileges later */
//             CheckRestore = TRUE;
//         }
//
//         /* Check if caller wants write access or if it's creating a file */
//         if ((WriteAccess & DesiredAccess) || (CheckRestore))
//         {
//             /* Setup the privilege lookup and do it */
//             Privileges.PrivilegeCount = 1;
//             Privileges.Control = PRIVILEGE_SET_ALL_NECESSARY;
//             Privileges.Privilege[0].Luid = SeRestorePrivilege;
//             Privileges.Privilege[0].Attributes = 0;
//             AccessGranted = SePrivilegeCheck(&Privileges,
//                                              &AccessState->SubjectSecurityContext,
//                                              PreviousMode);
//             if (AccessGranted)
//             {
//                 /* Remember that privilege was given */
//                 HaveBackupPriv = TRUE;
//
//                 /* Append the privileges and update the access state */
//                 SeAppendPrivileges(AccessState, &Privileges);
//                 AccessState->PreviouslyGrantedAccess |= (DesiredAccess & WriteAccess);
//                 AccessState->RemainingDesiredAccess &= ~WriteAccess;
//
//                 /* Set restore privilege for the token */
//                 AccessState->Flags |= TOKEN_HAS_RESTORE_PRIVILEGE;
//             }
//         }
//
//         /* If we don't have the privilege, remove the option */
//         if (!HaveBackupPriv) *CreateOptions &= ~FILE_OPEN_FOR_BACKUP_INTENT;
//     }
// }
//
// NTSTATUS
// NTAPI
// IopCheckDeviceAndDriver(IN POPEN_PACKET OpenPacket,
//                         IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Make sure the object is valid */
//     if ((IoGetDevObjExtension(DeviceObject)->ExtensionFlags &
//         (DOE_UNLOAD_PENDING |
//          DOE_DELETE_PENDING |
//          DOE_REMOVE_PENDING |
//          DOE_REMOVE_PROCESSED)) ||
//         (DeviceObject->Flags & DO_DEVICE_INITIALIZING))
//     {
//         /* It's unloading or initializing, so fail */
//         DPRINT1("You are seeing this because the following ROS driver: %wZ\n"
//                 " sucks. Please fix it's AddDevice Routine\n",
//                 &DeviceObject->DriverObject->DriverName);
//         return STATUS_NO_SUCH_DEVICE;
//     }
//     else if ((DeviceObject->Flags & DO_EXCLUSIVE) &&
//              (DeviceObject->ReferenceCount) &&
//              !(OpenPacket->RelatedFileObject) &&
//              !(OpenPacket->Options & IO_ATTACH_DEVICE))
//     {
//         return STATUS_ACCESS_DENIED;
//     }
//
//     else
//     {
//         /* Increase reference count */
//         InterlockedIncrement(&DeviceObject->ReferenceCount);
//         return STATUS_SUCCESS;
//     }
// }
//
// VOID
// NTAPI
// IopDoNameTransmogrify(IN PIRP Irp,
//                       IN PFILE_OBJECT FileObject,
//                       IN PREPARSE_DATA_BUFFER DataBuffer)
// {
//     PWSTR Buffer;
//     USHORT Length;
//     USHORT RequiredLength;
//     PWSTR NewBuffer;
//
//     PAGED_CODE();
//
//     ASSERT(Irp->IoStatus.Status == STATUS_REPARSE);
//     ASSERT(Irp->IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT);
//     ASSERT(Irp->Tail.Overlay.AuxiliaryBuffer != NULL);
//     ASSERT(DataBuffer != NULL);
//     ASSERT(DataBuffer->ReparseTag == IO_REPARSE_TAG_MOUNT_POINT);
//     ASSERT(DataBuffer->ReparseDataLength < MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     ASSERT(DataBuffer->Reserved < MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//
//     /* First of all, validate data */
//     if (DataBuffer->ReparseDataLength < REPARSE_DATA_BUFFER_HEADER_SIZE ||
//         (DataBuffer->SymbolicLinkReparseBuffer.PrintNameLength +
//          DataBuffer->SymbolicLinkReparseBuffer.SubstituteNameLength +
//          FIELD_OFFSET(REPARSE_DATA_BUFFER, MountPointReparseBuffer.PathBuffer[0])) > MAXIMUM_REPARSE_DATA_BUFFER_SIZE)
//     {
//         Irp->IoStatus.Status = STATUS_IO_REPARSE_DATA_INVALID;
//     }
//
//     /* Everything went right */
//     if (NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         /* Compute buffer & length */
//         Buffer = (PWSTR)((ULONG_PTR)DataBuffer->MountPointReparseBuffer.PathBuffer +
//                                     DataBuffer->MountPointReparseBuffer.SubstituteNameOffset);
//         Length = DataBuffer->MountPointReparseBuffer.SubstituteNameLength;
//
//         /* Check we don't overflow */
//         if (((ULONG)MAXUSHORT - DataBuffer->Reserved) <= (Length + sizeof(UNICODE_NULL)))
//         {
//             Irp->IoStatus.Status = STATUS_IO_REPARSE_DATA_INVALID;
//         }
//         else
//         {
//             /* Compute how much memory we'll need */
//             RequiredLength = DataBuffer->Reserved + Length + sizeof(UNICODE_NULL);
//
//             /* Check if FileObject can already hold what we need */
//             if (FileObject->FileName.MaximumLength >= RequiredLength)
//             {
//                 NewBuffer = FileObject->FileName.Buffer;
//             }
//             else
//             {
//                 /* Allocate otherwise */
//                 NewBuffer = ExAllocatePoolWithTag(PagedPool, RequiredLength, TAG_IO_NAME);
//                 if (NewBuffer == NULL)
//                 {
//                     Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//             }
//         }
//     }
//
//     /* Everything went right */
//     if (NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         /* Copy the reserved data */
//         if (DataBuffer->Reserved)
//         {
//             RtlMoveMemory((PWSTR)((ULONG_PTR)NewBuffer + Length),
//                           (PWSTR)((ULONG_PTR)FileObject->FileName.Buffer + FileObject->FileName.Length - DataBuffer->Reserved),
//                           DataBuffer->Reserved);
//         }
//
//         /* Then the buffer */
//         if (Length)
//         {
//             RtlCopyMemory(NewBuffer, Buffer, Length);
//         }
//
//         /* And finally replace buffer if new one was allocated */
//         FileObject->FileName.Length = RequiredLength - sizeof(UNICODE_NULL);
//         if (NewBuffer != FileObject->FileName.Buffer)
//         {
//             if (FileObject->FileName.Buffer)
//             {
//                 /*
//                  * Don't use TAG_IO_NAME since the FileObject's FileName
//                  * may have been re-allocated using a different tag
//                  * by a filesystem.
//                  */
//                 ExFreePoolWithTag(FileObject->FileName.Buffer, 0);
//             }
//
//             FileObject->FileName.Buffer = NewBuffer;
//             FileObject->FileName.MaximumLength = RequiredLength;
//             FileObject->FileName.Buffer[RequiredLength / sizeof(WCHAR) - 1] = UNICODE_NULL;
//         }
//     }
//
//     /* We don't need them anymore - it was allocated by the driver */
//     ExFreePool(DataBuffer);
// }
//
// NTSTATUS
// IopCheckTopDeviceHint(IN OUT PDEVICE_OBJECT * DeviceObject,
//                       IN POPEN_PACKET OpenPacket,
//                       BOOLEAN DirectOpen)
// {
//     PDEVICE_OBJECT LocalDevice;
//     DEVICE_TYPE DeviceType;
//
//     LocalDevice = *DeviceObject;
//
//     /* Direct open is not allowed */
//     if (DirectOpen)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate we have a file system device */
//     DeviceType = LocalDevice->DeviceType;
//     if (DeviceType != FILE_DEVICE_DISK_FILE_SYSTEM &&
//         DeviceType != FILE_DEVICE_CD_ROM_FILE_SYSTEM &&
//         DeviceType != FILE_DEVICE_TAPE_FILE_SYSTEM &&
//         DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM &&
//         DeviceType != FILE_DEVICE_DFS_FILE_SYSTEM)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Verify the hint and if it's OK, return it */
//     if (IopVerifyDeviceObjectOnStack(LocalDevice, OpenPacket->TopDeviceObjectHint))
//     {
//         *DeviceObject = OpenPacket->TopDeviceObjectHint;
//         return STATUS_SUCCESS;
//     }
//
//     /* Failure case here */
//     /* If we thought was had come through a mount point,
//      * actually update we didn't and return the error
//      */
//     if (OpenPacket->TraversedMountPoint)
//     {
//         OpenPacket->TraversedMountPoint = FALSE;
//         return STATUS_MOUNT_POINT_NOT_RESOLVED;
//     }
//
//     /* Otherwise, just return the fact the hint is invalid */
//     return STATUS_INVALID_DEVICE_OBJECT_PARAMETER;
// }
//
// NTSTATUS
// NTAPI
// IopParseDevice(IN PVOID ParseObject,
//                IN PVOID ObjectType,
//                IN OUT PACCESS_STATE AccessState,
//                IN KPROCESSOR_MODE AccessMode,
//                IN ULONG Attributes,
//                IN OUT PUNICODE_STRING CompleteName,
//                IN OUT PUNICODE_STRING RemainingName,
//                IN OUT PVOID Context,
//                IN PSECURITY_QUALITY_OF_SERVICE SecurityQos OPTIONAL,
//                OUT PVOID *Object)
// {
//     POPEN_PACKET OpenPacket = (POPEN_PACKET)Context;
//     PDEVICE_OBJECT OriginalDeviceObject = (PDEVICE_OBJECT)ParseObject;
//     PDEVICE_OBJECT DeviceObject, OwnerDevice;
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PVPB Vpb = NULL;
//     PIRP Irp;
//     PIO_STACK_LOCATION StackLoc;
//     IO_SECURITY_CONTEXT SecurityContext;
//     IO_STATUS_BLOCK IoStatusBlock;
//     BOOLEAN DirectOpen = FALSE, OpenCancelled, UseDummyFile;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     KIRQL OldIrql;
//     PDUMMY_FILE_OBJECT LocalFileObject;
//     PFILE_BASIC_INFORMATION FileBasicInfo;
//     ULONG ReturnLength;
//     KPROCESSOR_MODE CheckMode;
//     BOOLEAN VolumeOpen = FALSE;
//     ACCESS_MASK DesiredAccess, GrantedAccess;
//     BOOLEAN AccessGranted, LockHeld = FALSE;
//     PPRIVILEGE_SET Privileges = NULL;
//     UNICODE_STRING FileString;
//     USHORT Attempt;
//     IOTRACE(IO_FILE_DEBUG, "ParseObject: %p. RemainingName: %wZ\n",
//             ParseObject, RemainingName);
//
//     for (Attempt = 0; Attempt < IOP_MAX_REPARSE_TRAVERSAL; ++Attempt)
//     {
//         /* Assume failure */
//         *Object = NULL;
//
//         /* Validate the open packet */
//         if (!IopValidateOpenPacket(OpenPacket)) return STATUS_OBJECT_TYPE_MISMATCH;
//
//         /* Valide reparse point in case we traversed a mountpoint */
//         if (OpenPacket->TraversedMountPoint)
//         {
//             /* This is a reparse point we understand */
//             ASSERT(OpenPacket->Information == IO_REPARSE_TAG_MOUNT_POINT);
//
//             /* Make sure we're dealing with correct DO */
//             if (OriginalDeviceObject->DeviceType != FILE_DEVICE_DISK &&
//                 OriginalDeviceObject->DeviceType != FILE_DEVICE_CD_ROM &&
//                 OriginalDeviceObject->DeviceType != FILE_DEVICE_VIRTUAL_DISK &&
//                 OriginalDeviceObject->DeviceType != FILE_DEVICE_TAPE)
//             {
//                 OpenPacket->FinalStatus = STATUS_IO_REPARSE_DATA_INVALID;
//                 return STATUS_IO_REPARSE_DATA_INVALID;
//             }
//         }
//
//         /* Check if we have a related file object */
//         if (OpenPacket->RelatedFileObject)
//         {
//             /* Use the related file object's device object */
//             OriginalDeviceObject = OpenPacket->RelatedFileObject->DeviceObject;
//         }
//
//         /* Validate device status */
//         Status = IopCheckDeviceAndDriver(OpenPacket, OriginalDeviceObject);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed, return status */
//             OpenPacket->FinalStatus = Status;
//             return Status;
//         }
//
//         /* Map the generic mask and set the new mapping in the access state */
//         RtlMapGenericMask(&AccessState->RemainingDesiredAccess,
//                           &IoFileObjectType->TypeInfo.GenericMapping);
//         RtlMapGenericMask(&AccessState->OriginalDesiredAccess,
//                           &IoFileObjectType->TypeInfo.GenericMapping);
//         SeSetAccessStateGenericMapping(AccessState,
//                                        &IoFileObjectType->TypeInfo.GenericMapping);
//         DesiredAccess = AccessState->RemainingDesiredAccess;
//
//         /* Check what kind of access checks to do */
//         if ((AccessMode != KernelMode) ||
//             (OpenPacket->Options & IO_FORCE_ACCESS_CHECK))
//         {
//             /* Call is from user-mode or kernel is forcing checks */
//             CheckMode = UserMode;
//         }
//         else
//         {
//             /* Call is from the kernel */
//             CheckMode = KernelMode;
//         }
//
//         /* Check privilege for backup or restore operation */
//         IopCheckBackupRestorePrivilege(AccessState,
//                                        &OpenPacket->CreateOptions,
//                                        CheckMode,
//                                        OpenPacket->Disposition);
//
//         /* Check if we are re-parsing */
//         if (((OpenPacket->Override) && !(RemainingName->Length)) ||
//             (AccessState->Flags & SE_BACKUP_PRIVILEGES_CHECKED))
//         {
//             /* Get granted access from the last call */
//             DesiredAccess |= AccessState->PreviouslyGrantedAccess;
//         }
//
//         /* Check if this is a volume open */
//         if ((OpenPacket->RelatedFileObject) &&
//             (OpenPacket->RelatedFileObject->Flags & FO_VOLUME_OPEN) &&
//             !(RemainingName->Length))
//         {
//             /* It is */
//             VolumeOpen = TRUE;
//         }
//
//         /* Now check if we need access checks */
//         if (((AccessMode != KernelMode) ||
//              (OpenPacket->Options & IO_FORCE_ACCESS_CHECK)) &&
//             (!(OpenPacket->RelatedFileObject) || (VolumeOpen)) &&
//             !(OpenPacket->Override))
//         {
//             KeEnterCriticalRegion();
//             ExAcquireResourceSharedLite(&IopSecurityResource, TRUE);
//
//             /* Check if a device object is being parsed  */
//             if (!RemainingName->Length)
//             {
//                 /* Lock the subject context */
//                 SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//                 LockHeld = TRUE;
//
//                 /* Do access check */
//                 AccessGranted = SeAccessCheck(OriginalDeviceObject->
//                                               SecurityDescriptor,
//                                               &AccessState->SubjectSecurityContext,
//                                               LockHeld,
//                                               DesiredAccess,
//                                               0,
//                                               &Privileges,
//                                               &IoFileObjectType->
//                                               TypeInfo.GenericMapping,
//                                               UserMode,
//                                               &GrantedAccess,
//                                               &Status);
//                 if (Privileges)
//                 {
//                     /* Append and free the privileges */
//                     SeAppendPrivileges(AccessState, Privileges);
//                     SeFreePrivileges(Privileges);
//                 }
//
//                 /* Check if we got access */
//                 if (AccessGranted)
//                 {
//                     /* Update access state */
//                     AccessState->PreviouslyGrantedAccess |= GrantedAccess;
//                     AccessState->RemainingDesiredAccess &= ~(GrantedAccess |
//                                                              MAXIMUM_ALLOWED);
//                     OpenPacket->Override= TRUE;
//                 }
//
//                 FileString.Length = 8;
//                 FileString.MaximumLength = 8;
//                 FileString.Buffer = L"File";
//
//                 /* Do Audit/Alarm for open operation */
//                 SeOpenObjectAuditAlarm(&FileString,
//                                        OriginalDeviceObject,
//                                        CompleteName,
//                                        OriginalDeviceObject->SecurityDescriptor,
//                                        AccessState,
//                                        FALSE,
//                                        AccessGranted,
//                                        UserMode,
//                                        &AccessState->GenerateOnClose);
//             }
//             else
//             {
//                 /* Check if we need to do traverse validation */
//                 if (!(AccessState->Flags & TOKEN_HAS_TRAVERSE_PRIVILEGE) ||
//                     ((OriginalDeviceObject->DeviceType == FILE_DEVICE_DISK) ||
//                      (OriginalDeviceObject->DeviceType == FILE_DEVICE_CD_ROM)))
//                 {
//                     /* Check if this is a restricted token */
//                     if (!(AccessState->Flags & TOKEN_IS_RESTRICTED))
//                     {
//                         /* Do the FAST traverse check */
//                         AccessGranted = SeFastTraverseCheck(OriginalDeviceObject->SecurityDescriptor,
//                                                             AccessState,
//                                                             FILE_TRAVERSE,
//                                                             UserMode);
//                     }
//                     else
//                     {
//                         /* Fail */
//                         AccessGranted = FALSE;
//                     }
//
//                     /* Check if we failed to get access */
//                     if (!AccessGranted)
//                     {
//                         /* Lock the subject context */
//                         SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//                         LockHeld = TRUE;
//
//                         /* Do access check */
//                         AccessGranted = SeAccessCheck(OriginalDeviceObject->
//                                                       SecurityDescriptor,
//                                                       &AccessState->SubjectSecurityContext,
//                                                       LockHeld,
//                                                       FILE_TRAVERSE,
//                                                       0,
//                                                       &Privileges,
//                                                       &IoFileObjectType->
//                                                       TypeInfo.GenericMapping,
//                                                       UserMode,
//                                                       &GrantedAccess,
//                                                       &Status);
//                         if (Privileges)
//                         {
//                             /* Append and free the privileges */
//                             SeAppendPrivileges(AccessState, Privileges);
//                             SeFreePrivileges(Privileges);
//                         }
//                     }
//
//                     /* FIXME: Do Audit/Alarm for traverse check */
//                 }
//                 else
//                 {
//                     /* Access automatically granted */
//                     AccessGranted = TRUE;
//                 }
//             }
//
//             ExReleaseResourceLite(&IopSecurityResource);
//             KeLeaveCriticalRegion();
//
//             /* Check if we hold the lock */
//             if (LockHeld)
//             {
//                 /* Release it */
//                 SeUnlockSubjectContext(&AccessState->SubjectSecurityContext);
//             }
//
//             /* Check if access failed */
//             if (!AccessGranted)
//             {
//                 /* Dereference the device and fail */
//                 DPRINT1("Traverse access failed!\n");
//                 IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//                 return STATUS_ACCESS_DENIED;
//             }
//         }
//
//         /* Check if we can simply use a dummy file */
//         UseDummyFile = ((OpenPacket->QueryOnly) || (OpenPacket->DeleteOnly));
//
//         /* Check if this is a direct open */
//         if (!(RemainingName->Length) &&
//             !(OpenPacket->RelatedFileObject) &&
//             ((DesiredAccess & ~(SYNCHRONIZE |
//                                 FILE_READ_ATTRIBUTES |
//                                 READ_CONTROL |
//                                 ACCESS_SYSTEM_SECURITY |
//                                 WRITE_OWNER |
//                                 WRITE_DAC)) == 0) &&
//             !(UseDummyFile))
//         {
//             /* Remember this for later */
//             DirectOpen = TRUE;
//         }
//
//         /* Check if we have a related FO that wasn't a direct open */
//         if ((OpenPacket->RelatedFileObject) &&
//             !(OpenPacket->RelatedFileObject->Flags & FO_DIRECT_DEVICE_OPEN))
//         {
//             /* The device object is the one we were given */
//             DeviceObject = ParseObject;
//
//             /* Check if the related FO had a VPB */
//             if (OpenPacket->RelatedFileObject->Vpb)
//             {
//                 /* Yes, remember it */
//                 Vpb = OpenPacket->RelatedFileObject->Vpb;
//
//                 /* Reference it */
//                 InterlockedIncrement((PLONG)&Vpb->ReferenceCount);
//
//                 /* Check if we were given a specific top level device to use */
//                 if (OpenPacket->InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT)
//                 {
//                     DeviceObject = Vpb->DeviceObject;
//                 }
//             }
//         }
//         else
//         {
//             /* Check if it has a VPB */
//             if ((OriginalDeviceObject->Vpb) && !(DirectOpen))
//             {
//                 /* Check if the VPB is mounted, and mount it */
//                 Vpb = IopCheckVpbMounted(OpenPacket,
//                                          OriginalDeviceObject,
//                                          RemainingName,
//                                          &Status);
//                 if (!Vpb) return Status;
//
//                 /* Get the VPB's device object */
//                 DeviceObject = Vpb->DeviceObject;
//             }
//             else
//             {
//                 /* The device object is the one we were given */
//                 DeviceObject = OriginalDeviceObject;
//             }
//
//             /* If we weren't given a specific top level device, look for an attached device */
//             if (!(OpenPacket->InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT) &&
//                 DeviceObject->AttachedDevice)
//             {
//                 /* Get the attached device */
//                 DeviceObject = IoGetAttachedDevice(DeviceObject);
//             }
//         }
//
//         /* If we have a top level device hint, verify it */
//         if (OpenPacket->InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT)
//         {
//             Status = IopCheckTopDeviceHint(&DeviceObject, OpenPacket, DirectOpen);
//             if (!NT_SUCCESS(Status))
//             {
//                 IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//                 if (Vpb) IopDereferenceVpbAndFree(Vpb);
//                 return Status;
//             }
//         }
//
//         /* If we traversed a mount point, reset the information */
//         if (OpenPacket->TraversedMountPoint)
//         {
//             OpenPacket->TraversedMountPoint = FALSE;
//         }
//
//         /* Check if this is a secure FSD */
//         if ((DeviceObject->Characteristics & FILE_DEVICE_SECURE_OPEN) &&
//             ((OpenPacket->RelatedFileObject) || (RemainingName->Length)) &&
//             (!VolumeOpen))
//         {
//             Privileges = NULL;
//             GrantedAccess = 0;
//
//             KeEnterCriticalRegion();
//             ExAcquireResourceSharedLite(&IopSecurityResource, TRUE);
//
//             /* Lock the subject context */
//             SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//
//             /* Do access check */
//             AccessGranted = SeAccessCheck(OriginalDeviceObject->SecurityDescriptor,
//                                           &AccessState->SubjectSecurityContext,
//                                           TRUE,
//                                           DesiredAccess,
//                                           0,
//                                           &Privileges,
//                                           &IoFileObjectType->TypeInfo.GenericMapping,
//                                           UserMode,
//                                           &GrantedAccess,
//                                           &Status);
//             if (Privileges != NULL)
//             {
//                 /* Append and free the privileges */
//                 SeAppendPrivileges(AccessState, Privileges);
//                 SeFreePrivileges(Privileges);
//             }
//
//             /* Check if we got access */
//             if (GrantedAccess)
//             {
//                 AccessState->PreviouslyGrantedAccess |= GrantedAccess;
//                 AccessState->RemainingDesiredAccess &= ~(GrantedAccess | MAXIMUM_ALLOWED);
//             }
//
//             FileString.Length = 8;
//             FileString.MaximumLength = 8;
//             FileString.Buffer = L"File";
//
//             /* Do Audit/Alarm for open operation
//              * NOTA: we audit target device object
//              */
//             SeOpenObjectAuditAlarm(&FileString,
//                                    DeviceObject,
//                                    CompleteName,
//                                    OriginalDeviceObject->SecurityDescriptor,
//                                    AccessState,
//                                    FALSE,
//                                    AccessGranted,
//                                    UserMode,
//                                    &AccessState->GenerateOnClose);
//
//             SeUnlockSubjectContext(&AccessState->SubjectSecurityContext);
//
//             ExReleaseResourceLite(&IopSecurityResource);
//             KeLeaveCriticalRegion();
//
//             /* Check if access failed */
//             if (!AccessGranted)
//             {
//                 /* Dereference the device and fail */
//                 IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//                 if (Vpb) IopDereferenceVpbAndFree(Vpb);
//                 return STATUS_ACCESS_DENIED;
//             }
//         }
//
//         /* Allocate the IRP */
//         Irp = IoAllocateIrp(DeviceObject->StackSize, TRUE);
//         if (!Irp)
//         {
//             /* Dereference the device and VPB, then fail */
//             IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//             if (Vpb) IopDereferenceVpbAndFree(Vpb);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Now set the IRP data */
//         Irp->RequestorMode = AccessMode;
//         Irp->Flags = IRP_CREATE_OPERATION | IRP_SYNCHRONOUS_API | IRP_DEFER_IO_COMPLETION;
//         Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//         Irp->UserIosb = &IoStatusBlock;
//         Irp->MdlAddress = NULL;
//         Irp->PendingReturned = FALSE;
//         Irp->UserEvent = NULL;
//         Irp->Cancel = FALSE;
//         Irp->CancelRoutine = NULL;
//         Irp->Tail.Overlay.AuxiliaryBuffer = NULL;
//
//         /* Setup the security context */
//         SecurityContext.SecurityQos = SecurityQos;
//         SecurityContext.AccessState = AccessState;
//         SecurityContext.DesiredAccess = AccessState->RemainingDesiredAccess;
//         SecurityContext.FullCreateOptions = OpenPacket->CreateOptions;
//
//         /* Get the I/O Stack location */
//         StackLoc = IoGetNextIrpStackLocation(Irp);
//         StackLoc->Control = 0;
//
//         /* Check what kind of file this is */
//         switch (OpenPacket->CreateFileType)
//         {
//             /* Normal file */
//             case CreateFileTypeNone:
//
//                 /* Set the major function and EA Length */
//                 StackLoc->MajorFunction = IRP_MJ_CREATE;
//                 StackLoc->Parameters.Create.EaLength = OpenPacket->EaLength;
//
//                 /* Set the flags */
//                 StackLoc->Flags = (UCHAR)OpenPacket->Options;
//                 StackLoc->Flags |= !(Attributes & OBJ_CASE_INSENSITIVE) ? SL_CASE_SENSITIVE: 0;
//                 break;
//
//             /* Named pipe */
//             case CreateFileTypeNamedPipe:
//
//                 /* Set the named pipe MJ and set the parameters */
//                 StackLoc->MajorFunction = IRP_MJ_CREATE_NAMED_PIPE;
//                 StackLoc->Parameters.CreatePipe.Parameters = OpenPacket->ExtraCreateParameters;
//                 break;
//
//             /* Mailslot */
//             case CreateFileTypeMailslot:
//
//                 /* Set the mailslot MJ and set the parameters */
//                 StackLoc->MajorFunction = IRP_MJ_CREATE_MAILSLOT;
//                 StackLoc->Parameters.CreateMailslot.Parameters = OpenPacket->ExtraCreateParameters;
//                 break;
//         }
//
//         /* Set the common data */
//         Irp->Overlay.AllocationSize = OpenPacket->AllocationSize;
//         Irp->AssociatedIrp.SystemBuffer = OpenPacket->EaBuffer;
//         StackLoc->Parameters.Create.Options = (OpenPacket->Disposition << 24) |
//                                               (OpenPacket->CreateOptions &
//                                                0xFFFFFF);
//         StackLoc->Parameters.Create.FileAttributes = OpenPacket->FileAttributes;
//         StackLoc->Parameters.Create.ShareAccess = OpenPacket->ShareAccess;
//         StackLoc->Parameters.Create.SecurityContext = &SecurityContext;
//
//         /* Check if we really need to create an object */
//         if (!UseDummyFile)
//         {
//             ULONG ObjectSize = sizeof(FILE_OBJECT);
//
//             /* Tag on space for a file object extension */
//             if (OpenPacket->InternalFlags & IOP_CREATE_FILE_OBJECT_EXTENSION)
//                 ObjectSize += sizeof(FILE_OBJECT_EXTENSION);
//
//             /* Create the actual file object */
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        NULL,
//                                        Attributes,
//                                        NULL,
//                                        NULL);
//             Status = ObCreateObject(KernelMode,
//                                     IoFileObjectType,
//                                     &ObjectAttributes,
//                                     AccessMode,
//                                     NULL,
//                                     ObjectSize,
//                                     0,
//                                     0,
//                                     (PVOID*)&FileObject);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Create failed, free the IRP */
//                 IoFreeIrp(Irp);
//
//                 /* Dereference the device and VPB */
//                 IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//                 if (Vpb) IopDereferenceVpbAndFree(Vpb);
//
//                 /* We failed, return status */
//                 OpenPacket->FinalStatus = Status;
//                 return Status;
//             }
//
//             /* Clear the file object */
//             RtlZeroMemory(FileObject, ObjectSize);
//
//             /* Check if this is Synch I/O */
//             if (OpenPacket->CreateOptions &
//                 (FILE_SYNCHRONOUS_IO_ALERT | FILE_SYNCHRONOUS_IO_NONALERT))
//             {
//                 /* Set the synch flag */
//                 FileObject->Flags |= FO_SYNCHRONOUS_IO;
//
//                 /* Check if it's also alertable */
//                 if (OpenPacket->CreateOptions & FILE_SYNCHRONOUS_IO_ALERT)
//                 {
//                     /* It is, set the alertable flag */
//                     FileObject->Flags |= FO_ALERTABLE_IO;
//                 }
//             }
//
//             /* Check if this is synch I/O */
//             if (FileObject->Flags & FO_SYNCHRONOUS_IO)
//             {
//                 /* Initialize the event */
//                 KeInitializeEvent(&FileObject->Lock, SynchronizationEvent, FALSE);
//             }
//
//             /* Check if the caller requested no intermediate buffering */
//             if (OpenPacket->CreateOptions & FILE_NO_INTERMEDIATE_BUFFERING)
//             {
//                 /* Set the correct flag for the FSD to read */
//                 FileObject->Flags |= FO_NO_INTERMEDIATE_BUFFERING;
//             }
//
//             /* Check if the caller requested write through support */
//             if (OpenPacket->CreateOptions & FILE_WRITE_THROUGH)
//             {
//                 /* Set the correct flag for the FSD to read */
//                 FileObject->Flags |= FO_WRITE_THROUGH;
//             }
//
//             /* Check if the caller says the file will be only read sequentially */
//             if (OpenPacket->CreateOptions & FILE_SEQUENTIAL_ONLY)
//             {
//                 /* Set the correct flag for the FSD to read */
//                 FileObject->Flags |= FO_SEQUENTIAL_ONLY;
//             }
//
//             /* Check if the caller believes the file will be only read randomly */
//             if (OpenPacket->CreateOptions & FILE_RANDOM_ACCESS)
//             {
//                 /* Set the correct flag for the FSD to read */
//                 FileObject->Flags |= FO_RANDOM_ACCESS;
//             }
//
//             /* Check if we were asked to setup a file object extension */
//             if (OpenPacket->InternalFlags & IOP_CREATE_FILE_OBJECT_EXTENSION)
//             {
//                 PFILE_OBJECT_EXTENSION FileObjectExtension;
//
//                 /* Make sure the file object knows it has an extension */
//                 FileObject->Flags |= FO_FILE_OBJECT_HAS_EXTENSION;
//
//                 /* Initialize file object extension */
//                 FileObjectExtension = (PFILE_OBJECT_EXTENSION)(FileObject + 1);
//                 FileObject->FileObjectExtension = FileObjectExtension;
//
//                 /* Add the top level device which we'll send the request to */
//                 if (OpenPacket->InternalFlags & IOP_USE_TOP_LEVEL_DEVICE_HINT)
//                 {
//                     FileObjectExtension->TopDeviceObjectHint = DeviceObject;
//                 }
//             }
//         }
//         else
//         {
//             /* Use the dummy object instead */
//             LocalFileObject = OpenPacket->LocalFileObject;
//             RtlZeroMemory(LocalFileObject, sizeof(DUMMY_FILE_OBJECT));
//
//             /* Set it up */
//             FileObject = (PFILE_OBJECT)&LocalFileObject->ObjectHeader.Body;
//             LocalFileObject->ObjectHeader.Type = IoFileObjectType;
//             LocalFileObject->ObjectHeader.PointerCount = 1;
//         }
//
//         /* Setup the file header */
//         FileObject->Type = IO_TYPE_FILE;
//         FileObject->Size = sizeof(FILE_OBJECT);
//         FileObject->RelatedFileObject = OpenPacket->RelatedFileObject;
//         FileObject->DeviceObject = OriginalDeviceObject;
//
//         /* Check if this is a direct device open */
//         if (DirectOpen) FileObject->Flags |= FO_DIRECT_DEVICE_OPEN;
//
//         /* Check if the caller wants case sensitivity */
//         if (!(Attributes & OBJ_CASE_INSENSITIVE))
//         {
//             /* Tell the driver about it */
//             FileObject->Flags |= FO_OPENED_CASE_SENSITIVE;
//         }
//
//         /* Now set the file object */
//         Irp->Tail.Overlay.OriginalFileObject = FileObject;
//         StackLoc->FileObject = FileObject;
//
//         /* Check if the file object has a name */
//         if (RemainingName->Length)
//         {
//             /* Setup the unicode string */
//             FileObject->FileName.MaximumLength = RemainingName->Length + sizeof(WCHAR);
//             FileObject->FileName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                                 FileObject->FileName.MaximumLength,
//                                                                 TAG_IO_NAME);
//             if (!FileObject->FileName.Buffer)
//             {
//                 /* Failed to allocate the name, free the IRP */
//                 IoFreeIrp(Irp);
//
//                 /* Dereference the device object and VPB */
//                 IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//                 if (Vpb) IopDereferenceVpbAndFree(Vpb);
//
//                 /* Clear the FO and dereference it */
//                 FileObject->DeviceObject = NULL;
//                 if (!UseDummyFile) ObDereferenceObject(FileObject);
//
//                 /* Fail */
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//         }
//
//         /* Copy the name */
//         RtlCopyUnicodeString(&FileObject->FileName, RemainingName);
//
//         /* Initialize the File Object event and set the FO */
//         KeInitializeEvent(&FileObject->Event, NotificationEvent, FALSE);
//         OpenPacket->FileObject = FileObject;
//
//         /* Queue the IRP and call the driver */
//         IopQueueIrpToThread(Irp);
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for the driver to complete the create */
//             KeWaitForSingleObject(&FileObject->Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//
//             /* Get the new status */
//             Status = IoStatusBlock.Status;
//         }
//         else
//         {
//             /* We'll have to complete it ourselves */
//             ASSERT(!Irp->PendingReturned);
//             ASSERT(!Irp->MdlAddress);
//
//             /* Handle name change if required */
//             if (Status == STATUS_REPARSE)
//             {
//                 /* Check this is a mount point */
//                 if (Irp->IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT)
//                 {
//                     PREPARSE_DATA_BUFFER ReparseData;
//
//                     /* Reparse point attributes were passed by the driver in the auxiliary buffer */
//                     ASSERT(Irp->Tail.Overlay.AuxiliaryBuffer != NULL);
//                     ReparseData = (PREPARSE_DATA_BUFFER)Irp->Tail.Overlay.AuxiliaryBuffer;
//
//                     ASSERT(ReparseData->ReparseTag == IO_REPARSE_TAG_MOUNT_POINT);
//                     ASSERT(ReparseData->ReparseDataLength < MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//                     ASSERT(ReparseData->Reserved < MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//
//                     IopDoNameTransmogrify(Irp, FileObject, ReparseData);
//                 }
//             }
//
//             /* Completion happens at APC_LEVEL */
//             KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//             /* Get the new I/O Status block ourselves */
//             IoStatusBlock = Irp->IoStatus;
//             Status = IoStatusBlock.Status;
//
//             /* Manually signal the even, we can't have any waiters */
//             FileObject->Event.Header.SignalState = 1;
//
//             /* Now that we've signaled the events, de-associate the IRP */
//             IopUnQueueIrpFromThread(Irp);
//
//             /* Check if the IRP had an input buffer */
//             if ((Irp->Flags & IRP_BUFFERED_IO) &&
//                 (Irp->Flags & IRP_DEALLOCATE_BUFFER))
//             {
//                 /* Free it. A driver might've tacked one on */
//                 ExFreePool(Irp->AssociatedIrp.SystemBuffer);
//             }
//
//             /* Free the IRP and bring the IRQL back down */
//             IoFreeIrp(Irp);
//             KeLowerIrql(OldIrql);
//         }
//
//         /* Copy the I/O Status */
//         OpenPacket->Information = IoStatusBlock.Information;
//
//         /* The driver failed to create the file */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Check if we have a name and if so, free it */
//             if (FileObject->FileName.Length)
//             {
//                 /*
//                  * Don't use TAG_IO_NAME since the FileObject's FileName
//                  * may have been re-allocated using a different tag
//                  * by a filesystem.
//                  */
//                 ExFreePoolWithTag(FileObject->FileName.Buffer, 0);
//                 FileObject->FileName.Buffer = NULL;
//                 FileObject->FileName.Length = 0;
//             }
//
//             /* Clear its device object */
//             FileObject->DeviceObject = NULL;
//
//             /* Save this now because the FO might go away */
//             OpenCancelled = FileObject->Flags & FO_FILE_OPEN_CANCELLED ?
//                             TRUE : FALSE;
//
//             /* Clear the file object in the open packet */
//             OpenPacket->FileObject = NULL;
//
//             /* Dereference the file object */
//             if (!UseDummyFile) ObDereferenceObject(FileObject);
//
//             /* Dereference the device object */
//             IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//
//             /* Unless the driver cancelled the open, dereference the VPB */
//             if (!(OpenCancelled) && (Vpb)) IopDereferenceVpbAndFree(Vpb);
//
//             /* Set the status and return */
//             OpenPacket->FinalStatus = Status;
//             return Status;
//         }
//         else if (Status == STATUS_REPARSE)
//         {
//             if (OpenPacket->Information == IO_REPARSE ||
//                 OpenPacket->Information == IO_REPARSE_TAG_MOUNT_POINT)
//             {
//                 /* Update CompleteName with reparse info which got updated in IopDoNameTransmogrify() */
//                 if (CompleteName->MaximumLength < FileObject->FileName.Length)
//                 {
//                     PWSTR NewCompleteName;
//
//                     /* Allocate a new buffer for the string */
//                     NewCompleteName = ExAllocatePoolWithTag(PagedPool, FileObject->FileName.Length, TAG_IO_NAME);
//                     if (NewCompleteName == NULL)
//                     {
//                         OpenPacket->FinalStatus = STATUS_INSUFFICIENT_RESOURCES;
//                         return STATUS_INSUFFICIENT_RESOURCES;
//                     }
//
//                     /* Release the old one */
//                     if (CompleteName->Buffer != NULL)
//                     {
//                         /*
//                          * Don't use TAG_IO_NAME since the FileObject's FileName
//                          * may have been re-allocated using a different tag
//                          * by a filesystem.
//                          */
//                         ExFreePoolWithTag(CompleteName->Buffer, 0);
//                     }
//
//                     /* And setup the new one */
//                     CompleteName->Buffer = NewCompleteName;
//                     CompleteName->MaximumLength = FileObject->FileName.Length;
//                 }
//
//                 /* Copy our new complete name */
//                 RtlCopyUnicodeString(CompleteName, &FileObject->FileName);
//
//                 if (OpenPacket->Information == IO_REPARSE_TAG_MOUNT_POINT)
//                 {
//                     OpenPacket->RelatedFileObject = NULL;
//                 }
//             }
//
//             /* Check if we have a name and if so, free it */
//             if (FileObject->FileName.Length)
//             {
//                 /*
//                  * Don't use TAG_IO_NAME since the FileObject's FileName
//                  * may have been re-allocated using a different tag
//                  * by a filesystem.
//                  */
//                 ExFreePoolWithTag(FileObject->FileName.Buffer, 0);
//                 FileObject->FileName.Buffer = NULL;
//                 FileObject->FileName.Length = 0;
//             }
//
//             /* Clear its device object */
//             FileObject->DeviceObject = NULL;
//
//             /* Clear the file object in the open packet */
//             OpenPacket->FileObject = NULL;
//
//             /* Dereference the file object */
//             if (!UseDummyFile) ObDereferenceObject(FileObject);
//
//             /* Dereference the device object */
//             IopDereferenceDeviceObject(OriginalDeviceObject, FALSE);
//
//             /* Unless the driver cancelled the open, dereference the VPB */
//             if (Vpb != NULL) IopDereferenceVpbAndFree(Vpb);
//
//             if (OpenPacket->Information != IO_REMOUNT)
//             {
//                 OpenPacket->RelatedFileObject = NULL;
//
//                 /* Inform we traversed a mount point for later attempt */
//                 if (OpenPacket->Information == IO_REPARSE_TAG_MOUNT_POINT)
//                 {
//                     OpenPacket->TraversedMountPoint = 1;
//                 }
//
//                 /* In case we override checks, but got this on volume open, fail hard */
//                 if (OpenPacket->Override)
//                 {
//                     KeBugCheckEx(DRIVER_RETURNED_STATUS_REPARSE_FOR_VOLUME_OPEN,
//                                  (ULONG_PTR)OriginalDeviceObject,
//                                  (ULONG_PTR)DeviceObject,
//                                  (ULONG_PTR)CompleteName,
//                                  OpenPacket->Information);
//                 }
//
//                 /* Return to IO/OB so that information can be upgraded */
//                 return STATUS_REPARSE;
//             }
//
//             /* Loop again and reattempt an opening */
//             continue;
//         }
//
//         break;
//     }
//
//     if (Attempt == IOP_MAX_REPARSE_TRAVERSAL)
//         return STATUS_UNSUCCESSFUL;
//
//     /* Get the owner of the File Object */
//     OwnerDevice = IoGetRelatedDeviceObject(FileObject);
//
//     /*
//      * It's possible that the device to whom we sent the IRP to
//      * isn't actually the device that ended opening the file object
//      * internally.
//      */
//     if (OwnerDevice != DeviceObject)
//     {
//         /* We have to de-reference the VPB we had associated */
//         if (Vpb) IopDereferenceVpbAndFree(Vpb);
//
//         /* And re-associate with the actual one */
//         Vpb = FileObject->Vpb;
//         if (Vpb) InterlockedIncrement((PLONG)&Vpb->ReferenceCount);
//     }
//
//     /* Make sure we are not using a dummy */
//     if (!UseDummyFile)
//     {
//         /* Check if this was a volume open */
//         if ((!(FileObject->RelatedFileObject) ||
//               (FileObject->RelatedFileObject->Flags & FO_VOLUME_OPEN)) &&
//             !(FileObject->FileName.Length))
//         {
//             /* All signs point to it, but make sure it was actually an FSD */
//             if ((OwnerDevice->DeviceType == FILE_DEVICE_DISK_FILE_SYSTEM) ||
//                 (OwnerDevice->DeviceType == FILE_DEVICE_CD_ROM_FILE_SYSTEM) ||
//                 (OwnerDevice->DeviceType == FILE_DEVICE_TAPE_FILE_SYSTEM) ||
//                 (OwnerDevice->DeviceType == FILE_DEVICE_FILE_SYSTEM))
//             {
//                 /* The owner device is an FSD, so this is a volume open for real */
//                 FileObject->Flags |= FO_VOLUME_OPEN;
//             }
//         }
//
//         /* Reference the object and set the parse check */
//         ObReferenceObject(FileObject);
//         *Object = FileObject;
//         OpenPacket->FinalStatus = IoStatusBlock.Status;
//         OpenPacket->ParseCheck = TRUE;
//         return OpenPacket->FinalStatus;
//     }
//     else
//     {
//         /* Check if this was a query */
//         if (OpenPacket->QueryOnly)
//         {
//             /* Check if the caller wants basic info only */
//             if (!OpenPacket->FullAttributes)
//             {
//                 /* Allocate the buffer */
//                 FileBasicInfo = ExAllocatePoolWithTag(NonPagedPool,
//                                                       sizeof(*FileBasicInfo),
//                                                       TAG_IO);
//                 if (FileBasicInfo)
//                 {
//                     /* Do the query */
//                     Status = IoQueryFileInformation(FileObject,
//                                                     FileBasicInformation,
//                                                     sizeof(*FileBasicInfo),
//                                                     FileBasicInfo,
//                                                     &ReturnLength);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* Copy the data */
//                         RtlCopyMemory(OpenPacket->BasicInformation,
//                                       FileBasicInfo,
//                                       ReturnLength);
//                     }
//
//                     /* Free our buffer */
//                     ExFreePoolWithTag(FileBasicInfo, TAG_IO);
//                 }
//                 else
//                 {
//                     /* Fail */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//             }
//             else
//             {
//                 /* This is a full query */
//                 Status = IoQueryFileInformation(
//                     FileObject,
//                     FileNetworkOpenInformation,
//                     sizeof(FILE_NETWORK_OPEN_INFORMATION),
//                     OpenPacket->NetworkInformation,
//                     &ReturnLength);
//                 if (!NT_SUCCESS(Status)) ASSERT(Status != STATUS_NOT_IMPLEMENTED);
//             }
//         }
//
//         /* Delete the file object */
//         IopDeleteFile(FileObject);
//
//         /* Clear out the file */
//         OpenPacket->FileObject = NULL;
//
//         /* Set and return status */
//         OpenPacket->FinalStatus = Status;
//         OpenPacket->ParseCheck = TRUE;
//         return Status;
//     }
// }
//
// NTSTATUS
// NTAPI
// IopParseFile(IN PVOID ParseObject,
//              IN PVOID ObjectType,
//              IN OUT PACCESS_STATE AccessState,
//              IN KPROCESSOR_MODE AccessMode,
//              IN ULONG Attributes,
//              IN OUT PUNICODE_STRING CompleteName,
//              IN OUT PUNICODE_STRING RemainingName,
//              IN OUT PVOID Context OPTIONAL,
//              IN PSECURITY_QUALITY_OF_SERVICE SecurityQos OPTIONAL,
//              OUT PVOID *Object)
// {
//     PVOID DeviceObject;
//     POPEN_PACKET OpenPacket = (POPEN_PACKET)Context;
//
//     /* Validate the open packet */
//     if (!IopValidateOpenPacket(OpenPacket)) return STATUS_OBJECT_TYPE_MISMATCH;
//
//     /* Get the device object */
//     DeviceObject = IoGetRelatedDeviceObject(ParseObject);
//     OpenPacket->RelatedFileObject = ParseObject;
//
//     /* Call the main routine */
//     return IopParseDevice(DeviceObject,
//                           ObjectType,
//                           AccessState,
//                           AccessMode,
//                           Attributes,
//                           CompleteName,
//                           RemainingName,
//                           OpenPacket,
//                           SecurityQos,
//                           Object);
// }
//
// VOID
// NTAPI
// IopDeleteFile(IN PVOID ObjectBody)
// {
//     PFILE_OBJECT FileObject = (PFILE_OBJECT)ObjectBody;
//     PIRP Irp;
//     PIO_STACK_LOCATION StackPtr;
//     NTSTATUS Status;
//     KEVENT Event;
//     PDEVICE_OBJECT DeviceObject;
//     BOOLEAN DereferenceDone = FALSE;
//     PVPB Vpb;
//     KIRQL OldIrql;
//     IOTRACE(IO_FILE_DEBUG, "ObjectBody: %p\n", ObjectBody);
//
//     /* Check if the file has a device object */
//     if (FileObject->DeviceObject)
//     {
//         /* Check if this is a direct open or not */
//         if (FileObject->Flags & FO_DIRECT_DEVICE_OPEN)
//         {
//             /* Get the attached device */
//             DeviceObject = IoGetAttachedDevice(FileObject->DeviceObject);
//         }
//         else
//         {
//             /* Use the file object's device object */
//             DeviceObject = IoGetRelatedDeviceObject(FileObject);
//         }
//
//         /* Sanity check */
//         ASSERT(!(FileObject->Flags & FO_SYNCHRONOUS_IO) ||
//                (InterlockedExchange((PLONG)&FileObject->Busy, TRUE) == FALSE));
//
//         /* Check if the handle wasn't created yet */
//         if (!(FileObject->Flags & FO_HANDLE_CREATED))
//         {
//             /* Send the cleanup IRP */
//             IopCloseFile(NULL, ObjectBody, 0, 1, 1);
//         }
//
//         /* Clear and set up Events */
//         KeClearEvent(&FileObject->Event);
//         KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//         /* Allocate an IRP */
//         Irp = IopAllocateIrpMustSucceed(DeviceObject->StackSize);
//
//         /* Set it up */
//         Irp->UserEvent = &Event;
//         Irp->UserIosb = &Irp->IoStatus;
//         Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//         Irp->Tail.Overlay.OriginalFileObject = FileObject;
//         Irp->Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;
//
//         /* Set up Stack Pointer Data */
//         StackPtr = IoGetNextIrpStackLocation(Irp);
//         StackPtr->MajorFunction = IRP_MJ_CLOSE;
//         StackPtr->FileObject = FileObject;
//
//         /* Queue the IRP */
//         IopQueueIrpToThread(Irp);
//
//         /* Get the VPB and check if this isn't a direct open */
//         Vpb = FileObject->Vpb;
//         if ((Vpb) && !(FileObject->Flags & FO_DIRECT_DEVICE_OPEN))
//         {
//             /* Dereference the VPB before the close */
//             InterlockedDecrement((PLONG)&Vpb->ReferenceCount);
//         }
//
//         /* Check if the FS will never disappear by itself */
//         if (FileObject->DeviceObject->Flags & DO_NEVER_LAST_DEVICE)
//         {
//             /* Dereference it */
//             InterlockedDecrement(&FileObject->DeviceObject->ReferenceCount);
//             DereferenceDone = TRUE;
//         }
//
//         /* Call the FS Driver */
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         }
//
//         /* De-queue the IRP */
//         KeRaiseIrql(APC_LEVEL, &OldIrql);
//         IopUnQueueIrpFromThread(Irp);
//         KeLowerIrql(OldIrql);
//
//         /* Free the IRP */
//         IoFreeIrp(Irp);
//
//         /* Clear the file name */
//         if (FileObject->FileName.Buffer)
//         {
//             /*
//              * Don't use TAG_IO_NAME since the FileObject's FileName
//              * may have been re-allocated using a different tag
//              * by a filesystem.
//              */
//             ExFreePoolWithTag(FileObject->FileName.Buffer, 0);
//             FileObject->FileName.Buffer = NULL;
//         }
//
//         /* Check if the FO had a completion port */
//         if (FileObject->CompletionContext)
//         {
//             /* Free it */
//             ObDereferenceObject(FileObject->CompletionContext->Port);
//             ExFreePool(FileObject->CompletionContext);
//         }
//
//         /* Check if the FO had extension */
//         if (FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION)
//         {
//             /* Release filter context structure if any */
//             FsRtlPTeardownPerFileObjectContexts(FileObject);
//         }
//
//         /* Check if dereference has been done yet */
//         if (!DereferenceDone)
//         {
//             /* Dereference device object */
//             IopDereferenceDeviceObject(FileObject->DeviceObject, FALSE);
//         }
//     }
// }
//
// PDEVICE_OBJECT
// NTAPI
// IopGetDeviceAttachmentBase(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_OBJECT PDO = DeviceObject;
//
//     /* Go down the stack to attempt to get the PDO */
//     for (; ((PEXTENDED_DEVOBJ_EXTENSION)PDO->DeviceObjectExtension)->AttachedTo != NULL;
//            PDO = ((PEXTENDED_DEVOBJ_EXTENSION)PDO->DeviceObjectExtension)->AttachedTo);
//
//     return PDO;
// }
//
// PDEVICE_OBJECT
// NTAPI
// IopGetDevicePDO(IN PDEVICE_OBJECT DeviceObject)
// {
//     KIRQL OldIrql;
//     PDEVICE_OBJECT PDO;
//
//     ASSERT(DeviceObject != NULL);
//
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock as _);
//     /* Get the base DO */
//     PDO = IopGetDeviceAttachmentBase(DeviceObject);
//     /* Check whether that's really a PDO and if so, keep it */
//     if ((PDO->Flags & DO_BUS_ENUMERATED_DEVICE) != DO_BUS_ENUMERATED_DEVICE)
//     {
//         PDO = NULL;
//     }
//     else
//     {
//         ObReferenceObject(PDO);
//     }
//     KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock as _, OldIrql);
//
//     return PDO;
// }
//
// NTSTATUS
// NTAPI
// IopSetDeviceSecurityDescriptor(IN PDEVICE_OBJECT DeviceObject,
//                                IN PSECURITY_INFORMATION SecurityInformation,
//                                IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                IN POOL_TYPE PoolType,
//                                IN PGENERIC_MAPPING GenericMapping)
// {
//     NTSTATUS Status;
//     PSECURITY_DESCRIPTOR OldSecurityDescriptor, CachedSecurityDescriptor, NewSecurityDescriptor;
//
//     PAGED_CODE();
//
//     /* Keep attempting till we find our old SD or fail */
//     while (TRUE)
//     {
//         KeEnterCriticalRegion();
//         ExAcquireResourceSharedLite(&IopSecurityResource, TRUE);
//
//         /* Get our old SD and reference it */
//         OldSecurityDescriptor = DeviceObject->SecurityDescriptor;
//         if (OldSecurityDescriptor != NULL)
//         {
//             ObReferenceSecurityDescriptor(OldSecurityDescriptor, 1);
//         }
//
//         ExReleaseResourceLite(&IopSecurityResource);
//         KeLeaveCriticalRegion();
//
//         /* Set the SD information */
//         NewSecurityDescriptor = OldSecurityDescriptor;
//         Status = SeSetSecurityDescriptorInfo(NULL, SecurityInformation,
//                                              SecurityDescriptor, &NewSecurityDescriptor,
//                                              PoolType, GenericMapping);
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (OldSecurityDescriptor != NULL)
//             {
//                 ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
//             }
//
//             break;
//         }
//
//         /* Add the new DS to the internal cache */
//         Status = ObLogSecurityDescriptor(NewSecurityDescriptor,
//                                          &CachedSecurityDescriptor, 1);
//         ExFreePool(NewSecurityDescriptor);
//         if (!NT_SUCCESS(Status))
//         {
//             ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
//             break;
//         }
//
//         KeEnterCriticalRegion();
//         ExAcquireResourceExclusiveLite(&IopSecurityResource, TRUE);
//         /* Check if someone changed it in our back */
//         if (DeviceObject->SecurityDescriptor == OldSecurityDescriptor)
//         {
//             /* We're clear, do the swap */
//             DeviceObject->SecurityDescriptor = CachedSecurityDescriptor;
//             ExReleaseResourceLite(&IopSecurityResource);
//             KeLeaveCriticalRegion();
//
//             /* And dereference old SD (twice - us + not in use) */
//             ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 2);
//
//             break;
//         }
//         ExReleaseResourceLite(&IopSecurityResource);
//         KeLeaveCriticalRegion();
//
//         /* If so, try again */
//         ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
//         ObDereferenceSecurityDescriptor(CachedSecurityDescriptor, 1);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopSetDeviceSecurityDescriptors(IN PDEVICE_OBJECT UpperDeviceObject,
//                                 IN PDEVICE_OBJECT PhysicalDeviceObject,
//                                 IN PSECURITY_INFORMATION SecurityInformation,
//                                 IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                 IN POOL_TYPE PoolType,
//                                 IN PGENERIC_MAPPING GenericMapping)
// {
//     PDEVICE_OBJECT CurrentDO = PhysicalDeviceObject, NextDevice;
//     NTSTATUS Status = STATUS_SUCCESS, TmpStatus;
//
//     PAGED_CODE();
//
//     ASSERT(PhysicalDeviceObject != NULL);
//
//     /* We always reference the DO we're working on */
//     ObReferenceObject(CurrentDO);
//
//     /* Go up from PDO to latest DO */
//     do
//     {
//         /* Attempt to set the new SD on it */
//         TmpStatus = IopSetDeviceSecurityDescriptor(CurrentDO, SecurityInformation,
//                                                    SecurityDescriptor, PoolType,
//                                                    GenericMapping);
//         /* Was our last one? Remember that status then */
//         if (CurrentDO == UpperDeviceObject)
//         {
//             Status = TmpStatus;
//         }
//
//         /* Try to move to the next DO (and thus, reference it) */
//         NextDevice = CurrentDO->AttachedDevice;
//         if (NextDevice)
//         {
//             ObReferenceObject(NextDevice);
//         }
//
//         /* Dereference current DO and move to the next one */
//         ObDereferenceObject(CurrentDO);
//         CurrentDO = NextDevice;
//     }
//     while (CurrentDO != NULL);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopGetSetSecurityObject(IN PVOID ObjectBody,
//                         IN SECURITY_OPERATION_CODE OperationCode,
//                         IN PSECURITY_INFORMATION SecurityInformation,
//                         IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                         IN OUT PULONG BufferLength,
//                         IN OUT PSECURITY_DESCRIPTOR *OldSecurityDescriptor,
//                         IN POOL_TYPE PoolType,
//                         IN OUT PGENERIC_MAPPING GenericMapping)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION StackPtr;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     PIRP Irp;
//     BOOLEAN LocalEvent = FALSE;
//     KEVENT Event;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//     IOTRACE(IO_FILE_DEBUG, "ObjectBody: %p\n", ObjectBody);
//
//     /* Check if this is a device or file */
//     if (((PFILE_OBJECT)ObjectBody)->Type == IO_TYPE_DEVICE)
//     {
//         /* It's a device */
//         DeviceObject = (PDEVICE_OBJECT)ObjectBody;
//         FileObject = NULL;
//     }
//     else
//     {
//         /* It's a file */
//         FileObject = (PFILE_OBJECT)ObjectBody;
//
//         /* Check if this is a direct open */
//         if (FileObject->Flags & FO_DIRECT_DEVICE_OPEN)
//         {
//             /* Get the Device Object */
//             DeviceObject = IoGetAttachedDevice(FileObject->DeviceObject);
//         }
//         else
//         {
//             /* Otherwise, use the direct device*/
//             DeviceObject = FileObject->DeviceObject;
//         }
//     }
//
//     /* Check if the request was for a device object */
//     if (!(FileObject) ||
//         (!(FileObject->FileName.Length) && !(FileObject->RelatedFileObject)) ||
//         (FileObject->Flags & FO_DIRECT_DEVICE_OPEN))
//     {
//         /* Check what kind of request this was */
//         if (OperationCode == QuerySecurityDescriptor)
//         {
//             return SeQuerySecurityDescriptorInfo(SecurityInformation,
//                                                  SecurityDescriptor,
//                                                  BufferLength,
//                                                  &DeviceObject->SecurityDescriptor);
//         }
//         else if (OperationCode == DeleteSecurityDescriptor)
//         {
//             /* Simply return success */
//             return STATUS_SUCCESS;
//         }
//         else if (OperationCode == AssignSecurityDescriptor)
//         {
//             Status = STATUS_SUCCESS;
//
//             /* Make absolutely sure this is a device object */
//             if (!(FileObject) || !(FileObject->Flags & FO_STREAM_FILE))
//             {
//                 PSECURITY_DESCRIPTOR CachedSecurityDescriptor;
//
//                 /* Add the security descriptor in cache */
//                 Status = ObLogSecurityDescriptor(SecurityDescriptor, &CachedSecurityDescriptor, 1);
//                 if (NT_SUCCESS(Status))
//                 {
//                     KeEnterCriticalRegion();
//                     ExAcquireResourceExclusiveLite(&IopSecurityResource, TRUE);
//
//                     /* Assign the Security Descriptor */
//                     DeviceObject->SecurityDescriptor = CachedSecurityDescriptor;
//
//                     ExReleaseResourceLite(&IopSecurityResource);
//                     KeLeaveCriticalRegion();
//                 }
//             }
//
//             /* Return status */
//             return Status;
//         }
//         else if (OperationCode == SetSecurityDescriptor)
//         {
//             /* Get the Physical Device Object if any */
//             PDEVICE_OBJECT PDO = IopGetDevicePDO(DeviceObject);
//
//             if (PDO != NULL)
//             {
//                 /* Apply the new SD to any DO in the path from PDO to current DO */
//                 Status = IopSetDeviceSecurityDescriptors(DeviceObject, PDO,
//                                                          SecurityInformation,
//                                                          SecurityDescriptor,
//                                                          PoolType, GenericMapping);
//                 ObDereferenceObject(PDO);
//             }
//             else
//             {
//                 /* Otherwise, just set for ourselves */
//                 Status = IopSetDeviceSecurityDescriptor(DeviceObject,
//                                                         SecurityInformation,
//                                                         SecurityDescriptor,
//                                                         PoolType, GenericMapping);
//             }
//
//             return STATUS_SUCCESS;
//         }
//
//         /* Shouldn't happen */
//         return STATUS_SUCCESS;
//     }
//     else if (OperationCode == DeleteSecurityDescriptor)
//     {
//         /* Same as for devices, do nothing */
//         return STATUS_SUCCESS;
//     }
//
//     /* At this point, we know we're a file. Reference it */
//     ObReferenceObject(FileObject);
//
//     /* Check if we should use Sync IO or not */
//     if (FileObject->Flags & FO_SYNCHRONOUS_IO)
//     {
//         /* Lock the file object */
//         Status = IopLockFileObject(FileObject, ExGetPreviousMode());
//         if (Status != STATUS_SUCCESS)
//         {
//             ObDereferenceObject(FileObject);
//             return Status;
//         }
//     }
//     else
//     {
//         /* Use local event */
//         KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//         LocalEvent = TRUE;
//     }
//
//     /* Clear the File Object event */
//     KeClearEvent(&FileObject->Event);
//
//     /* Get the device object */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//
//     /* Allocate the IRP */
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//     if (!Irp) return IopCleanupFailedIrp(FileObject, NULL, NULL);
//
//     /* Set the IRP */
//     Irp->Tail.Overlay.OriginalFileObject = FileObject;
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//     Irp->RequestorMode = ExGetPreviousMode();
//     Irp->UserIosb = &IoStatusBlock;
//     Irp->UserEvent = (LocalEvent) ? &Event : NULL;
//     Irp->Flags = (LocalEvent) ? IRP_SYNCHRONOUS_API : 0;
//     Irp->Overlay.AsynchronousParameters.UserApcRoutine = NULL;
//
//     /* Set Stack Parameters */
//     StackPtr = IoGetNextIrpStackLocation(Irp);
//     StackPtr->FileObject = FileObject;
//
//     /* Check if this is a query or set */
//     if (OperationCode == QuerySecurityDescriptor)
//     {
//         /* Set the major function and parameters */
//         StackPtr->MajorFunction = IRP_MJ_QUERY_SECURITY;
//         StackPtr->Parameters.QuerySecurity.SecurityInformation =
//             *SecurityInformation;
//         StackPtr->Parameters.QuerySecurity.Length = *BufferLength;
//         Irp->UserBuffer = SecurityDescriptor;
//     }
//     else
//     {
//         /* Set the major function and parameters for a set */
//         StackPtr->MajorFunction = IRP_MJ_SET_SECURITY;
//         StackPtr->Parameters.SetSecurity.SecurityInformation =
//             *SecurityInformation;
//         StackPtr->Parameters.SetSecurity.SecurityDescriptor =
//             SecurityDescriptor;
//     }
//
//     /* Queue the IRP */
//     IopQueueIrpToThread(Irp);
//
//     /* Update operation counts */
//     IopUpdateOperationCount(IopOtherTransfer);
//
//     /* Call the Driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     /* Check if this was async I/O */
//     if (LocalEvent)
//     {
//         /* Check if the IRP is pending completion */
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait on the local event */
//             KeWaitForSingleObject(&Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//             Status = IoStatusBlock.Status;
//         }
//     }
//     else
//     {
//         /* Check if the IRP is pending completion */
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait on the file object */
//             KeWaitForSingleObject(&FileObject->Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//             Status = FileObject->FinalStatus;
//         }
//
//         /* Release the lock */
//         IopUnlockFileObject(FileObject);
//     }
//
//     /* This Driver doesn't implement Security, so try to give it a default */
//     if (Status == STATUS_INVALID_DEVICE_REQUEST)
//     {
//         /* Was this a query? */
//         if (OperationCode == QuerySecurityDescriptor)
//         {
//             /* Set a World Security Descriptor */
//             Status = SeSetWorldSecurityDescriptor(*SecurityInformation,
//                                                   SecurityDescriptor,
//                                                   BufferLength);
//         }
//         else
//         {
//             /* It wasn't a query, so just fake success */
//             Status = STATUS_SUCCESS;
//         }
//     }
//     else if (OperationCode == QuerySecurityDescriptor)
//     {
//         /* Callers usually expect the normalized form */
//         if (Status == STATUS_BUFFER_OVERFLOW) Status = STATUS_BUFFER_TOO_SMALL;
//
//         _SEH2_TRY
//         {
//             /* Return length */
//             *BufferLength = (ULONG)IoStatusBlock.Information;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopQueryName(IN PVOID ObjectBody,
//              IN BOOLEAN HasName,
//              OUT POBJECT_NAME_INFORMATION ObjectNameInfo,
//              IN ULONG Length,
//              OUT PULONG ReturnLength,
//              IN KPROCESSOR_MODE PreviousMode)
// {
//     return IopQueryNameInternal(ObjectBody,
//                                 HasName,
//                                 FALSE,
//                                 ObjectNameInfo,
//                                 Length,
//                                 ReturnLength,
//                                 PreviousMode);
// }
//
// NTSTATUS
// NTAPI
// IopQueryNameInternal(IN PVOID ObjectBody,
//                      IN BOOLEAN HasName,
//                      IN BOOLEAN QueryDosName,
//                      OUT POBJECT_NAME_INFORMATION ObjectNameInfo,
//                      IN ULONG Length,
//                      OUT PULONG ReturnLength,
//                      IN KPROCESSOR_MODE PreviousMode)
// {
//     POBJECT_NAME_INFORMATION LocalInfo;
//     PFILE_NAME_INFORMATION LocalFileInfo;
//     PFILE_OBJECT FileObject = (PFILE_OBJECT)ObjectBody;
//     ULONG LocalReturnLength, FileLength;
//     BOOLEAN LengthMismatch = FALSE;
//     NTSTATUS Status;
//     PWCHAR p;
//     PDEVICE_OBJECT DeviceObject;
//     BOOLEAN NoObCall;
//
//     IOTRACE(IO_FILE_DEBUG, "ObjectBody: %p\n", ObjectBody);
//
//     /* Validate length */
//     if (Length < sizeof(OBJECT_NAME_INFORMATION))
//     {
//         /* Wrong length, fail */
//         *ReturnLength = sizeof(OBJECT_NAME_INFORMATION);
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     /* Allocate Buffer */
//     LocalInfo = ExAllocatePoolWithTag(PagedPool, Length, TAG_IO);
//     if (!LocalInfo) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Query DOS name if the caller asked to */
//     NoObCall = FALSE;
//     if (QueryDosName)
//     {
//         DeviceObject = FileObject->DeviceObject;
//
//         /* In case of a network file system, don't call mountmgr */
//         if (DeviceObject->DeviceType == FILE_DEVICE_NETWORK_FILE_SYSTEM)
//         {
//             /* We'll store separator and terminator */
//             LocalReturnLength = sizeof(OBJECT_NAME_INFORMATION) + 2 * sizeof(WCHAR);
//             if (Length < LocalReturnLength)
//             {
//                 Status = STATUS_BUFFER_OVERFLOW;
//             }
//             else
//             {
//                 LocalInfo->Name.Length = sizeof(WCHAR);
//                 LocalInfo->Name.MaximumLength = sizeof(WCHAR);
//                 LocalInfo->Name.Buffer = (PVOID)((ULONG_PTR)LocalInfo + sizeof(OBJECT_NAME_INFORMATION));
//                 LocalInfo->Name.Buffer[0] = OBJ_NAME_PATH_SEPARATOR;
//                 Status = STATUS_SUCCESS;
//             }
//         }
//         /* Otherwise, call mountmgr to get DOS name */
//         else
//         {
//             Status = IoVolumeDeviceToDosName(DeviceObject, &LocalInfo->Name);
//             LocalReturnLength = LocalInfo->Name.Length + sizeof(OBJECT_NAME_INFORMATION) + sizeof(WCHAR);
//         }
//     }
//
//     /* Fall back if querying DOS name failed or if caller never wanted it ;-) */
//     if (!QueryDosName || !NT_SUCCESS(Status))
//     {
//         /* Query the name */
//         Status = ObQueryNameString(FileObject->DeviceObject,
//                                    LocalInfo,
//                                    Length,
//                                    &LocalReturnLength);
//     }
//     else
//     {
//         NoObCall = TRUE;
//     }
//
//     if (!NT_SUCCESS(Status) && (Status != STATUS_INFO_LENGTH_MISMATCH))
//     {
//         /* Free the buffer and fail */
//         ExFreePoolWithTag(LocalInfo, TAG_IO);
//         return Status;
//     }
//
//     /* Get buffer pointer */
//     p = (PWCHAR)(ObjectNameInfo + 1);
//
//     _SEH2_TRY
//     {
//         /* Copy the information */
//         if (QueryDosName && NoObCall)
//         {
//             ASSERT(PreviousMode == KernelMode);
//
//             /* Copy structure first */
//             RtlCopyMemory(ObjectNameInfo,
//                           LocalInfo,
//                           (Length >= LocalReturnLength ? sizeof(OBJECT_NAME_INFORMATION) : Length));
//             /* Name then */
//             RtlCopyMemory(p, LocalInfo->Name.Buffer,
//                           (Length >= LocalReturnLength ? LocalInfo->Name.Length : Length - sizeof(OBJECT_NAME_INFORMATION)));
//
//             if (FileObject->DeviceObject->DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM)
//             {
//                 ExFreePool(LocalInfo->Name.Buffer);
//             }
//         }
//         else
//         {
//             RtlCopyMemory(ObjectNameInfo,
//                           LocalInfo,
//                           (LocalReturnLength > Length) ?
//                           Length : LocalReturnLength);
//         }
//
//         /* Set buffer pointer */
//         ObjectNameInfo->Name.Buffer = p;
//
//         /* Advance in buffer */
//         p += (LocalInfo->Name.Length / sizeof(WCHAR));
//
//         /* Check if this already filled our buffer */
//         if (LocalReturnLength > Length)
//         {
//             /* Set the length mismatch to true, so that we can return
//              * the proper buffer size to the caller later
//              */
//             LengthMismatch = TRUE;
//
//             /* Save the initial buffer length value */
//             *ReturnLength = LocalReturnLength;
//         }
//
//         /* Now get the file name buffer and check the length needed */
//         LocalFileInfo = (PFILE_NAME_INFORMATION)LocalInfo;
//         FileLength = Length -
//                      LocalReturnLength +
//                      FIELD_OFFSET(FILE_NAME_INFORMATION, FileName);
//
//         /* Query the File name */
//         if (PreviousMode == KernelMode &&
//             BooleanFlagOn(FileObject->Flags, FO_SYNCHRONOUS_IO))
//         {
//             Status = IopGetFileInformation(FileObject,
//                                            LengthMismatch ? Length : FileLength,
//                                            FileNameInformation,
//                                            LocalFileInfo,
//                                            &LocalReturnLength);
//         }
//         else
//         {
//             Status = IoQueryFileInformation(FileObject,
//                                             FileNameInformation,
//                                             LengthMismatch ? Length : FileLength,
//                                             LocalFileInfo,
//                                             &LocalReturnLength);
//         }
//         if (NT_ERROR(Status))
//         {
//             /* Allow status that would mean it's not implemented in the storage stack */
//             if (Status != STATUS_INVALID_PARAMETER && Status != STATUS_INVALID_DEVICE_REQUEST &&
//                 Status != STATUS_NOT_IMPLEMENTED && Status != STATUS_INVALID_INFO_CLASS)
//             {
//                 _SEH2_LEAVE;
//             }
//
//             /* In such case, zero the output and reset the status */
//             LocalReturnLength = FIELD_OFFSET(FILE_NAME_INFORMATION, FileName);
//             LocalFileInfo->FileNameLength = 0;
//             LocalFileInfo->FileName[0] = OBJ_NAME_PATH_SEPARATOR;
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* We'll at least return the name length */
//             if (LocalReturnLength < FIELD_OFFSET(FILE_NAME_INFORMATION, FileName))
//             {
//                 LocalReturnLength = FIELD_OFFSET(FILE_NAME_INFORMATION, FileName);
//             }
//         }
//
//         /* If the provided buffer is too small, return the required size */
//         if (LengthMismatch)
//         {
//             /* Add the required length */
//             *ReturnLength += LocalFileInfo->FileNameLength;
//
//             /* Free the allocated buffer and return failure */
//             Status = STATUS_BUFFER_OVERFLOW;
//             _SEH2_LEAVE;
//         }
//
//         /* Now calculate the new lengths left */
//         FileLength = LocalReturnLength -
//                      FIELD_OFFSET(FILE_NAME_INFORMATION, FileName);
//         LocalReturnLength = (ULONG)((ULONG_PTR)p -
//                                     (ULONG_PTR)ObjectNameInfo +
//                                     LocalFileInfo->FileNameLength);
//
//         /* Don't copy the name if it's not valid */
//         if (LocalFileInfo->FileName[0] != OBJ_NAME_PATH_SEPARATOR)
//         {
//             /* Free the allocated buffer and return failure */
//             Status = STATUS_OBJECT_PATH_INVALID;
//             _SEH2_LEAVE;
//         }
//
//         /* Write the Name and null-terminate it */
//         RtlCopyMemory(p, LocalFileInfo->FileName, FileLength);
//         p += (FileLength / sizeof(WCHAR));
//         *p = UNICODE_NULL;
//         LocalReturnLength += sizeof(UNICODE_NULL);
//
//         /* Return the length needed */
//         *ReturnLength = LocalReturnLength;
//
//         /* Setup the length and maximum length */
//         FileLength = (ULONG)((ULONG_PTR)p - (ULONG_PTR)ObjectNameInfo);
//         ObjectNameInfo->Name.Length = (USHORT)FileLength -
//                                               sizeof(OBJECT_NAME_INFORMATION);
//         ObjectNameInfo->Name.MaximumLength = (USHORT)ObjectNameInfo->Name.Length +
//                                                      sizeof(UNICODE_NULL);
//     }
//     _SEH2_FINALLY
//     {
//         /* Free buffer and return */
//         ExFreePoolWithTag(LocalInfo, TAG_IO);
//     } _SEH2_END;
//
//     return Status;
// }
//
// VOID
// NTAPI
// IopCloseFile(IN PEPROCESS Process OPTIONAL,
//              IN PVOID ObjectBody,
//              IN ACCESS_MASK GrantedAccess,
//              IN ULONG HandleCount,
//              IN ULONG SystemHandleCount)
// {
//     PFILE_OBJECT FileObject = (PFILE_OBJECT)ObjectBody;
//     KEVENT Event;
//     PIRP Irp;
//     PIO_STACK_LOCATION StackPtr;
//     NTSTATUS Status;
//     PDEVICE_OBJECT DeviceObject;
//     KIRQL OldIrql;
//     IO_STATUS_BLOCK IoStatusBlock;
//     IOTRACE(IO_FILE_DEBUG, "ObjectBody: %p\n", ObjectBody);
//
//     /* If this isn't the last handle for the current process, quit */
//     if (HandleCount != 1) return;
//
//     /* Check if the file is locked and has more then one handle opened */
//     if ((FileObject->LockOperation) && (SystemHandleCount != 1))
//     {
//         /* Check if this is a direct open or not */
//         if (BooleanFlagOn(FileObject->Flags, FO_DIRECT_DEVICE_OPEN))
//         {
//             /* Get the attached device */
//             DeviceObject = IoGetAttachedDevice(FileObject->DeviceObject);
//         }
//         else
//         {
//             /* Get the FO's device */
//             DeviceObject = IoGetRelatedDeviceObject(FileObject);
//         }
//
//         /* Check if this is a sync FO and lock it */
//         if (BooleanFlagOn(FileObject->Flags, FO_SYNCHRONOUS_IO))
//         {
//             (VOID)IopLockFileObject(FileObject, KernelMode);
//         }
//
//         /* Go the FastIO path if possible, otherwise fall back to IRP */
//         if (DeviceObject->DriverObject->FastIoDispatch == NULL ||
//             DeviceObject->DriverObject->FastIoDispatch->FastIoUnlockAll == NULL ||
//             !DeviceObject->DriverObject->FastIoDispatch->FastIoUnlockAll(FileObject, PsGetCurrentProcess(), &IoStatusBlock, DeviceObject))
//         {
//             /* Clear and set up Events */
//             KeClearEvent(&FileObject->Event);
//             KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//             /* Allocate an IRP */
//             Irp = IopAllocateIrpMustSucceed(DeviceObject->StackSize);
//
//             /* Set it up */
//             Irp->UserEvent = &Event;
//             Irp->UserIosb = &Irp->IoStatus;
//             Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//             Irp->Tail.Overlay.OriginalFileObject = FileObject;
//             Irp->RequestorMode = KernelMode;
//             Irp->Flags = IRP_SYNCHRONOUS_API;
//             Irp->Overlay.AsynchronousParameters.UserApcRoutine = NULL;
//             ObReferenceObject(FileObject);
//
//             /* Set up Stack Pointer Data */
//             StackPtr = IoGetNextIrpStackLocation(Irp);
//             StackPtr->MajorFunction = IRP_MJ_LOCK_CONTROL;
//             StackPtr->MinorFunction = IRP_MN_UNLOCK_ALL;
//             StackPtr->FileObject = FileObject;
//
//             /* Queue the IRP */
//             IopQueueIrpToThread(Irp);
//
//             /* Call the FS Driver */
//             Status = IoCallDriver(DeviceObject, Irp);
//             if (Status == STATUS_PENDING)
//             {
//                 /* Wait for completion */
//                 KeWaitForSingleObject(&Event, UserRequest, KernelMode, FALSE, NULL);
//             }
//
//             /* IO will unqueue & free for us */
//         }
//
//         /* Release the lock if we were holding it */
//         if (BooleanFlagOn(FileObject->Flags, FO_SYNCHRONOUS_IO))
//         {
//             IopUnlockFileObject(FileObject);
//         }
//     }
//
//     /* Make sure this is the last handle */
//     if (SystemHandleCount != 1) return;
//
//     /* Check if this is a direct open or not */
//     if (FileObject->Flags & FO_DIRECT_DEVICE_OPEN)
//     {
//         /* Get the attached device */
//         DeviceObject = IoGetAttachedDevice(FileObject->DeviceObject);
//     }
//     else
//     {
//         /* Get the FO's device */
//         DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     }
//
//     /* Set the handle created flag */
//     FileObject->Flags |= FO_HANDLE_CREATED;
//
//     /* Check if this is a sync FO and lock it */
//     if (Process != NULL &&
//         BooleanFlagOn(FileObject->Flags, FO_SYNCHRONOUS_IO))
//     {
//         (VOID)IopLockFileObject(FileObject, KernelMode);
//     }
//
//     /* Clear and set up Events */
//     KeClearEvent(&FileObject->Event);
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     /* Allocate an IRP */
//     Irp = IopAllocateIrpMustSucceed(DeviceObject->StackSize);
//
//     /* Set it up */
//     Irp->UserEvent = &Event;
//     Irp->UserIosb = &Irp->IoStatus;
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//     Irp->Tail.Overlay.OriginalFileObject = FileObject;
//     Irp->Overlay.AsynchronousParameters.UserApcRoutine = NULL;
//     Irp->Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;
//
//     /* Set up Stack Pointer Data */
//     StackPtr = IoGetNextIrpStackLocation(Irp);
//     StackPtr->MajorFunction = IRP_MJ_CLEANUP;
//     StackPtr->FileObject = FileObject;
//
//     /* Queue the IRP */
//     IopQueueIrpToThread(Irp);
//
//     /* Update operation counts */
//     IopUpdateOperationCount(IopOtherTransfer);
//
//     /* Call the FS Driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for completion */
//         KeWaitForSingleObject(&Event, UserRequest, KernelMode, FALSE, NULL);
//     }
//
//     /* Unqueue the IRP */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//     IopUnQueueIrpFromThread(Irp);
//     KeLowerIrql(OldIrql);
//
//     /* Free the IRP */
//     IoFreeIrp(Irp);
//
//     /* Release the lock if we were holding it */
//     if (Process != NULL &&
//         BooleanFlagOn(FileObject->Flags, FO_SYNCHRONOUS_IO))
//     {
//         IopUnlockFileObject(FileObject);
//     }
// }
//
// NTSTATUS
// NTAPI
// IopQueryAttributesFile(IN POBJECT_ATTRIBUTES ObjectAttributes,
//                        IN FILE_INFORMATION_CLASS FileInformationClass,
//                        IN ULONG FileInformationSize,
//                        OUT PVOID FileInformation)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE AccessMode = ExGetPreviousMode();
//     DUMMY_FILE_OBJECT LocalFileObject;
//     FILE_NETWORK_OPEN_INFORMATION NetworkOpenInfo;
//     HANDLE Handle;
//     OPEN_PACKET OpenPacket;
//     BOOLEAN IsBasic;
//     PAGED_CODE();
//     IOTRACE(IO_FILE_DEBUG, "Class: %lx\n", FileInformationClass);
//
//     /* Check if the caller was user mode */
//     if (AccessMode != KernelMode)
//     {
//         /* Protect probe in SEH */
//         _SEH2_TRY
//         {
//             /* Probe the buffer */
//             ProbeForWrite(FileInformation, FileInformationSize, sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Check if this is a basic or full request */
//     IsBasic = (FileInformationSize == sizeof(FILE_BASIC_INFORMATION));
//
//     /* Setup the Open Packet */
//     RtlZeroMemory(&OpenPacket, sizeof(OPEN_PACKET));
//     OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
//     OpenPacket.Size = sizeof(OPEN_PACKET);
//     OpenPacket.CreateOptions = FILE_OPEN_REPARSE_POINT;
//     OpenPacket.ShareAccess = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
//     OpenPacket.Disposition = FILE_OPEN;
//     OpenPacket.BasicInformation = IsBasic ? FileInformation : NULL;
//     OpenPacket.NetworkInformation = IsBasic ? &NetworkOpenInfo :
//                                     (AccessMode != KernelMode) ?
//                                     &NetworkOpenInfo : FileInformation;
//     OpenPacket.QueryOnly = TRUE;
//     OpenPacket.FullAttributes = IsBasic ? FALSE : TRUE;
//     OpenPacket.LocalFileObject = &LocalFileObject;
//
//     /* Update the operation count */
//     IopUpdateOperationCount(IopOtherTransfer);
//
//     /*
//      * Attempt opening the file. This will call the I/O Parse Routine for
//      * the File Object (IopParseDevice) which will use the dummy file obejct
//      * send the IRP to its device object. Note that we have two statuses
//      * to worry about: the Object Manager's status (in Status) and the I/O
//      * status, which is in the Open Packet's Final Status, and determined
//      * by the Parse Check member.
//      */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 NULL,
//                                 AccessMode,
//                                 NULL,
//                                 FILE_READ_ATTRIBUTES,
//                                 &OpenPacket,
//                                 &Handle);
//     if (OpenPacket.ParseCheck == FALSE)
//     {
//         /* Parse failed */
//         DPRINT("IopQueryAttributesFile failed for '%wZ' with 0x%lx\n",
//                ObjectAttributes->ObjectName, Status);
//         return Status;
//     }
//     else
//     {
//         /* Use the Io status */
//         Status = OpenPacket.FinalStatus;
//     }
//
//     /* Check if we were succesful and this was user mode and a full query */
//     if ((NT_SUCCESS(Status)) && (AccessMode != KernelMode) && !(IsBasic))
//     {
//         /* Enter SEH for copy */
//         _SEH2_TRY
//         {
//             /* Copy the buffer back */
//             RtlCopyMemory(FileInformation,
//                           &NetworkOpenInfo,
//                           FileInformationSize);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopAcquireFileObjectLock(
//     _In_ PFILE_OBJECT FileObject,
//     _In_ KPROCESSOR_MODE WaitMode,
//     _In_ BOOLEAN Alertable,
//     _Out_ PBOOLEAN LockFailed)
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     InterlockedIncrement((PLONG)&FileObject->Waiters);
//
//     Status = STATUS_SUCCESS;
//     do
//     {
//         if (!InterlockedExchange((PLONG)&FileObject->Busy, TRUE))
//         {
//             break;
//         }
//         Status = KeWaitForSingleObject(&FileObject->Lock,
//                                        Executive,
//                                        WaitMode,
//                                        Alertable,
//                                        NULL);
//     } while (Status == STATUS_SUCCESS);
//
//     InterlockedDecrement((PLONG)&FileObject->Waiters);
//     if (Status == STATUS_SUCCESS)
//     {
//         ObReferenceObject(FileObject);
//         *LockFailed = FALSE;
//     }
//     else
//     {
//         if (!FileObject->Busy && FileObject->Waiters)
//         {
//             KeSetEvent(&FileObject->Lock, IO_NO_INCREMENT, FALSE);
//         }
//         *LockFailed = TRUE;
//     }
//
//     return Status;
// }
//
// PVOID
// NTAPI
// IoGetFileObjectFilterContext(IN PFILE_OBJECT FileObject)
// {
//     if (BooleanFlagOn(FileObject->Flags, FO_FILE_OBJECT_HAS_EXTENSION))
//     {
//         PFILE_OBJECT_EXTENSION FileObjectExtension;
//
//         FileObjectExtension = FileObject->FileObjectExtension;
//         return FileObjectExtension->FilterContext;
//     }
//
//     return NULL;
// }
//
// NTSTATUS
// NTAPI
// IoChangeFileObjectFilterContext(IN PFILE_OBJECT FileObject,
//                                 IN PVOID FilterContext,
//                                 IN BOOLEAN Define)
// {
//     ULONG_PTR Success;
//     PFILE_OBJECT_EXTENSION FileObjectExtension;
//
//     if (!BooleanFlagOn(FileObject->Flags, FO_FILE_OBJECT_HAS_EXTENSION))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     FileObjectExtension = FileObject->FileObjectExtension;
//     if (Define)
//     {
//         /* If define, just set the new value if not value is set
//          * Success will only contain old value. It is valid if it is NULL
//          */
//         Success = (ULONG_PTR)InterlockedCompareExchangePointer(&FileObjectExtension->FilterContext, FilterContext, NULL);
//     }
//     else
//     {
//         /* If not define, we want to reset filter context.
//          * We will remove value (provided by the caller) and set NULL instead.
//          * This will only success if caller provides correct previous value.
//          * To catch whether it worked, we substract previous value to expect value:
//          * If it matches (and thus, we reset), Success will contain 0
//          * Otherwise, it will contain a non-zero value.
//          */
//         Success = (ULONG_PTR)InterlockedCompareExchangePointer(&FileObjectExtension->FilterContext, NULL, FilterContext) - (ULONG_PTR)FilterContext;
//     }
//
//     /* If success isn't 0, it means we failed somewhere (set or unset) */
//     if (Success != 0)
//     {
//         return STATUS_ALREADY_COMMITTED;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// IopCreateFile(OUT PHANDLE FileHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN POBJECT_ATTRIBUTES ObjectAttributes,
//               OUT PIO_STATUS_BLOCK IoStatusBlock,
//               IN PLARGE_INTEGER AllocationSize OPTIONAL,
//               IN ULONG FileAttributes,
//               IN ULONG ShareAccess,
//               IN ULONG Disposition,
//               IN ULONG CreateOptions,
//               IN PVOID EaBuffer OPTIONAL,
//               IN ULONG EaLength,
//               IN CREATE_FILE_TYPE CreateFileType,
//               IN PVOID ExtraCreateParameters OPTIONAL,
//               IN ULONG Options,
//               IN ULONG Flags,
//               IN PDEVICE_OBJECT DeviceObject OPTIONAL)
// {
//     KPROCESSOR_MODE AccessMode;
//     HANDLE LocalHandle = 0;
//     LARGE_INTEGER SafeAllocationSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PNAMED_PIPE_CREATE_PARAMETERS NamedPipeCreateParameters;
//     POPEN_PACKET OpenPacket;
//     ULONG EaErrorOffset;
//     PAGED_CODE();
//
//     IOTRACE(IO_FILE_DEBUG, "FileName: %wZ\n", ObjectAttributes->ObjectName);
//
//
//     /* Check if we have no parameter checking to do */
//     if (Options & IO_NO_PARAMETER_CHECKING)
//     {
//         /* Then force kernel-mode access to avoid checks */
//         AccessMode = KernelMode;
//     }
//     else
//     {
//         /* Otherwise, use the actual mode */
//         AccessMode = ExGetPreviousMode();
//     }
//
//     /* Check if we need to do parameter checking */
//     if ((AccessMode != KernelMode) || (Options & IO_CHECK_CREATE_PARAMETERS))
//     {
//         /* Validate parameters */
//         if (FileAttributes & ~FILE_ATTRIBUTE_VALID_FLAGS)
//         {
//             DPRINT1("File Create 'FileAttributes' Parameter contains invalid flags!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if (ShareAccess & ~FILE_SHARE_VALID_FLAGS)
//         {
//             DPRINT1("File Create 'ShareAccess' Parameter contains invalid flags!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if (Disposition > FILE_MAXIMUM_DISPOSITION)
//         {
//             DPRINT1("File Create 'Disposition' Parameter is out of range!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if (CreateOptions & ~FILE_VALID_OPTION_FLAGS)
//         {
//             DPRINT1("File Create 'CreateOptions' parameter contains invalid flags!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & (FILE_SYNCHRONOUS_IO_ALERT | FILE_SYNCHRONOUS_IO_NONALERT)) &&
//             (!(DesiredAccess & SYNCHRONIZE)))
//         {
//             DPRINT1("File Create 'CreateOptions' parameter FILE_SYNCHRONOUS_IO_* requested, but 'DesiredAccess' does not have SYNCHRONIZE!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & FILE_DELETE_ON_CLOSE) && (!(DesiredAccess & DELETE)))
//         {
//             DPRINT1("File Create 'CreateOptions' parameter FILE_DELETE_ON_CLOSE requested, but 'DesiredAccess' does not have DELETE!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & (FILE_SYNCHRONOUS_IO_NONALERT | FILE_SYNCHRONOUS_IO_ALERT)) ==
//             (FILE_SYNCHRONOUS_IO_NONALERT | FILE_SYNCHRONOUS_IO_ALERT))
//         {
//             DPRINT1("File Create 'FileAttributes' parameter both FILE_SYNCHRONOUS_IO_NONALERT and FILE_SYNCHRONOUS_IO_ALERT specified!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & FILE_DIRECTORY_FILE) && !(CreateOptions & FILE_NON_DIRECTORY_FILE) &&
//             (CreateOptions & ~(FILE_DIRECTORY_FILE |
//                                FILE_SYNCHRONOUS_IO_ALERT |
//                                FILE_SYNCHRONOUS_IO_NONALERT |
//                                FILE_WRITE_THROUGH |
//                                FILE_COMPLETE_IF_OPLOCKED |
//                                FILE_OPEN_FOR_BACKUP_INTENT |
//                                FILE_DELETE_ON_CLOSE |
//                                FILE_OPEN_FOR_FREE_SPACE_QUERY |
//                                FILE_OPEN_BY_FILE_ID |
//                                FILE_NO_COMPRESSION |
//                                FILE_OPEN_REPARSE_POINT)))
//         {
//             DPRINT1("File Create 'CreateOptions' Parameter has flags incompatible with FILE_DIRECTORY_FILE!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & FILE_DIRECTORY_FILE) && !(CreateOptions & FILE_NON_DIRECTORY_FILE) &&
//             (Disposition != FILE_CREATE) && (Disposition != FILE_OPEN) && (Disposition != FILE_OPEN_IF))
//         {
//             DPRINT1("File Create 'CreateOptions' Parameter FILE_DIRECTORY_FILE requested, but 'Disposition' is not FILE_CREATE/FILE_OPEN/FILE_OPEN_IF!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & FILE_COMPLETE_IF_OPLOCKED) && (CreateOptions & FILE_RESERVE_OPFILTER))
//         {
//             DPRINT1("File Create 'CreateOptions' Parameter both FILE_COMPLETE_IF_OPLOCKED and FILE_RESERVE_OPFILTER specified!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         if ((CreateOptions & FILE_NO_INTERMEDIATE_BUFFERING) && (DesiredAccess & FILE_APPEND_DATA))
//         {
//             DPRINT1("File Create 'CreateOptions' parameter FILE_NO_INTERMEDIATE_BUFFERING requested, but 'DesiredAccess' FILE_APPEND_DATA requires it!\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Now check if this is a named pipe */
//         if (CreateFileType == CreateFileTypeNamedPipe)
//         {
//             /* Make sure we have extra parameters */
//             if (!ExtraCreateParameters)
//             {
//                 DPRINT1("Invalid parameter: ExtraCreateParameters == 0!\n");
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Get the parameters and validate them */
//             NamedPipeCreateParameters = ExtraCreateParameters;
//             if ((NamedPipeCreateParameters->NamedPipeType > FILE_PIPE_MESSAGE_TYPE) ||
//                 (NamedPipeCreateParameters->ReadMode > FILE_PIPE_MESSAGE_MODE) ||
//                 (NamedPipeCreateParameters->CompletionMode > FILE_PIPE_COMPLETE_OPERATION) ||
//                 (ShareAccess & FILE_SHARE_DELETE) ||
//                 ((Disposition < FILE_OPEN) || (Disposition > FILE_OPEN_IF)) ||
//                 (CreateOptions & ~FILE_VALID_PIPE_OPTION_FLAGS))
//             {
//                 /* Invalid named pipe create */
//                 DPRINT1("Invalid named pipe create\n");
//                 return STATUS_INVALID_PARAMETER;
//             }
//         }
//         else if (CreateFileType == CreateFileTypeMailslot)
//         {
//             /* Make sure we have extra parameters */
//             if (!ExtraCreateParameters)
//             {
//                 DPRINT1("Invalid parameter: ExtraCreateParameters == 0!\n");
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Get the parameters and validate them */
//             if ((ShareAccess & FILE_SHARE_DELETE) ||
//                 !(ShareAccess & ~FILE_SHARE_WRITE) ||
//                 (Disposition != FILE_CREATE) ||
//                 (CreateOptions & ~FILE_VALID_MAILSLOT_OPTION_FLAGS))
//             {
//                 /* Invalid mailslot create */
//                 DPRINT1("Invalid mailslot create\n");
//                 return STATUS_INVALID_PARAMETER;
//             }
//         }
//     }
//
//     /* Allocate the open packet */
//     OpenPacket = ExAllocatePoolWithTag(NonPagedPool, sizeof(*OpenPacket), 'pOoI');
//     if (!OpenPacket) return STATUS_INSUFFICIENT_RESOURCES;
//     RtlZeroMemory(OpenPacket, sizeof(*OpenPacket));
//
//     /* Check if the call came from user mode */
//     if (AccessMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the output parameters */
//             ProbeForWriteHandle(FileHandle);
//             ProbeForWriteIoStatusBlock(IoStatusBlock);
//
//             /* Probe the allocation size if one was passed in */
//             if (AllocationSize)
//             {
//                 SafeAllocationSize = ProbeForReadLargeInteger(AllocationSize);
//             }
//             else
//             {
//                 SafeAllocationSize.QuadPart = 0;
//             }
//
//             /* Make sure it's valid */
//             if (SafeAllocationSize.QuadPart < 0)
//             {
//                 RtlRaiseStatus(STATUS_INVALID_PARAMETER);
//             }
//
//             /* Check if EA was passed in */
//             if ((EaBuffer) && (EaLength))
//             {
//                 /* Probe it */
//                 ProbeForRead(EaBuffer, EaLength, sizeof(ULONG));
//
//                 /* And marshall it */
//                 OpenPacket->EaBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                              EaLength,
//                                                              TAG_EA);
//                 OpenPacket->EaLength = EaLength;
//                 RtlCopyMemory(OpenPacket->EaBuffer, EaBuffer, EaLength);
//
//                 /* Validate the buffer */
//                 Status = IoCheckEaBufferValidity(OpenPacket->EaBuffer,
//                                                  EaLength,
//                                                  &EaErrorOffset);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Undo everything if it's invalid */
//                     DPRINT1("Invalid EA buffer\n");
//                     IoStatusBlock->Status = Status;
//                     IoStatusBlock->Information = EaErrorOffset;
//                     RtlRaiseStatus(Status);
//                 }
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             if (OpenPacket->EaBuffer != NULL) ExFreePool(OpenPacket->EaBuffer);
//             ExFreePool(OpenPacket);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Check if this is a device attach */
//         if (CreateOptions & IO_ATTACH_DEVICE_API)
//         {
//             /* Set the flag properly */
//             Options |= IO_ATTACH_DEVICE;
//             CreateOptions &= ~IO_ATTACH_DEVICE_API;
//         }
//
//         /* Check if we have allocation size */
//         if (AllocationSize)
//         {
//             /* Capture it */
//             SafeAllocationSize = *AllocationSize;
//         }
//         else
//         {
//             /* Otherwise, no size */
//             SafeAllocationSize.QuadPart = 0;
//         }
//
//         /* Check if we have an EA packet */
//         if ((EaBuffer) && (EaLength))
//         {
//             /* Allocate the kernel copy */
//             OpenPacket->EaBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                          EaLength,
//                                                          TAG_EA);
//             if (!OpenPacket->EaBuffer)
//             {
//                 ExFreePool(OpenPacket);
//                 DPRINT1("Failed to allocate open packet EA buffer\n");
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Copy the data */
//             OpenPacket->EaLength = EaLength;
//             RtlCopyMemory(OpenPacket->EaBuffer, EaBuffer, EaLength);
//
//             /* Validate the buffer */
//             Status = IoCheckEaBufferValidity(OpenPacket->EaBuffer,
//                                              EaLength,
//                                              &EaErrorOffset);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Undo everything if it's invalid */
//                 DPRINT1("Invalid EA buffer\n");
//                 ExFreePool(OpenPacket->EaBuffer);
//                 IoStatusBlock->Status = Status;
//                 IoStatusBlock->Information = EaErrorOffset;
//                 ExFreePool(OpenPacket);
//                 return Status;
//             }
//         }
//     }
//
//     /* Setup the Open Packet */
//     OpenPacket->Type = IO_TYPE_OPEN_PACKET as i16;
//     OpenPacket->Size = sizeof(*OpenPacket);
//     OpenPacket->AllocationSize = SafeAllocationSize;
//     OpenPacket->CreateOptions = CreateOptions;
//     OpenPacket->FileAttributes = (USHORT)FileAttributes;
//     OpenPacket->ShareAccess = (USHORT)ShareAccess;
//     OpenPacket->Options = Options;
//     OpenPacket->Disposition = Disposition;
//     OpenPacket->CreateFileType = CreateFileType;
//     OpenPacket->ExtraCreateParameters = ExtraCreateParameters;
//     OpenPacket->InternalFlags = Flags;
//     OpenPacket->TopDeviceObjectHint = DeviceObject;
//
//     /* Update the operation count */
//     IopUpdateOperationCount(IopOtherTransfer);
//
//     /*
//      * Attempt opening the file. This will call the I/O Parse Routine for
//      * the File Object (IopParseDevice) which will create the object and
//      * send the IRP to its device object. Note that we have two statuses
//      * to worry about: the Object Manager's status (in Status) and the I/O
//      * status, which is in the Open Packet's Final Status, and determined
//      * by the Parse Check member.
//      */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 NULL,
//                                 AccessMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 OpenPacket,
//                                 &LocalHandle);
//
//     /* Free the EA Buffer */
//     if (OpenPacket->EaBuffer) ExFreePool(OpenPacket->EaBuffer);
//
//     /* Now check for Ob or Io failure */
//     if (!(NT_SUCCESS(Status)) || (OpenPacket->ParseCheck == FALSE))
//     {
//         /* Check if Ob thinks well went well */
//         if (NT_SUCCESS(Status))
//         {
//             /*
//              * Tell it otherwise. Because we didn't use an ObjectType,
//              * it incorrectly returned us a handle to God knows what.
//              */
//             ZwClose(LocalHandle);
//             Status = STATUS_OBJECT_TYPE_MISMATCH;
//         }
//
//         /* Now check the Io status */
//         if (!NT_SUCCESS(OpenPacket->FinalStatus))
//         {
//             /* Use this status instead of Ob's */
//             Status = OpenPacket->FinalStatus;
//
//             /* Check if it was only a warning */
//             if (NT_WARNING(Status))
//             {
//                 /* Protect write with SEH */
//                 _SEH2_TRY
//                 {
//                     /* In this case, we copy the I/O Status back */
//                     IoStatusBlock->Information = OpenPacket->Information;
//                     IoStatusBlock->Status = OpenPacket->FinalStatus;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     /* Get exception code */
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//             }
//         }
//         else if ((OpenPacket->FileObject) && (OpenPacket->ParseCheck == FALSE))
//         {
//             /*
//              * This can happen in the very bizarre case where the parse routine
//              * actually executed more then once (due to a reparse) and ended
//              * up failing after already having created the File Object.
//              */
//             if (OpenPacket->FileObject->FileName.Length)
//             {
//                 /* It had a name, free it */
//                 ExFreePoolWithTag(OpenPacket->FileObject->FileName.Buffer, TAG_IO_NAME);
//             }
//
//             /* Clear the device object to invalidate the FO, and dereference */
//             OpenPacket->FileObject->DeviceObject = NULL;
//             ObDereferenceObject(OpenPacket->FileObject);
//         }
//     }
//     else
//     {
//         /* We reached success and have a valid file handle */
//         OpenPacket->FileObject->Flags |= FO_HANDLE_CREATED;
//         ASSERT(OpenPacket->FileObject->Type == IO_TYPE_FILE);
//
//         /* Enter SEH for write back */
//         _SEH2_TRY
//         {
//             /* Write back the handle and I/O Status */
//             *FileHandle = LocalHandle;
//             IoStatusBlock->Information = OpenPacket->Information;
//             IoStatusBlock->Status = OpenPacket->FinalStatus;
//
//             /* Get the Io status */
//             Status = OpenPacket->FinalStatus;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the exception status */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Check if we were 100% successful */
//     if ((OpenPacket->ParseCheck != FALSE) && (OpenPacket->FileObject))
//     {
//         /* Dereference the File Object */
//         ObDereferenceObject(OpenPacket->FileObject);
//     }
//
//     /* Return status */
//     ExFreePool(OpenPacket);
//     return Status;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoCheckQuerySetFileInformation(IN FILE_INFORMATION_CLASS FileInformationClass,
//                                IN ULONG Length,
//                                IN BOOLEAN SetOperation)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoCheckQuotaBufferValidity(IN PFILE_QUOTA_INFORMATION QuotaBuffer,
//                            IN ULONG QuotaLength,
//                            OUT PULONG ErrorOffset)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCreateFile(OUT PHANDLE FileHandle,
//              IN ACCESS_MASK DesiredAccess,
//              IN POBJECT_ATTRIBUTES ObjectAttributes,
//              OUT PIO_STATUS_BLOCK IoStatusBlock,
//              IN PLARGE_INTEGER AllocationSize OPTIONAL,
//              IN ULONG FileAttributes,
//              IN ULONG ShareAccess,
//              IN ULONG Disposition,
//              IN ULONG CreateOptions,
//              IN PVOID EaBuffer OPTIONAL,
//              IN ULONG EaLength,
//              IN CREATE_FILE_TYPE CreateFileType,
//              IN PVOID ExtraCreateParameters OPTIONAL,
//              IN ULONG Options)
// {
//     PAGED_CODE();
//
//     return IopCreateFile(FileHandle,
//                          DesiredAccess,
//                          ObjectAttributes,
//                          IoStatusBlock,
//                          AllocationSize,
//                          FileAttributes,
//                          ShareAccess,
//                          Disposition,
//                          CreateOptions,
//                          EaBuffer,
//                          EaLength,
//                          CreateFileType,
//                          ExtraCreateParameters,
//                          Options,
//                          0,
//                          NULL);
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoCreateFileSpecifyDeviceObjectHint(OUT PHANDLE FileHandle,
//                                     IN ACCESS_MASK DesiredAccess,
//                                     IN POBJECT_ATTRIBUTES ObjectAttributes,
//                                     OUT PIO_STATUS_BLOCK IoStatusBlock,
//                                     IN PLARGE_INTEGER AllocationSize OPTIONAL,
//                                     IN ULONG FileAttributes,
//                                     IN ULONG ShareAccess,
//                                     IN ULONG Disposition,
//                                     IN ULONG CreateOptions,
//                                     IN PVOID EaBuffer OPTIONAL,
//                                     IN ULONG EaLength,
//                                     IN CREATE_FILE_TYPE CreateFileType,
//                                     IN PVOID ExtraCreateParameters OPTIONAL,
//                                     IN ULONG Options,
//                                     IN PVOID DeviceObject)
// {
//     ULONG Flags = 0;
//
//     PAGED_CODE();
//
//     /* Check if we were passed a device to send the create request to*/
//     if (DeviceObject)
//     {
//         /* We'll tag this request into a file object extension */
//         Flags = (IOP_CREATE_FILE_OBJECT_EXTENSION | IOP_USE_TOP_LEVEL_DEVICE_HINT);
//     }
//
//     return IopCreateFile(FileHandle,
//                          DesiredAccess,
//                          ObjectAttributes,
//                          IoStatusBlock,
//                          AllocationSize,
//                          FileAttributes,
//                          ShareAccess,
//                          Disposition,
//                          CreateOptions,
//                          EaBuffer,
//                          EaLength,
//                          CreateFileType,
//                          ExtraCreateParameters,
//                          Options | IO_NO_PARAMETER_CHECKING,
//                          Flags,
//                          DeviceObject);
// }
//
// /*
//  * @implemented
//  */
// PFILE_OBJECT
// NTAPI
// IoCreateStreamFileObjectEx(IN PFILE_OBJECT FileObject OPTIONAL,
//                            IN PDEVICE_OBJECT DeviceObject OPTIONAL,
//                            OUT PHANDLE FileObjectHandle OPTIONAL)
// {
//     PFILE_OBJECT CreatedFileObject;
//     NTSTATUS Status;
//     HANDLE FileHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PAGED_CODE();
//     IOTRACE(IO_FILE_DEBUG, "FileObject: %p\n", FileObject);
//
//     /* Choose Device Object */
//     if (FileObject) DeviceObject = FileObject->DeviceObject;
//
//     /* Reference the device object and initialize attributes */
//     InterlockedIncrement(&DeviceObject->ReferenceCount);
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//
//     /* Create the File Object */
//     Status = ObCreateObject(KernelMode,
//                             IoFileObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(FILE_OBJECT),
//                             sizeof(FILE_OBJECT),
//                             0,
//                             (PVOID*)&CreatedFileObject);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         IopDereferenceDeviceObject(DeviceObject, FALSE);
//         ExRaiseStatus(Status);
//     }
//
//     /* Set File Object Data */
//     RtlZeroMemory(CreatedFileObject, sizeof(FILE_OBJECT));
//     CreatedFileObject->DeviceObject = DeviceObject;
//     CreatedFileObject->Type = IO_TYPE_FILE;
//     CreatedFileObject->Size = sizeof(FILE_OBJECT);
//     CreatedFileObject->Flags = FO_STREAM_FILE;
//
//     /* Initialize the wait event */
//     KeInitializeEvent(&CreatedFileObject->Event, SynchronizationEvent, FALSE);
//
//     /* Insert it to create a handle for it */
//     Status = ObInsertObject(CreatedFileObject,
//                             NULL,
//                             FILE_READ_DATA,
//                             1,
//                             (PVOID*)&CreatedFileObject,
//                             &FileHandle);
//     if (!NT_SUCCESS(Status)) ExRaiseStatus(Status);
//
//     /* Set the handle created flag */
//     CreatedFileObject->Flags |= FO_HANDLE_CREATED;
//     ASSERT(CreatedFileObject->Type == IO_TYPE_FILE);
//
//     /* Check if we have a VPB */
//     if (DeviceObject->Vpb)
//     {
//         /* Reference it */
//          InterlockedIncrement((PLONG)&DeviceObject->Vpb->ReferenceCount);
//     }
//
//     /* Check if the caller wants the handle */
//     if (FileObjectHandle)
//     {
//         /* Return it */
//         *FileObjectHandle = FileHandle;
//         ObDereferenceObject(CreatedFileObject);
//     }
//     else
//     {
//         /* Otherwise, close it */
//         ObCloseHandle(FileHandle, KernelMode);
//     }
//
//     /* Return the file object */
//     return CreatedFileObject;
// }
//
// /*
//  * @implemented
//  */
// PFILE_OBJECT
// NTAPI
// IoCreateStreamFileObject(IN PFILE_OBJECT FileObject,
//                          IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Call the newer function */
//     return IoCreateStreamFileObjectEx(FileObject, DeviceObject, NULL);
// }
//
// /*
//  * @implemented
//  */
// PFILE_OBJECT
// NTAPI
// IoCreateStreamFileObjectLite(IN PFILE_OBJECT FileObject OPTIONAL,
//                              IN PDEVICE_OBJECT DeviceObject OPTIONAL)
// {
//     PFILE_OBJECT CreatedFileObject;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PAGED_CODE();
//     IOTRACE(IO_FILE_DEBUG, "FileObject: %p\n", FileObject);
//
//     /* Choose Device Object */
//     if (FileObject) DeviceObject = FileObject->DeviceObject;
//
//     /* Reference the device object and initialize attributes */
//     InterlockedIncrement(&DeviceObject->ReferenceCount);
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//
//     /* Create the File Object */
//     Status = ObCreateObject(KernelMode,
//                             IoFileObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(FILE_OBJECT),
//                             sizeof(FILE_OBJECT),
//                             0,
//                             (PVOID*)&CreatedFileObject);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         IopDereferenceDeviceObject(DeviceObject, FALSE);
//         ExRaiseStatus(Status);
//     }
//
//     /* Set File Object Data */
//     RtlZeroMemory(CreatedFileObject, sizeof(FILE_OBJECT));
//     CreatedFileObject->DeviceObject = DeviceObject;
//     CreatedFileObject->Type = IO_TYPE_FILE;
//     CreatedFileObject->Size = sizeof(FILE_OBJECT);
//     CreatedFileObject->Flags = FO_STREAM_FILE;
//
//     /* Initialize the wait event */
//     KeInitializeEvent(&CreatedFileObject->Event, SynchronizationEvent, FALSE);
//
//     /* Destroy create information */
//     ObFreeObjectCreateInfoBuffer(OBJECT_TO_OBJECT_HEADER(CreatedFileObject)->
//                                  ObjectCreateInfo);
//     OBJECT_TO_OBJECT_HEADER(CreatedFileObject)->ObjectCreateInfo = NULL;
//
//     /* Set the handle created flag */
//     CreatedFileObject->Flags |= FO_HANDLE_CREATED;
//     ASSERT(CreatedFileObject->Type == IO_TYPE_FILE);
//
//     /* Check if we have a VPB */
//     if (DeviceObject->Vpb)
//     {
//         /* Reference it */
//          InterlockedIncrement((PLONG)&DeviceObject->Vpb->ReferenceCount);
//     }
//
//     /* Return the file object */
//     return CreatedFileObject;
// }
//
// /*
//  * @implemented
//  */
// PGENERIC_MAPPING
// NTAPI
// IoGetFileObjectGenericMapping(VOID)
// {
//     /* Return the mapping */
//     return &IopFileMapping;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoIsFileOriginRemote(IN PFILE_OBJECT FileObject)
// {
//     /* Return the flag status */
//     return FileObject->Flags & FO_REMOTE_ORIGIN ? TRUE : FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoFastQueryNetworkAttributes(IN POBJECT_ATTRIBUTES ObjectAttributes,
//                              IN ACCESS_MASK DesiredAccess,
//                              IN ULONG OpenOptions,
//                              OUT PIO_STATUS_BLOCK IoStatus,
//                              OUT PFILE_NETWORK_OPEN_INFORMATION Buffer)
// {
//     NTSTATUS Status;
//     DUMMY_FILE_OBJECT LocalFileObject;
//     HANDLE Handle;
//     OPEN_PACKET OpenPacket;
//     PAGED_CODE();
//     IOTRACE(IO_FILE_DEBUG, "FileName: %wZ\n", ObjectAttributes->ObjectName);
//
//     /* Setup the Open Packet */
//     RtlZeroMemory(&OpenPacket, sizeof(OPEN_PACKET));
//     OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
//     OpenPacket.Size = sizeof(OPEN_PACKET);
//     OpenPacket.CreateOptions = OpenOptions | FILE_OPEN_REPARSE_POINT;
//     OpenPacket.ShareAccess = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
//     OpenPacket.Options = IO_FORCE_ACCESS_CHECK;
//     OpenPacket.Disposition = FILE_OPEN;
//     OpenPacket.NetworkInformation = Buffer;
//     OpenPacket.QueryOnly = TRUE;
//     OpenPacket.FullAttributes = TRUE;
//     OpenPacket.LocalFileObject = &LocalFileObject;
//
//     /*
//      * Attempt opening the file. This will call the I/O Parse Routine for
//      * the File Object (IopParseDevice) which will use the dummy file obejct
//      * send the IRP to its device object. Note that we have two statuses
//      * to worry about: the Object Manager's status (in Status) and the I/O
//      * status, which is in the Open Packet's Final Status, and determined
//      * by the Parse Check member.
//      */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 NULL,
//                                 KernelMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 &OpenPacket,
//                                 &Handle);
//     if (OpenPacket.ParseCheck == FALSE)
//     {
//         /* Parse failed */
//         IoStatus->Status = Status;
//     }
//     else
//     {
//         /* Use the Io status */
//         IoStatus->Status = OpenPacket.FinalStatus;
//         IoStatus->Information = OpenPacket.Information;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoUpdateShareAccess(IN PFILE_OBJECT FileObject,
//                     OUT PSHARE_ACCESS ShareAccess)
// {
//     PAGED_CODE();
//
//     /* Check if the file has an extension */
//     if (FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION)
//     {
//         /* Check if caller specified to ignore access checks */
//         //if (FileObject->FoExtFlags & IO_IGNORE_SHARE_ACCESS_CHECK)
//         {
//             /* Don't update share access */
//             return;
//         }
//     }
//
//     /* Otherwise, check if there's any access present */
//     if ((FileObject->ReadAccess) ||
//         (FileObject->WriteAccess) ||
//         (FileObject->DeleteAccess))
//     {
//         /* Increase the open count */
//         ShareAccess->OpenCount++;
//
//         /* Add new share access */
//         ShareAccess->Readers += FileObject->ReadAccess;
//         ShareAccess->Writers += FileObject->WriteAccess;
//         ShareAccess->Deleters += FileObject->DeleteAccess;
//         ShareAccess->SharedRead += FileObject->SharedRead;
//         ShareAccess->SharedWrite += FileObject->SharedWrite;
//         ShareAccess->SharedDelete += FileObject->SharedDelete;
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCheckShareAccess(IN ACCESS_MASK DesiredAccess,
//                    IN ULONG DesiredShareAccess,
//                    IN PFILE_OBJECT FileObject,
//                    IN PSHARE_ACCESS ShareAccess,
//                    IN BOOLEAN Update)
// {
//     BOOLEAN ReadAccess;
//     BOOLEAN WriteAccess;
//     BOOLEAN DeleteAccess;
//     BOOLEAN SharedRead;
//     BOOLEAN SharedWrite;
//     BOOLEAN SharedDelete;
//     PAGED_CODE();
//
//     /* Get access masks */
//     ReadAccess = (DesiredAccess & (FILE_READ_DATA | FILE_EXECUTE)) != 0;
//     WriteAccess = (DesiredAccess & (FILE_WRITE_DATA | FILE_APPEND_DATA)) != 0;
//     DeleteAccess = (DesiredAccess & DELETE) != 0;
//
//     /* Set them in the file object */
//     FileObject->ReadAccess = ReadAccess;
//     FileObject->WriteAccess = WriteAccess;
//     FileObject->DeleteAccess = DeleteAccess;
//
//     /* Check if the file has an extension */
//     if (FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION)
//     {
//         /* Check if caller specified to ignore access checks */
//         //if (FileObject->FoExtFlags & IO_IGNORE_SHARE_ACCESS_CHECK)
//         {
//             /* Don't check share access */
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Check if we have any access */
//     if ((ReadAccess) || (WriteAccess) || (DeleteAccess))
//     {
//         /* Get shared access masks */
//         SharedRead = (DesiredShareAccess & FILE_SHARE_READ) != 0;
//         SharedWrite = (DesiredShareAccess & FILE_SHARE_WRITE) != 0;
//         SharedDelete = (DesiredShareAccess & FILE_SHARE_DELETE) != 0;
//
//         /* Check if the shared access is violated */
//         if ((ReadAccess &&
//              (ShareAccess->SharedRead < ShareAccess->OpenCount)) ||
//             (WriteAccess &&
//              (ShareAccess->SharedWrite < ShareAccess->OpenCount)) ||
//             (DeleteAccess &&
//              (ShareAccess->SharedDelete < ShareAccess->OpenCount)) ||
//             ((ShareAccess->Readers != 0) && !SharedRead) ||
//             ((ShareAccess->Writers != 0) && !SharedWrite) ||
//             ((ShareAccess->Deleters != 0) && !SharedDelete))
//         {
//             /* Sharing violation, fail */
//             return STATUS_SHARING_VIOLATION;
//         }
//
//         /* Set them */
//         FileObject->SharedRead = SharedRead;
//         FileObject->SharedWrite = SharedWrite;
//         FileObject->SharedDelete = SharedDelete;
//
//         /* It's not, check if caller wants us to update it */
//         if (Update)
//         {
//             /* Increase open count */
//             ShareAccess->OpenCount++;
//
//             /* Update shared access */
//             ShareAccess->Readers += ReadAccess;
//             ShareAccess->Writers += WriteAccess;
//             ShareAccess->Deleters += DeleteAccess;
//             ShareAccess->SharedRead += SharedRead;
//             ShareAccess->SharedWrite += SharedWrite;
//             ShareAccess->SharedDelete += SharedDelete;
//         }
//     }
//
//     /* Validation successful */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoRemoveShareAccess(IN PFILE_OBJECT FileObject,
//                     IN PSHARE_ACCESS ShareAccess)
// {
//     PAGED_CODE();
//
//     /* Check if the file has an extension */
//     if (FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION)
//     {
//         /* Check if caller specified to ignore access checks */
//         //if (FileObject->FoExtFlags & IO_IGNORE_SHARE_ACCESS_CHECK)
//         {
//             /* Don't update share access */
//             return;
//         }
//     }
//
//     /* Otherwise, check if there's any access present */
//     if ((FileObject->ReadAccess) ||
//         (FileObject->WriteAccess) ||
//         (FileObject->DeleteAccess))
//     {
//         /* Decrement the open count */
//         ShareAccess->OpenCount--;
//
//         /* Remove share access */
//         ShareAccess->Readers -= FileObject->ReadAccess;
//         ShareAccess->Writers -= FileObject->WriteAccess;
//         ShareAccess->Deleters -= FileObject->DeleteAccess;
//         ShareAccess->SharedRead -= FileObject->SharedRead;
//         ShareAccess->SharedWrite -= FileObject->SharedWrite;
//         ShareAccess->SharedDelete -= FileObject->SharedDelete;
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoSetShareAccess(IN ACCESS_MASK DesiredAccess,
//                  IN ULONG DesiredShareAccess,
//                  IN PFILE_OBJECT FileObject,
//                  OUT PSHARE_ACCESS ShareAccess)
// {
//     BOOLEAN ReadAccess;
//     BOOLEAN WriteAccess;
//     BOOLEAN DeleteAccess;
//     BOOLEAN SharedRead;
//     BOOLEAN SharedWrite;
//     BOOLEAN SharedDelete;
//     BOOLEAN Update = TRUE;
//     PAGED_CODE();
//
//     ReadAccess = (DesiredAccess & (FILE_READ_DATA | FILE_EXECUTE)) != 0;
//     WriteAccess = (DesiredAccess & (FILE_WRITE_DATA | FILE_APPEND_DATA)) != 0;
//     DeleteAccess = (DesiredAccess & DELETE) != 0;
//
//     /* Check if the file has an extension */
//     if (FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION)
//     {
//         /* Check if caller specified to ignore access checks */
//         //if (FileObject->FoExtFlags & IO_IGNORE_SHARE_ACCESS_CHECK)
//         {
//             /* Don't update share access */
//             Update = FALSE;
//         }
//     }
//
//     /* Update basic access */
//     FileObject->ReadAccess = ReadAccess;
//     FileObject->WriteAccess = WriteAccess;
//     FileObject->DeleteAccess = DeleteAccess;
//
//     /* Check if we have no access as all */
//     if (!(ReadAccess) && !(WriteAccess) && !(DeleteAccess))
//     {
//         /* Check if we need to update the structure */
//         if (!Update) return;
//
//         /* Otherwise, clear data */
//         ShareAccess->OpenCount = 0;
//         ShareAccess->Readers = 0;
//         ShareAccess->Writers = 0;
//         ShareAccess->Deleters = 0;
//         ShareAccess->SharedRead = 0;
//         ShareAccess->SharedWrite = 0;
//         ShareAccess->SharedDelete = 0;
//     }
//     else
//     {
//         /* Calculate shared access */
//         SharedRead = (DesiredShareAccess & FILE_SHARE_READ) != 0;
//         SharedWrite = (DesiredShareAccess & FILE_SHARE_WRITE) != 0;
//         SharedDelete = (DesiredShareAccess & FILE_SHARE_DELETE) != 0;
//
//         /* Set it in the FO */
//         FileObject->SharedRead = SharedRead;
//         FileObject->SharedWrite = SharedWrite;
//         FileObject->SharedDelete = SharedDelete;
//
//         /* Check if we need to update the structure */
//         if (!Update) return;
//
//         /* Otherwise, set data */
//         ShareAccess->OpenCount = 1;
//         ShareAccess->Readers = ReadAccess;
//         ShareAccess->Writers = WriteAccess;
//         ShareAccess->Deleters = DeleteAccess;
//         ShareAccess->SharedRead = SharedRead;
//         ShareAccess->SharedWrite = SharedWrite;
//         ShareAccess->SharedDelete = SharedDelete;
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoCancelFileOpen(IN PDEVICE_OBJECT DeviceObject,
//                  IN PFILE_OBJECT FileObject)
// {
//     PIRP Irp;
//     KEVENT Event;
//     KIRQL OldIrql;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION Stack;
//
//     /* Check if handles were already created for the
//      * open file. If so, that's over.
//      */
//     if (FileObject->Flags & FO_HANDLE_CREATED)
//         KeBugCheckEx(INVALID_CANCEL_OF_FILE_OPEN,
//                      (ULONG_PTR)FileObject,
//                      (ULONG_PTR)DeviceObject, 0, 0);
//
//     /* Reset the events */
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//     KeClearEvent(&FileObject->Event);
//
//     /* Allocate the IRP we'll use */
//     Irp = IopAllocateIrpMustSucceed(DeviceObject->StackSize);
//     /* Properly set it */
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//     Irp->UserEvent = &Event;
//     Irp->UserIosb = &Irp->IoStatus;
//     Irp->Overlay.AsynchronousParameters.UserApcRoutine = NULL;
//     Irp->Tail.Overlay.OriginalFileObject = FileObject;
//     Irp->RequestorMode = KernelMode;
//     Irp->Flags = IRP_CLOSE_OPERATION | IRP_SYNCHRONOUS_API;
//
//     Stack = IoGetNextIrpStackLocation(Irp);
//     Stack->MajorFunction = IRP_MJ_CLEANUP;
//     Stack->FileObject = FileObject;
//
//     /* Put on top of IRPs list of the thread */
//     IopQueueIrpToThread(Irp);
//
//     /* Call the driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, UserRequest,
//                               KernelMode, FALSE, NULL);
//     }
//
//     /* Remove from IRPs list */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//     IopUnQueueIrpFromThread(Irp);
//     KeLowerIrql(OldIrql);
//
//     /* Free the IRP */
//     IoFreeIrp(Irp);
//
//     /* Clear the event */
//     KeClearEvent(&FileObject->Event);
//     /* And finally, mark the open operation as canceled */
//     FileObject->Flags |= FO_FILE_OPEN_CANCELLED;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoQueryFileDosDeviceName(IN PFILE_OBJECT FileObject,
//                          OUT POBJECT_NAME_INFORMATION *ObjectNameInformation)
// {
//     NTSTATUS Status;
//     ULONG Length, ReturnLength;
//     POBJECT_NAME_INFORMATION LocalInfo;
//
//     /* Start with a buffer length of 200 */
//     ReturnLength = 200;
//     /*
//      * We'll loop until query works.
//      * We will use returned length for next loop
//      * iteration, trying to have a big enough buffer.
//      */
//     for (Length = 200; ; Length = ReturnLength)
//     {
//         /* Allocate our work buffer */
//         LocalInfo = ExAllocatePoolWithTag(PagedPool, Length, 'nDoI');
//         if (LocalInfo == NULL)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Query the DOS name */
//         Status = IopQueryNameInternal(FileObject,
//                                       TRUE,
//                                       TRUE,
//                                       LocalInfo,
//                                       Length,
//                                       &ReturnLength,
//                                       KernelMode);
//         /* If it succeed, nothing more to do */
//         if (Status == STATUS_SUCCESS)
//         {
//             break;
//         }
//
//         /* Otherwise, prepare for re-allocation */
//         ExFreePoolWithTag(LocalInfo, 'nDoI');
//
//         /*
//          * If we failed because of something else
//          * than memory, simply stop and fail here
//          */
//         if (Status != STATUS_BUFFER_OVERFLOW)
//         {
//             return Status;
//         }
//     }
//
//     /* Success case here: return our buffer */
//     *ObjectNameInformation = LocalInfo;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoSetFileOrigin(IN PFILE_OBJECT FileObject,
//                 IN BOOLEAN Remote)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     BOOLEAN FlagSet;
//
//     /* Get the flag status */
//     FlagSet = FileObject->Flags & FO_REMOTE_ORIGIN ? TRUE : FALSE;
//
//     /* Don't set the flag if it was set already, and don't remove it if it wasn't set */
//     if (Remote && !FlagSet)
//     {
//         /* Set the flag */
//         FileObject->Flags |= FO_REMOTE_ORIGIN;
//     }
//     else if (!Remote && FlagSet)
//     {
//         /* Remove the flag */
//         FileObject->Flags &= ~FO_REMOTE_ORIGIN;
//     }
//     else
//     {
//         /* Fail */
//         Status = STATUS_INVALID_PARAMETER_MIX;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateFile(PHANDLE FileHandle,
//              ACCESS_MASK DesiredAccess,
//              POBJECT_ATTRIBUTES ObjectAttributes,
//              PIO_STATUS_BLOCK IoStatusBlock,
//              PLARGE_INTEGER AllocateSize,
//              ULONG FileAttributes,
//              ULONG ShareAccess,
//              ULONG CreateDisposition,
//              ULONG CreateOptions,
//              PVOID EaBuffer,
//              ULONG EaLength)
// {
//     NTSTATUS Status;
//     /* Call the I/O Function */
//     Status = IoCreateFile(FileHandle,
//                         DesiredAccess,
//                         ObjectAttributes,
//                         IoStatusBlock,
//                         AllocateSize,
//                         FileAttributes,
//                         ShareAccess,
//                         CreateDisposition,
//                         CreateOptions,
//                         EaBuffer,
//                         EaLength,
//                         CreateFileTypeNone,
//                         NULL,
//                         0);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtCreateMailslotFile(OUT PHANDLE FileHandle,
//                      IN ACCESS_MASK DesiredAccess,
//                      IN POBJECT_ATTRIBUTES ObjectAttributes,
//                      OUT PIO_STATUS_BLOCK IoStatusBlock,
//                      IN ULONG CreateOptions,
//                      IN ULONG MailslotQuota,
//                      IN ULONG MaxMessageSize,
//                      IN PLARGE_INTEGER TimeOut)
// {
//     MAILSLOT_CREATE_PARAMETERS Buffer;
//     PAGED_CODE();
//
//     /* Check for Timeout */
//     if (TimeOut)
//     {
//         /* check if the call came from user mode */
//         if (KeGetPreviousMode() != KernelMode)
//         {
//             /* Enter SEH for Probe */
//             _SEH2_TRY
//             {
//                 /* Probe the timeout */
//                 Buffer.ReadTimeout = ProbeForReadLargeInteger(TimeOut);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return the exception code */
//                 _SEH2_YIELD(return _SEH2_GetExceptionCode());
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             /* Otherwise, capture directly */
//             Buffer.ReadTimeout = *TimeOut;
//         }
//
//         /* Set the correct setting */
//         Buffer.TimeoutSpecified = TRUE;
//     }
//     else
//     {
//         /* Tell the FSD we don't have a timeout */
//         Buffer.TimeoutSpecified = FALSE;
//     }
//
//     /* Set Settings */
//     Buffer.MailslotQuota = MailslotQuota;
//     Buffer.MaximumMessageSize = MaxMessageSize;
//
//     /* Call I/O */
//     return IoCreateFile(FileHandle,
//                         DesiredAccess,
//                         ObjectAttributes,
//                         IoStatusBlock,
//                         NULL,
//                         0,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_CREATE,
//                         CreateOptions,
//                         NULL,
//                         0,
//                         CreateFileTypeMailslot,
//                         (PVOID)&Buffer,
//                         0);
// }
//
// NTSTATUS
// NTAPI
// NtCreateNamedPipeFile(OUT PHANDLE FileHandle,
//                       IN ACCESS_MASK DesiredAccess,
//                       IN POBJECT_ATTRIBUTES ObjectAttributes,
//                       OUT PIO_STATUS_BLOCK IoStatusBlock,
//                       IN ULONG ShareAccess,
//                       IN ULONG CreateDisposition,
//                       IN ULONG CreateOptions,
//                       IN ULONG NamedPipeType,
//                       IN ULONG ReadMode,
//                       IN ULONG CompletionMode,
//                       IN ULONG MaximumInstances,
//                       IN ULONG InboundQuota,
//                       IN ULONG OutboundQuota,
//                       IN PLARGE_INTEGER DefaultTimeout)
// {
//     NAMED_PIPE_CREATE_PARAMETERS Buffer;
//     PAGED_CODE();
//
//     /* Check for Timeout */
//     if (DefaultTimeout)
//     {
//         /* check if the call came from user mode */
//         if (KeGetPreviousMode() != KernelMode)
//         {
//             /* Enter SEH for Probe */
//             _SEH2_TRY
//             {
//                 /* Probe the timeout */
//                 Buffer.DefaultTimeout =
//                     ProbeForReadLargeInteger(DefaultTimeout);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return the exception code */
//                 _SEH2_YIELD(return _SEH2_GetExceptionCode());
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             /* Otherwise, capture directly */
//             Buffer.DefaultTimeout = *DefaultTimeout;
//         }
//
//         /* Set the correct setting */
//         Buffer.TimeoutSpecified = TRUE;
//     }
//     else
//     {
//         /* Tell the FSD we don't have a timeout */
//         Buffer.TimeoutSpecified = FALSE;
//     }
//
//     /* Set Settings */
//     Buffer.NamedPipeType = NamedPipeType;
//     Buffer.ReadMode = ReadMode;
//     Buffer.CompletionMode = CompletionMode;
//     Buffer.MaximumInstances = MaximumInstances;
//     Buffer.InboundQuota = InboundQuota;
//     Buffer.OutboundQuota = OutboundQuota;
//
//     /* Call I/O */
//     return IoCreateFile(FileHandle,
//                         DesiredAccess,
//                         ObjectAttributes,
//                         IoStatusBlock,
//                         NULL,
//                         0,
//                         ShareAccess,
//                         CreateDisposition,
//                         CreateOptions,
//                         NULL,
//                         0,
//                         CreateFileTypeNamedPipe,
//                         (PVOID)&Buffer,
//                         0);
// }
//
// NTSTATUS
// NTAPI
// NtFlushWriteBuffer(VOID)
// {
//     PAGED_CODE();
//
//     /* Call the kernel */
//     KeFlushWriteBuffer();
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenFile(OUT PHANDLE FileHandle,
//            IN ACCESS_MASK DesiredAccess,
//            IN POBJECT_ATTRIBUTES ObjectAttributes,
//            OUT PIO_STATUS_BLOCK IoStatusBlock,
//            IN ULONG ShareAccess,
//            IN ULONG OpenOptions)
// {
//     NTSTATUS Status;
//     /* Call the I/O Function */
//     Status = IoCreateFile(FileHandle,
//                         DesiredAccess,
//                         ObjectAttributes,
//                         IoStatusBlock,
//                         NULL,
//                         0,
//                         ShareAccess,
//                         FILE_OPEN,
//                         OpenOptions,
//                         NULL,
//                         0,
//                         CreateFileTypeNone,
//                         NULL,
//                         0);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryAttributesFile(IN POBJECT_ATTRIBUTES ObjectAttributes,
//                       OUT PFILE_BASIC_INFORMATION FileInformation)
// {
//     /* Call the internal helper API */
//     return IopQueryAttributesFile(ObjectAttributes,
//                                   FileBasicInformation,
//                                   sizeof(FILE_BASIC_INFORMATION),
//                                   FileInformation);
// }
//
// NTSTATUS
// NTAPI
// NtQueryFullAttributesFile(IN POBJECT_ATTRIBUTES ObjectAttributes,
//                           OUT PFILE_NETWORK_OPEN_INFORMATION FileInformation)
// {
//     /* Call the internal helper API */
//     return IopQueryAttributesFile(ObjectAttributes,
//                                   FileNetworkOpenInformation,
//                                   sizeof(FILE_NETWORK_OPEN_INFORMATION),
//                                   FileInformation);
// }
//
// /**
//  * @name NtCancelIoFile
//  *
//  * Cancel all pending I/O operations in the current thread for specified
//  * file object.
//  *
//  * @param FileHandle
//  *        Handle to file object to cancel requests for. No specific
//  *        access rights are needed.
//  * @param IoStatusBlock
//  *        Pointer to status block which is filled with final completition
//  *        status on successful return.
//  *
//  * @return Status.
//  *
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCancelIoFile(IN HANDLE FileHandle,
//                OUT PIO_STATUS_BLOCK IoStatusBlock)
// {
//     PFILE_OBJECT FileObject;
//     PETHREAD Thread;
//     PIRP Irp;
//     KIRQL OldIrql;
//     BOOLEAN OurIrpsInList = FALSE;
//     LARGE_INTEGER Interval;
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     NTSTATUS Status;
//     PLIST_ENTRY ListHead, NextEntry;
//     PAGED_CODE();
//     IOTRACE(IO_API_DEBUG, "FileHandle: %p\n", FileHandle);
//
//     /* Check the previous mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the I/O Status Block */
//             ProbeForWriteIoStatusBlock(IoStatusBlock);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Reference the file object */
//     Status = ObReferenceObjectByHandle(FileHandle,
//                                        0,
//                                        IoFileObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&FileObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* IRP cancellations are synchronized at APC_LEVEL. */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Get the current thread */
//     Thread = PsGetCurrentThread();
//
//     /* Update the operation counts */
//     IopUpdateOperationCount(IopOtherTransfer);
//
//     /* Loop the list */
//     ListHead = &Thread->IrpList;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the IRP and check if the File Object matches */
//         Irp = CONTAINING_RECORD(NextEntry, IRP, ThreadListEntry);
//         if (Irp->Tail.Overlay.OriginalFileObject == FileObject)
//         {
//             /* Cancel this IRP and keep looping */
//             IoCancelIrp(Irp);
//             OurIrpsInList = TRUE;
//         }
//
//         /* Go to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Lower the IRQL */
//     KeLowerIrql(OldIrql);
//
//     /* Check if we had found an IRP */
//     if (OurIrpsInList)
//     {
//         /* Setup a 10ms wait */
//         Interval.QuadPart = -100000;
//
//         /* Start looping */
//         while (OurIrpsInList)
//         {
//             /* Do the wait */
//             KeDelayExecutionThread(KernelMode, FALSE, &Interval);
//             OurIrpsInList = FALSE;
//
//             /* Raise IRQL */
//             KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//             /* Now loop the list again */
//             NextEntry = ListHead->Flink;
//             while (NextEntry != ListHead)
//             {
//                 /* Get the IRP and check if the File Object matches */
//                 Irp = CONTAINING_RECORD(NextEntry, IRP, ThreadListEntry);
//                 if (Irp->Tail.Overlay.OriginalFileObject == FileObject)
//                 {
//                     /* Keep looping */
//                     OurIrpsInList = TRUE;
//                     break;
//                 }
//
//                 /* Go to the next entry */
//                 NextEntry = NextEntry->Flink;
//             }
//
//             /* Lower the IRQL */
//             KeLowerIrql(OldIrql);
//         }
//     }
//
//     /* Enter SEH for writing back the I/O Status */
//     _SEH2_TRY
//     {
//         /* Write success */
//         IoStatusBlock->Status = STATUS_SUCCESS;
//         IoStatusBlock->Information = 0;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Ignore exception */
//     }
//     _SEH2_END;
//
//     /* Dereference the file object and return success */
//     ObDereferenceObject(FileObject);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtDeleteFile(IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     NTSTATUS Status;
//     DUMMY_FILE_OBJECT LocalFileObject;
//     HANDLE Handle;
//     KPROCESSOR_MODE AccessMode = KeGetPreviousMode();
//     OPEN_PACKET OpenPacket;
//     PAGED_CODE();
//     IOTRACE(IO_API_DEBUG, "FileMame: %wZ\n", ObjectAttributes->ObjectName);
//
//     /* Setup the Open Packet */
//     RtlZeroMemory(&OpenPacket, sizeof(OPEN_PACKET));
//     OpenPacket.Type = IO_TYPE_OPEN_PACKET as i16;
//     OpenPacket.Size = sizeof(OPEN_PACKET);
//     OpenPacket.CreateOptions = FILE_DELETE_ON_CLOSE;
//     OpenPacket.ShareAccess = FILE_SHARE_READ |
//                              FILE_SHARE_WRITE |
//                              FILE_SHARE_DELETE;
//     OpenPacket.Disposition = FILE_OPEN;
//     OpenPacket.DeleteOnly = TRUE;
//     OpenPacket.LocalFileObject = &LocalFileObject;
//
//     /* Update the operation counts */
//     IopUpdateOperationCount(IopOtherTransfer);
//
//     /*
//      * Attempt opening the file. This will call the I/O Parse Routine for
//      * the File Object (IopParseDevice) which will use the dummy file obejct
//      * send the IRP to its device object. Note that we have two statuses
//      * to worry about: the Object Manager's status (in Status) and the I/O
//      * status, which is in the Open Packet's Final Status, and determined
//      * by the Parse Check member.
//      */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 NULL,
//                                 AccessMode,
//                                 NULL,
//                                 DELETE,
//                                 &OpenPacket,
//                                 &Handle);
//     if (OpenPacket.ParseCheck == FALSE) return Status;
//
//     /* Retrn the Io status */
//     return OpenPacket.FinalStatus;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
