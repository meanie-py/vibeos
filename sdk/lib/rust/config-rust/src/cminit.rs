use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::{KeGetCurrentThread, NonPagedPool, PagedPool};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ddk::{
    CMHIVE, CM_CHECK_REGISTRY_STATUS, CmCheckRegistry, CmpAllocate, CmpCreateEvent,
    CmpDestroyHiveViewList, CmpDestroySecurityCache, CmpFileFlush, CmpFileRead,
    CmpFileSetSize, CmpFileWrite, CmpFree, CmpInitHiveViewList, CmpInitSecurityCache,
    ERESOURCE, EX_PUSH_LOCK, ExAcquirePushLockExclusive, ExDeleteResourceLite,
    ExInitializePushLock, ExInitializeResourceLite, ExReleasePushLock,
    FILE_FS_SIZE_INFORMATION,
    FILE_STANDARD_INFORMATION, HCELL_INDEX, _IO_STATUS_BLOCK,
    KeInitializeGuardedMutex, KeWaitForSingleObject, _OBJECT_ATTRIBUTES,
    PCELL_DATA, PCHAR, PCMHIVE, PCUNICODE_STRING, PCWSTR,
    PHANDLE, PHHIVE, PKEVENT, PKGUARDED_MUTEX, PKTHREAD, PULONG_PTR,
    RtlAppendUnicodeStringToString, RtlCopyUnicodeString, RtlInitEmptyUnicodeString,
    RtlInitUnicodeString, ZwClose, ZwCreateFile, ZwFsControlFile,
    ZwQueryInformationFile, ZwQueryVolumeInformationFile, _UNICODE_STRING,
    _KGUARDED_MUTEX,
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

// TAG constants (from sdk/lib/cmlib/cmlib.h). C uses big-endian char literals;
// 'vHMC' → bytes 'C','M','H','v' in memory → little-endian u32.
const TAG_CMHIVE: u32 = u32::from_le_bytes(*b"CMHv");
const TAG_CM: u32 = u32::from_le_bytes(*b"CM  ");
// HFILE_TYPE_MAX from hivedata.h
const HFILE_TYPE_MAX: u32 = 4;
const FILE_SUPERSEDE: u32 = 0;

// ARM64_UART_TRACE — no-op shim provided by C side (and inline-asm on ARM64).
extern "C" {
    fn ARM64_UART_TRACE(msg: PCHAR);
}

extern "system" {
    fn HvInitialize(
        RegistryHive: PHHIVE,
        OperationType: u32,
        HiveFlags: u32,
        FileType: u32,
        HiveData: *mut core::ffi::c_void,
        Allocate: *mut core::ffi::c_void,
        Free: *mut core::ffi::c_void,
        SetSize: *mut core::ffi::c_void,
        Write: *mut core::ffi::c_void,
        Read: *mut core::ffi::c_void,
        Flush: *mut core::ffi::c_void,
        Cluster: u32,
        FileName: PCUNICODE_STRING,
    ) -> i32;
    fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, Unpin: u8) -> u8;
    fn HvFree(Hive: PHHIVE);
}
extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
}

#[inline(always)]
unsafe fn CM_CHECK_REGISTRY_SUCCESS(Status: CM_CHECK_REGISTRY_STATUS) -> u8 {
    (Status == 0) as u8
}

extern "C" {
    static mut CmpShareSystemHives: u8;
    static mut CmpHiveListHeadLock: EX_PUSH_LOCK;
    static mut CmpHiveListHead: LIST_ENTRY;
    static mut CmpMiniNTBoot: u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeHive(
    CmHive: *mut PCMHIVE,
    OperationType: u32,
    HiveFlags: u32,
    FileType: u32,
    HiveData: *mut core::ffi::c_void,
    Primary: *mut core::ffi::c_void,
    Log: *mut core::ffi::c_void,
    External: *mut core::ffi::c_void,
    Alternate: *mut core::ffi::c_void,
    FileName: PCUNICODE_STRING,
    CheckFlags: u32,
) -> i32 {
    let CmHive = opaque_ptr(CmHive);
    let HiveData = opaque_ptr(HiveData);
    let Primary = opaque_ptr(Primary);
    let Log = opaque_ptr(Log);
    let External = opaque_ptr(External);
    let Alternate = opaque_ptr(Alternate);
    let FileName = opaque_ptr_const(FileName);

    const HINIT_MEMORY: u32 = 1;
    const HINIT_FILE: u32 = 2;
    const HINIT_MEMORY_INPLACE: u32 = 3;
    const HINIT_MAPFILE: u32 = 5;
    const HFILE_TYPE_PRIMARY: usize = 0;
    const HFILE_TYPE_LOG: usize = 1;
    const HFILE_TYPE_EXTERNAL: usize = 2;
    const HFILE_TYPE_ALTERNATE: usize = 3;
    const HBLOCK_SIZE: u32 = 4096;
    const HSECTOR_SIZE: u32 = 512;
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
    const STATUS_INVALID_PARAMETER: i32 = -1073741811i32;
    const STATUS_REGISTRY_IO_FAILED: i32 = 0xC000_014Du32 as i32;
    const STATUS_REGISTRY_CORRUPT: i32 = 0xC000_014Cu32 as i32;
    const FileFsSizeInformation: u32 = 3;
    const Volatile: usize = 1;
    const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;

    let Hive: PCMHIVE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileSizeInformation: FILE_FS_SIZE_INFORMATION = core::mem::zeroed();
    #[allow(unused_assignments)]
    let mut Cluster: u32 = 1;

    *CmHive = core::ptr::null_mut();

    if (!External.is_null() && (!Primary.is_null() || !Log.is_null())) ||
        (!Log.is_null() && Primary.is_null()) ||
        (CmpShareSystemHives == 0 && (HiveFlags & 0x00000001) != 0 &&
            (!Primary.is_null() || !External.is_null() || !Log.is_null())) ||
        ((OperationType == HINIT_MEMORY) && HiveData.is_null()) ||
        (!Log.is_null() && (FileType != HFILE_TYPE_LOG as u32)) ||
        (!Alternate.is_null() && (FileType != HFILE_TYPE_ALTERNATE as u32)) ||
        (!Alternate.is_null() && Primary.is_null())
    {
        return STATUS_INVALID_PARAMETER;
    }

    if !Primary.is_null() {
        let Status = ZwQueryVolumeInformationFile(
            Primary,
            &mut IoStatusBlock,
            &mut FileSizeInformation as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as u32,
            FileFsSizeInformation,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if FileSizeInformation.BytesPerSector > HBLOCK_SIZE {
            return STATUS_REGISTRY_IO_FAILED;
        }

        Cluster = FileSizeInformation.BytesPerSector / HSECTOR_SIZE;
        Cluster = max(1, Cluster);
    } else {
        Cluster = 1;
    }

    Hive = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<CMHIVE>(), TAG_CMHIVE) as PCMHIVE;
    if Hive.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Hive).UnloadEvent = core::ptr::null_mut();
    (*Hive).RootKcb = core::ptr::null_mut();
    (*Hive).Frozen = 0;
    (*Hive).UnloadWorkItem = core::ptr::null_mut();
    (*Hive).GrowOnlyMode = 0;
    (*Hive).GrowOffset = 0;
    (*Hive).CellRemapArray = core::ptr::null_mut();
    (*Hive).UseCountLog.Next = 0;
    (*Hive).LockHiveLog.Next = 0;
    (*Hive).FileObject = core::ptr::null_mut();
    (*Hive).NotifyList.Flink = core::ptr::null_mut();
    (*Hive).NotifyList.Blink = core::ptr::null_mut();

    (*Hive).HiveIsLoading = 1;

    (*Hive).CreatorOwner = KeGetCurrentThread() as PKTHREAD;

    InitializeListHead(&mut (*Hive).KcbConvertListHead);
    InitializeListHead(&mut (*Hive).KnodeConvertListHead);
    InitializeListHead(&mut (*Hive).TrustClassEntry);

    (*Hive).ViewLock = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_KGUARDED_MUTEX>(),
        TAG_CMHIVE,
    ) as PKGUARDED_MUTEX;
    if (*Hive).ViewLock.is_null() {
        ExFreePoolWithTag(Hive as *mut core::ffi::c_void, TAG_CMHIVE);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Hive).FlusherLock = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ERESOURCE>(),
        TAG_CMHIVE,
    ) as *mut ERESOURCE;
    if (*Hive).FlusherLock.is_null() {
        ExFreePoolWithTag((*Hive).ViewLock as *mut core::ffi::c_void, TAG_CMHIVE);
        ExFreePoolWithTag(Hive as *mut core::ffi::c_void, TAG_CMHIVE);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Hive).FileHandles[HFILE_TYPE_PRIMARY] = Primary;
    (*Hive).FileHandles[HFILE_TYPE_LOG] = Log;
    (*Hive).FileHandles[HFILE_TYPE_EXTERNAL] = External;
    (*Hive).FileHandles[HFILE_TYPE_ALTERNATE] = Alternate;

    KeInitializeGuardedMutex((*Hive).ViewLock);
    (*Hive).ViewLockOwner = core::ptr::null_mut();

    ExInitializeResourceLite((*Hive).FlusherLock);

    ExInitializePushLock(core::ptr::addr_of_mut!((*Hive).HiveLock) as PULONG_PTR);
    (*Hive).HiveLockOwner = core::ptr::null_mut();
    ExInitializePushLock(core::ptr::addr_of_mut!((*Hive).WriterLock) as PULONG_PTR);
    (*Hive).WriterLockOwner = core::ptr::null_mut();
    ExInitializePushLock(core::ptr::addr_of_mut!((*Hive).SecurityLock) as PULONG_PTR);
    (*Hive).HiveSecurityLockOwner = core::ptr::null_mut();

    RtlInitEmptyUnicodeString(&mut (*Hive).FileUserName, core::ptr::null_mut(), 0);
    RtlInitEmptyUnicodeString(&mut (*Hive).FileFullPath, core::ptr::null_mut(), 0);
    // Upstream 396019984c9: CmpRemoveFromHiveFileList() keys off a NULL Buffer to decide
    // whether a path was ever captured, so it must START NULL.
    RtlInitEmptyUnicodeString(&mut (*Hive).HiveRootPath, core::ptr::null_mut(), 0);

    CmpInitHiveViewList(Hive);

    CmpInitSecurityCache(Hive);

    (*Hive).Flags = 0;
    (*Hive).FlushCount = 0;

    ARM64_UART_TRACE(b"[HIVE] HvInitialize\r\n\0".as_ptr() as PCHAR);
    let Status = HvInitialize(
        &mut (*Hive).Hive,
        OperationType,
        HiveFlags,
        FileType,
        HiveData,
        CmpAllocate as *mut core::ffi::c_void,
        CmpFree as *mut core::ffi::c_void,
        CmpFileSetSize as *mut core::ffi::c_void,
        CmpFileWrite as *mut core::ffi::c_void,
        CmpFileRead as *mut core::ffi::c_void,
        CmpFileFlush as *mut core::ffi::c_void,
        Cluster,
        FileName,
    );
    if !NT_SUCCESS(Status) {
        ExDeleteResourceLite((*Hive).FlusherLock);
        ExFreePoolWithTag((*Hive).FlusherLock as *mut core::ffi::c_void, TAG_CMHIVE);
        ExFreePoolWithTag((*Hive).ViewLock as *mut core::ffi::c_void, TAG_CMHIVE);
        ExFreePoolWithTag(Hive as *mut core::ffi::c_void, TAG_CMHIVE);
        return Status;
    }

    ARM64_UART_TRACE(b"[HIVE] HvInitialize done\r\n\0".as_ptr() as PCHAR);

    if (OperationType == HINIT_FILE) ||
        (OperationType == HINIT_MEMORY) ||
        (OperationType == HINIT_MEMORY_INPLACE) ||
        (OperationType == HINIT_MAPFILE)
    {
        ARM64_UART_TRACE(b"[HIVE] CmCheckRegistry\r\n\0".as_ptr() as PCHAR);
        if OperationType == HINIT_MEMORY {
            let HivePtr = &mut (*Hive).Hive;
            let RootCell = (*(*HivePtr).BaseBlock).RootCell;
            if RootCell != HCELL_NIL {
                let RootData = HvGetCell(HivePtr, RootCell) as PCELL_DATA;
                if !RootData.is_null() && (CheckFlags & CM_CHECK_REGISTRY_PURGE_VOLATILES) != 0 &&
                    (*RootData).u.KeyNode.SubKeyCounts[Volatile] != 0
                {
                    HvMarkCellDirty(HivePtr, RootCell, 0);
                    if (*HivePtr).Version >= 6 {
                        (*RootData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile] = 0xBAADF00D;
                    } else {
                        (*RootData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile] = HCELL_NIL;
                    }
                    (*RootData).u.KeyNode.SubKeyCounts[Volatile] = 0;
                }
            }
            ARM64_UART_TRACE(b"[HIVE] CmCheckRegistry skipped (HINIT_MEMORY)\r\n\0".as_ptr() as PCHAR);
        } else {
            let CheckStatus = CmCheckRegistry(Hive, CheckFlags);
            if CM_CHECK_REGISTRY_SUCCESS(CheckStatus) == 0 {
                // Not in the C. cmcheck's own DPRINT1s were dropped in translation, so this
                // was a bare REGISTRY_CORRUPT: the status names which check failed.
                crate::dbg_print!(
                    b"CmpInitializeHive: CmCheckRegistry failed (CM_CHECK_REGISTRY status %lu)\n\0",
                    CheckStatus as u32,
                );
                ExDeleteResourceLite((*Hive).FlusherLock);
                ExFreePoolWithTag((*Hive).FlusherLock as *mut core::ffi::c_void, TAG_CMHIVE);
                ExFreePoolWithTag((*Hive).ViewLock as *mut core::ffi::c_void, TAG_CMHIVE);
                ExFreePoolWithTag(Hive as *mut core::ffi::c_void, TAG_CMHIVE);
                return STATUS_REGISTRY_CORRUPT;
            }
        }
    }

    ARM64_UART_TRACE(b"[HIVE] CmCheckRegistry done\r\n\0".as_ptr() as PCHAR);

    (*Hive).HiveIsLoading = 0;

    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(CmpHiveListHeadLock));

    InsertHeadList(core::ptr::addr_of_mut!(CmpHiveListHead), &mut (*Hive).HiveList);

    ExReleasePushLock(core::ptr::addr_of_mut!(CmpHiveListHeadLock));

    *CmHive = Hive;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpDestroyHive(CmHive: PCMHIVE) -> NTSTATUS {
    let CmHive = opaque_ptr(CmHive);
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(CmpHiveListHeadLock));
    RemoveEntryList(core::ptr::addr_of_mut!((*CmHive).HiveList));
    ExReleasePushLock(core::ptr::addr_of_mut!(CmpHiveListHeadLock));

    CmpDestroySecurityCache(CmHive);

    CmpDestroyHiveViewList(CmHive);

    ExDeleteResourceLite((*CmHive).FlusherLock);
    ExFreePoolWithTag((*CmHive).FlusherLock as PVOID, TAG_CMHIVE);

    ExFreePoolWithTag((*CmHive).ViewLock as PVOID, TAG_CMHIVE);

    HvFree(core::ptr::addr_of_mut!((*CmHive).Hive));

    CmpFree(CmHive as PVOID, TAG_CM);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpOpenHiveFiles(
    BaseName: PCUNICODE_STRING,
    Extension: PCWSTR,
    Primary: PHANDLE,
    Log: PHANDLE,
    PrimaryDisposition: *mut u32,
    LogDisposition: *mut u32,
    CreateAllowed: u8,
    MarkAsSystemHive: u8,
    NoBuffering: u8,
    ClusterSize: *mut u32,
) -> i32 {
    let BaseName = opaque_ptr_const(BaseName);
    let Extension = opaque_ptr_const(Extension);
    let Primary = opaque_ptr(Primary);
    let Log = opaque_ptr(Log);
    let PrimaryDisposition = opaque_ptr(PrimaryDisposition);
    let LogDisposition = opaque_ptr(LogDisposition);
    let ClusterSize = opaque_ptr(ClusterSize);
    let mut EventHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut Status: i32;
    let mut FullName: _UNICODE_STRING = core::mem::zeroed();
    let mut ExtensionName: _UNICODE_STRING = core::mem::zeroed();
    let mut NameBuffer: *mut u16;
    let mut Length: u16;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let AttributeFlags: u32;
    let ShareMode: u32;
    let DesiredAccess: u32;
    let mut CreateDisposition: u32;
    let IoFlags: u32;
    let mut CompressionState: u16;
    let mut FileInformation: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut FsSizeInformation: FILE_FS_SIZE_INFORMATION = core::mem::zeroed();

    Status = CmpCreateEvent(0, &mut EventHandle, &mut Event);
    if Status < 0 {
        return Status;
    }

    RtlInitEmptyUnicodeString(&mut FullName, core::ptr::null_mut(), 0);
    Length = (*BaseName).Length;

    if !Extension.is_null() {
        Length = (Length as usize + wcslen(Extension) * core::mem::size_of::<u16>() + core::mem::size_of::<u16>()) as u16;

        NameBuffer = ExAllocatePoolWithTag(PagedPool, Length as usize, 0x20436d43u32) as *mut u16;
        if NameBuffer.is_null() {
            ObDereferenceObject(Event as *mut core::ffi::c_void);
            ZwClose(EventHandle);
            return STATUS_NO_MEMORY;
        }

        FullName.Buffer = NameBuffer;
        FullName.MaximumLength = Length;
        RtlCopyUnicodeString(&mut FullName, BaseName as *mut _UNICODE_STRING);
    } else {
        FullName = *BaseName;
        NameBuffer = core::ptr::null_mut();
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut FullName;
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if CreateAllowed != 0 && CmpShareSystemHives == 0 {
        CreateDisposition = 3;
    } else {
        CreateDisposition = 1;
    }

    IoFlags = 0x00000008 | 0x00000400 | if NoBuffering != 0 { 0x00000004 } else { 0 };

    if CmpMiniNTBoot != 0 && CmpShareSystemHives != 0 {
        DesiredAccess = 0x0001;
        ShareMode = 0x00000001;
    } else {
        ShareMode = 0;
        DesiredAccess = 0x0001 | 0x0002;
    };

    AttributeFlags = 0x00000080;

    Status = ZwCreateFile(
        Primary,
        DesiredAccess | 0x00100000,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        AttributeFlags,
        ShareMode,
        CreateDisposition,
        0x00000020 | IoFlags, // FILE_SYNCHRONOUS_IO_NONALERT
        core::ptr::null_mut(),
        0,
    );

    if Status < 0 {
        crate::dbg_print!(
            b"ZwCreateFile(%wZ) failed, Status 0x%08lx.\n\0",
            ObjectAttributes.ObjectName,
            Status
        );
        if !NameBuffer.is_null() {
            ExFreePoolWithTag(NameBuffer as *mut core::ffi::c_void, 0x20436d43u32);
        }
        ObDereferenceObject(Event as *mut core::ffi::c_void);
        ZwClose(EventHandle);
        *Primary = core::ptr::null_mut();
        return Status;
    }

    if MarkAsSystemHive != 0 {
        Status = ZwFsControlFile(
            *Primary,
            EventHandle,
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            0x0009C0C8,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if Status == 0x00000103 {
            KeWaitForSingleObject(
                Event as *mut core::ffi::c_void,
                0,
                0,
                0,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if Status == 0xC0000010u32 as i32 {
            Status = STATUS_SUCCESS;
        }

        if Status < 0 {
            if !NameBuffer.is_null() {
                ExFreePoolWithTag(NameBuffer as *mut core::ffi::c_void, 0x20436d43u32);
            }
            ObDereferenceObject(Event as *mut core::ffi::c_void);
            ZwClose(EventHandle);
            ZwClose(*Primary);
            *Primary = core::ptr::null_mut();
            return Status;
        }
    }

    CompressionState = 0;
    Status = ZwFsControlFile(
        *Primary,
        EventHandle,
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        0x0009C040,
        &mut CompressionState as *mut u16 as *mut core::ffi::c_void,
        core::mem::size_of::<u16>() as u32,
        core::ptr::null_mut(),
        0,
    );
    if Status == 0x00000103 {
        KeWaitForSingleObject(
            Event as *mut core::ffi::c_void,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
    }

    *PrimaryDisposition = IoStatusBlock.Information as u32;
    if IoStatusBlock.Information != 2 {
        Status = ZwQueryInformationFile(
            *Primary,
            &mut IoStatusBlock,
            &mut FileInformation as *mut FILE_STANDARD_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
            ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileStandardInformation as _,
        );
        if Status >= 0 {
            if FileInformation.EndOfFile.QuadPart == 0 {
                *PrimaryDisposition = 2;
            }
        }
    }

    if !ClusterSize.is_null() {
        Status = ZwQueryVolumeInformationFile(
            *Primary,
            &mut IoStatusBlock,
            &mut FsSizeInformation as *mut FILE_FS_SIZE_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as u32,
            3,
        );
        if Status < 0 {
            if !NameBuffer.is_null() {
                ExFreePoolWithTag(NameBuffer as *mut core::ffi::c_void, 0x20436d43u32);
            }
            ObDereferenceObject(Event as *mut core::ffi::c_void);
            ZwClose(EventHandle);
            return Status;
        }

        if FsSizeInformation.BytesPerSector > 4096 {
            if !NameBuffer.is_null() {
                ExFreePoolWithTag(NameBuffer as *mut core::ffi::c_void, 0x20436d43u32);
            }
            ObDereferenceObject(Event as *mut core::ffi::c_void);
            ZwClose(EventHandle);
            return 0xC0000218u32 as i32;
        }

        *ClusterSize = max(1, FsSizeInformation.BytesPerSector / 512);
    }

    if Extension.is_null() {
        ObDereferenceObject(Event as *mut core::ffi::c_void);
        ZwClose(EventHandle);
        return STATUS_SUCCESS;
    }

    CreateDisposition = if CmpShareSystemHives != 0 { 1 } else { 3 };
    if *PrimaryDisposition == 2 {
        CreateDisposition = FILE_SUPERSEDE;
    }

    RtlInitUnicodeString(&mut ExtensionName, Extension);
    RtlAppendUnicodeStringToString(&mut FullName, &mut ExtensionName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut FullName;
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let IoFlags = 0x00000008 | 0x00000004;

    let mut AttributeFlags = 0x00000080;
    if _wcsnicmp(Extension, w!(".log").as_ptr() as *const u16, 4) == 0 {
        AttributeFlags |= 0x00000002;
    }

    Status = ZwCreateFile(
        Log,
        DesiredAccess | 0x00100000,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        AttributeFlags,
        ShareMode,
        CreateDisposition,
        0x00000020 | IoFlags, // FILE_SYNCHRONOUS_IO_NONALERT
        core::ptr::null_mut(),
        0,
    );

    if Status >= 0 && MarkAsSystemHive != 0 {
        Status = ZwFsControlFile(
            *Log,
            EventHandle,
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            0x0009C0C8,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if Status == 0x00000103 {
            KeWaitForSingleObject(
                Event as *mut core::ffi::c_void,
                0,
                0,
                0,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if Status == 0xC0000010u32 as i32 {
            Status = STATUS_SUCCESS;
        }

        if Status < 0 {
            ZwClose(*Log);
        }
    }

    if Status < 0 {
        *Log = core::ptr::null_mut();
    } else {
        Status = ZwFsControlFile(
            *Log,
            EventHandle,
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            0x0009C040,
            &mut CompressionState as *mut u16 as *mut core::ffi::c_void,
            core::mem::size_of::<u16>() as u32,
            core::ptr::null_mut(),
            0,
        );
        if Status == 0x00000103 {
            KeWaitForSingleObject(
                Event as *mut core::ffi::c_void,
                0,
                0,
                0,
                core::ptr::null_mut(),
            );
        }

        *LogDisposition = IoStatusBlock.Information as u32;
    }

    if !NameBuffer.is_null() {
        ExFreePoolWithTag(NameBuffer as *mut core::ffi::c_void, 0x20436d43u32);
    }
    ObDereferenceObject(Event as *mut core::ffi::c_void);
    ZwClose(EventHandle);
    STATUS_SUCCESS
}

unsafe fn wcslen(s: *const u16) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

unsafe fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32 {
    for i in 0..n {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 == 0 && c2 == 0 {
            return 0;
        }
        let c1_lower = if c1 >= 65 && c1 <= 90 { c1 + 32 } else { c1 };
        let c2_lower = if c2 >= 65 && c2 <= 90 { c2 + 32 } else { c2 };
        if c1_lower != c2_lower {
            return if c1_lower < c2_lower { -1 } else { 1 };
        }
    }
    0
}

unsafe fn max(a: u32, b: u32) -> u32 {
    if a > b { a } else { b }
}

#[no_mangle]
pub unsafe extern "system" fn CmpCloseHiveFiles(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    for i in 0..HFILE_TYPE_MAX {
        if (*Hive).FileHandles[i as usize] != core::ptr::null_mut() {
            ZwClose((*Hive).FileHandles[i as usize]);
            (*Hive).FileHandles[i as usize] = core::ptr::null_mut();
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cminit.c
//  * PURPOSE:         Configuration Manager - Hive Initialization
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// CmpInitializeHive(
//     _Out_ PCMHIVE *CmHive,
//     _In_ ULONG OperationType,
//     _In_ ULONG HiveFlags,
//     _In_ ULONG FileType,
//     _In_opt_ PVOID HiveData,
//     _In_ HANDLE Primary,
//     _In_ HANDLE Log,
//     _In_ HANDLE External,
//     _In_ HANDLE Alternate,
//     _In_opt_ PCUNICODE_STRING FileName,
//     _In_ ULONG CheckFlags)
// {
//     PCMHIVE Hive;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_FS_SIZE_INFORMATION FileSizeInformation;
//     NTSTATUS Status;
//     ULONG Cluster;
//
//     /* Assume failure */
//     *CmHive = NULL;
//
//     /*
//      * The following are invalid:
//      * - An external hive that is also internal.
//      * - A log hive that is not a primary hive too.
//      * - A volatile hive that is linked to permanent storage,
//      *   unless this hive is a shared system hive.
//      * - An in-memory initialization without hive data.
//      * - A log hive that is not linked to a correct file type.
//      * - An alternate hive that is not linked to a correct file type.
//      * - A lonely alternate hive not backed up with its corresponding primary hive.
//      */
//     if ((External && (Primary || Log)) ||
//         (Log && !Primary) ||
//         (!CmpShareSystemHives && (HiveFlags & HIVE_VOLATILE) &&
//             (Primary || External || Log)) ||
//         ((OperationType == HINIT_MEMORY) && !HiveData) ||
//         (Log && (FileType != HFILE_TYPE_LOG)) ||
//         (Alternate && (FileType != HFILE_TYPE_ALTERNATE)) ||
//         (Alternate && !Primary))
//     {
//         /* Fail the request */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if this is a primary hive */
//     if (Primary)
//     {
//         /* Get the cluster size */
//         Status = ZwQueryVolumeInformationFile(Primary,
//                                               &IoStatusBlock,
//                                               &FileSizeInformation,
//                                               sizeof(FILE_FS_SIZE_INFORMATION),
//                                               FileFsSizeInformation);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Make sure it's not larger then the block size */
//         if (FileSizeInformation.BytesPerSector > HBLOCK_SIZE)
//         {
//             /* Fail */
//             return STATUS_REGISTRY_IO_FAILED;
//         }
//
//         /* Otherwise, calculate the cluster */
//         Cluster = FileSizeInformation.BytesPerSector / HSECTOR_SIZE;
//         Cluster = max(1, Cluster);
//     }
//     else
//     {
//         /* Otherwise use cluster 1 */
//         Cluster = 1;
//     }
//
//     /* Allocate the hive */
//     Hive = ExAllocatePoolWithTag(NonPagedPool, sizeof(CMHIVE), TAG_CMHIVE);
//     if (!Hive) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Setup null fields */
//     Hive->UnloadEvent = NULL;
//     Hive->RootKcb = NULL;
//     Hive->Frozen = FALSE;
//     Hive->UnloadWorkItem = NULL;
//     Hive->GrowOnlyMode = FALSE;
//     Hive->GrowOffset = 0;
//     Hive->CellRemapArray = NULL;
//     Hive->UseCountLog.Next = 0;
//     Hive->LockHiveLog.Next = 0;
//     Hive->FileObject = NULL;
//     Hive->NotifyList.Flink = NULL;
//     Hive->NotifyList.Blink = NULL;
//
//     /* Set the loading flag */
//     Hive->HiveIsLoading = TRUE;
//
//     /* Set the current thread as creator */
//     Hive->CreatorOwner = KeGetCurrentThread();
//
//     /* Initialize lists */
//     InitializeListHead(&Hive->KcbConvertListHead);
//     InitializeListHead(&Hive->KnodeConvertListHead);
//     InitializeListHead(&Hive->TrustClassEntry);
//
//     /* Allocate the view log */
//     Hive->ViewLock = ExAllocatePoolWithTag(NonPagedPool,
//                                            sizeof(KGUARDED_MUTEX),
//                                            TAG_CMHIVE);
//     if (!Hive->ViewLock)
//     {
//         /* Cleanup allocation and fail */
//         ExFreePoolWithTag(Hive, TAG_CMHIVE);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Allocate the flush lock */
//     Hive->FlusherLock = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(ERESOURCE),
//                                               TAG_CMHIVE);
//     if (!Hive->FlusherLock)
//     {
//         /* Cleanup allocations and fail */
//         ExFreePoolWithTag(Hive->ViewLock, TAG_CMHIVE);
//         ExFreePoolWithTag(Hive, TAG_CMHIVE);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Setup the handles */
//     Hive->FileHandles[HFILE_TYPE_PRIMARY] = Primary;
//     Hive->FileHandles[HFILE_TYPE_LOG] = Log;
//     Hive->FileHandles[HFILE_TYPE_EXTERNAL] = External;
//     Hive->FileHandles[HFILE_TYPE_ALTERNATE] = Alternate;
//
//     /* Initailize the guarded mutex */
//     KeInitializeGuardedMutex(Hive->ViewLock);
//     Hive->ViewLockOwner = NULL;
//
//     /* Initialize the flush lock */
//     ExInitializeResourceLite(Hive->FlusherLock);
//
//     /* Setup hive locks */
//     ExInitializePushLock(&Hive->HiveLock);
//     Hive->HiveLockOwner = NULL;
//     ExInitializePushLock(&Hive->WriterLock);
//     Hive->WriterLockOwner = NULL;
//     ExInitializePushLock(&Hive->SecurityLock);
//     Hive->HiveSecurityLockOwner = NULL;
//
//     /* Clear file names */
//     RtlInitEmptyUnicodeString(&Hive->FileUserName, NULL, 0);
//     RtlInitEmptyUnicodeString(&Hive->FileFullPath, NULL, 0);
//
//     /* Initialize the view list */
//     CmpInitHiveViewList(Hive);
//
//     /* Initailize the security cache */
//     CmpInitSecurityCache(Hive);
//
//     /* Setup flags */
//     Hive->Flags = 0;
//     Hive->FlushCount = 0;
//
//     /* Initialize it */
//     ARM64_UART_TRACE("[HIVE] HvInitialize\r\n");
//     Status = HvInitialize(&Hive->Hive,
//                           OperationType,
//                           HiveFlags,
//                           FileType,
//                           HiveData,
//                           CmpAllocate,
//                           CmpFree,
//                           CmpFileSetSize,
//                           CmpFileWrite,
//                           CmpFileRead,
//                           CmpFileFlush,
//                           Cluster,
//                           FileName);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup allocations and fail */
//         ExDeleteResourceLite(Hive->FlusherLock);
//         ExFreePoolWithTag(Hive->FlusherLock, TAG_CMHIVE);
//         ExFreePoolWithTag(Hive->ViewLock, TAG_CMHIVE);
//         ExFreePoolWithTag(Hive, TAG_CMHIVE);
//         return Status;
//     }
//
//     ARM64_UART_TRACE("[HIVE] HvInitialize done\r\n");
//
//     /* Check if we should verify the registry */
//     if ((OperationType == HINIT_FILE) ||
//         (OperationType == HINIT_MEMORY) ||
//         (OperationType == HINIT_MEMORY_INPLACE) ||
//         (OperationType == HINIT_MAPFILE))
//     {
//         /* Verify integrity */
//         ARM64_UART_TRACE("[HIVE] CmCheckRegistry\r\n");
//         /* Skip deep validation for HINIT_MEMORY hives during boot.
//          * The hive was loaded from the boot loader and validated there.
//          * Still do volatile purging for the root cell only. */
//         if (OperationType == HINIT_MEMORY)
//         {
//             PHHIVE HivePtr = &Hive->Hive;
//             HCELL_INDEX RootCell = HivePtr->BaseBlock->RootCell;
//             if (RootCell != HCELL_NIL)
//             {
//                 PCELL_DATA RootData = (PCELL_DATA)HvGetCell(HivePtr, RootCell);
//                 if (RootData && (CheckFlags & CM_CHECK_REGISTRY_PURGE_VOLATILES) &&
//                     (RootData->u.KeyNode.SubKeyCounts[Volatile] != 0))
//                 {
//                     HvMarkCellDirty(HivePtr, RootCell, FALSE);
//                     if (HivePtr->Version >= 6) /* HSYS_WHISTLER_BETA1 */
//                         RootData->u.KeyNode.SubKeyLists[Volatile] = 0xBAADF00D;
//                     else
//                         RootData->u.KeyNode.SubKeyLists[Volatile] = HCELL_NIL;
//                     RootData->u.KeyNode.SubKeyCounts[Volatile] = 0;
//                 }
//             }
//             ARM64_UART_TRACE("[HIVE] CmCheckRegistry skipped (HINIT_MEMORY)\r\n");
//         }
//         else
//         {
//             CM_CHECK_REGISTRY_STATUS CheckStatus = CmCheckRegistry(Hive, CheckFlags);
//             if (!CM_CHECK_REGISTRY_SUCCESS(CheckStatus))
//             {
//                 /* Cleanup allocations and fail */
//                 ExDeleteResourceLite(Hive->FlusherLock);
//                 ExFreePoolWithTag(Hive->FlusherLock, TAG_CMHIVE);
//                 ExFreePoolWithTag(Hive->ViewLock, TAG_CMHIVE);
//                 ExFreePoolWithTag(Hive, TAG_CMHIVE);
//                 return STATUS_REGISTRY_CORRUPT;
//             }
//         }
//     }
//
//     ARM64_UART_TRACE("[HIVE] CmCheckRegistry done\r\n");
//
//     /* Reset the loading flag */
//     Hive->HiveIsLoading = FALSE;
//
//     /* Lock the hive list */
//     ExAcquirePushLockExclusive(&CmpHiveListHeadLock);
//
//     /* Insert this hive */
//     InsertHeadList(&CmpHiveListHead, &Hive->HiveList);
//
//     /* Release the lock */
//     ExReleasePushLock(&CmpHiveListHeadLock);
//
//     /* Return the hive and success */
//     *CmHive = Hive;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// CmpDestroyHive(IN PCMHIVE CmHive)
// {
//     /* Remove the hive from the list */
//     ExAcquirePushLockExclusive(&CmpHiveListHeadLock);
//     RemoveEntryList(&CmHive->HiveList);
//     ExReleasePushLock(&CmpHiveListHeadLock);
//
//     /* Destroy the security descriptor cache */
//     CmpDestroySecurityCache(CmHive);
//
//     /* Destroy the view list */
//     CmpDestroyHiveViewList(CmHive);
//
//     /* Delete the flusher lock */
//     ExDeleteResourceLite(CmHive->FlusherLock);
//     ExFreePoolWithTag(CmHive->FlusherLock, TAG_CMHIVE);
//
//     /* Delete the view lock */
//     ExFreePoolWithTag(CmHive->ViewLock, TAG_CMHIVE);
//
//     /* Free the hive storage */
//     HvFree(&CmHive->Hive);
//
//     /* Free the hive */
//     CmpFree(CmHive, TAG_CM);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// CmpOpenHiveFiles(IN PCUNICODE_STRING BaseName,
//                  IN PCWSTR Extension OPTIONAL,
//                  OUT PHANDLE Primary,
//                  OUT PHANDLE Log,
//                  OUT PULONG PrimaryDisposition,
//                  OUT PULONG LogDisposition,
//                  IN BOOLEAN CreateAllowed,
//                  IN BOOLEAN MarkAsSystemHive,
//                  IN BOOLEAN NoBuffering,
//                  OUT PULONG ClusterSize OPTIONAL)
// {
//     HANDLE EventHandle;
//     PKEVENT Event;
//     NTSTATUS Status;
//     UNICODE_STRING FullName, ExtensionName;
//     PWCHAR NameBuffer;
//     USHORT Length;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     ULONG AttributeFlags, ShareMode, DesiredAccess, CreateDisposition, IoFlags;
//     USHORT CompressionState;
//     FILE_STANDARD_INFORMATION FileInformation;
//     FILE_FS_SIZE_INFORMATION FsSizeInformation;
//
//     /* Create event */
//     Status = CmpCreateEvent(NotificationEvent, &EventHandle, &Event);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the full name */
//     RtlInitEmptyUnicodeString(&FullName, NULL, 0);
//     Length = BaseName->Length;
//
//     /* Check if we have an extension */
//     if (Extension)
//     {
//         /* Update the name length */
//         Length += (USHORT)wcslen(Extension) * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//
//         /* Allocate the buffer for the full name */
//         NameBuffer = ExAllocatePoolWithTag(PagedPool, Length, TAG_CM);
//         if (!NameBuffer)
//         {
//             /* Fail */
//             ObDereferenceObject(Event);
//             ZwClose(EventHandle);
//             return STATUS_NO_MEMORY;
//         }
//
//         /* Build the full name */
//         FullName.Buffer = NameBuffer;
//         FullName.MaximumLength = Length;
//         RtlCopyUnicodeString(&FullName, BaseName);
//     }
//     else
//     {
//         /* The base name is the full name */
//         FullName = *BaseName;
//         NameBuffer = NULL;
//     }
//
//     /* Initialize the attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FullName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Check if we can create the hive */
//     if (CreateAllowed && !CmpShareSystemHives)
//     {
//         /* Open only or create */
//         CreateDisposition = FILE_OPEN_IF;
//     }
//     else
//     {
//         /* Open only */
//         CreateDisposition = FILE_OPEN;
//     }
//
//     /* Setup the flags */
//     // FIXME : FILE_OPEN_FOR_BACKUP_INTENT is unimplemented and breaks 3rd stage boot
//     IoFlags = //FILE_OPEN_FOR_BACKUP_INTENT |
//               FILE_NO_COMPRESSION |
//               FILE_RANDOM_ACCESS |
//               (NoBuffering ? FILE_NO_INTERMEDIATE_BUFFERING : 0);
//
//     /* Set share and access modes */
//     if (CmpMiniNTBoot && CmpShareSystemHives)
//     {
//         /* We're on Live CD or otherwise sharing */
//         DesiredAccess = FILE_READ_DATA;
//         ShareMode = FILE_SHARE_READ;
//     }
//     else
//     {
//         /* We want to write exclusively */
//         ShareMode = 0;
//         DesiredAccess = FILE_READ_DATA | FILE_WRITE_DATA;
//     }
//
//     /* Default attributes */
//     AttributeFlags = FILE_ATTRIBUTE_NORMAL;
//
//     /* Now create the file.
//      * Note: We use FILE_SYNCHRONOUS_IO_NONALERT here to simplify CmpFileRead/CmpFileWrite.
//      *       Windows does async I/O and therefore does not use this flag (or SYNCHRONIZE).
//      */
//     Status = ZwCreateFile(Primary,
//                           DesiredAccess | SYNCHRONIZE,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           AttributeFlags,
//                           ShareMode,
//                           CreateDisposition,
//                           FILE_SYNCHRONOUS_IO_NONALERT | IoFlags,
//                           NULL,
//                           0);
//     /* Check if anything failed until now */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwCreateFile(%wZ) failed, Status 0x%08lx.\n", ObjectAttributes.ObjectName, Status);
//
//         /* Close handles and free buffers */
//         if (NameBuffer) ExFreePoolWithTag(NameBuffer, TAG_CM);
//         ObDereferenceObject(Event);
//         ZwClose(EventHandle);
//         *Primary = NULL;
//         return Status;
//     }
//
//     if (MarkAsSystemHive)
//     {
//         /* We opened it, mark it as a system hive */
//         Status = ZwFsControlFile(*Primary,
//                                  EventHandle,
//                                  NULL,
//                                  NULL,
//                                  &IoStatusBlock,
//                                  FSCTL_MARK_AS_SYSTEM_HIVE,
//                                  NULL,
//                                  0,
//                                  NULL,
//                                  0);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* If we don't support it, ignore the failure */
//         if (Status == STATUS_INVALID_DEVICE_REQUEST) Status = STATUS_SUCCESS;
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* Close handles and free buffers */
//             if (NameBuffer) ExFreePoolWithTag(NameBuffer, TAG_CM);
//             ObDereferenceObject(Event);
//             ZwClose(EventHandle);
//             ZwClose(*Primary);
//             *Primary = NULL;
//             return Status;
//         }
//     }
//
//     /* Disable compression */
//     CompressionState = 0;
//     Status = ZwFsControlFile(*Primary,
//                              EventHandle,
//                              NULL,
//                              NULL,
//                              &IoStatusBlock,
//                              FSCTL_SET_COMPRESSION,
//                              &CompressionState,
//                              sizeof(CompressionState),
//                              NULL,
//                              0);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for completion */
//         KeWaitForSingleObject(Event,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     /* Get the disposition */
//     *PrimaryDisposition = (ULONG)IoStatusBlock.Information;
//     if (IoStatusBlock.Information != FILE_CREATED)
//     {
//         /* Check how large the file is */
//         Status = ZwQueryInformationFile(*Primary,
//                                         &IoStatusBlock,
//                                         &FileInformation,
//                                         sizeof(FileInformation),
//                                         FileStandardInformation);
//         if (NT_SUCCESS(Status))
//         {
//             /* Check if it's 0 bytes */
//             if (!FileInformation.EndOfFile.QuadPart)
//             {
//                 /* Assume it's a new file */
//                 *PrimaryDisposition = FILE_CREATED;
//             }
//         }
//     }
//
//     /* Check if the caller wants cluster size returned */
//     if (ClusterSize)
//     {
//         /* Query it */
//         Status = ZwQueryVolumeInformationFile(*Primary,
//                                               &IoStatusBlock,
//                                               &FsSizeInformation,
//                                               sizeof(FsSizeInformation),
//                                               FileFsSizeInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Close handles and free buffers */
//             if (NameBuffer) ExFreePoolWithTag(NameBuffer, TAG_CM);
//             ObDereferenceObject(Event);
//             ZwClose(EventHandle);
//             return Status;
//         }
//
//         /* Check if the sector size is invalid */
//         if (FsSizeInformation.BytesPerSector > HBLOCK_SIZE)
//         {
//             /* Close handles and free buffers */
//             if (NameBuffer) ExFreePoolWithTag(NameBuffer, TAG_CM);
//             ObDereferenceObject(Event);
//             ZwClose(EventHandle);
//             return STATUS_CANNOT_LOAD_REGISTRY_FILE;
//         }
//
//         /* Return cluster size */
//         *ClusterSize = max(1, FsSizeInformation.BytesPerSector / HSECTOR_SIZE);
//     }
//
//     /* Check if we don't need to create a log file */
//     if (!Extension)
//     {
//         /* We're done, close handles */
//         ObDereferenceObject(Event);
//         ZwClose(EventHandle);
//         return STATUS_SUCCESS;
//     }
//
//     /* Check if we can create the hive */
//     CreateDisposition = CmpShareSystemHives ? FILE_OPEN : FILE_OPEN_IF;
//     if (*PrimaryDisposition == FILE_CREATED)
//     {
//         /* Over-write the existing log file, since this is a new hive */
//         CreateDisposition = FILE_SUPERSEDE;
//     }
//
//     /* Setup the name */
//     RtlInitUnicodeString(&ExtensionName, Extension);
//     RtlAppendUnicodeStringToString(&FullName, &ExtensionName);
//
//     /* Initialize the attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FullName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Setup the flags */
//     IoFlags = FILE_NO_COMPRESSION | FILE_NO_INTERMEDIATE_BUFFERING;
//
//     /* Check if this is a log file */
//     if (!_wcsnicmp(Extension, L".log", 4))
//     {
//         /* Hide log files */
//         AttributeFlags |= FILE_ATTRIBUTE_HIDDEN;
//     }
//
//     /* Now create the file.
//      * Note: We use FILE_SYNCHRONOUS_IO_NONALERT here to simplify CmpFileRead/CmpFileWrite.
//      *       Windows does async I/O and therefore does not use this flag (or SYNCHRONIZE).
//      */
//     Status = ZwCreateFile(Log,
//                           DesiredAccess | SYNCHRONIZE,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           AttributeFlags,
//                           ShareMode,
//                           CreateDisposition,
//                           FILE_SYNCHRONOUS_IO_NONALERT | IoFlags,
//                           NULL,
//                           0);
//     if (NT_SUCCESS(Status) && MarkAsSystemHive)
//     {
//         /* We opened it, mark it as a system hive */
//         Status = ZwFsControlFile(*Log,
//                                  EventHandle,
//                                  NULL,
//                                  NULL,
//                                  &IoStatusBlock,
//                                  FSCTL_MARK_AS_SYSTEM_HIVE,
//                                  NULL,
//                                  0,
//                                  NULL,
//                                  0);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* If we don't support it, ignore the failure */
//         if (Status == STATUS_INVALID_DEVICE_REQUEST) Status = STATUS_SUCCESS;
//
//         /* If we failed, close the handle */
//         if (!NT_SUCCESS(Status)) ZwClose(*Log);
//     }
//
//     /* Check if anything failed until now */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Clear the handle */
//         *Log = NULL;
//     }
//     else
//     {
//         /* Disable compression */
//         Status = ZwFsControlFile(*Log,
//                                  EventHandle,
//                                  NULL,
//                                  NULL,
//                                  &IoStatusBlock,
//                                  FSCTL_SET_COMPRESSION,
//                                  &CompressionState,
//                                  sizeof(CompressionState),
//                                  NULL,
//                                  0);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//         }
//
//         /* Return the disposition */
//         *LogDisposition = (ULONG)IoStatusBlock.Information;
//     }
//
//     /* We're done, close handles and free buffers */
//     if (NameBuffer) ExFreePoolWithTag(NameBuffer, TAG_CM);
//     ObDereferenceObject(Event);
//     ZwClose(EventHandle);
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// CmpCloseHiveFiles(IN PCMHIVE Hive)
// {
//     ULONG i;
//
//     for (i = 0; i < HFILE_TYPE_MAX; i++)
//     {
//         if (Hive->FileHandles[i] != NULL)
//         {
//             ZwClose(Hive->FileHandles[i]);
//             Hive->FileHandles[i] = NULL;
//         }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
