use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    _CREATE_FILE_TYPE_CreateFileTypeNone,
    _WORK_QUEUE_TYPE_DelayedWorkQueue,
    _FILE_INFORMATION_CLASS_FileAllocationInformation,
    _FILE_INFORMATION_CLASS_FileEndOfFileInformation,
    _FILE_INFORMATION_CLASS_FileNameInformation,
    _FILE_INFORMATION_CLASS_FileReparsePointInformation,
    _FILE_INFORMATION_CLASS_FileStandardInformation,
    _KWAIT_REASON_Executive,
    FILE_ALLOCATION_INFORMATION, FILE_END_OF_FILE_INFORMATION,
    FILE_NAME_INFORMATION, FILE_REPARSE_POINT_INFORMATION, FILE_STANDARD_INFORMATION,
    _IO_STATUS_BLOCK, IoAllocateWorkItem, IoCreateFile, IoFreeWorkItem,
    IoGetDeviceObjectPointer, IoQueueWorkItem, IoSetThreadHardErrorMode,
    KEVENT, KeInitializeEvent, KeReleaseSemaphore, LARGE_INTEGER,
    _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES,
    PDEVICE_OBJECT, PFILE_NAME_INFORMATION, PFILE_OBJECT,
    PFILE_REPARSE_POINT_INFORMATION, PHANDLE, PREPARSE_DATA_BUFFER, PUNICODE_STRING,
    RTL_QUERY_REGISTRY_TABLE,
    RtlAppendUnicodeStringToString, RtlCompareMemory,
    RtlCreateSystemVolumeInformationFolder, RtlDeleteRegistryValue,
    RtlEqualUnicodeString, RtlInitUnicodeString, RtlWriteRegistryValue,
    ZwClose, ZwFsControlFile, ZwOpenEvent, ZwOpenFile, ZwQueryDirectoryFile,
    ZwQueryInformationFile, ZwReadFile, ZwSetInformationFile,
    ZwWaitForSingleObject, ZwWriteFile,
    DEVICE_OBJECT, HANDLE, _UNICODE_STRING,
};
use core::ffi::c_void;

// NTAPI kernel exports — extern "system" (stdcall on i386, _Name@N).
unsafe extern "system" {
    fn KeWaitForSingleObject( Object: *mut core::ffi::c_void, WaitReason: i32, WaitMode: i32, Alertable: u8, Timeout: *const LARGE_INTEGER, ) -> NTSTATUS;
    fn RtlQueryRegistryValues( RelativeTo: u32, Path: *const u16, QueryTable: *mut RTL_QUERY_REGISTRY_TABLE, Context: *mut core::ffi::c_void, Environment: *mut core::ffi::c_void, ) -> NTSTATUS;
}

// Aliases so translated code can use the short names
const FileStandardInformation: u32 = _FILE_INFORMATION_CLASS_FileStandardInformation;
const FileEndOfFileInformation: u32 = _FILE_INFORMATION_CLASS_FileEndOfFileInformation;
const FileAllocationInformation: u32 = _FILE_INFORMATION_CLASS_FileAllocationInformation;
const FileNameInformation: u32 = _FILE_INFORMATION_CLASS_FileNameInformation;
const FileReparsePointInformation: u32 = _FILE_INFORMATION_CLASS_FileReparsePointInformation;
const CreateFileTypeNone: u32 = _CREATE_FILE_TYPE_CreateFileTypeNone;

extern "C" {
    fn MountMgrCreatePointWorker(DeviceExtension: *mut DEVICE_EXTENSION, SymbolicString: *const _UNICODE_STRING, DeviceName: *const _UNICODE_STRING) -> u32;
    static mut Unloading: i32;
    static mut UnloadEvent: KEVENT;
    static mut SafeVolumes: _UNICODE_STRING;
    static ReparseIndex: _UNICODE_STRING;
    fn IsUniqueIdPresent( DeviceExtension: *mut DEVICE_EXTENSION, DatabaseEntry: *mut DATABASE_ENTRY, ) -> BOOLEAN;
    fn UpdateReplicatedUniqueIds( ListDeviceInfo: *mut DEVICE_INFORMATION, DatabaseEntry: *mut DATABASE_ENTRY, );
}
extern "C" {
    fn DPRINT1(format: *const i8, ...);
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;

const DATABASE_PATH_WSTR: [u16; 40] = [
    0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
    0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C,
    0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x004D, 0x006F,
    0x0075, 0x006E, 0x0074, 0x0065, 0x0064, 0x0044, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x0000,
];

#[no_mangle]
pub static mut DatabasePath: *mut u16 = unsafe {
    &DATABASE_PATH_WSTR as *const [u16; 40] as *const u16 as *mut u16
};

const OFFLINE_PATH_WSTR: [u16; 48] = [
    0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
    0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C,
    0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x004D, 0x006F,
    0x0075, 0x006E, 0x0074, 0x0065, 0x0064, 0x0044, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x004F, 0x0066, 0x0066, 0x006C, 0x0069,
    0x006E, 0x0065, 0x0000,
];

#[no_mangle]
pub static mut OfflinePath: *mut u16 = unsafe {
    &OFFLINE_PATH_WSTR as *const [u16; 48] as *const u16 as *mut u16
};

const REMOTE_DB_WSTR: [u16; 59] = [
    0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0020, 0x0056,
    0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0020, 0x0049, 0x006E, 0x0066,
    0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x005C,
    0x004D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0050, 0x006F, 0x0069, 0x006E,
    0x0074, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0052,
    0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0044, 0x0061, 0x0074, 0x0061,
    0x0062, 0x0061, 0x0073, 0x0065, 0x0000,
];

#[no_mangle]
pub static mut RemoteDatabase: _UNICODE_STRING = _UNICODE_STRING {
    Length: 116,
    MaximumLength: 116,
    Buffer: unsafe { &REMOTE_DB_WSTR as *const [u16; 59] as *const u16 as *mut u16 },
};

const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as i32;

const REG_BINARY: u32 = 3;


const MAXIMUM_REPARSE_DATA_BUFFER_SIZE: u32 = 16384;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00000020;
const FILE_OPEN_BY_FILE_ID: u32 = 0x2000;
const FILE_OPEN_REPARSE_POINT: u32 = 0x00200000;
const FSCTL_GET_REPARSE_POINT: u32 = 0x000900A8;
const OBJ_KERNEL_HANDLE: u32 = 0x00000200;
const OBJ_CASE_INSENSITIVE: u32 = 0x00000040;

#[repr(C)]
struct MOUNT_POINT_REPARSE_BUFFER {
    SubstituteNameOffset: u16,
    SubstituteNameLength: u16,
    PrintNameOffset: u16,
    PrintNameLength: u16,
    PathBuffer: u16,
}

#[repr(C)]
struct REPARSE_DATA_BUFFER {
    ReparseTag: u32,
    ReparseDataLength: u16,
    Reserved: u16,
    MountPointReparseBuffer: MOUNT_POINT_REPARSE_BUFFER,
}

const UNICODE_NULL: u16 = 0;
const STATUS_STOPPED_ON_SYMLINK: NTSTATUS = 0x8000002Du32 as i32;
const IO_STOP_ON_SYMLINK: u32 = 0x08;
const IO_NO_PARAMETER_CHECKING: u32 = 0x0100;
const FILE_NON_DIRECTORY_FILE: u32 = 0x00000040;
const FILE_SYNCHRONOUS_IO_ALERT: u32 = 0x00000010;
const FILE_CREATE: u32 = 2;

const RTL_REGISTRY_ABSOLUTE: u32 = 0;

const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA0000003;
const FILE_GENERIC_READ: u32 = 0x00120089;
const FILE_SHARE_READ: u32 = 0x00000001;
const FILE_SHARE_WRITE: u32 = 0x00000002;
const DO_UNLOAD_PENDING: u32 = 0x0001;
// Shadow ntkrnl_sys::constants::IO_NO_INCREMENT (i8) with i32 to match KPRIORITY.
#[allow(non_upper_case_globals)]
const IO_NO_INCREMENT: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn GetRemoteDatabaseSize(Database: HANDLE) -> i32 {
    let mut IoStatusBlock = core::mem::MaybeUninit::<_IO_STATUS_BLOCK>::uninit();
    let mut StandardInfo = core::mem::MaybeUninit::<FILE_STANDARD_INFORMATION>::uninit();
    let Status: i32;

    Status = ZwQueryInformationFile(
        Database,
        IoStatusBlock.as_mut_ptr(),
        StandardInfo.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
        FileStandardInformation,
    );

    if NT_SUCCESS(Status) {
        return StandardInfo.assume_init().EndOfFile.__bindgen_anon_1.LowPart as i32;
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn AddRemoteDatabaseEntry(
    Database: HANDLE,
    Entry: *mut DATABASE_ENTRY,
) -> NTSTATUS {
    let mut Size: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    Size.QuadPart = GetRemoteDatabaseSize(Database) as i64;

    ZwWriteFile(
        Database,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        Entry as *mut core::ffi::c_void,
        (*Entry).EntrySize,
        core::ptr::addr_of_mut!(Size),
        core::ptr::null_mut(),
    )
}
#[no_mangle]
pub unsafe extern "C" fn CloseRemoteDatabase(Database: HANDLE) -> NTSTATUS {
    ZwClose(Database)
}
#[no_mangle]
pub unsafe extern "C" fn TruncateRemoteDatabase(
    Database: HANDLE,
    NewSize: LONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut EndOfFile: FILE_END_OF_FILE_INFORMATION;
    let mut Allocation: FILE_ALLOCATION_INFORMATION;

    // Initialize the structures with zero-initialized memory
    IoStatusBlock = core::mem::zeroed();
    EndOfFile = core::mem::zeroed();
    Allocation = core::mem::zeroed();

    EndOfFile.EndOfFile.QuadPart = NewSize as i64;
    Allocation.AllocationSize.QuadPart = NewSize as i64;

    Status = ZwSetInformationFile(
        Database,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut EndOfFile as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as u32,
        FileEndOfFileInformation,
    );

    if NT_SUCCESS(Status) {
        Status = ZwSetInformationFile(
            Database,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut Allocation as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_ALLOCATION_INFORMATION>() as u32,
            FileAllocationInformation,
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn GetRemoteDatabaseEntry(Database: *mut core::ffi::c_void, StartingOffset: i32) -> *mut DATABASE_ENTRY {
    let mut Status: i32;
    let mut EntrySize: u32 = core::mem::zeroed();
    let mut Entry: *mut DATABASE_ENTRY;
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    /* Get the entry at the given position */
    ByteOffset.QuadPart = StartingOffset as i64;
    Status = ZwReadFile(Database,
                        core::ptr::null_mut(),
                        None,
                        core::ptr::null_mut(),
                        &mut IoStatusBlock,
                        &mut EntrySize as *mut u32 as *mut core::ffi::c_void,
                        core::mem::size_of::<u32>() as u32,
                        &mut ByteOffset,
                        core::ptr::null_mut());
    if Status < 0 {
        return core::ptr::null_mut();
    }

    /* If entry doesn't exist, truncate database */
    if EntrySize == 0 {
        TruncateRemoteDatabase(Database, StartingOffset);
        return core::ptr::null_mut();
    }

    /* Allocate the entry */
    Entry = AllocatePool(EntrySize) as *mut DATABASE_ENTRY;
    if Entry.is_null() {
        return core::ptr::null_mut();
    }

    /* Effectively read the entry */
    Status = ZwReadFile(Database,
                        core::ptr::null_mut(),
                        None,
                        core::ptr::null_mut(),
                        &mut IoStatusBlock,
                        Entry as *mut core::ffi::c_void,
                        EntrySize,
                        &mut ByteOffset,
                        core::ptr::null_mut());
    /* If it fails or returns inconsistent data, drop it (= truncate) */
    if Status < 0 || (IoStatusBlock.Information as u32) != EntrySize || (EntrySize as usize) < core::mem::size_of::<DATABASE_ENTRY>() {
        TruncateRemoteDatabase(Database, StartingOffset);
        FreePool(Entry as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    /* Validate entry */
    let sym_sum = (*Entry).SymbolicNameOffset.wrapping_add((*Entry).SymbolicNameLength);
    let uid_sum = (*Entry).UniqueIdOffset.wrapping_add((*Entry).UniqueIdLength);
    let max_offset = if sym_sum > uid_sum { sym_sum } else { uid_sum };
    if max_offset as u32 > EntrySize {
        TruncateRemoteDatabase(Database, StartingOffset);
        FreePool(Entry as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    return Entry;
}
#[no_mangle]
pub unsafe extern "C" fn WriteRemoteDatabaseEntry(
    Database: HANDLE,
    Offset: i32,
    Entry: *mut DATABASE_ENTRY,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    ByteOffset.QuadPart = Offset as i64;

    Status = ZwWriteFile(
        Database,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        Entry as *mut core::ffi::c_void,
        (*Entry).EntrySize as u32,
        &mut ByteOffset,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(Status) {
        if (IoStatusBlock.Information as u32) < ((*Entry).EntrySize as u32) {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn DeleteRemoteDatabaseEntry(
    Database: HANDLE,
    StartingOffset: LONG,
) -> NTSTATUS {
    let mut EndSize: u32;
    let mut TmpBuffer: *mut core::ffi::c_void;
    let mut Status: NTSTATUS;
    let mut DatabaseSize: u32;
    let mut Entry: *mut DATABASE_ENTRY;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut EndEntriesOffset: LARGE_INTEGER = core::mem::zeroed();

    // First, get database size
    let db_size = GetRemoteDatabaseSize(Database);
    if db_size == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    DatabaseSize = db_size as u32;

    // Then, get the entry to remove
    Entry = GetRemoteDatabaseEntry(Database, StartingOffset);
    if Entry.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    // Validate parameters: ensure we won't get zero or negative size
    if ((*Entry).EntrySize as u32).wrapping_add(StartingOffset as u32) >= DatabaseSize {
        // If we get invalid parameters, truncate the whole database
        // starting the wrong entry. We can't rely on the rest
        FreePool(Entry as *mut core::ffi::c_void);
        return TruncateRemoteDatabase(Database, StartingOffset);
    }

    // Now, get the size of the remaining entries (those after the one to remove)
    EndSize = DatabaseSize - (*Entry).EntrySize - (StartingOffset as u32);
    // Allocate a buffer big enough to hold them
    TmpBuffer = AllocatePool(EndSize);
    if TmpBuffer.is_null() {
        FreePool(Entry as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Get the offset of the entry right after the one to delete
    EndEntriesOffset.QuadPart = ((*Entry).EntrySize + StartingOffset as u32) as i64;
    // We don't need the entry any more
    FreePool(Entry as *mut core::ffi::c_void);

    // Read the ending entries
    Status = ZwReadFile(
        Database,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        TmpBuffer,
        EndSize,
        &mut EndEntriesOffset,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        FreePool(TmpBuffer);
        return Status;
    }

    // Ensure nothing went wrong - we don't want to corrupt the DB
    if IoStatusBlock.Information as u32 != EndSize {
        FreePool(TmpBuffer);
        return STATUS_INVALID_PARAMETER;
    }

    // Remove the entry
    Status = TruncateRemoteDatabase(Database, ((StartingOffset as u32) + EndSize) as LONG);
    if Status < 0 {
        FreePool(TmpBuffer);
        return Status;
    }

    // Now, shift the ending entries to erase the entry
    EndEntriesOffset.QuadPart = StartingOffset as i64;
    Status = ZwWriteFile(
        Database,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        TmpBuffer,
        EndSize,
        &mut EndEntriesOffset,
        core::ptr::null_mut(),
    );

    FreePool(TmpBuffer);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFromLocalDatabaseRoutine(
    ValueName: *mut u16,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let UniqueId = Context as *mut MOUNTDEV_UNIQUE_ID;

    let _ = ValueType;
    let _ = EntryContext;

    if ((*UniqueId).UniqueIdLength as u32 == ValueLength)
        && (RtlCompareMemory(
            core::ptr::addr_of!((*UniqueId).UniqueId) as *const core::ffi::c_void,
            ValueData,
            ValueLength as _,
        ) == ValueLength as _)
    {
        RtlDeleteRegistryValue(
            0, // RTL_REGISTRY_ABSOLUTE
            DatabasePath as *const u16,
            ValueName as *const u16,
        );
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn DeleteFromLocalDatabase(
    SymbolicLink: PUNICODE_STRING,
    UniqueId: PMOUNTDEV_UNIQUE_ID,
) {
    // RTL_QUERY_REGISTRY_TABLE QueryTable[2];
    // RtlZeroMemory(QueryTable, sizeof(QueryTable));
    let mut query_table = core::mem::MaybeUninit::<[RTL_QUERY_REGISTRY_TABLE; 2]>::uninit();
    core::ptr::write_bytes(
        query_table.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );
    let mut query_table = query_table.assume_init();

    // QueryTable[0].QueryRoutine = DeleteFromLocalDatabaseRoutine;
    query_table[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(DeleteFromLocalDatabaseRoutine));

    // QueryTable[0].Name = SymbolicLink->Buffer;
    query_table[0].Name = (*SymbolicLink).Buffer;

    // RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
    //                        DatabasePath,
    //                        QueryTable,
    //                        UniqueId,
    //                        NULL);
    let _ = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        DatabasePath as *const u16,
        &mut query_table[0] as *mut RTL_QUERY_REGISTRY_TABLE,
        UniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn WaitForRemoteDatabaseSemaphore(DeviceExtension: *mut DEVICE_EXTENSION) -> NTSTATUS {
    let Timeout: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0xFA0A1F00_i64 };

    let lock_ptr = core::ptr::addr_of_mut!((*DeviceExtension).RemoteDatabaseLock) as *mut core::ffi::c_void;

    let Status = KeWaitForSingleObject(
        lock_ptr,
        _KWAIT_REASON_Executive as i32,
        0i32,
        0u8, // FALSE
        core::ptr::addr_of!(Timeout),
    );

    if Status != STATUS_TIMEOUT {
        return Status;
    }

    STATUS_IO_TIMEOUT
}
#[no_mangle]
pub unsafe extern "C" fn ReleaseRemoteDatabaseSemaphore(DeviceExtension: *mut DEVICE_EXTENSION) {
    KeReleaseSemaphore(core::ptr::addr_of_mut!((*DeviceExtension).RemoteDatabaseLock), IO_NO_INCREMENT, 1, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn QueryUniqueIdQueryRoutine(
    ValueName: PWSTR,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let _ = ValueName;
    let _ = ValueType;
    let _ = EntryContext;

    if ValueLength >= 0x10000u32 {
        return 0;
    }

    let IntUniqueId = AllocatePool(core::mem::size_of::<*mut MOUNTDEV_UNIQUE_ID>() as u32 + ValueLength ) as *mut MOUNTDEV_UNIQUE_ID;
    if !IntUniqueId.is_null() {
        (*IntUniqueId).UniqueIdLength = ValueLength as u16;
        core::ptr::copy_nonoverlapping(
            ValueData as *const u8,
            core::ptr::addr_of_mut!((*IntUniqueId).UniqueId) as *mut u8,
            ValueLength as usize,
        );
        let UniqueId = Context as *mut *mut MOUNTDEV_UNIQUE_ID;
        *UniqueId = IntUniqueId;
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn QueryUniqueIdFromMaster(
    DeviceExtension: *mut DEVICE_EXTENSION,
    SymbolicName: *mut _UNICODE_STRING,
    UniqueId: *mut *mut MOUNTDEV_UNIQUE_ID,
) -> i32 {
    unsafe {
        let Status: i32;
        let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
        let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();

        QueryTable[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(QueryUniqueIdQueryRoutine));
        QueryTable[0].Name = (*SymbolicName).Buffer;

        *UniqueId = core::ptr::null_mut();
        let _ = RtlQueryRegistryValues(
            RTL_REGISTRY_ABSOLUTE,
            DatabasePath as *const u16,
            core::ptr::addr_of_mut!(QueryTable[0]) as *mut RTL_QUERY_REGISTRY_TABLE,
            UniqueId as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );

        if !(*UniqueId).is_null() {
            return STATUS_SUCCESS;
        }

        Status = FindDeviceInfo(
            DeviceExtension,
            SymbolicName,
            0u8,
            core::ptr::addr_of_mut!(DeviceInformation),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        *UniqueId = AllocatePool(
            (*(*DeviceInformation).UniqueId).UniqueIdLength as u32
                + core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as u32,
        ) as *mut MOUNTDEV_UNIQUE_ID;
        if (*UniqueId).is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*(*UniqueId)).UniqueIdLength = (*(*DeviceInformation).UniqueId).UniqueIdLength;
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*(*DeviceInformation).UniqueId).UniqueId) as *const u8,
            core::ptr::addr_of_mut!((*(*UniqueId)).UniqueId) as *mut u8,
            (*(*UniqueId)).UniqueIdLength as usize,
        );

        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn WriteUniqueIdToMaster(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DatabaseEntry: *const DATABASE_ENTRY,
) -> NTSTATUS {
    let symbolic_name_size = ((*DatabaseEntry).SymbolicNameLength as usize)
        .wrapping_add(core::mem::size_of::<u16>());
    let symbolic_name = AllocatePool(symbolic_name_size as u32) as *mut u8;
    if symbolic_name.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let src_bytes: *const u8 = (DatabaseEntry as *const u8)
        .offset((*DatabaseEntry).SymbolicNameOffset as isize);
    core::ptr::copy_nonoverlapping(
        src_bytes,
        symbolic_name,
        (*DatabaseEntry).SymbolicNameLength as usize,
    );

    let null_offset = (*DatabaseEntry).SymbolicNameLength as usize / core::mem::size_of::<u16>();
    let symbolic_name_wide = symbolic_name as *mut u16;
    *symbolic_name_wide.add(null_offset) = 0;

    let status = RtlWriteRegistryValue(
        RTL_REGISTRY_ABSOLUTE,
        DatabasePath as *const u16,
        symbolic_name_wide as *const u16,
        REG_BINARY,
        (DatabaseEntry as *const u8)
            .offset((*DatabaseEntry).UniqueIdOffset as isize) as *mut core::ffi::c_void,
        (*DatabaseEntry).UniqueIdLength as u32,
    );

    FreePool(symbolic_name as *mut core::ffi::c_void);

    let mut symbolic_string: _UNICODE_STRING = core::mem::zeroed();
    symbolic_string.Length = (*DatabaseEntry).SymbolicNameLength;
    symbolic_string.MaximumLength = (*DatabaseEntry).SymbolicNameLength;
    symbolic_string.Buffer = (DatabaseEntry as *const u8)
        .offset((*DatabaseEntry).SymbolicNameOffset as isize) as *mut u16;

    let list_head: *mut LIST_ENTRY =
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead);
    let mut next_entry: *mut LIST_ENTRY = (*list_head).Flink;
    let mut device_information: *mut DEVICE_INFORMATION = core::ptr::null_mut();

    loop {
        if next_entry == list_head {
            break;
        }
        device_information = ((next_entry as *const u8)
            .offset(-(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry) as isize)))
            as *mut DEVICE_INFORMATION;

        if (*(*device_information).UniqueId).UniqueIdLength
            != (*DatabaseEntry).UniqueIdLength
        {
            next_entry = (*next_entry).Flink;
            continue;
        }

        if RtlCompareMemory(
            (DatabaseEntry as *const u8)
                .offset((*DatabaseEntry).UniqueIdOffset as isize) as *const core::ffi::c_void,
            core::ptr::addr_of!((*(*device_information).UniqueId).UniqueId) as *const core::ffi::c_void,
            (*DatabaseEntry).UniqueIdLength as _,
        ) == (*DatabaseEntry).UniqueIdLength as _
        {
            break;
        }

        next_entry = (*next_entry).Flink;
    }

    if next_entry != list_head {
        MountMgrCreatePointWorker(
            DeviceExtension,
            &symbolic_string as *const _UNICODE_STRING,
            &(*device_information).DeviceName as *const _UNICODE_STRING,
        );
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn ReconcileThisDatabaseWithMasterWorker(Parameter: *mut core::ffi::c_void) {
    let mut Offset: u32;
    let mut Status: NTSTATUS;
    let mut FileObject: PFILE_OBJECT = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::mem::zeroed();
    let mut UniqueId: *mut MOUNTDEV_UNIQUE_ID = core::mem::zeroed();
    let mut DatabaseEntry: *mut DATABASE_ENTRY;
    let mut DatabaseHandle: HANDLE;
    let mut Handle: HANDLE = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ListDeviceInfo: *mut DEVICE_INFORMATION = core::mem::zeroed();
    let mut Entry: PLIST_ENTRY;
    let mut EntryInfo: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut AssociatedDevice: *mut ASSOCIATED_DEVICE_ENTRY;
    let mut HardwareErrors: u8;
    let mut Restart: u8;
    let mut FailedFinding: u8;
    let mut FileNameBuffer: [u16; 8] = core::mem::zeroed();
    let mut SymbolicNameBuffer: [u16; 100] = core::mem::zeroed();
    let mut ReparseFile: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut SymbolicName: _UNICODE_STRING = core::mem::zeroed();
    let mut VolumeName: _UNICODE_STRING = core::mem::zeroed();
    let mut ReparsePointInformation: FILE_REPARSE_POINT_INFORMATION = core::mem::zeroed();
    let mut SavedReparsePointInformation: FILE_REPARSE_POINT_INFORMATION = core::mem::zeroed();
    let DeviceExtension = (*(Parameter as *const RECONCILE_WORK_ITEM_CONTEXT)).DeviceExtension;
    let DeviceInformation = (*(Parameter as *const RECONCILE_WORK_ITEM_CONTEXT)).DeviceInformation;

    if Unloading != 0 {
        return;
    }

    if !NT_SUCCESS(WaitForRemoteDatabaseSemaphore(DeviceExtension)) {
        return;
    }

    // Use a labeled block so all "goto XYZ" cleanup jumps are valid.
    // break 'release_rds exits here; ReleaseRemoteDatabaseSemaphore runs below.
    'release_rds: {

    if Unloading != 0 {
        break 'release_rds;
    }

    KeWaitForSingleObject(
        &mut (*DeviceExtension).DeviceLock as *mut _ as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive as i32,
        0i32,
        0u8,
        core::ptr::null_mut(),
    );
    Entry = (*DeviceExtension).DeviceListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        ListDeviceInfo = (Entry as *const u8)
            .sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;
        if ListDeviceInfo == DeviceInformation {
            break;
        }
        Entry = (*Entry).Flink;
    }

    if Entry == core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) || (*DeviceInformation).Removable != 0 {
        KeReleaseSemaphore(
            &mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            0u8,
        );
        break 'release_rds;
    }

    Status = IoGetDeviceObjectPointer(
        &(*ListDeviceInfo).DeviceName as *const _UNICODE_STRING as *mut _UNICODE_STRING,
        FILE_READ_ATTRIBUTES,
        &mut FileObject,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        KeReleaseSemaphore(
            &mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            0u8,
        );
        break 'release_rds;
    }

    if ((*DeviceObject).Flags & DO_UNLOAD_PENDING) == 0 {
        InterlockedExchangeAdd(
            &mut (*ListDeviceInfo).MountState as *mut i32,
            1,
        );
    }

    ObDereferenceObject(FileObject as *mut c_void);

    (*DeviceInformation).NeedsReconcile = 1;
    (*DeviceInformation).NoDatabase = 1;
    FailedFinding = 0;

    Entry = (*DeviceExtension).DeviceListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        ListDeviceInfo = (Entry as *const u8)
            .sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;

        EntryInfo = (*ListDeviceInfo).AssociatedDevicesHead.Flink;
        while EntryInfo != core::ptr::addr_of_mut!((*ListDeviceInfo).AssociatedDevicesHead) {
            AssociatedDevice = (EntryInfo as *const u8)
                .sub(core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry))
                as *mut ASSOCIATED_DEVICE_ENTRY;
            NextEntry = (*EntryInfo).Flink;

            if (*AssociatedDevice).DeviceInformation == DeviceInformation {
                RemoveEntryList(&mut (*AssociatedDevice).AssociatedDevicesEntry);
                FreePool((*AssociatedDevice).String.Buffer as *mut core::ffi::c_void);
                FreePool(AssociatedDevice as *mut core::ffi::c_void);
            }

            EntryInfo = NextEntry;
        }
        Entry = (*Entry).Flink;
    }

    DatabaseHandle = OpenRemoteDatabase(DeviceInformation, 0u8);

    ReparseFile.Length = 0;
    ReparseFile.MaximumLength = (*DeviceInformation).DeviceName.Length
        + ReparseIndex.Length
        + core::mem::size_of::<u16>() as u16;
    ReparseFile.Buffer = AllocatePool(ReparseFile.MaximumLength as u32) as *mut u16;
    if ReparseFile.Buffer.is_null() {
        if !DatabaseHandle.is_null() {
            CloseRemoteDatabase(DatabaseHandle);
        }
        KeReleaseSemaphore(
            &mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            0u8,
        );
        break 'release_rds;
    }

    RtlAppendUnicodeStringToString(&mut ReparseFile, &(*DeviceInformation).DeviceName);
    RtlAppendUnicodeStringToString(&mut ReparseFile, &ReparseIndex);
    *ReparseFile.Buffer.add((ReparseFile.Length / 2) as usize) = UNICODE_NULL;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &ReparseFile as *const _UNICODE_STRING as *mut _UNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    HardwareErrors = IoSetThreadHardErrorMode(0u8);
    Status = ZwOpenFile(
        &mut Handle,
        FILE_GENERIC_READ,
        &ObjectAttributes as *const _OBJECT_ATTRIBUTES as *mut _OBJECT_ATTRIBUTES,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_ALERT,
    );
    IoSetThreadHardErrorMode(HardwareErrors);

    FreePool(ReparseFile.Buffer as *mut core::ffi::c_void);

    if !NT_SUCCESS(Status) {
        if !DatabaseHandle.is_null() {
            TruncateRemoteDatabase(DatabaseHandle, 0);
            CloseRemoteDatabase(DatabaseHandle);
        }
        KeReleaseSemaphore(
            &mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            0u8,
        );
        break 'release_rds;
    }

    core::ptr::write_bytes(
        FileNameBuffer.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of::<[u16; 8]>(),
    );
    FileName.Buffer = FileNameBuffer.as_mut_ptr();
    FileName.Length = core::mem::size_of::<[u16; 8]>() as u16;
    FileName.MaximumLength = core::mem::size_of::<[u16; 8]>() as u16;
    *(FileNameBuffer.as_mut_ptr() as *mut u32) = IO_REPARSE_TAG_MOUNT_POINT;
    Status = ZwQueryDirectoryFile(
        Handle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        &mut ReparsePointInformation as *mut FILE_REPARSE_POINT_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_REPARSE_POINT_INFORMATION>() as u32,
        FileReparsePointInformation,
        1u8,
        &FileName as *const _UNICODE_STRING as *mut _UNICODE_STRING,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(Handle);
        if !DatabaseHandle.is_null() {
            TruncateRemoteDatabase(DatabaseHandle, 0);
            CloseRemoteDatabase(DatabaseHandle);
        }
        KeReleaseSemaphore(
            &mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            0u8,
        );
        break 'release_rds;
    }

    if DatabaseHandle.is_null() {
        DatabaseHandle = OpenRemoteDatabase(DeviceInformation, 1u8);
        if DatabaseHandle.is_null() {
            KeReleaseSemaphore(
                &mut (*DeviceExtension).DeviceLock,
                IO_NO_INCREMENT,
                1,
                0u8,
            );
            break 'release_rds;
        }
    }

    KeReleaseSemaphore(
        &mut (*DeviceExtension).DeviceLock,
        IO_NO_INCREMENT,
        1,
        0u8,
    );

    // Nested labeled cleanup blocks implementing the C goto-chain pattern.
    // break 'X exits that block; cleanup code runs sequentially after each block.
    // Normal exit path: break 'close_rdb (skips all inner cleanup, runs close_rdb cleanup).
    'close_rdb: {
    'close_reparse: {
    'free_volume: {
    'free_db_entry: {
    'release_device_lock: {
    'free_unique_id: {

    // ---- inner main code (goto-chain body) ----
    Offset = 0;
    loop {
        DatabaseEntry = GetRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
        if DatabaseEntry.is_null() {
            break;
        }

        (*DatabaseEntry).EntryReferences = 0;
        Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset as i32, DatabaseEntry);
        if !NT_SUCCESS(Status) {
            FreePool(DatabaseEntry as *mut core::ffi::c_void);
            break 'close_reparse;
        }

        Offset += (*DatabaseEntry).EntrySize;
        FreePool(DatabaseEntry as *mut core::ffi::c_void);
    }

    SymbolicName.MaximumLength = core::mem::size_of::<[u16; 100]>() as u16;
    SymbolicName.Length = 0;
    SymbolicName.Buffer = SymbolicNameBuffer.as_mut_ptr();
    Restart = 1;

    loop {
        core::ptr::copy_nonoverlapping(
            &ReparsePointInformation as *const FILE_REPARSE_POINT_INFORMATION as *const u8,
            &mut SavedReparsePointInformation as *mut FILE_REPARSE_POINT_INFORMATION as *mut u8,
            core::mem::size_of::<FILE_REPARSE_POINT_INFORMATION>(),
        );
        Status = ZwQueryDirectoryFile(
            Handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            &mut ReparsePointInformation as *mut FILE_REPARSE_POINT_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_REPARSE_POINT_INFORMATION>() as u32,
            FileReparsePointInformation,
            1u8,
            if Restart != 0 { &FileName as *const _UNICODE_STRING as *mut _UNICODE_STRING } else { core::ptr::null_mut() },
            Restart,
        );
        if Restart != 0 {
            Restart = 0;
        } else {
            if ReparsePointInformation.FileReference == SavedReparsePointInformation.FileReference &&
                ReparsePointInformation.Tag == SavedReparsePointInformation.Tag
            {
                break;
            }
        }

        if !NT_SUCCESS(Status) || Unloading != 0 || ReparsePointInformation.Tag != IO_REPARSE_TAG_MOUNT_POINT {
            break;
        }

        Status = QueryVolumeName(
            Handle,
            &ReparsePointInformation as *const _ as *mut FILE_REPARSE_POINT_INFORMATION,
            core::ptr::null_mut(),
            &mut SymbolicName,
            &mut VolumeName,
        );
        if !NT_SUCCESS(Status) {
            continue;
        }

        Offset = 0;
        loop {
            let mut DbName: _UNICODE_STRING = core::mem::zeroed();

            DatabaseEntry = GetRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
            if DatabaseEntry.is_null() {
                break;
            }

            DbName.MaximumLength = (*DatabaseEntry).SymbolicNameLength;
            DbName.Length = DbName.MaximumLength;
            DbName.Buffer = (DatabaseEntry as *const u8)
                .add((*DatabaseEntry).SymbolicNameOffset as usize) as *mut u16;
            if RtlEqualUnicodeString(&DbName, &SymbolicName, 1u8) != 0 {
                break;
            }

            Offset += (*DatabaseEntry).EntrySize;
            FreePool(DatabaseEntry as *mut core::ffi::c_void);
        }

        if !DatabaseEntry.is_null() {
            if (*DatabaseEntry).EntryReferences != 0 {
                (*DatabaseEntry).EntryReferences += 1;
                Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset as i32, DatabaseEntry);
                if !NT_SUCCESS(Status) {
                    break 'free_db_entry;
                }
                FreePool(DatabaseEntry as *mut core::ffi::c_void);
            } else {
                KeWaitForSingleObject(
                    &mut (*DeviceExtension).DeviceLock as *mut _ as *mut core::ffi::c_void,
                    _KWAIT_REASON_Executive as i32,
                    0i32,
                    0u8,
                    core::ptr::null_mut(),
                );
                Status = QueryUniqueIdFromMaster(DeviceExtension, &mut SymbolicName, &mut UniqueId);
                if !NT_SUCCESS(Status) {
                    Status = WriteUniqueIdToMaster(DeviceExtension, DatabaseEntry);
                    if !NT_SUCCESS(Status) {
                        break 'release_device_lock;
                    }

                    (*DatabaseEntry).EntryReferences += 1;
                    Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset as i32, DatabaseEntry);
                    if !NT_SUCCESS(Status) {
                        break 'release_device_lock;
                    }

                    KeReleaseSemaphore(
                        &mut (*DeviceExtension).DeviceLock,
                        IO_NO_INCREMENT,
                        1,
                        0u8,
                    );
                    FreePool(DatabaseEntry as *mut core::ffi::c_void);
                } else if (*UniqueId).UniqueIdLength == (*DatabaseEntry).UniqueIdLength &&
                    RtlCompareMemory(
                        core::ptr::addr_of!((*UniqueId).UniqueId) as *const core::ffi::c_void,
                        (DatabaseEntry as *const u8).add((*DatabaseEntry).UniqueIdOffset as usize) as *const core::ffi::c_void,
                        (*UniqueId).UniqueIdLength as _,
                    ) as u32 == (*UniqueId).UniqueIdLength as u32
                {
                    (*DatabaseEntry).EntryReferences += 1;
                    Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset as i32, DatabaseEntry);
                    if !NT_SUCCESS(Status) {
                        break 'free_unique_id;
                    }

                    FreePool(UniqueId as *mut core::ffi::c_void);
                    KeReleaseSemaphore(
                        &mut (*DeviceExtension).DeviceLock,
                        IO_NO_INCREMENT,
                        1,
                        0u8,
                    );
                    FreePool(DatabaseEntry as *mut core::ffi::c_void);
                } else if IsUniqueIdPresent(DeviceExtension, DatabaseEntry) != 0 {
                    Status = WriteUniqueIdToMaster(DeviceExtension, DatabaseEntry);
                    if !NT_SUCCESS(Status) {
                        break 'free_unique_id;
                    }

                    (*DatabaseEntry).EntryReferences += 1;
                    Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset as i32, DatabaseEntry);
                    if !NT_SUCCESS(Status) {
                        break 'free_unique_id;
                    }

                    FreePool(UniqueId as *mut core::ffi::c_void);
                    KeReleaseSemaphore(
                        &mut (*DeviceExtension).DeviceLock,
                        IO_NO_INCREMENT,
                        1,
                        0u8,
                    );
                    FreePool(DatabaseEntry as *mut core::ffi::c_void);
                } else {
                    Status = DeleteRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
                    if !NT_SUCCESS(Status) {
                        break 'free_unique_id;
                    }

                    FreePool(DatabaseEntry as *mut core::ffi::c_void);
                    DatabaseEntry = AllocatePool(
                        (*UniqueId).UniqueIdLength as u32 + SymbolicName.Length as u32 + core::mem::size_of::<DATABASE_ENTRY>() as u32,
                    ) as *mut DATABASE_ENTRY;
                    if DatabaseEntry.is_null() {
                        break 'free_unique_id;
                    }

                    (*DatabaseEntry).EntrySize = (*UniqueId).UniqueIdLength as u32
                        + SymbolicName.Length as u32
                        + core::mem::size_of::<DATABASE_ENTRY>() as u32;
                    (*DatabaseEntry).EntryReferences = 1;
                    (*DatabaseEntry).SymbolicNameOffset = core::mem::size_of::<DATABASE_ENTRY>() as u16;
                    (*DatabaseEntry).SymbolicNameLength = SymbolicName.Length;
                    (*DatabaseEntry).UniqueIdOffset = (SymbolicName.Length as u32 + core::mem::size_of::<DATABASE_ENTRY>() as u32) as u16;
                    (*DatabaseEntry).UniqueIdLength = (*UniqueId).UniqueIdLength;
                    core::ptr::copy_nonoverlapping(
                        SymbolicName.Buffer as *const u8,
                        (DatabaseEntry as *mut u8).add((*DatabaseEntry).SymbolicNameOffset as usize),
                        (*DatabaseEntry).SymbolicNameLength as usize,
                    );
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*UniqueId).UniqueId) as *const u8,
                        (DatabaseEntry as *mut u8).add((*DatabaseEntry).UniqueIdOffset as usize),
                        (*UniqueId).UniqueIdLength as usize,
                    );

                    Status = AddRemoteDatabaseEntry(DatabaseHandle, DatabaseEntry);
                    if !NT_SUCCESS(Status) {
                        FreePool(DatabaseEntry as *mut core::ffi::c_void);
                        break 'free_unique_id;
                    }

                    FreePool(UniqueId as *mut core::ffi::c_void);
                    FreePool(DatabaseEntry as *mut core::ffi::c_void);
                    KeReleaseSemaphore(
                        &mut (*DeviceExtension).DeviceLock,
                        IO_NO_INCREMENT,
                        1,
                        0u8,
                    );
                }
            }
        } else {
            KeWaitForSingleObject(
                &mut (*DeviceExtension).DeviceLock as *mut _ as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive as i32,
                0i32,
                0u8,
                core::ptr::null_mut(),
            );
            Status = QueryUniqueIdFromMaster(DeviceExtension, &mut SymbolicName, &mut UniqueId);
            KeReleaseSemaphore(
                &mut (*DeviceExtension).DeviceLock,
                IO_NO_INCREMENT,
                1,
                0u8,
            );
            if NT_SUCCESS(Status) {
                DatabaseEntry = AllocatePool(
                    (*UniqueId).UniqueIdLength as u32 + SymbolicName.Length as u32 + core::mem::size_of::<DATABASE_ENTRY>() as u32,
                ) as *mut DATABASE_ENTRY;
                if !DatabaseEntry.is_null() {
                    (*DatabaseEntry).EntrySize = (*UniqueId).UniqueIdLength as u32
                        + SymbolicName.Length as u32
                        + core::mem::size_of::<DATABASE_ENTRY>() as u32;
                    (*DatabaseEntry).EntryReferences = 1;
                    (*DatabaseEntry).SymbolicNameOffset = core::mem::size_of::<DATABASE_ENTRY>() as u16;
                    (*DatabaseEntry).SymbolicNameLength = SymbolicName.Length;
                    (*DatabaseEntry).UniqueIdOffset = (SymbolicName.Length as u32 + core::mem::size_of::<DATABASE_ENTRY>() as u32) as u16;
                    (*DatabaseEntry).UniqueIdLength = (*UniqueId).UniqueIdLength;
                    core::ptr::copy_nonoverlapping(
                        SymbolicName.Buffer as *const u8,
                        (DatabaseEntry as *mut u8).add((*DatabaseEntry).SymbolicNameOffset as usize),
                        (*DatabaseEntry).SymbolicNameLength as usize,
                    );
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*UniqueId).UniqueId) as *const u8,
                        (DatabaseEntry as *mut u8).add((*DatabaseEntry).UniqueIdOffset as usize),
                        (*UniqueId).UniqueIdLength as usize,
                    );

                    Status = AddRemoteDatabaseEntry(DatabaseHandle, DatabaseEntry);
                    FreePool(DatabaseEntry as *mut core::ffi::c_void);
                    FreePool(UniqueId as *mut core::ffi::c_void);

                    if !NT_SUCCESS(Status) {
                        break 'free_volume;
                    }
                } else {
                    FreePool(UniqueId as *mut core::ffi::c_void);
                }
            }
        }

        KeWaitForSingleObject(
            &mut (*DeviceExtension).DeviceLock as *mut _ as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive as i32,
            0i32,
            0u8,
            core::ptr::null_mut(),
        );
        Status = FindDeviceInfo(
            DeviceExtension,
            &SymbolicName,
            0u8,
            &mut ListDeviceInfo,
        );
        if !NT_SUCCESS(Status) {
            FailedFinding = 1;
            FreePool(VolumeName.Buffer as *mut core::ffi::c_void);
        } else {
            AssociatedDevice = AllocatePool(core::mem::size_of::<ASSOCIATED_DEVICE_ENTRY>() as u32)
                as *mut ASSOCIATED_DEVICE_ENTRY;
            if AssociatedDevice.is_null() {
                FreePool(VolumeName.Buffer as *mut core::ffi::c_void);
            } else {
                (*AssociatedDevice).DeviceInformation = DeviceInformation;
                (*AssociatedDevice).String.Length = VolumeName.Length;
                (*AssociatedDevice).String.MaximumLength = VolumeName.MaximumLength;
                (*AssociatedDevice).String.Buffer = VolumeName.Buffer;
                InsertTailList(
                    &mut (*ListDeviceInfo).AssociatedDevicesHead,
                    &mut (*AssociatedDevice).AssociatedDevicesEntry,
                );
            }

            if (*ListDeviceInfo).SkipNotifications == 0 {
                PostOnlineNotification(DeviceExtension, &(*ListDeviceInfo).SymbolicName);
            }
        }
        KeReleaseSemaphore(
            &mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            0u8,
        );
    }

    ZwClose(Handle);

    KeWaitForSingleObject(
        &mut (*DeviceExtension).DeviceLock as *mut _ as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive as i32,
        0i32,
        0u8,
        core::ptr::null_mut(),
    );
    Entry = (*DeviceExtension).DeviceListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        ListDeviceInfo = (Entry as *const u8)
            .sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;
        if ListDeviceInfo == DeviceInformation {
            break;
        }
        Entry = (*Entry).Flink;
    }

    if Entry == core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        ListDeviceInfo = core::ptr::null_mut();
    }

    Offset = 0;
    loop {
        DatabaseEntry = GetRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
        if DatabaseEntry.is_null() {
            break;
        }

        if (*DatabaseEntry).EntryReferences == 0 {
            Status = DeleteRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
            if !NT_SUCCESS(Status) {
                FreePool(DatabaseEntry as *mut core::ffi::c_void);
                KeReleaseSemaphore(
                    &mut (*DeviceExtension).DeviceLock,
                    IO_NO_INCREMENT,
                    1,
                    0u8,
                );
                break 'close_rdb;
            }
        } else {
            if !ListDeviceInfo.is_null() {
                UpdateReplicatedUniqueIds(ListDeviceInfo, DatabaseEntry);
            }
            Offset += (*DatabaseEntry).EntrySize;
        }

        FreePool(DatabaseEntry as *mut core::ffi::c_void);
    }

    if !ListDeviceInfo.is_null() && FailedFinding == 0 {
        (*DeviceInformation).NoDatabase = 0;
    }

    KeReleaseSemaphore(
        &mut (*DeviceExtension).DeviceLock,
        IO_NO_INCREMENT,
        1,
        0u8,
    );

    break 'close_rdb; // normal exit: go to close_rdb cleanup

    } // end 'free_unique_id
    // 'free_unique_id cleanup:
    if !UniqueId.is_null() { FreePool(UniqueId as *mut core::ffi::c_void); }
    } // end 'release_device_lock
    // 'release_device_lock cleanup:
    KeReleaseSemaphore(&mut (*DeviceExtension).DeviceLock, IO_NO_INCREMENT, 1, 0u8);
    } // end 'free_db_entry
    // 'free_db_entry cleanup:
    if !DatabaseEntry.is_null() { FreePool(DatabaseEntry as *mut core::ffi::c_void); }
    } // end 'free_volume
    // 'free_volume cleanup:
    FreePool(VolumeName.Buffer as *mut core::ffi::c_void);
    } // end 'close_reparse
    // 'close_reparse cleanup:
    ZwClose(Handle);
    } // end 'close_rdb
    // 'close_rdb cleanup:
    CloseRemoteDatabase(DatabaseHandle);

    } // end 'release_rds labeled block

    ReleaseRemoteDatabaseSemaphore(DeviceExtension);
}

#[no_mangle]
pub unsafe extern "system" fn WorkerThread(
    DeviceObject: *mut DEVICE_OBJECT,
    Context: *mut core::ffi::c_void,
) {
    unsafe {
        let mut i: u32 = 0;
        let mut Event: KEVENT = core::mem::zeroed();
        let mut OldIrql: KIRQL = 0;
        let mut Status: NTSTATUS;
        let mut SafeEvent: *mut core::ffi::c_void = core::mem::zeroed();
        let mut Entry: PLIST_ENTRY;
        let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
        let mut WorkItem: *mut RECONCILE_WORK_ITEM;
        let DeviceExtension: *mut DEVICE_EXTENSION;
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

        UNREFERENCED_PARAMETER!(DeviceObject as *mut core::ffi::c_void);

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = (0x40 | 0x200) as u32; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(SafeVolumes);
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
        Timeout.QuadPart = -10000000i64; /* Wait for 1 second */

        i = 0;
        loop {
            if Unloading != 0 {
                i = 999;
            } else {
                Status = ZwOpenEvent(
                    core::ptr::addr_of_mut!(SafeEvent),
                    0x001F0003, // EVENT_ALL_ACCESS
                    core::ptr::addr_of_mut!(ObjectAttributes),
                );
                if Status >= 0 {
                    break;
                }
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                    _KWAIT_REASON_Executive as i32,
                    0i32,
                    0u8,
                    core::ptr::addr_of_mut!(Timeout),
                );
            }
            i += 1;
            if i >= 1000 {
                break;
            }
        }

        if i < 1000 {
            loop {
                Status = ZwWaitForSingleObject(
                    SafeEvent,
                    0u8,
                    core::ptr::addr_of_mut!(Timeout),
                );
                if !(Status == STATUS_TIMEOUT && Unloading == 0) {
                    break;
                }
            }
            ZwClose(SafeEvent);
        }

        DeviceExtension = Context as *mut DEVICE_EXTENSION;

        // InterlockedExchange(&(DeviceExtension->WorkerThreadStatus), 1)
        {
            let ptr = core::ptr::addr_of_mut!((*DeviceExtension).WorkerThreadStatus) as *mut core::sync::atomic::AtomicI32;
            (*ptr).store(1i32, core::sync::atomic::Ordering::SeqCst);
        }

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*DeviceExtension).WorkerSemaphore) as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive as i32,
            0i32,
            0u8,
            core::ptr::null_mut(),
        );
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*DeviceExtension).WorkerLock) as *mut usize,
            core::ptr::addr_of_mut!(OldIrql),
        );

        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).WorkerQueueListHead)) {
            Entry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceExtension).WorkerQueueListHead));
            WorkItem = ((Entry as *const u8).offset(
                -(core::mem::offset_of!(RECONCILE_WORK_ITEM, WorkerQueueListEntry) as isize),
            )) as *mut RECONCILE_WORK_ITEM;

            KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).WorkerLock) as *mut usize, OldIrql);

            if let Some(f) = (*WorkItem).WorkerRoutine {
                f((*WorkItem).Context);
            }

            IoFreeWorkItem((*WorkItem).WorkItem);
            FreePool(WorkItem as *mut core::ffi::c_void);

            if InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).WorkerReferences)) < 0 {
                return;
            }

            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*DeviceExtension).WorkerSemaphore) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive as i32,
                0i32,
                0u8,
                core::ptr::null_mut(),
            );
            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*DeviceExtension).WorkerLock) as *mut usize,
                core::ptr::addr_of_mut!(OldIrql),
            );
        }

        KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).WorkerLock) as *mut usize, OldIrql);

        InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).WorkerReferences));

        KeSetEvent(
            core::ptr::addr_of_mut!(UnloadEvent) as *mut core::ffi::c_void,
            IO_NO_INCREMENT,
            0u8,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn QueueWorkItem(
    DeviceExtension: *mut DEVICE_EXTENSION,
    WorkItem: *mut RECONCILE_WORK_ITEM,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut OldIrql: KIRQL = 0;

    (*WorkItem).Context = Context;

    if InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceExtension).WorkerReferences)) == 0 {
        IoQueueWorkItem(
            (*WorkItem).WorkItem,
            Some(core::mem::transmute::<unsafe extern "system" fn(*mut DEVICE_OBJECT, *mut core::ffi::c_void), _>(WorkerThread)),
            _WORK_QUEUE_TYPE_DelayedWorkQueue,
            DeviceExtension as *mut core::ffi::c_void,
        );
    }

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).WorkerLock) as *mut usize,
        core::ptr::addr_of_mut!(OldIrql),
    );
    InsertTailList(
        core::ptr::addr_of_mut!((*DeviceExtension).WorkerQueueListHead),
        core::ptr::addr_of_mut!((*WorkItem).WorkerQueueListEntry),
    );
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).WorkerLock) as *mut usize,
        OldIrql,
    );

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*DeviceExtension).WorkerSemaphore),
        IO_NO_INCREMENT,
        1,
        0u8,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn QueryVolumeName(
    RootDirectory: HANDLE,
    ReparsePointInformation: *mut FILE_REPARSE_POINT_INFORMATION,
    FileName: *mut _UNICODE_STRING,
    SymbolicName: *mut _UNICODE_STRING,
    VolumeName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut needed_length: u32;
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut file_name_info: *mut FILE_NAME_INFORMATION;
    let mut reparse_data_buffer: *mut REPARSE_DATA_BUFFER;

    if FileName.is_null() {
        let mut reference: _UNICODE_STRING = core::mem::zeroed();
        reference.Length = core::mem::size_of_val(&(*ReparsePointInformation).FileReference) as u16;
        reference.MaximumLength = reference.Length;
        reference.Buffer = core::ptr::addr_of_mut!((*ReparsePointInformation).FileReference) as *mut u16;

        object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        object_attributes.RootDirectory = RootDirectory;
        object_attributes.Attributes = OBJ_KERNEL_HANDLE;
        object_attributes.ObjectName = &mut reference;
        object_attributes.SecurityDescriptor = core::ptr::null_mut();
        object_attributes.SecurityQualityOfService = core::ptr::null_mut();
    } else {
        object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        object_attributes.RootDirectory = core::ptr::null_mut();
        object_attributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
        object_attributes.ObjectName = FileName;
        object_attributes.SecurityDescriptor = core::ptr::null_mut();
        object_attributes.SecurityQualityOfService = core::ptr::null_mut();
    }

    let open_options = if !FileName.is_null() {
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT
    } else {
        FILE_OPEN_BY_FILE_ID | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT
    };
    status = ZwOpenFile(
        &mut handle,
        SYNCHRONIZE | FILE_READ_ATTRIBUTES,
        &object_attributes as *const _OBJECT_ATTRIBUTES as *mut _OBJECT_ATTRIBUTES,
        &mut io_status_block,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        open_options,
    );
    if status < 0 {
        return status;
    }

    reparse_data_buffer = AllocatePool(MAXIMUM_REPARSE_DATA_BUFFER_SIZE) as *mut REPARSE_DATA_BUFFER;
    if reparse_data_buffer.is_null() {
        ZwClose(handle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ZwFsControlFile(
        handle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut io_status_block,
        FSCTL_GET_REPARSE_POINT,
        core::ptr::null_mut(),
        0,
        reparse_data_buffer as *mut core::ffi::c_void,
        MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
    );
    if status < 0 {
        FreePool(reparse_data_buffer as *mut core::ffi::c_void);
        ZwClose(handle);
        return status;
    }

    let substitute_name_length = (*reparse_data_buffer).MountPointReparseBuffer.SubstituteNameLength;
    if substitute_name_length + 2u16 > (*SymbolicName).MaximumLength {
        FreePool(reparse_data_buffer as *mut core::ffi::c_void);
        ZwClose(handle);
        return STATUS_BUFFER_TOO_SMALL;
    }

    (*SymbolicName).Length = substitute_name_length;
    let src = (core::ptr::addr_of!((*reparse_data_buffer).MountPointReparseBuffer.PathBuffer) as *mut u8)
        .add((*reparse_data_buffer).MountPointReparseBuffer.SubstituteNameOffset as usize) as *mut u16;
    RtlCopyMemory(
        (*SymbolicName).Buffer as *mut core::ffi::c_void,
        src as *const core::ffi::c_void,
        substitute_name_length as usize,
    );

    FreePool(reparse_data_buffer as *mut core::ffi::c_void);

    let len = (*SymbolicName).Length as usize;
    let idx = (len / core::mem::size_of::<u16>()) - 1;
    if (*SymbolicName).Buffer.add(idx).read() != 0x005Cu16 {
        ZwClose(handle);
        return STATUS_INVALID_PARAMETER;
    }

    (*SymbolicName).Length -= core::mem::size_of::<u16>() as u16;
    let new_idx = ((*SymbolicName).Length / core::mem::size_of::<u16>() as u16) as usize;
    (*SymbolicName).Buffer.add(new_idx).write(0x0000u16);

    if MOUNTMGR_IS_VOLUME_NAME(SymbolicName) == 0 {
        ZwClose(handle);
        return STATUS_INVALID_PARAMETER;
    }

    let alloc_size = core::mem::size_of::<FILE_NAME_INFORMATION>() + 2 * core::mem::size_of::<u16>();
    file_name_info = AllocatePool(alloc_size as u32) as *mut FILE_NAME_INFORMATION;
    if file_name_info.is_null() {
        ZwClose(handle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ZwQueryInformationFile(
        handle,
        &mut io_status_block,
        file_name_info as *mut core::ffi::c_void,
        alloc_size as u32,
        FileNameInformation,
    );
    if status == STATUS_BUFFER_OVERFLOW {
        needed_length = (*file_name_info).FileNameLength;
        FreePool(file_name_info as *mut core::ffi::c_void);
        file_name_info = AllocatePool(core::mem::size_of::<FILE_NAME_INFORMATION>() as u32 + needed_length) as *mut FILE_NAME_INFORMATION;
        if file_name_info.is_null() {
            ZwClose(handle);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        status = ZwQueryInformationFile(
            handle,
            &mut io_status_block,
            file_name_info as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_NAME_INFORMATION>() as u32 + needed_length,
            FileNameInformation,
        );
    }

    ZwClose(handle);

    if status < 0 {
        return status;
    }

    (*VolumeName).Length = (*file_name_info).FileNameLength as u16;
    (*VolumeName).MaximumLength = ((*file_name_info).FileNameLength + core::mem::size_of::<u16>() as u32) as u16;
    (*VolumeName).Buffer = AllocatePool((*VolumeName).MaximumLength as u32) as *mut u16;
    if (*VolumeName).Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        (*VolumeName).Buffer as *mut core::ffi::c_void,
        core::ptr::addr_of!((*file_name_info).FileName) as *const core::ffi::c_void,
        (*file_name_info).FileNameLength as usize,
    );

    let term_idx = ((*file_name_info).FileNameLength / core::mem::size_of::<u16>() as u32) as usize;
    (*VolumeName).Buffer.add(term_idx).write(0x0000u16);

    FreePool(file_name_info as *mut core::ffi::c_void);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn OnlineMountedVolumes(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DeviceInformation: *mut DEVICE_INFORMATION,
) {
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut restart_scan: i32;
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut object_attributes: _OBJECT_ATTRIBUTES;
    let mut volume_device_information: *mut DEVICE_INFORMATION = core::ptr::null_mut();
    let mut file_name_buffer: [u16; 0x8] = [0u16; 0x8];
    let mut symbolic_name_buffer: [u16; 0x64] = [0u16; 0x64];
    let mut reparse_file: _UNICODE_STRING = core::mem::zeroed();
    let mut file_name: _UNICODE_STRING = core::mem::zeroed();
    let mut symbolic_name: _UNICODE_STRING = core::mem::zeroed();
    let mut volume_name: _UNICODE_STRING = core::mem::zeroed();
    let mut reparse_point_information: FILE_REPARSE_POINT_INFORMATION = core::mem::zeroed();
    let mut saved_reparse_point_information: FILE_REPARSE_POINT_INFORMATION = core::mem::zeroed();

    if (*DeviceInformation).Removable != 0 {
        return;
    }

    reparse_file.Length = 0;
    reparse_file.MaximumLength = (*DeviceInformation).DeviceName.Length
        + ReparseIndex.Length
        + core::mem::size_of::<u16>() as u16;
    reparse_file.Buffer =
        AllocatePool(reparse_file.MaximumLength as u32) as *mut u16;
    if reparse_file.Buffer.is_null() {
        return;
    }

    RtlAppendUnicodeStringToString(
        core::ptr::addr_of_mut!(reparse_file),
        core::ptr::addr_of!((*DeviceInformation).DeviceName),
    );
    RtlAppendUnicodeStringToString(
        core::ptr::addr_of_mut!(reparse_file),
        core::ptr::addr_of!(ReparseIndex),
    );
    *reparse_file
        .Buffer
        .add(reparse_file.Length as usize / core::mem::size_of::<u16>()) = 0u16;

    object_attributes = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName =
        core::ptr::addr_of!(reparse_file) as *mut _UNICODE_STRING;
    object_attributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    status = ZwOpenFile(
        &raw mut handle,
        FILE_GENERIC_READ,
        &raw mut object_attributes,
        &raw mut io_status_block,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_ALERT | FILE_OPEN_REPARSE_POINT,
    );
    FreePool(reparse_file.Buffer as *mut core::ffi::c_void);
    if !NT_SUCCESS(status) {
        (*DeviceInformation).NoDatabase = 0;
        return;
    }

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(file_name_buffer) as *mut u8,
        0,
        core::mem::size_of_val(&file_name_buffer),
    );
    file_name.Buffer = file_name_buffer.as_mut_ptr();
    file_name.Length = core::mem::size_of_val(&file_name_buffer) as u16;
    file_name.MaximumLength = core::mem::size_of_val(&file_name_buffer) as u16;
    *(file_name_buffer.as_mut_ptr() as *mut u32) = IO_REPARSE_TAG_MOUNT_POINT;

    status = ZwQueryDirectoryFile(
        handle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &raw mut io_status_block,
        &raw mut reparse_point_information as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_REPARSE_POINT_INFORMATION>() as u32,
        FileReparsePointInformation,
        1u8,
        &raw mut file_name,
        0u8,
    );
    if !NT_SUCCESS(status) {
        ZwClose(handle);
        return;
    }

    restart_scan = 1;

    loop {
        symbolic_name.Length = 0;
        symbolic_name.MaximumLength = core::mem::size_of_val(&symbolic_name_buffer) as u16;
        symbolic_name.Buffer = symbolic_name_buffer.as_mut_ptr();

        core::ptr::copy_nonoverlapping(
            &raw const reparse_point_information as *const u8,
            &raw mut saved_reparse_point_information as *mut u8,
            core::mem::size_of::<FILE_REPARSE_POINT_INFORMATION>(),
        );

        let file_name_param = if restart_scan != 0 {
            &raw mut file_name
        } else {
            core::ptr::null_mut()
        };
        status = ZwQueryDirectoryFile(
            handle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &raw mut io_status_block,
            &raw mut reparse_point_information as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_REPARSE_POINT_INFORMATION>() as u32,
            FileReparsePointInformation,
            1u8,
            file_name_param,
            (restart_scan != 0) as u8,
        );

        if restart_scan == 0 {
            if reparse_point_information.FileReference == saved_reparse_point_information.FileReference
                && reparse_point_information.Tag == saved_reparse_point_information.Tag
            {
                break;
            }
        } else {
            restart_scan = 0;
        }

        if !NT_SUCCESS(status) || reparse_point_information.Tag != IO_REPARSE_TAG_MOUNT_POINT {
            break;
        }

        status = QueryVolumeName(
            handle,
            &raw mut reparse_point_information,
            core::ptr::null_mut(),
            &raw mut symbolic_name,
            &raw mut volume_name,
        );
        if !NT_SUCCESS(status) {
            continue;
        }

        FreePool(volume_name.Buffer as *mut core::ffi::c_void);

        status = FindDeviceInfo(
            DeviceExtension,
            &raw const symbolic_name as *const _UNICODE_STRING,
            0u8,
            &raw mut volume_device_information,
        );
        if !NT_SUCCESS(status) {
            (*DeviceInformation).NoDatabase = 1;
            continue;
        }

        if (*DeviceInformation).SkipNotifications == 0 {
            PostOnlineNotification(
                DeviceExtension,
                &raw const (*volume_device_information).SymbolicName as *const _UNICODE_STRING,
            );
        }
    }

    ZwClose(handle);
}
#[no_mangle]
pub unsafe extern "C" fn ReconcileThisDatabaseWithMaster(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DeviceInformation: *mut DEVICE_INFORMATION,
) {
    unsafe {
        let WorkItem: *mut RECONCILE_WORK_ITEM;

        if (*DeviceInformation).Removable != 0 {
            return;
        }

        WorkItem = AllocatePool(core::mem::size_of::<RECONCILE_WORK_ITEM>() as u32) as *mut RECONCILE_WORK_ITEM;
        if WorkItem.is_null() {
            return;
        }

        (*WorkItem).WorkItem = IoAllocateWorkItem((*DeviceExtension).DeviceObject);
        if (*WorkItem).WorkItem.is_null() {
            FreePool(WorkItem as *mut core::ffi::c_void);
            return;
        }

        (*WorkItem).WorkerRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut core::ffi::c_void), _>(ReconcileThisDatabaseWithMasterWorker));
        // RECONCILE_WORK_ITEM_CONTEXT holds DeviceExtension+DeviceInformation;
        // allocate separately (C struct embedded it anonymously; bindgen separates them).
        let ctx = AllocatePool(core::mem::size_of::<RECONCILE_WORK_ITEM_CONTEXT>() as u32) as *mut RECONCILE_WORK_ITEM_CONTEXT;
        if ctx.is_null() {
            IoFreeWorkItem((*WorkItem).WorkItem);
            FreePool(WorkItem as *mut core::ffi::c_void);
            return;
        }
        (*ctx).DeviceExtension = DeviceExtension;
        (*ctx).DeviceInformation = DeviceInformation;
        (*WorkItem).Context = ctx as *mut core::ffi::c_void;
        QueueWorkItem(
            DeviceExtension,
            WorkItem,
            ctx as *mut core::ffi::c_void,
        );

        if (*DeviceExtension).WorkerThreadStatus == 0
            && (*DeviceExtension).AutomaticDriveLetter != 0
            && (*DeviceExtension).NoAutoMount != 0
        {
            OnlineMountedVolumes(DeviceExtension, DeviceInformation);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ReconcileAllDatabasesWithMaster(DeviceExtension: *mut DEVICE_EXTENSION) {
    let mut NextEntry: PLIST_ENTRY;
    let mut DeviceInformation: *mut DEVICE_INFORMATION;

    NextEntry = (*DeviceExtension).DeviceListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        DeviceInformation = ((NextEntry as *const u8).sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))) as *mut DEVICE_INFORMATION;
        if (*DeviceInformation).Removable == 0 {
            ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
        }
        NextEntry = (*NextEntry).Flink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CreateRemoteDatabaseWorker(
    DeviceObject: *mut DEVICE_OBJECT,
    Context: *mut core::ffi::c_void,
) {
    let mut Status: NTSTATUS;
    let mut Database: HANDLE = core::ptr::null_mut();
    let mut DatabaseName: _UNICODE_STRING = core::mem::zeroed();
    let WorkItem: *mut MIGRATE_WORK_ITEM;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let DeviceInformation: *mut DEVICE_INFORMATION;

    UNREFERENCED_PARAMETER!(DeviceObject as *mut core::ffi::c_void);

    /* Extract context */
    WorkItem = Context as *mut MIGRATE_WORK_ITEM;
    DeviceInformation = (*WorkItem).DeviceInformation;

    'cleanup: {
        /* Reconstruct appropriate string */
        DatabaseName.Length = 0;
        DatabaseName.MaximumLength = (*DeviceInformation).DeviceName.Length
                                     + RemoteDatabase.Length
                                     + core::mem::size_of::<u16>() as u16;
        DatabaseName.Buffer = AllocatePool(DatabaseName.MaximumLength as u32) as *mut u16;
        if DatabaseName.Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        /* Create the required folder (in which the database will be stored
         * \System Volume Information at root of the volume
         */
        Status = RtlCreateSystemVolumeInformationFolder(&(*DeviceInformation).DeviceName as *const _UNICODE_STRING);
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        /* Finish initiating strings */
        RtlAppendUnicodeStringToString(&mut DatabaseName, &(*DeviceInformation).DeviceName);
        RtlAppendUnicodeStringToString(&mut DatabaseName, &raw const RemoteDatabase);
        *DatabaseName.Buffer.add((DatabaseName.Length / core::mem::size_of::<u16>() as u16) as usize) = UNICODE_NULL;

        /* Create database */
        // InitializeObjectAttributes expanded inline
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.ObjectName = &mut DatabaseName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = IoCreateFile(
            &mut Database,
            SYNCHRONIZE | READ_CONTROL | FILE_WRITE_ATTRIBUTES
                | FILE_READ_ATTRIBUTES | FILE_WRITE_EA | FILE_READ_EA
                | FILE_APPEND_DATA | FILE_WRITE_DATA | FILE_READ_DATA,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
            0,
            FILE_CREATE,
            FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_ALERT,
            core::ptr::null_mut(),
            0,
            CreateFileTypeNone,
            core::ptr::null_mut(),
            IO_STOP_ON_SYMLINK | IO_NO_PARAMETER_CHECKING,
        );
        if !NT_SUCCESS(Status) {
            if Status == STATUS_STOPPED_ON_SYMLINK {
                // DPRINT removed
            }
            Database = core::ptr::null_mut();
            break 'cleanup;
        }
    } // 'cleanup

    if !DatabaseName.Buffer.is_null() {
        FreePool(DatabaseName.Buffer as *mut core::ffi::c_void);
    }
    if NT_SUCCESS(Status) {
        (*DeviceInformation).Migrated = 1;
    } else if !Database.is_null() {
        ZwClose(Database);
    }
    IoFreeWorkItem((*WorkItem).WorkItem);
    (*WorkItem).WorkItem = core::ptr::null_mut();
    (*WorkItem).Status = Status;
    (*WorkItem).Database = Database;
    KeSetEvent((*WorkItem).Event as *mut core::ffi::c_void, IO_NO_INCREMENT, 0u8);
}
#[no_mangle]
pub unsafe extern "C" fn OpenRemoteDatabase(
    DeviceInformation: *mut DEVICE_INFORMATION,
    MigrateDatabase: u8,
) -> *mut core::ffi::c_void {
    const FILE_OPEN_IF: u32 = 3;
    const FILE_OPEN: u32 = 1;

    let mut Database: *mut core::ffi::c_void;
    let mut Status: NTSTATUS;
    let mut PreviousMode: u8;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceRemoteDatabase: _UNICODE_STRING = core::mem::zeroed();

    Database = core::ptr::null_mut();

    DeviceRemoteDatabase.Length = 0u16;
    DeviceRemoteDatabase.MaximumLength = (*DeviceInformation).DeviceName.Length
        .wrapping_add(RemoteDatabase.Length)
        .wrapping_add(2);
    DeviceRemoteDatabase.Buffer = AllocatePool(DeviceRemoteDatabase.MaximumLength as u32) as *mut u16;
    if DeviceRemoteDatabase.Buffer.is_null() {
        return core::ptr::null_mut();
    }

    RtlAppendUnicodeStringToString(
        &mut DeviceRemoteDatabase,
        &(*DeviceInformation).DeviceName as *const _UNICODE_STRING,
    );
    RtlAppendUnicodeStringToString(
        &mut DeviceRemoteDatabase,
        &raw const RemoteDatabase as *const _UNICODE_STRING,
    );
    *DeviceRemoteDatabase.Buffer.add(
        DeviceRemoteDatabase.Length as usize / core::mem::size_of::<u16>(),
    ) = 0u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DeviceRemoteDatabase;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    PreviousMode = IoSetThreadHardErrorMode(0u8);

    Status = IoCreateFile(
        &mut Database,
        SYNCHRONIZE
            | READ_CONTROL
            | FILE_WRITE_ATTRIBUTES
            | FILE_READ_ATTRIBUTES
            | FILE_WRITE_EA
            | FILE_READ_EA
            | FILE_APPEND_DATA
            | FILE_WRITE_DATA
            | FILE_READ_DATA,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
        0,
        if MigrateDatabase == 0 || (*DeviceInformation).Migrated == 0 {
            FILE_OPEN_IF
        } else {
            FILE_OPEN
        },
        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_ALERT,
        core::ptr::null_mut(),
        0,
        CreateFileTypeNone as u32,
        core::ptr::null_mut(),
        IO_STOP_ON_SYMLINK | IO_NO_PARAMETER_CHECKING,
    );

    if Status == STATUS_STOPPED_ON_SYMLINK {
        // DPRINT1("Attempt to exploit CVE-2015-1769. See CORE-10216\n");
    }

    if MigrateDatabase != 0 && Status >= 0 {
        CreateRemoteDatabase(DeviceInformation, &mut Database);
    }

    IoSetThreadHardErrorMode(PreviousMode);
    FreePool(DeviceRemoteDatabase.Buffer as *mut core::ffi::c_void);

    Database
}
#[no_mangle]
pub unsafe extern "C" fn ChangeRemoteDatabaseUniqueId(
    DeviceInformation: *mut DEVICE_INFORMATION,
    OldUniqueId: *const MOUNTDEV_UNIQUE_ID,
    NewUniqueId: *const MOUNTDEV_UNIQUE_ID,
) {
    let mut Offset: i32 = 0;
    let Database: *mut core::ffi::c_void;
    let mut Entry: *mut DATABASE_ENTRY;
    let mut NewEntry: *mut DATABASE_ENTRY;
    let mut Status: i32 = STATUS_SUCCESS as i32;

    Database = OpenRemoteDatabase(DeviceInformation, 0u8);
    if Database.is_null() {
        return;
    }

    loop {
        Entry = GetRemoteDatabaseEntry(Database, Offset);
        if Entry.is_null() {
            break;
        }

        if (*Entry).UniqueIdLength != (*OldUniqueId).UniqueIdLength {
            Offset += (*Entry).EntrySize as i32;
            FreePool(Entry as *mut core::ffi::c_void);
            continue;
        }

        if (RtlCompareMemory(
            core::ptr::addr_of!((*OldUniqueId).UniqueId) as *const core::ffi::c_void,
            (Entry as *const u8).add((*Entry).UniqueIdOffset as usize) as *const core::ffi::c_void,
            (*Entry).UniqueIdLength as _,
        ) as u32)
            != (*Entry).UniqueIdLength as u32
        {
            Offset += (*Entry).EntrySize as i32;
            FreePool(Entry as *mut core::ffi::c_void);
            continue;
        }

        let new_size = (*Entry).EntrySize as usize
            + (*NewUniqueId).UniqueIdLength as usize
            - (*OldUniqueId).UniqueIdLength as usize;
        NewEntry = AllocatePool(new_size as u32) as *mut DATABASE_ENTRY;
        if NewEntry.is_null() {
            Offset += (*Entry).EntrySize as i32;
            FreePool(Entry as *mut core::ffi::c_void);
            continue;
        }

        (*NewEntry).EntrySize = ((*Entry).EntrySize as i32
            + (*NewUniqueId).UniqueIdLength as i32
            - (*OldUniqueId).UniqueIdLength as i32) as u32;
        (*NewEntry).EntryReferences = (*Entry).EntryReferences;
        (*NewEntry).SymbolicNameOffset = core::mem::size_of::<DATABASE_ENTRY>() as u16;
        (*NewEntry).SymbolicNameLength = (*Entry).SymbolicNameLength;
        (*NewEntry).UniqueIdOffset =
            ((*Entry).SymbolicNameLength as usize + core::mem::size_of::<DATABASE_ENTRY>()) as u16;
        (*NewEntry).UniqueIdLength = (*NewUniqueId).UniqueIdLength;

        RtlCopyMemory(
            (NewEntry as *mut u8).add((*NewEntry).SymbolicNameOffset as usize)
                as *mut core::ffi::c_void,
            (Entry as *const u8).add((*Entry).SymbolicNameOffset as usize)
                as *const core::ffi::c_void,
            (*NewEntry).SymbolicNameLength as usize,
        );
        RtlCopyMemory(
            (NewEntry as *mut u8).add((*NewEntry).UniqueIdOffset as usize)
                as *mut core::ffi::c_void,
            core::ptr::addr_of!((*NewUniqueId).UniqueId) as *const core::ffi::c_void,
            (*NewEntry).UniqueIdLength as usize,
        );

        Status = DeleteRemoteDatabaseEntry(Database, Offset);
        if !NT_SUCCESS(Status) {
            FreePool(Entry as *mut core::ffi::c_void);
            FreePool(NewEntry as *mut core::ffi::c_void);
            break;
        }

        Status = AddRemoteDatabaseEntry(Database, NewEntry);
        FreePool(Entry as *mut core::ffi::c_void);
        FreePool(NewEntry as *mut core::ffi::c_void);
    }

    CloseRemoteDatabase(Database);
}

#[no_mangle]
pub unsafe extern "system" fn DeleteDriveLetterRoutine(
    ValueName: *mut u16,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let _ = EntryContext;

    if ValueType != REG_BINARY {
        return STATUS_SUCCESS;
    }

    let UniqueId = Context as *mut MOUNTDEV_UNIQUE_ID;

    if (*UniqueId).UniqueIdLength as u32 != ValueLength {
        return STATUS_SUCCESS;
    }

    if RtlCompareMemory(
        core::ptr::addr_of!((*UniqueId).UniqueId) as *const core::ffi::c_void,
        ValueData,
        ValueLength as _,
    ) != ValueLength as _ {
        return STATUS_SUCCESS;
    }

    let mut RegistryEntry: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut RegistryEntry, ValueName as *const u16);

    if IsDriveLetter(&RegistryEntry) != 0 {
        RtlDeleteRegistryValue(
            RTL_REGISTRY_ABSOLUTE,
            DatabasePath as *const u16,
            ValueName as *const u16,
        );
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn DeleteRegistryDriveLetter(UniqueId: *mut MOUNTDEV_UNIQUE_ID) {
    let mut QueryTable: core::mem::MaybeUninit<[RTL_QUERY_REGISTRY_TABLE; 2]> =
        core::mem::MaybeUninit::zeroed();
    let query_table_ptr: *mut RTL_QUERY_REGISTRY_TABLE =
        QueryTable.as_mut_ptr() as *mut RTL_QUERY_REGISTRY_TABLE;
    (*query_table_ptr).QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(DeleteDriveLetterRoutine));
    let _ = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        DatabasePath as *const u16,
        query_table_ptr,
        UniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DeleteNoDriveLetterEntryRoutine(
    ValueName: *mut u16,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let _ = EntryContext;
    let UniqueId = Context as *mut MOUNTDEV_UNIQUE_ID;
    if *ValueName != '#' as u16 || ValueType != REG_BINARY || (*UniqueId).UniqueIdLength as u32 != ValueLength {
        return STATUS_SUCCESS;
    }
    if RtlCompareMemory(
        core::ptr::addr_of!((*UniqueId).UniqueId) as *const core::ffi::c_void,
        ValueData as *const core::ffi::c_void,
        ValueLength as _,
    ) != ValueLength as _
    {
        RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE, DatabasePath as *const u16, ValueName as *const u16);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteNoDriveLetterEntry(
    UniqueId: *mut MOUNTDEV_UNIQUE_ID,
) {
    let mut query_table: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    query_table[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(DeleteNoDriveLetterEntryRoutine));
    let _ = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        DatabasePath as *const u16,
        query_table.as_mut_ptr(),
        UniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
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
//  * FILE:             drivers/filesystem/mountmgr/mountmgr.c
//  * PURPOSE:          Mount Manager - remote/local database handler
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #define NDEBUG
// #include <debug.h>
//
// PWSTR DatabasePath = L"\\Registry\\Machine\\System\\MountedDevices";
// PWSTR OfflinePath = L"\\Registry\\Machine\\System\\MountedDevices\\Offline";
//
// UNICODE_STRING RemoteDatabase = RTL_CONSTANT_STRING(L"\\System Volume Information\\MountPointManagerRemoteDatabase");
//
// /*
//  * @implemented
//  */
// LONG
// GetRemoteDatabaseSize(IN HANDLE Database)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_STANDARD_INFORMATION StandardInfo;
//
//     /* Just query the size */
//     Status = ZwQueryInformationFile(Database,
//                                     &IoStatusBlock,
//                                     &StandardInfo,
//                                     sizeof(FILE_STANDARD_INFORMATION),
//                                     FileStandardInformation);
//     if (NT_SUCCESS(Status))
//     {
//         return StandardInfo.EndOfFile.LowPart;
//     }
//
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// AddRemoteDatabaseEntry(IN HANDLE Database,
//                        IN PDATABASE_ENTRY Entry)
// {
//     LARGE_INTEGER Size;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Get size to append data */
//     Size.QuadPart = GetRemoteDatabaseSize(Database);
//
//     return ZwWriteFile(Database, NULL, NULL, NULL,
//                        &IoStatusBlock, Entry,
//                        Entry->EntrySize, &Size, NULL);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// CloseRemoteDatabase(IN HANDLE Database)
// {
//     return ZwClose(Database);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// TruncateRemoteDatabase(IN HANDLE Database,
//                        IN LONG NewSize)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_END_OF_FILE_INFORMATION EndOfFile;
//     FILE_ALLOCATION_INFORMATION Allocation;
//
//     EndOfFile.EndOfFile.QuadPart = NewSize;
//     Allocation.AllocationSize.QuadPart = NewSize;
//
//     /* First set EOF */
//     Status = ZwSetInformationFile(Database,
//                                   &IoStatusBlock,
//                                   &EndOfFile,
//                                   sizeof(FILE_END_OF_FILE_INFORMATION),
//                                   FileEndOfFileInformation);
//     if (NT_SUCCESS(Status))
//     {
//         /* And then, properly set allocation information */
//         Status = ZwSetInformationFile(Database,
//                                       &IoStatusBlock,
//                                       &Allocation,
//                                       sizeof(FILE_ALLOCATION_INFORMATION),
//                                       FileAllocationInformation);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// PDATABASE_ENTRY
// GetRemoteDatabaseEntry(IN HANDLE Database,
//                        IN LONG StartingOffset)
// {
//     NTSTATUS Status;
//     ULONG EntrySize;
//     PDATABASE_ENTRY Entry;
//     LARGE_INTEGER ByteOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Get the entry at the given position */
//     ByteOffset.QuadPart = StartingOffset;
//     Status = ZwReadFile(Database,
//                         NULL,
//                         NULL,
//                         NULL,
//                         &IoStatusBlock,
//                         &EntrySize,
//                         sizeof(EntrySize),
//                         &ByteOffset,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return NULL;
//     }
//
//     /* If entry doesn't exist, truncate database */
//     if (!EntrySize)
//     {
//         TruncateRemoteDatabase(Database, StartingOffset);
//         return NULL;
//     }
//
//     /* Allocate the entry */
//     Entry = AllocatePool(EntrySize);
//     if (!Entry)
//     {
//         return NULL;
//     }
//
//     /* Effectively read the entry */
//     Status = ZwReadFile(Database,
//                         NULL,
//                         NULL,
//                         NULL,
//                         &IoStatusBlock,
//                         Entry,
//                         EntrySize,
//                         &ByteOffset,
//                         NULL);
//     /* If it fails or returns inconsistent data, drop it (= truncate) */
//     if (!NT_SUCCESS(Status) ||
//         (IoStatusBlock.Information != EntrySize) ||
//         (EntrySize < sizeof(DATABASE_ENTRY)) )
//     {
//         TruncateRemoteDatabase(Database, StartingOffset);
//         FreePool(Entry);
//         return NULL;
//     }
//
//     /* Validate entry */
//     if (MAX(Entry->SymbolicNameOffset + Entry->SymbolicNameLength,
//             Entry->UniqueIdOffset + Entry->UniqueIdLength) > (LONG)EntrySize)
//     {
//         TruncateRemoteDatabase(Database, StartingOffset);
//         FreePool(Entry);
//         return NULL;
//     }
//
//     return Entry;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WriteRemoteDatabaseEntry(IN HANDLE Database,
//                          IN LONG Offset,
//                          IN PDATABASE_ENTRY Entry)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER ByteOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     ByteOffset.QuadPart = Offset;
//     Status = ZwWriteFile(Database,
//                          NULL,
//                          NULL,
//                          NULL,
//                          &IoStatusBlock,
//                          Entry,
//                          Entry->EntrySize,
//                          &ByteOffset,
//                          NULL);
//     if (NT_SUCCESS(Status))
//     {
//         if (IoStatusBlock.Information < Entry->EntrySize)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// DeleteRemoteDatabaseEntry(IN HANDLE Database,
//                           IN LONG StartingOffset)
// {
//     ULONG EndSize;
//     PVOID TmpBuffer;
//     NTSTATUS Status;
//     ULONG DatabaseSize;
//     PDATABASE_ENTRY Entry;
//     IO_STATUS_BLOCK IoStatusBlock;
//     LARGE_INTEGER EndEntriesOffset;
//
//     /* First, get database size */
//     DatabaseSize = GetRemoteDatabaseSize(Database);
//     if (!DatabaseSize)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Then, get the entry to remove */
//     Entry = GetRemoteDatabaseEntry(Database, StartingOffset);
//     if (!Entry)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate parameters: ensure we won't get zero or negative size */
//     if (Entry->EntrySize + StartingOffset >= DatabaseSize)
//     {
//         /* If we get invalid parameters, truncate the whole database
//          * starting the wrong entry. We can't rely on the rest
//          */
//         FreePool(Entry);
//         return TruncateRemoteDatabase(Database, StartingOffset);
//     }
//
//     /* Now, get the size of the remaining entries (those after the one to remove) */
//     EndSize = DatabaseSize - Entry->EntrySize - StartingOffset;
//     /* Allocate a buffer big enough to hold them */
//     TmpBuffer = AllocatePool(EndSize);
//     if (!TmpBuffer)
//     {
//         FreePool(Entry);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Get the offset of the entry right after the one to delete */
//     EndEntriesOffset.QuadPart = Entry->EntrySize + StartingOffset;
//     /* We don't need the entry any more */
//     FreePool(Entry);
//
//     /* Read the ending entries */
//     Status = ZwReadFile(Database, NULL, NULL, NULL, &IoStatusBlock,
//                         TmpBuffer, EndSize, &EndEntriesOffset, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(TmpBuffer);
//         return Status;
//     }
//
//     /* Ensure nothing went wrong - we don't want to corrupt the DB */
//     if (IoStatusBlock.Information != EndSize)
//     {
//         FreePool(TmpBuffer);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Remove the entry */
//     Status = TruncateRemoteDatabase(Database, StartingOffset + EndSize);
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(TmpBuffer);
//         return Status;
//     }
//
//     /* Now, shift the ending entries to erase the entry */
//     EndEntriesOffset.QuadPart = StartingOffset;
//     Status = ZwWriteFile(Database, NULL, NULL, NULL, &IoStatusBlock,
//                          TmpBuffer, EndSize, &EndEntriesOffset, NULL);
//
//     FreePool(TmpBuffer);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// DeleteFromLocalDatabaseRoutine(IN PWSTR ValueName,
//                                IN ULONG ValueType,
//                                IN PVOID ValueData,
//                                IN ULONG ValueLength,
//                                IN PVOID Context,
//                                IN PVOID EntryContext)
// {
//     PMOUNTDEV_UNIQUE_ID UniqueId = Context;
//
//     UNREFERENCED_PARAMETER(ValueType);
//     UNREFERENCED_PARAMETER(EntryContext);
//
//     /* Ensure it matches, and delete */
//     if ((UniqueId->UniqueIdLength == ValueLength) &&
//         (RtlCompareMemory(UniqueId->UniqueId, ValueData, ValueLength) ==
//          ValueLength))
//     {
//         RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                DatabasePath,
//                                ValueName);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// DeleteFromLocalDatabase(IN PUNICODE_STRING SymbolicLink,
//                         IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = DeleteFromLocalDatabaseRoutine;
//     QueryTable[0].Name = SymbolicLink->Buffer;
//
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            UniqueId,
//                            NULL);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WaitForRemoteDatabaseSemaphore(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER Timeout;
//
//     /* Wait for 7 minutes */
//     Timeout.QuadPart = 0xFA0A1F00;
//     Status = KeWaitForSingleObject(&(DeviceExtension->RemoteDatabaseLock), Executive, KernelMode, FALSE, &Timeout);
//     if (Status != STATUS_TIMEOUT)
//     {
//         return Status;
//     }
//
//     return STATUS_IO_TIMEOUT;
// }
//
// /*
//  * @implemented
//  */
// VOID
// ReleaseRemoteDatabaseSemaphore(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     KeReleaseSemaphore(&(DeviceExtension->RemoteDatabaseLock), IO_NO_INCREMENT, 1, FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// QueryUniqueIdQueryRoutine(IN PWSTR ValueName,
//                           IN ULONG ValueType,
//                           IN PVOID ValueData,
//                           IN ULONG ValueLength,
//                           IN PVOID Context,
//                           IN PVOID EntryContext)
// {
//     PMOUNTDEV_UNIQUE_ID IntUniqueId;
//     PMOUNTDEV_UNIQUE_ID * UniqueId;
//
//     UNREFERENCED_PARAMETER(ValueName);
//     UNREFERENCED_PARAMETER(ValueType);
//     UNREFERENCED_PARAMETER(EntryContext);
//
//     /* Sanity check */
//     if (ValueLength >= 0x10000)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Allocate the Unique ID */
//     IntUniqueId = AllocatePool(sizeof(UniqueId) + ValueLength);
//     if (IntUniqueId)
//     {
//         /* Copy data & return */
//         IntUniqueId->UniqueIdLength = (USHORT)ValueLength;
//         RtlCopyMemory(&(IntUniqueId->UniqueId), ValueData, ValueLength);
//
//         UniqueId = Context;
//         *UniqueId = IntUniqueId;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QueryUniqueIdFromMaster(IN PDEVICE_EXTENSION DeviceExtension,
//                         IN PUNICODE_STRING SymbolicName,
//                         OUT PMOUNTDEV_UNIQUE_ID * UniqueId)
// {
//     NTSTATUS Status;
//     PDEVICE_INFORMATION DeviceInformation;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     /* Query the unique ID */
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = QueryUniqueIdQueryRoutine;
//     QueryTable[0].Name = SymbolicName->Buffer;
//
//     *UniqueId = NULL;
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            UniqueId,
//                            NULL);
//     /* Unique ID found, no need to go farther */
//     if (*UniqueId)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Otherwise, find associate device information */
//     Status = FindDeviceInfo(DeviceExtension, SymbolicName, FALSE, &DeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     *UniqueId = AllocatePool(DeviceInformation->UniqueId->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//     if (!*UniqueId)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Return this unique ID (better than nothing) */
//     (*UniqueId)->UniqueIdLength = DeviceInformation->UniqueId->UniqueIdLength;
//     RtlCopyMemory(&((*UniqueId)->UniqueId), &(DeviceInformation->UniqueId->UniqueId), (*UniqueId)->UniqueIdLength);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WriteUniqueIdToMaster(IN PDEVICE_EXTENSION DeviceExtension,
//                       IN PDATABASE_ENTRY DatabaseEntry)
// {
//     NTSTATUS Status;
//     PWCHAR SymbolicName;
//     PLIST_ENTRY NextEntry;
//     UNICODE_STRING SymbolicString;
//     PDEVICE_INFORMATION DeviceInformation;
//
//     /* Create symbolic name from database entry */
//     SymbolicName = AllocatePool(DatabaseEntry->SymbolicNameLength + sizeof(WCHAR));
//     if (!SymbolicName)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyMemory(SymbolicName,
//                   (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset),
//                   DatabaseEntry->SymbolicNameLength);
//     SymbolicName[DatabaseEntry->SymbolicNameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Associate the unique ID with the name from remote database */
//     Status = RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                    DatabasePath,
//                                    SymbolicName,
//                                    REG_BINARY,
//                                    (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                                    DatabaseEntry->UniqueIdLength);
//     FreePool(SymbolicName);
//
//     /* Reget symbolic name */
//     SymbolicString.Length = DatabaseEntry->SymbolicNameLength;
//     SymbolicString.MaximumLength = DatabaseEntry->SymbolicNameLength;
//     SymbolicString.Buffer = (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset);
//
//     /* Find the device using this unique ID */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry,
//                                               DEVICE_INFORMATION,
//                                               DeviceListEntry);
//
//         if (DeviceInformation->UniqueId->UniqueIdLength != DatabaseEntry->UniqueIdLength)
//         {
//             continue;
//         }
//
//         if (RtlCompareMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                              DeviceInformation->UniqueId->UniqueId,
//                              DatabaseEntry->UniqueIdLength) == DatabaseEntry->UniqueIdLength)
//         {
//             break;
//         }
//     }
//
//     /* If found, create a mount point */
//     if (NextEntry != &(DeviceExtension->DeviceListHead))
//     {
//         MountMgrCreatePointWorker(DeviceExtension, &SymbolicString, &(DeviceInformation->DeviceName));
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ReconcileThisDatabaseWithMasterWorker(IN PVOID Parameter)
// {
//     ULONG Offset;
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     PDATABASE_ENTRY DatabaseEntry;
//     HANDLE DatabaseHandle, Handle;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDEVICE_INFORMATION ListDeviceInfo;
//     PLIST_ENTRY Entry, EntryInfo, NextEntry;
//     PASSOCIATED_DEVICE_ENTRY AssociatedDevice;
//     BOOLEAN HardwareErrors, Restart, FailedFinding;
//     WCHAR FileNameBuffer[0x8], SymbolicNameBuffer[100];
//     UNICODE_STRING ReparseFile, FileName, SymbolicName, VolumeName;
//     FILE_REPARSE_POINT_INFORMATION ReparsePointInformation, SavedReparsePointInformation;
//     PDEVICE_EXTENSION DeviceExtension = ((PRECONCILE_WORK_ITEM_CONTEXT)Parameter)->DeviceExtension;
//     PDEVICE_INFORMATION DeviceInformation = ((PRECONCILE_WORK_ITEM_CONTEXT)Parameter)->DeviceInformation;
//
//     /* We're unloading, do nothing */
//     if (Unloading)
//     {
//         return;
//     }
//
//     /* Lock remote DB */
//     if (!NT_SUCCESS(WaitForRemoteDatabaseSemaphore(DeviceExtension)))
//     {
//         return;
//     }
//
//     /* Recheck for unloading */
//     if (Unloading)
//     {
//         goto ReleaseRDS;
//     }
//
//     /* Find the DB to reconcile */
//     KeWaitForSingleObject(&DeviceExtension->DeviceLock, Executive, KernelMode, FALSE, NULL);
//     for (Entry = DeviceExtension->DeviceListHead.Flink;
//          Entry != &DeviceExtension->DeviceListHead;
//          Entry = Entry->Flink)
//     {
//         ListDeviceInfo = CONTAINING_RECORD(Entry, DEVICE_INFORMATION, DeviceListEntry);
//         if (ListDeviceInfo == DeviceInformation)
//         {
//             break;
//         }
//     }
//
//     /* If not found, or if removable, bail out */
//     if (Entry == &DeviceExtension->DeviceListHead || DeviceInformation->Removable)
//     {
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//         goto ReleaseRDS;
//     }
//
//     /* Get our device object */
//     Status = IoGetDeviceObjectPointer(&ListDeviceInfo->DeviceName, FILE_READ_ATTRIBUTES, &FileObject, &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//         goto ReleaseRDS;
//     }
//
//     /* Mark mounted only if not unloading */
//     if (!(DeviceObject->Flags & DO_UNLOAD_PENDING))
//     {
//         InterlockedExchangeAdd(&ListDeviceInfo->MountState, 1);
//     }
//
//     ObDereferenceObject(FileObject as *mut c_void);
//
//     /* Force default: no DB, and need for reconcile */
//     DeviceInformation->NeedsReconcile = TRUE;
//     DeviceInformation->NoDatabase = TRUE;
//     FailedFinding = FALSE;
//
//     /* Remove any associated device that refers to the DB to reconcile */
//     for (Entry = DeviceExtension->DeviceListHead.Flink;
//          Entry != &DeviceExtension->DeviceListHead;
//          Entry = Entry->Flink)
//     {
//         ListDeviceInfo = CONTAINING_RECORD(Entry, DEVICE_INFORMATION, DeviceListEntry);
//
//         EntryInfo = ListDeviceInfo->AssociatedDevicesHead.Flink;
//         while (EntryInfo != &ListDeviceInfo->AssociatedDevicesHead)
//         {
//             AssociatedDevice = CONTAINING_RECORD(EntryInfo, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//             NextEntry = EntryInfo->Flink;
//
//             if (AssociatedDevice->DeviceInformation == DeviceInformation)
//             {
//                 RemoveEntryList(&AssociatedDevice->AssociatedDevicesEntry);
//                 FreePool(AssociatedDevice->String.Buffer);
//                 FreePool(AssociatedDevice);
//             }
//
//             EntryInfo = NextEntry;
//         }
//     }
//
//     /* Open the remote database */
//     DatabaseHandle = OpenRemoteDatabase(DeviceInformation, FALSE);
//
//     /* Prepare a string with reparse point index */
//     ReparseFile.Length = 0;
//     ReparseFile.MaximumLength = DeviceInformation->DeviceName.Length
//                                 + ReparseIndex.Length
//                                 + sizeof(UNICODE_NULL);
//     ReparseFile.Buffer = AllocatePool(ReparseFile.MaximumLength);
//     if (!ReparseFile.Buffer)
//     {
//         if (DatabaseHandle != 0)
//         {
//             CloseRemoteDatabase(DatabaseHandle);
//         }
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//         goto ReleaseRDS;
//     }
//
//     RtlAppendUnicodeStringToString(&ReparseFile, &DeviceInformation->DeviceName);
//     RtlAppendUnicodeStringToString(&ReparseFile, &ReparseIndex);
//     ReparseFile.Buffer[ReparseFile.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &ReparseFile,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open reparse point directory */
//     HardwareErrors = IoSetThreadHardErrorMode(FALSE);
//     Status = ZwOpenFile(&Handle,
//                         FILE_GENERIC_READ,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_ALERT);
//     IoSetThreadHardErrorMode(HardwareErrors);
//
//     FreePool(ReparseFile.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (DatabaseHandle != 0)
//         {
//             TruncateRemoteDatabase(DatabaseHandle, 0);
//             CloseRemoteDatabase(DatabaseHandle);
//         }
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//         goto ReleaseRDS;
//     }
//
//     /* Query reparse point information
//      * We only pay attention to mout point
//      */
//     RtlZeroMemory(FileNameBuffer, sizeof(FileNameBuffer));
//     FileName.Buffer = FileNameBuffer;
//     FileName.Length = sizeof(FileNameBuffer);
//     FileName.MaximumLength = sizeof(FileNameBuffer);
//     ((PULONG)FileNameBuffer)[0] = IO_REPARSE_TAG_MOUNT_POINT;
//     Status = ZwQueryDirectoryFile(Handle,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   &IoStatusBlock,
//                                   &ReparsePointInformation,
//                                   sizeof(FILE_REPARSE_POINT_INFORMATION),
//                                   FileReparsePointInformation,
//                                   TRUE,
//                                   &FileName,
//                                   FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         ZwClose(Handle);
//         if (DatabaseHandle != 0)
//         {
//             TruncateRemoteDatabase(DatabaseHandle, 0);
//             CloseRemoteDatabase(DatabaseHandle);
//         }
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//         goto ReleaseRDS;
//     }
//
//     /* If we failed to open the remote DB previously,
//      * retry this time allowing migration (and thus, creation if required)
//      */
//     if (DatabaseHandle == 0)
//     {
//         DatabaseHandle = OpenRemoteDatabase(DeviceInformation, TRUE);
//         if (DatabaseHandle == 0)
//         {
//             KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//             goto ReleaseRDS;
//         }
//     }
//
//     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//
//     /* Reset all the references to our DB entries */
//     Offset = 0;
//     for (;;)
//     {
//         DatabaseEntry = GetRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
//         if (DatabaseEntry == NULL)
//         {
//             break;
//         }
//
//         DatabaseEntry->EntryReferences = 0;
//         Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset, DatabaseEntry);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(DatabaseEntry);
//             goto CloseReparse;
//         }
//
//         Offset += DatabaseEntry->EntrySize;
//         FreePool(DatabaseEntry);
//     }
//
//     /* Init string for QueryVolumeName call */
//     SymbolicName.MaximumLength = sizeof(SymbolicNameBuffer);
//     SymbolicName.Length = 0;
//     SymbolicName.Buffer = SymbolicNameBuffer;
//     Restart = TRUE;
//
//     /* Start looping on reparse points */
//     for (;;)
//     {
//         RtlCopyMemory(&SavedReparsePointInformation, &ReparsePointInformation, sizeof(FILE_REPARSE_POINT_INFORMATION));
//         Status = ZwQueryDirectoryFile(Handle,
//                                       NULL,
//                                       NULL,
//                                       NULL,
//                                       &IoStatusBlock,
//                                       &ReparsePointInformation,
//                                       sizeof(FILE_REPARSE_POINT_INFORMATION),
//                                       FileReparsePointInformation,
//                                       TRUE,
//                                       Restart ? &FileName : NULL,
//                                       Restart);
//         /* Restart only once */
//         if (Restart)
//         {
//             Restart = FALSE;
//         }
//         else
//         {
//             /* If we get the same one, we're done, bail out */
//             if (ReparsePointInformation.FileReference == SavedReparsePointInformation.FileReference &&
//                 ReparsePointInformation.Tag == SavedReparsePointInformation.Tag)
//             {
//                 break;
//             }
//         }
//
//         /* If querying failed, or if onloading, or if not returning mount points, bail out */
//         if (!NT_SUCCESS(Status) || Unloading || ReparsePointInformation.Tag != IO_REPARSE_TAG_MOUNT_POINT)
//         {
//             break;
//         }
//
//         /* Get the volume name associated to the mount point */
//         Status = QueryVolumeName(Handle, &ReparsePointInformation, 0, &SymbolicName, &VolumeName);
//         if (!NT_SUCCESS(Status))
//         {
//             continue;
//         }
//
//         /* Browse the DB to find the name */
//         Offset = 0;
//         for (;;)
//         {
//             UNICODE_STRING DbName;
//
//             DatabaseEntry = GetRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
//             if (DatabaseEntry == NULL)
//             {
//                 break;
//             }
//
//             DbName.MaximumLength = DatabaseEntry->SymbolicNameLength;
//             DbName.Length = DbName.MaximumLength;
//             DbName.Buffer = (PWSTR)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset);
//             /* Found, we're done! */
//             if (RtlEqualUnicodeString(&DbName, &SymbolicName, TRUE))
//             {
//                 break;
//             }
//
//             Offset += DatabaseEntry->EntrySize;
//             FreePool(DatabaseEntry);
//         }
//
//         /* If we found the mount point.... */
//         if (DatabaseEntry != NULL)
//         {
//             /* If it was referenced, reference it once more and update to remote */
//             if (DatabaseEntry->EntryReferences)
//             {
//                 ++DatabaseEntry->EntryReferences;
//                 Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset, DatabaseEntry);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     goto FreeDBEntry;
//                 }
//
//                 FreePool(DatabaseEntry);
//             }
//             else
//             {
//                 /* Query the Unique ID associated to that mount point in case it changed */
//                 KeWaitForSingleObject(&DeviceExtension->DeviceLock, Executive, KernelMode, FALSE, NULL);
//                 Status = QueryUniqueIdFromMaster(DeviceExtension, &SymbolicName, &UniqueId);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* If we failed doing so, reuse the old Unique ID and push it to master */
//                     Status = WriteUniqueIdToMaster(DeviceExtension, DatabaseEntry);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto ReleaseDeviceLock;
//                     }
//
//                     /* And then, reference & write the entry */
//                     ++DatabaseEntry->EntryReferences;
//                     Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset, DatabaseEntry);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto ReleaseDeviceLock;
//                     }
//
//                     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//                     FreePool(DatabaseEntry);
//                 }
//                 /* If the Unique ID didn't change */
//                 else if (UniqueId->UniqueIdLength == DatabaseEntry->UniqueIdLength &&
//                          RtlCompareMemory(UniqueId->UniqueId,
//                                           (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                                           UniqueId->UniqueIdLength) == UniqueId->UniqueIdLength)
//                 {
//                     /* Reference the entry, and update to remote */
//                     ++DatabaseEntry->EntryReferences;
//                     Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset, DatabaseEntry);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto FreeUniqueId;
//                     }
//
//                     FreePool(UniqueId);
//                     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//                     FreePool(DatabaseEntry);
//                 }
//                 /* Would, by chance, the Unique ID be present elsewhere? */
//                 else if (IsUniqueIdPresent(DeviceExtension, DatabaseEntry))
//                 {
//                     /* Push the ID to master */
//                     Status = WriteUniqueIdToMaster(DeviceExtension, DatabaseEntry);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto FreeUniqueId;
//                     }
//
//                     /* And then, reference & write the entry */
//                     ++DatabaseEntry->EntryReferences;
//                     Status = WriteRemoteDatabaseEntry(DatabaseHandle, Offset, DatabaseEntry);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto FreeUniqueId;
//                     }
//
//                     FreePool(UniqueId);
//                     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//                     FreePool(DatabaseEntry);
//                 }
//                 else
//                 {
//                     /* OK, at that point, we're facing a totally unknown unique ID
//                      * So, get rid of the old entry, and recreate a new one with
//                      * the know unique ID
//                      */
//                     Status = DeleteRemoteDatabaseEntry(DatabaseHandle, Offset);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto FreeUniqueId;
//                     }
//
//                     FreePool(DatabaseEntry);
//                     /* Allocate a new entry big enough */
//                     DatabaseEntry = AllocatePool(UniqueId->UniqueIdLength + SymbolicName.Length + sizeof(DATABASE_ENTRY));
//                     if (DatabaseEntry == NULL)
//                     {
//                        goto FreeUniqueId;
//                     }
//
//                     /* Configure it */
//                     DatabaseEntry->EntrySize = UniqueId->UniqueIdLength + SymbolicName.Length + sizeof(DATABASE_ENTRY);
//                     DatabaseEntry->EntryReferences = 1;
//                     DatabaseEntry->SymbolicNameOffset = sizeof(DATABASE_ENTRY);
//                     DatabaseEntry->SymbolicNameLength = SymbolicName.Length;
//                     DatabaseEntry->UniqueIdOffset = SymbolicName.Length + sizeof(DATABASE_ENTRY);
//                     DatabaseEntry->UniqueIdLength = UniqueId->UniqueIdLength;
//                     RtlCopyMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset), SymbolicName.Buffer, DatabaseEntry->SymbolicNameLength);
//                     RtlCopyMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset), UniqueId->UniqueId, UniqueId->UniqueIdLength);
//
//                     /* And write it remotely */
//                     Status = AddRemoteDatabaseEntry(DatabaseHandle, DatabaseEntry);
//                     if (!NT_SUCCESS(Status))
//                     {
//                        FreePool(DatabaseEntry);
//                        goto FreeUniqueId;
//                     }
//
//                     FreePool(UniqueId);
//                     FreePool(DatabaseEntry);
//                     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//                 }
//             }
//         }
//         else
//         {
//             /* We failed finding it remotely
//              * So, let's allocate a new remote DB entry
//              */
//             KeWaitForSingleObject(&DeviceExtension->DeviceLock, Executive, KernelMode, FALSE, NULL);
//             /* To be able to do so, we need the device Unique ID, ask master */
//             Status = QueryUniqueIdFromMaster(DeviceExtension, &SymbolicName, &UniqueId);
//             KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Allocate a new entry big enough */
//                 DatabaseEntry = AllocatePool(UniqueId->UniqueIdLength + SymbolicName.Length + sizeof(DATABASE_ENTRY));
//                 if (DatabaseEntry != NULL)
//                 {
//                     /* Configure it */
//                     DatabaseEntry->EntrySize = UniqueId->UniqueIdLength + SymbolicName.Length + sizeof(DATABASE_ENTRY);
//                     DatabaseEntry->EntryReferences = 1;
//                     DatabaseEntry->SymbolicNameOffset = sizeof(DATABASE_ENTRY);
//                     DatabaseEntry->SymbolicNameLength = SymbolicName.Length;
//                     DatabaseEntry->UniqueIdOffset = SymbolicName.Length + sizeof(DATABASE_ENTRY);
//                     DatabaseEntry->UniqueIdLength = UniqueId->UniqueIdLength;
//                     RtlCopyMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset), SymbolicName.Buffer, DatabaseEntry->SymbolicNameLength);
//                     RtlCopyMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset), UniqueId->UniqueId, UniqueId->UniqueIdLength);
//
//                     /* And write it remotely */
//                     Status = AddRemoteDatabaseEntry(DatabaseHandle, DatabaseEntry);
//                     FreePool(DatabaseEntry);
//                     FreePool(UniqueId);
//
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto FreeVolume;
//                     }
//                 }
//                 else
//                 {
//                     FreePool(UniqueId);
//                 }
//             }
//         }
//
//         /* Find info about the device associated associated with the mount point */
//         KeWaitForSingleObject(&DeviceExtension->DeviceLock, Executive, KernelMode, FALSE, NULL);
//         Status = FindDeviceInfo(DeviceExtension, &SymbolicName, FALSE, &ListDeviceInfo);
//         if (!NT_SUCCESS(Status))
//         {
//             FailedFinding = TRUE;
//             FreePool(VolumeName.Buffer);
//         }
//         else
//         {
//             /* Associate the device with the currrent DB */
//             AssociatedDevice = AllocatePool(sizeof(ASSOCIATED_DEVICE_ENTRY));
//             if (AssociatedDevice == NULL)
//             {
//                 FreePool(VolumeName.Buffer);
//             }
//             else
//             {
//                 AssociatedDevice->DeviceInformation = DeviceInformation;
//                 AssociatedDevice->String.Length = VolumeName.Length;
//                 AssociatedDevice->String.MaximumLength = VolumeName.MaximumLength;
//                 AssociatedDevice->String.Buffer = VolumeName.Buffer;
//                 InsertTailList(&ListDeviceInfo->AssociatedDevicesHead, &AssociatedDevice->AssociatedDevicesEntry);
//             }
//
//             /* If we don't have to skip notifications, notify */
//             if (!ListDeviceInfo->SkipNotifications)
//             {
//                 PostOnlineNotification(DeviceExtension, &ListDeviceInfo->SymbolicName);
//             }
//         }
//
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//     }
//
//     /* We don't need mount points any longer */
//     ZwClose(Handle);
//
//     /* Look for the DB again */
//     KeWaitForSingleObject(&DeviceExtension->DeviceLock, Executive, KernelMode, FALSE, NULL);
//     for (Entry = DeviceExtension->DeviceListHead.Flink;
//          Entry != &DeviceExtension->DeviceListHead;
//          Entry = Entry->Flink)
//     {
//         ListDeviceInfo = CONTAINING_RECORD(Entry, DEVICE_INFORMATION, DeviceListEntry);
//         if (ListDeviceInfo == DeviceInformation)
//         {
//             break;
//         }
//     }
//
//     if (Entry == &DeviceExtension->DeviceListHead)
//     {
//         ListDeviceInfo = NULL;
//     }
//
//     /* Start the pruning loop */
//     Offset = 0;
//     for (;;)
//     {
//         /* Get the entry */
//         DatabaseEntry = GetRemoteDatabaseEntry(DatabaseHandle, Offset as i32);
//         if (DatabaseEntry == NULL)
//         {
//             break;
//         }
//
//         /* It's not referenced anylonger? Prune it */
//         if (DatabaseEntry->EntryReferences == 0)
//         {
//             Status = DeleteRemoteDatabaseEntry(DatabaseHandle, Offset);
//             if (!NT_SUCCESS(Status))
//             {
//                 FreePool(DatabaseEntry);
//                 KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//                 goto CloseRDB;
//             }
//         }
//         /* Update the Unique IDs to reflect the changes we might have done previously */
//         else
//         {
//             if (ListDeviceInfo != NULL)
//             {
//                 UpdateReplicatedUniqueIds(ListDeviceInfo, DatabaseEntry);
//             }
//
//             Offset += DatabaseEntry->EntrySize;
//         }
//
//         FreePool(DatabaseEntry);
//     }
//
//     /* We do have a DB now :-) */
//     if (ListDeviceInfo != NULL && !FailedFinding)
//     {
//         DeviceInformation->NoDatabase = FALSE;
//     }
//
//     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//
//     goto CloseRDB;
//
// FreeUniqueId:
//     FreePool(UniqueId);
// ReleaseDeviceLock:
//     KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
// FreeDBEntry:
//     FreePool(DatabaseEntry);
// FreeVolume:
//     FreePool(VolumeName.Buffer);
// CloseReparse:
//     ZwClose(Handle);
// CloseRDB:
//     CloseRemoteDatabase(DatabaseHandle);
// ReleaseRDS:
//     ReleaseRemoteDatabaseSemaphore(DeviceExtension);
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// WorkerThread(IN PDEVICE_OBJECT DeviceObject,
//              IN PVOID Context)
// {
//     ULONG i;
//     KEVENT Event;
//     KIRQL OldIrql;
//     NTSTATUS Status;
//     HANDLE SafeEvent;
//     PLIST_ENTRY Entry;
//     LARGE_INTEGER Timeout;
//     PRECONCILE_WORK_ITEM WorkItem;
//     PDEVICE_EXTENSION DeviceExtension;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SafeVolumes,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Timeout.QuadPart = -10000000LL; /* Wait for 1 second */
//
//     /* Wait as long as possible for clearance from autochk
//      * We will write remote databases only if it is safe
//      * to access volumes.
//      * First, given we start before SMSS, wait for the
//      * event creation.
//      */
//     i = 0;
//     do
//     {
//         /* If we started to shutdown, stop waiting forever and jump to last attempt */
//         if (Unloading)
//         {
//             i = 999;
//         }
//         else
//         {
//             /* Attempt to open the event */
//             Status = ZwOpenEvent(&SafeEvent, EVENT_ALL_ACCESS, &ObjectAttributes);
//             if (NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             /* Wait a bit to give SMSS a chance to create the event */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, &Timeout);
//         }
//
//         ++i;
//     }
//     while (i < 1000);
//
//     /* We managed to open the event, wait until autochk signals it */
//     if (i < 1000)
//     {
//         do
//         {
//             Status = ZwWaitForSingleObject(SafeEvent, FALSE, &Timeout);
//         }
//         while (Status == STATUS_TIMEOUT && !Unloading);
//
//         ZwClose(SafeEvent);
//     }
//
//     DeviceExtension = Context;
//
//     InterlockedExchange(&(DeviceExtension->WorkerThreadStatus), 1);
//
//     /* Acquire workers lock */
//     KeWaitForSingleObject(&(DeviceExtension->WorkerSemaphore), Executive, KernelMode, FALSE, NULL);
//
//     KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//
//     /* Ensure there are workers */
//     while (!IsListEmpty(&(DeviceExtension->WorkerQueueListHead)))
//     {
//         /* Unqueue a worker */
//         Entry = RemoveHeadList(&(DeviceExtension->WorkerQueueListHead));
//         WorkItem = CONTAINING_RECORD(Entry,
//                                      RECONCILE_WORK_ITEM,
//                                      WorkerQueueListEntry);
//
//         KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//
//         /* Call it */
//         WorkItem->WorkerRoutine(WorkItem->Context);
//
//         IoFreeWorkItem(WorkItem->WorkItem);
//         FreePool(WorkItem);
//
//         if (InterlockedDecrement(&(DeviceExtension->WorkerReferences)) < 0)
//         {
//             return;
//         }
//
//         KeWaitForSingleObject(&(DeviceExtension->WorkerSemaphore), Executive, KernelMode, FALSE, NULL);
//         KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//     }
//     KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//
//     InterlockedDecrement(&(DeviceExtension->WorkerReferences));
//
//     /* Reset event */
//     KeSetEvent(&UnloadEvent, IO_NO_INCREMENT, FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QueueWorkItem(IN PDEVICE_EXTENSION DeviceExtension,
//               IN PRECONCILE_WORK_ITEM WorkItem,
//               IN PVOID Context)
// {
//     KIRQL OldIrql;
//
//     WorkItem->Context = Context;
//
//     /* When called, lock is already acquired */
//
//     /* If noone (-1 as references), start to work */
//     if (InterlockedIncrement(&(DeviceExtension->WorkerReferences)) == 0)
//     {
//         IoQueueWorkItem(WorkItem->WorkItem, WorkerThread, DelayedWorkQueue, DeviceExtension);
//     }
//
//     /* Otherwise queue worker for delayed execution */
//     KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//     InsertTailList(&(DeviceExtension->WorkerQueueListHead),
//                    &(WorkItem->WorkerQueueListEntry));
//     KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//
//     KeReleaseSemaphore(&(DeviceExtension->WorkerSemaphore), IO_NO_INCREMENT, 1, FALSE);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QueryVolumeName(IN HANDLE RootDirectory,
//                 IN PFILE_REPARSE_POINT_INFORMATION ReparsePointInformation,
//                 IN PUNICODE_STRING FileName OPTIONAL,
//                 OUT PUNICODE_STRING SymbolicName,
//                 OUT PUNICODE_STRING VolumeName)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     ULONG NeededLength;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILE_NAME_INFORMATION FileNameInfo;
//     PREPARSE_DATA_BUFFER ReparseDataBuffer;
//
//     if (!FileName)
//     {
//         UNICODE_STRING Reference;
//
//         Reference.Length = Reference.MaximumLength = sizeof(ReparsePointInformation->FileReference);
//         Reference.Buffer = (PWSTR)&(ReparsePointInformation->FileReference);
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &Reference,
//                                    OBJ_KERNEL_HANDLE,
//                                    RootDirectory,
//                                    NULL);
//     }
//     else
//     {
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    FileName,
//                                    OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//     }
//
//     /* Open volume */
//     Status = ZwOpenFile(&Handle,
//                         SYNCHRONIZE | FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         (FileName) ? FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT :
//                                      FILE_OPEN_BY_FILE_ID | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get the reparse point data */
//     ReparseDataBuffer = AllocatePool(MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     if (!ReparseDataBuffer)
//     {
//         ZwClose(Handle);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = ZwFsControlFile(Handle,
//                              0,
//                              NULL,
//                              NULL,
//                              &IoStatusBlock,
//                              FSCTL_GET_REPARSE_POINT,
//                              NULL,
//                              0,
//                              ReparseDataBuffer,
//                              MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(ReparseDataBuffer);
//         ZwClose(Handle);
//         return Status;
//     }
//
//     /* Check that name can fit in buffer */
//     if (ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameLength + sizeof(UNICODE_NULL) > SymbolicName->MaximumLength)
//     {
//         FreePool(ReparseDataBuffer);
//         ZwClose(Handle);
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Copy symbolic name */
//     SymbolicName->Length = ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameLength;
//     RtlCopyMemory(SymbolicName->Buffer,
//                   (PWSTR)((ULONG_PTR)ReparseDataBuffer->MountPointReparseBuffer.PathBuffer +
//                                      ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameOffset),
//                   ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameLength);
//
//     FreePool(ReparseDataBuffer);
//
//     /* Name has to \ terminated */
//     if (SymbolicName->Buffer[SymbolicName->Length / sizeof(WCHAR) - 1] != L'\\')
//     {
//         ZwClose(Handle);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* So that we can delete it, and match mountmgr requirements */
//     SymbolicName->Length -= sizeof(WCHAR);
//     SymbolicName->Buffer[SymbolicName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Also ensure it's really a volume name... */
//     if (!MOUNTMGR_IS_VOLUME_NAME(SymbolicName))
//     {
//         ZwClose(Handle);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Now prepare to really get the name */
//     FileNameInfo = AllocatePool(sizeof(FILE_NAME_INFORMATION) + 2 * sizeof(WCHAR));
//     if (!FileNameInfo)
//     {
//         ZwClose(Handle);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = ZwQueryInformationFile(Handle,
//                                     &IoStatusBlock,
//                                     FileNameInfo,
//                                     sizeof(FILE_NAME_INFORMATION) + 2 * sizeof(WCHAR),
//                                     FileNameInformation);
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         /* As expected... Reallocate with proper size */
//         NeededLength = FileNameInfo->FileNameLength;
//         FreePool(FileNameInfo);
//
//         FileNameInfo = AllocatePool(sizeof(FILE_NAME_INFORMATION) + NeededLength);
//         if (!FileNameInfo)
//         {
//             ZwClose(Handle);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* And query name */
//         Status = ZwQueryInformationFile(Handle,
//                                         &IoStatusBlock,
//                                         FileNameInfo,
//                                         sizeof(FILE_NAME_INFORMATION) + NeededLength,
//                                         FileNameInformation);
//     }
//
//     ZwClose(Handle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Return the volume name */
//     VolumeName->Length = (USHORT)FileNameInfo->FileNameLength;
//     VolumeName->MaximumLength = (USHORT)FileNameInfo->FileNameLength + sizeof(WCHAR);
//     VolumeName->Buffer = AllocatePool(VolumeName->MaximumLength);
//     if (!VolumeName->Buffer)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyMemory(VolumeName->Buffer, FileNameInfo->FileName, FileNameInfo->FileNameLength);
//     VolumeName->Buffer[FileNameInfo->FileNameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//     FreePool(FileNameInfo);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// OnlineMountedVolumes(IN PDEVICE_EXTENSION DeviceExtension,
//                      IN PDEVICE_INFORMATION DeviceInformation)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     BOOLEAN RestartScan;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDEVICE_INFORMATION VolumeDeviceInformation;
//     WCHAR FileNameBuffer[0x8], SymbolicNameBuffer[0x64];
//     UNICODE_STRING ReparseFile, FileName, SymbolicName, VolumeName;
//     FILE_REPARSE_POINT_INFORMATION ReparsePointInformation, SavedReparsePointInformation;
//
//     /* Removable devices don't have remote database on them */
//     if (DeviceInformation->Removable)
//     {
//         return;
//     }
//
//     /* Prepare a string with reparse point index */
//     ReparseFile.Length = 0;
//     ReparseFile.MaximumLength = DeviceInformation->DeviceName.Length
//                                 + ReparseIndex.Length
//                                 + sizeof(UNICODE_NULL);
//     ReparseFile.Buffer = AllocatePool(ReparseFile.MaximumLength);
//     if (!ReparseFile.Buffer)
//     {
//         return;
//     }
//
//     RtlAppendUnicodeStringToString(&ReparseFile, &DeviceInformation->DeviceName);
//     RtlAppendUnicodeStringToString(&ReparseFile, &ReparseIndex);
//     ReparseFile.Buffer[ReparseFile.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &ReparseFile,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open reparse point */
//     Status = ZwOpenFile(&Handle,
//                         FILE_GENERIC_READ,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_ALERT | FILE_OPEN_REPARSE_POINT);
//     FreePool(ReparseFile.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         DeviceInformation->NoDatabase = FALSE;
//         return;
//     }
//
//     /* Query reparse point information
//      * We only pay attention to mout point
//      */
//     RtlZeroMemory(FileNameBuffer, sizeof(FileNameBuffer));
//     FileName.Buffer = FileNameBuffer;
//     FileName.Length = sizeof(FileNameBuffer);
//     FileName.MaximumLength = sizeof(FileNameBuffer);
//     ((PULONG)FileNameBuffer)[0] = IO_REPARSE_TAG_MOUNT_POINT;
//     Status = ZwQueryDirectoryFile(Handle,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   &IoStatusBlock,
//                                   &ReparsePointInformation,
//                                   sizeof(FILE_REPARSE_POINT_INFORMATION),
//                                   FileReparsePointInformation,
//                                   TRUE,
//                                   &FileName,
//                                   FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         ZwClose(Handle);
//         return;
//     }
//
//     RestartScan = TRUE;
//
//     /* Query mount points */
//     while (TRUE)
//     {
//         SymbolicName.Length = 0;
//         SymbolicName.MaximumLength = sizeof(SymbolicNameBuffer);
//         SymbolicName.Buffer = SymbolicNameBuffer;
//         RtlCopyMemory(&SavedReparsePointInformation, &ReparsePointInformation, sizeof(FILE_REPARSE_POINT_INFORMATION));
//
//         Status = ZwQueryDirectoryFile(Handle,
//                                       NULL,
//                                       NULL,
//                                       NULL,
//                                       &IoStatusBlock,
//                                       &ReparsePointInformation,
//                                       sizeof(FILE_REPARSE_POINT_INFORMATION),
//                                       FileReparsePointInformation,
//                                       TRUE,
//                                       (RestartScan) ? &FileName : NULL,
//                                       RestartScan);
//          if (!RestartScan)
//          {
//              if (ReparsePointInformation.FileReference == SavedReparsePointInformation.FileReference &&
//                  ReparsePointInformation.Tag == SavedReparsePointInformation.Tag)
//              {
//                  break;
//              }
//          }
//          else
//          {
//              RestartScan = FALSE;
//          }
//
//          if (!NT_SUCCESS(Status) || ReparsePointInformation.Tag != IO_REPARSE_TAG_MOUNT_POINT)
//          {
//              break;
//          }
//
//          /* Get the volume name associated to the mount point */
//          Status = QueryVolumeName(Handle,
//                                   &ReparsePointInformation,
//                                   NULL, &SymbolicName,
//                                   &VolumeName);
//          if (!NT_SUCCESS(Status))
//          {
//              continue;
//          }
//
//          FreePool(VolumeName.Buffer);
//
//          /* Get its information */
//          Status = FindDeviceInfo(DeviceExtension, &SymbolicName,
//                                  FALSE, &VolumeDeviceInformation);
//          if (!NT_SUCCESS(Status))
//          {
//              DeviceInformation->NoDatabase = TRUE;
//              continue;
//          }
//
//          /* If notification are enabled, mark it online */
//          if (!DeviceInformation->SkipNotifications)
//          {
//              PostOnlineNotification(DeviceExtension, &VolumeDeviceInformation->SymbolicName);
//          }
//     }
//
//     ZwClose(Handle);
// }
//
// /*
//  * @implemented
//  */
// VOID
// ReconcileThisDatabaseWithMaster(IN PDEVICE_EXTENSION DeviceExtension,
//                                 IN PDEVICE_INFORMATION DeviceInformation)
// {
//     PRECONCILE_WORK_ITEM WorkItem;
//
//     /* Removable devices don't have remote database */
//     if (DeviceInformation->Removable)
//     {
//         return;
//     }
//
//     /* Allocate a work item */
//     WorkItem = AllocatePool(sizeof(RECONCILE_WORK_ITEM));
//     if (!WorkItem)
//     {
//         return;
//     }
//
//     WorkItem->WorkItem = IoAllocateWorkItem(DeviceExtension->DeviceObject);
//     if (!WorkItem->WorkItem)
//     {
//         FreePool(WorkItem);
//         return;
//     }
//
//     /* And queue it */
//     WorkItem->WorkerRoutine = ReconcileThisDatabaseWithMasterWorker;
//     WorkItem->DeviceExtension = DeviceExtension;
//     WorkItem->DeviceInformation = DeviceInformation;
//     QueueWorkItem(DeviceExtension, WorkItem, &(WorkItem->DeviceExtension));
//
//     /* If the worker thread isn't started yet, automatic drive letter is
//      * enabled but automount disabled, manually set mounted volumes online.
//      * Otherwise, they will be set online during database reconciliation. */
//     if (DeviceExtension->WorkerThreadStatus == 0 &&
//         DeviceExtension->AutomaticDriveLetter &&
//         DeviceExtension->NoAutoMount)
//     {
//         OnlineMountedVolumes(DeviceExtension, DeviceInformation);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// ReconcileAllDatabasesWithMaster(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     PLIST_ENTRY NextEntry;
//     PDEVICE_INFORMATION DeviceInformation;
//
//     /* Browse all the devices */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry,
//                                               DEVICE_INFORMATION,
//                                               DeviceListEntry);
//         /* If it's not removable, then, it might have a database to sync */
//         if (!DeviceInformation->Removable)
//         {
//             ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CreateRemoteDatabaseWorker(IN PDEVICE_OBJECT DeviceObject,
//                            IN PVOID Context)
// {
//     NTSTATUS Status;
//     HANDLE Database = 0;
//     UNICODE_STRING DatabaseName;
//     PMIGRATE_WORK_ITEM WorkItem;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDEVICE_INFORMATION DeviceInformation;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     /* Extract context */
//     WorkItem = Context;
//     DeviceInformation = WorkItem->DeviceInformation;
//
//     /* Reconstruct appropriate string */
//     DatabaseName.Length = 0;
//     DatabaseName.MaximumLength = DeviceInformation->DeviceName.Length
//                                  + RemoteDatabase.Length
//                                  + sizeof(UNICODE_NULL);
//     DatabaseName.Buffer = AllocatePool(DatabaseName.MaximumLength);
//     if (DatabaseName.Buffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     /* Create the required folder (in which the database will be stored
//      * \System Volume Information at root of the volume
//      */
//     Status = RtlCreateSystemVolumeInformationFolder(&(DeviceInformation->DeviceName));
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* Finish initiating strings */
//     RtlAppendUnicodeStringToString(&DatabaseName, &DeviceInformation->DeviceName);
//     RtlAppendUnicodeStringToString(&DatabaseName, &RemoteDatabase);
//     DatabaseName.Buffer[DatabaseName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Create database */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DatabaseName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     Status = IoCreateFile(&Database,
//                           SYNCHRONIZE | READ_CONTROL | FILE_WRITE_ATTRIBUTES |
//                           FILE_READ_ATTRIBUTES | FILE_WRITE_EA | FILE_READ_EA |
//                           FILE_APPEND_DATA | FILE_WRITE_DATA | FILE_READ_DATA,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
//                           0,
//                           FILE_CREATE,
//                           FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_ALERT,
//                           NULL,
//                           0,
//                           CreateFileTypeNone,
//                           NULL,
//                           IO_STOP_ON_SYMLINK | IO_NO_PARAMETER_CHECKING);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_STOPPED_ON_SYMLINK)
//         {
//             DPRINT1("Attempt to exploit CVE-2015-1769. See CORE-10216\n");
//         }
//
//         Database = 0;
//         goto Cleanup;
//     }
//
// Cleanup:
//     if (DatabaseName.Buffer)
//     {
//         FreePool(DatabaseName.Buffer);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         DeviceInformation->Migrated = 1;
//     }
//     else if (Database != 0)
//     {
//         ZwClose(Database);
//     }
//
//     IoFreeWorkItem(WorkItem->WorkItem);
//
//     WorkItem->WorkItem = NULL;
//     WorkItem->Status = Status;
//     WorkItem->Database = Database;
//
//     KeSetEvent(WorkItem->Event, 0, FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// CreateRemoteDatabase(IN PDEVICE_INFORMATION DeviceInformation,
//                      IN OUT PHANDLE Database)
// {
//     KEVENT Event;
//     NTSTATUS Status;
//     PMIGRATE_WORK_ITEM WorkItem;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Allocate a work item dedicated to migration */
//     WorkItem = AllocatePool(sizeof(MIGRATE_WORK_ITEM));
//     if (!WorkItem)
//     {
//         *Database = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(WorkItem, sizeof(MIGRATE_WORK_ITEM));
//     WorkItem->Event = &Event;
//     WorkItem->DeviceInformation = DeviceInformation;
//     WorkItem->WorkItem = IoAllocateWorkItem(DeviceInformation->DeviceExtension->DeviceObject);
//     if (!WorkItem->WorkItem)
//     {
//         FreePool(WorkItem);
//         *Database = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* And queue it */
//     IoQueueWorkItem(WorkItem->WorkItem,
//                     CreateRemoteDatabaseWorker,
//                     DelayedWorkQueue,
//                     WorkItem);
//
//     KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//     Status = WorkItem->Status;
//
//     *Database = (NT_SUCCESS(Status) ? WorkItem->Database : 0);
//
//     FreePool(WorkItem);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// OpenRemoteDatabase(IN PDEVICE_INFORMATION DeviceInformation,
//                    IN BOOLEAN MigrateDatabase)
// {
//     HANDLE Database;
//     NTSTATUS Status;
//     BOOLEAN PreviousMode;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING DeviceRemoteDatabase;
//
//     Database = 0;
//
//     /* Get database name */
//     DeviceRemoteDatabase.Length = 0;
//     DeviceRemoteDatabase.MaximumLength = DeviceInformation->DeviceName.Length
//                                          + RemoteDatabase.Length
//                                          + sizeof(UNICODE_NULL);
//     DeviceRemoteDatabase.Buffer = AllocatePool(DeviceRemoteDatabase.MaximumLength);
//     if (!DeviceRemoteDatabase.Buffer)
//     {
//         return 0;
//     }
//
//     RtlAppendUnicodeStringToString(&DeviceRemoteDatabase, &DeviceInformation->DeviceName);
//     RtlAppendUnicodeStringToString(&DeviceRemoteDatabase, &RemoteDatabase);
//     DeviceRemoteDatabase.Buffer[DeviceRemoteDatabase.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Open database */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DeviceRemoteDatabase,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Disable hard errors */
//     PreviousMode = IoSetThreadHardErrorMode(FALSE);
//
//     Status = IoCreateFile(&Database,
//                           SYNCHRONIZE | READ_CONTROL | FILE_WRITE_ATTRIBUTES |
//                           FILE_READ_ATTRIBUTES | FILE_WRITE_EA | FILE_READ_EA |
//                           FILE_APPEND_DATA | FILE_WRITE_DATA | FILE_READ_DATA,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
//                           0,
//                           (!MigrateDatabase || DeviceInformation->Migrated == 0) ? FILE_OPEN_IF : FILE_OPEN,
//                           FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_ALERT,
//                           NULL,
//                           0,
//                           CreateFileTypeNone,
//                           NULL,
//                           IO_STOP_ON_SYMLINK | IO_NO_PARAMETER_CHECKING);
//     if (Status == STATUS_STOPPED_ON_SYMLINK)
//     {
//         DPRINT1("Attempt to exploit CVE-2015-1769. See CORE-10216\n");
//     }
//
//     /* If base it to be migrated and was opened successfully, go ahead */
//     if (MigrateDatabase && NT_SUCCESS(Status))
//     {
//         CreateRemoteDatabase(DeviceInformation, &Database);
//     }
//
//     IoSetThreadHardErrorMode(PreviousMode);
//     FreePool(DeviceRemoteDatabase.Buffer);
//
//     return Database;
// }
//
// /*
//  * @implemented
//  */
// VOID
// ChangeRemoteDatabaseUniqueId(IN PDEVICE_INFORMATION DeviceInformation,
//                              IN PMOUNTDEV_UNIQUE_ID OldUniqueId,
//                              IN PMOUNTDEV_UNIQUE_ID NewUniqueId)
// {
//     LONG Offset = 0;
//     HANDLE Database;
//     PDATABASE_ENTRY Entry, NewEntry;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Open the remote database */
//     Database = OpenRemoteDatabase(DeviceInformation, FALSE);
//     if (!Database)
//     {
//         return;
//     }
//
//     /* Get all the entries */
//     do
//     {
//         Entry = GetRemoteDatabaseEntry(Database, Offset);
//         if (!Entry)
//         {
//             break;
//         }
//
//         /* Not the correct entry, skip it */
//         if (Entry->UniqueIdLength != OldUniqueId->UniqueIdLength)
//         {
//             Offset += Entry->EntrySize;
//             FreePool(Entry);
//             continue;
//         }
//
//         /* Not the correct entry, skip it */
//         if (RtlCompareMemory(OldUniqueId->UniqueId,
//                              (PVOID)((ULONG_PTR)Entry + Entry->UniqueIdOffset),
//                              Entry->UniqueIdLength) != Entry->UniqueIdLength)
//         {
//             Offset += Entry->EntrySize;
//             FreePool(Entry);
//             continue;
//         }
//
//         /* Here, we have the correct entry */
//         NewEntry = AllocatePool(Entry->EntrySize + NewUniqueId->UniqueIdLength - OldUniqueId->UniqueIdLength);
//         if (!NewEntry)
//         {
//             Offset += Entry->EntrySize;
//             FreePool(Entry);
//             continue;
//         }
//
//         /* Recreate the entry from the previous one */
//         NewEntry->EntrySize = Entry->EntrySize + NewUniqueId->UniqueIdLength - OldUniqueId->UniqueIdLength;
//         NewEntry->EntryReferences = Entry->EntryReferences;
//         NewEntry->SymbolicNameOffset = sizeof(DATABASE_ENTRY);
//         NewEntry->SymbolicNameLength = Entry->SymbolicNameLength;
//         NewEntry->UniqueIdOffset = Entry->SymbolicNameLength + sizeof(DATABASE_ENTRY);
//         NewEntry->UniqueIdLength = NewUniqueId->UniqueIdLength;
//         RtlCopyMemory((PVOID)((ULONG_PTR)NewEntry + NewEntry->SymbolicNameOffset),
//                       (PVOID)((ULONG_PTR)Entry + Entry->SymbolicNameOffset),
//                       NewEntry->SymbolicNameLength);
//         RtlCopyMemory((PVOID)((ULONG_PTR)NewEntry + NewEntry->UniqueIdOffset),
//                       NewUniqueId->UniqueId, NewEntry->UniqueIdLength);
//
//         /* Delete old entry */
//         Status = DeleteRemoteDatabaseEntry(Database, Offset);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(Entry);
//             FreePool(NewEntry);
//             break;
//         }
//
//         /* And replace with new one */
//         Status = AddRemoteDatabaseEntry(Database, NewEntry);
//         FreePool(Entry);
//         FreePool(NewEntry);
//     } while (NT_SUCCESS(Status));
//
//     CloseRemoteDatabase(Database);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// DeleteDriveLetterRoutine(IN PWSTR ValueName,
//                          IN ULONG ValueType,
//                          IN PVOID ValueData,
//                          IN ULONG ValueLength,
//                          IN PVOID Context,
//                          IN PVOID EntryContext)
// {
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     UNICODE_STRING RegistryEntry;
//
//     UNREFERENCED_PARAMETER(EntryContext);
//
//     if (ValueType != REG_BINARY)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     UniqueId = Context;
//
//     /* First ensure we have the correct data */
//     if (UniqueId->UniqueIdLength != ValueLength)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (RtlCompareMemory(UniqueId->UniqueId, ValueData, ValueLength) != ValueLength)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     RtlInitUnicodeString(&RegistryEntry, ValueName);
//
//     /* Then, it's a drive letter, erase it */
//     if (IsDriveLetter(&RegistryEntry))
//     {
//         RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                DatabasePath,
//                                ValueName);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// DeleteRegistryDriveLetter(IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = DeleteDriveLetterRoutine;
//
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            UniqueId,
//                            NULL);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI

// NTAPI cross-module decls — extern "system" to match the defs (i386 stdcall).

// driver-internal cdecl decls (types_*.rs authority = extern "C").

// ddk_shims-provided fns — extern "system" (match the shim def).
unsafe extern "system" {
    fn MOUNTMGR_IS_VOLUME_NAME(name: *const _UNICODE_STRING) -> u8; // BOOLEAN
    fn AllocatePool(NumberOfBytes: u32) -> *mut core::ffi::c_void;
    fn FreePool(P: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn FindDeviceInfo( DeviceExtension: *mut DEVICE_EXTENSION, SymbolicName: *const _UNICODE_STRING, Boolean: u8, ListDeviceInfo: *mut *mut DEVICE_INFORMATION, ) -> NTSTATUS;
    fn PostOnlineNotification( DeviceExtension: *mut DEVICE_EXTENSION, SymbolicName: *const _UNICODE_STRING, );
    fn IsDriveLetter(SymbolicName: *const _UNICODE_STRING) -> BOOLEAN;
}
// DeleteNoDriveLetterEntryRoutine(IN PWSTR ValueName,
//                                 IN ULONG ValueType,
//                                 IN PVOID ValueData,
//                                 IN ULONG ValueLength,
//                                 IN PVOID Context,
//                                 IN PVOID EntryContext)
// {
//     PMOUNTDEV_UNIQUE_ID UniqueId = Context;
//
//     UNREFERENCED_PARAMETER(EntryContext);
//
//     /* Ensure we have correct input */
//     if (ValueName[0] != L'#' || ValueType != REG_BINARY ||
//         UniqueId->UniqueIdLength != ValueLength)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* And then, if unique ID matching, delete entry */
//     if (RtlCompareMemory(UniqueId->UniqueId, ValueData, ValueLength) != ValueLength)
//     {
//         RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                DatabasePath,
//                                ValueName);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// DeleteNoDriveLetterEntry(IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = DeleteNoDriveLetterEntryRoutine;
//
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            UniqueId,
//                            NULL);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// CreateRemoteDatabase — re-integrated from the translation cache
// (.responses/mountmgr__database/); the stitcher dropped its body. Queues a
// migration work item and waits for the remote DB to be created. (database.c)
#[no_mangle]
pub unsafe extern "system" fn CreateRemoteDatabase(
    DeviceInformation: *mut DEVICE_INFORMATION,
    Database: *mut HANDLE,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut WorkItem: *mut MIGRATE_WORK_ITEM;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    WorkItem = AllocatePool(core::mem::size_of::<MIGRATE_WORK_ITEM>() as u32) as *mut MIGRATE_WORK_ITEM;
    if WorkItem.is_null() {
        *Database = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(WorkItem as *mut u8, 0, core::mem::size_of::<MIGRATE_WORK_ITEM>());
    (*WorkItem).Event = core::ptr::addr_of_mut!(Event);
    (*WorkItem).DeviceInformation = DeviceInformation;
    (*WorkItem).WorkItem = IoAllocateWorkItem((*(*DeviceInformation).DeviceExtension).DeviceObject);
    if (*WorkItem).WorkItem.is_null() {
        FreePool(WorkItem as *mut core::ffi::c_void);
        *Database = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoQueueWorkItem(
        (*WorkItem).WorkItem,
        Some(core::mem::transmute::<unsafe extern "system" fn(*mut DEVICE_OBJECT, *mut core::ffi::c_void), _>(CreateRemoteDatabaseWorker)),
        _WORK_QUEUE_TYPE_DelayedWorkQueue,
        WorkItem as *mut core::ffi::c_void,
    );

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive as i32,
        0i32,  // KernelMode
        0u8,   // not alertable
        core::ptr::null_mut(),
    );
    Status = (*WorkItem).Status;

    *Database = if Status >= 0 { (*WorkItem).Database } else { core::ptr::null_mut() };

    FreePool(WorkItem as *mut core::ffi::c_void);
    return Status;
}
