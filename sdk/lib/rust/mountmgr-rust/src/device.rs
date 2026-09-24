use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, FILE_FS_DEVICE_INFORMATION, FILE_NAME_INFORMATION, _IO_STATUS_BLOCK, IoAcquireCancelSpinLock, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoGetDeviceObjectPointer, IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCancelRoutine, KeReleaseSemaphore, KeWaitForSingleObject, LIST_ENTRY, MOUNTMGR_CHANGE_NOTIFY_INFO, MOUNTMGR_CREATE_POINT_INPUT, MOUNTMGR_DRIVE_LETTER_INFORMATION, MOUNTMGR_DRIVE_LETTER_TARGET, MOUNTMGR_MOUNT_POINT, MOUNTMGR_MOUNT_POINTS, MOUNTMGR_QUERY_AUTO_MOUNT, MOUNTMGR_SET_AUTO_MOUNT, MOUNTMGR_TARGET_NAME, MOUNTMGR_VOLUME_MOUNT_POINT, MOUNTMGR_VOLUME_PATHS, _OBJECT_ATTRIBUTES, OBJECT_NAME_INFORMATION, ObQueryNameString, ObReferenceObjectByHandle, PARTITION_INFORMATION, PDEVICE_OBJECT, PFILE_NAME_INFORMATION, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PMOUNTMGR_CHANGE_NOTIFY_INFO, PMOUNTMGR_CREATE_POINT_INPUT, PMOUNTMGR_DRIVE_LETTER_INFORMATION, PMOUNTMGR_DRIVE_LETTER_TARGET, PMOUNTMGR_MOUNT_POINT, PMOUNTMGR_MOUNT_POINTS, PMOUNTMGR_QUERY_AUTO_MOUNT, PMOUNTMGR_SET_AUTO_MOUNT, PMOUNTMGR_TARGET_NAME, PMOUNTMGR_VOLUME_MOUNT_POINT, PMOUNTMGR_VOLUME_PATHS, POBJECT_NAME_INFORMATION, PREPARSE_DATA_BUFFER, PUNICODE_STRING, PsGetCurrentThread, PsGetThreadHardErrorsAreDisabled, PsSetThreadHardErrorsAreDisabled, RTL_QUERY_REGISTRY_TABLE, RtlAppendUnicodeStringToString, RtlCompareUnicodeString, RtlDeleteRegistryValue, RtlInitUnicodeString, RtlPrefixUnicodeString, RtlQueryRegistryValues, ZwClose, ZwFsControlFile, ZwOpenFile, ZwQueryInformationFile, ZwQueryVolumeInformationFile};
use ntkrnl_sys::ddk::{_MOUNTMGR_AUTO_MOUNT_STATE_Enabled as Enabled_AutoMount, _FSINFOCLASS_FileFsDeviceInformation as FileFsDeviceInformation, _FILE_INFORMATION_CLASS_FileNameInformation as FileNameInformation};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{GUID, IO_STACK_LOCATION, IRP, KPROCESSOR_MODE, MOUNTMGR_AUTO_MOUNT_STATE, PRTL_QUERY_REGISTRY_ROUTINE, PRTL_QUERY_REGISTRY_TABLE, _FILE_NAME_INFORMATION, _FILE_OBJECT, _IO_STACK_LOCATION, _LARGE_INTEGER, _MOUNTMGR_QUERY_AUTO_MOUNT, _MOUNTMGR_VOLUME_MOUNT_POINT, _OBJECT_HANDLE_INFORMATION, _OBJECT_NAME_INFORMATION, _UNICODE_STRING};

// IoFileObjectType — kernel DATA export (POBJECT_TYPE); raw-dylib import (i386 needs the
// __imp_ thunk; works on all arches).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" { static IoFileObjectType: *mut ntkrnl_sys::ddk::_OBJECT_TYPE; }

extern "C" {
    fn MountMgrCreatePointWorker( DeviceExtension: PDEVICE_EXTENSION, SymbolicName: *mut _UNICODE_STRING, TargetDeviceName: *mut _UNICODE_STRING, ) -> NTSTATUS;
    static DosDevices: _UNICODE_STRING;
    static DeviceFloppy: _UNICODE_STRING;
    static DeviceCdRom: _UNICODE_STRING;
    fn DeleteSymbolicLinkNameFromMemory(DeviceExtension: PDEVICE_EXTENSION, SymbolicName: *mut _UNICODE_STRING, b: u8);
    static DatabasePath: PWSTR;
    fn ReconcileAllDatabasesWithMaster(DeviceExtension: PDEVICE_EXTENSION);
    fn WaitForOnlinesToComplete(DeviceExtension: PDEVICE_EXTENSION);
}
extern "system" {
    fn MountMgrCancel(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
    fn RtlWriteRegistryValue( RelativeTo: u32, Path: *const u16, ValueName: *const u16, ValueType: u32, ValueData: *mut core::ffi::c_void, ValueLength: u32, ) -> NTSTATUS;
    fn RtlCompareMemory(Source1: *const core::ffi::c_void, Source2: *const core::ffi::c_void, Length: usize) -> usize;
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: usize, Tag: u32) -> *mut ::core::ffi::c_void;
    fn MOUNTMGR_IS_DRIVE_LETTER(name: *const _UNICODE_STRING) -> BOOLEAN;
    fn RtlEqualUnicodeString( String1: *mut _UNICODE_STRING, String2: *mut _UNICODE_STRING, CaseInSensitive: BOOLEAN, ) -> BOOLEAN;
    fn ReconcileThisDatabaseWithMasterWorker(Context: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
const MAX_DEVICES: u32 = 0x3E8;
const LETTER_POSITION: usize = 12;
const COLON_POSITION: usize = 13;
const DRIVE_LETTER_LENGTH: u16 = 28;
const UNICODE_NULL: u16 = 0;
const PagedPool: u32 = 1;
const FALSE: BOOLEAN = 0u8;
const TRUE: BOOLEAN = 1u8;

const MAXIMUM_REPARSE_DATA_BUFFER_SIZE: u32 = 16 * 1024;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
const FILE_OPEN_REPARSE_POINT: u32 = 0x00200000;
const FSCTL_GET_REPARSE_POINT: u32 = 0x000900A8;

const MAXSHORT: u16 = 32767;

// CTL_CODE('m'=0x6D, fn, METHOD_BUFFERED, access) — header-exact (ddk/mountmgr.h).
// The previous block was FABRICATED (0x6DE0xx family, wrong fn+access encoding);
// no mountmgr ioctl could ever match, so AUTO_DL/NEXT_DRIVE_LETTER were dead and
// no volume ever received a drive letter (USB-disk e2e, 2026-07-06).
const IOCTL_MOUNTMGR_CREATE_POINT: u32 = 0x6DC000;
const IOCTL_MOUNTMGR_DELETE_POINTS: u32 = 0x6DC004;
const IOCTL_MOUNTMGR_QUERY_POINTS: u32 = 0x6D0008;
const IOCTL_MOUNTMGR_DELETE_POINTS_DBONLY: u32 = 0x6DC00C;
const IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER: u32 = 0x6DC010;
const IOCTL_MOUNTMGR_AUTO_DL_ASSIGNMENTS: u32 = 0x6DC014;
const IOCTL_MOUNTMGR_VOLUME_MOUNT_POINT_CREATED: u32 = 0x6DC018;
const IOCTL_MOUNTMGR_VOLUME_MOUNT_POINT_DELETED: u32 = 0x6DC01C;
const IOCTL_MOUNTMGR_CHANGE_NOTIFY: u32 = 0x6D4020;
const IOCTL_MOUNTMGR_KEEP_LINKS_WHEN_OFFLINE: u32 = 0x6DC024;
const IOCTL_MOUNTMGR_CHECK_UNPROCESSED_VOLUMES: u32 = 0x6D4028;
const IOCTL_MOUNTMGR_VOLUME_ARRIVAL_NOTIFICATION: u32 = 0x6D402C;
const IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH: u32 = 0x6D0030;
const IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS: u32 = 0x6D0034;
const IOCTL_MOUNTMGR_SCRUB_REGISTRY: u32 = 0x6DC038;
const IOCTL_MOUNTMGR_QUERY_AUTO_MOUNT: u32 = 0x6D003C;
const IOCTL_MOUNTMGR_SET_AUTO_MOUNT: u32 = 0x6DC040;
const FILE_DEVICE_DISK: u32 = 7;
const FILE_DEVICE_VIRTUAL_DISK: u32 = 36;
const FILE_REMOTE_DEVICE: u32 = 0x00000010;
const PARTITION_NTFT: u32 = 0x80;
const VALID_NTFT: u32 = 0xC0;
const PARTITION_HUGE: u32 = 6;
const PARTITION_IFS: u32 = 7;
const PARTITION_FAT32: u32 = 11;
const PARTITION_FAT32_XINT13: u32 = 12;
#[no_mangle]
pub unsafe extern "C" fn MountMgrChangeNotify(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS = 0;
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let ChangeNotify: PMOUNTMGR_CHANGE_NOTIFY_INFO = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_CHANGE_NOTIFY_INFO;
    if ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_CHANGE_NOTIFY_INFO>()
        || ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
            < core::mem::size_of::<MOUNTMGR_CHANGE_NOTIFY_INFO>()
    {
        return STATUS_INVALID_PARAMETER;
    }
    if (*DeviceExtension).EpicNumber != (*ChangeNotify).EpicNumber {
        (*ChangeNotify).EpicNumber = (*DeviceExtension).EpicNumber;
        (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTMGR_CHANGE_NOTIFY_INFO>() as _;
        return STATUS_SUCCESS;
    }
    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldIrql));
    if (*Irp).Cancel != 0 {
        Status = STATUS_CANCELLED;
    } else {
        InsertTailList(
            core::ptr::addr_of_mut!((*DeviceExtension).IrpListHead),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
        );
        IoMarkIrpPending(Irp);
        IoSetCancelRoutine(
            Irp,
            Some(core::mem::transmute::<
                unsafe extern "system" fn(PDEVICE_OBJECT, PIRP),
                _,
            >(MountMgrCancel)),
        );
        Status = STATUS_PENDING;
    }
    IoReleaseCancelSpinLock(OldIrql);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn MountmgrWriteNoAutoMount(
    DeviceExtension: *mut DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Value: u32 = (*DeviceExtension).NoAutoMount as u32;
    let wide_str: [u16; 12] = [78, 111, 65, 117, 116, 111, 77, 111, 117, 110, 116, 0];
    let p_wide = &wide_str as *const u16;
    return RtlWriteRegistryValue(
        RTL_REGISTRY_ABSOLUTE,
        (*DeviceExtension).RegistryPath.Buffer,
        p_wide,
        REG_DWORD,
        &raw mut Value as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrSetAutoMount(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Stack: PIO_STACK_LOCATION = ::core::ptr::null_mut::<IO_STACK_LOCATION>();
    let mut SetState: PMOUNTMGR_SET_AUTO_MOUNT = ::core::ptr::null_mut::<MOUNTMGR_SET_AUTO_MOUNT>();

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < ::core::mem::size_of::<MOUNTMGR_SET_AUTO_MOUNT>() as usize
    {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_INVALID_PARAMETER;
    }

    SetState = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_SET_AUTO_MOUNT;

    if ((*SetState).NewState != Enabled_AutoMount)
        == ((*DeviceExtension).NoAutoMount != 0)
    {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_SUCCESS;
    }

    (*DeviceExtension).NoAutoMount = ((*SetState).NewState != Enabled_AutoMount) as u8;
    (*Irp).IoStatus.Information = 0 as _;
    return MountmgrWriteNoAutoMount(DeviceExtension);
}

#[no_mangle]
pub unsafe extern "C" fn MountMgrQueryAutoMount(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Stack: PIO_STACK_LOCATION = ::core::ptr::null_mut::<_IO_STACK_LOCATION>();
    let mut QueryState: PMOUNTMGR_QUERY_AUTO_MOUNT =
        ::core::ptr::null_mut::<_MOUNTMGR_QUERY_AUTO_MOUNT>();
    Stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < ::core::mem::size_of::<MOUNTMGR_QUERY_AUTO_MOUNT>() as usize
    {
        (*Irp).IoStatus.Information = 0 as _;
        return STATUS_INVALID_PARAMETER;
    }
    QueryState = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_QUERY_AUTO_MOUNT;
    (*QueryState).CurrentState =
        ((*DeviceExtension).NoAutoMount == 0) as MOUNTMGR_AUTO_MOUNT_STATE;
    (*Irp).IoStatus.Information =
        ::core::mem::size_of::<MOUNTMGR_QUERY_AUTO_MOUNT>() as _;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ScrubRegistryRoutine(
    ValueName: PWSTR,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut::<LIST_ENTRY>();
    let mut DeviceInfo: *mut DEVICE_INFORMATION = core::ptr::null_mut::<DEVICE_INFORMATION>();
    let Continue: *mut u8 = EntryContext as *mut u8;
    let DeviceExtension: *mut DEVICE_EXTENSION = Context as *mut DEVICE_EXTENSION;

    if ValueType != REG_BINARY {
        return STATUS_SUCCESS;
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead)) {
        NextEntry = (*DeviceExtension).DeviceListHead.Flink as *mut LIST_ENTRY;
        while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
            DeviceInfo = (NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
            ) as *mut DEVICE_INFORMATION;

            if !((*DeviceInfo).UniqueId.is_null()
                || (*(*DeviceInfo).UniqueId).UniqueIdLength as u32 != ValueLength)
            {
                if RtlCompareMemory(
                    core::ptr::addr_of_mut!((*(*DeviceInfo).UniqueId).UniqueId) as *mut u8 as *const core::ffi::c_void,
                    ValueData as *const core::ffi::c_void,
                    ValueLength as usize,
                ) == ValueLength as usize
                {
                    return STATUS_SUCCESS;
                }
            }
            NextEntry = (*NextEntry).Flink as *mut LIST_ENTRY;
        }
    }

    Status = RtlDeleteRegistryValue(
        RTL_REGISTRY_ABSOLUTE as u32,
        DatabasePath as *const u16,
        ValueName as *const u16,
    );

    if !(Status >= 0) {
        *Continue = 1u8;
        return STATUS_UNSUCCESSFUL;
    }

    *Continue = 0u8;
    Status
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrScrubRegistry(mut DeviceExtension: PDEVICE_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Continue: u8 = 0;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    loop {
        ntkrnl_sys::rtl::RtlZeroMemory(
            &raw mut QueryTable as *mut ::core::ffi::c_void,
            core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
        );
        QueryTable[0].QueryRoutine = Some(core::mem::transmute::<
            unsafe extern "system" fn(PWSTR, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> NTSTATUS,
            _,
        >(ScrubRegistryRoutine));
        QueryTable[0].EntryContext = &raw mut Continue as *mut ::core::ffi::c_void;
        Continue = 0;
        Status = RtlQueryRegistryValues(
            RTL_REGISTRY_ABSOLUTE as u32,
            DatabasePath,
            &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
            DeviceExtension as *mut ::core::ffi::c_void,
            ::core::ptr::null_mut(),
        );
        if Continue == 0 {
            break;
        }
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrCreatePoint(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut MaxLength: ULONG = 0;
    let Stack: PIO_STACK_LOCATION;
    let Point: PMOUNTMGR_CREATE_POINT_INPUT;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>()
    {
        return STATUS_INVALID_PARAMETER;
    }

    Point = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_CREATE_POINT_INPUT;

    MaxLength = if (*Point).DeviceNameOffset as i32 + (*Point).DeviceNameLength as i32
        > (*Point).SymbolicLinkNameLength as i32 + (*Point).SymbolicLinkNameOffset as i32
    {
        ((*Point).DeviceNameOffset as i32 + (*Point).DeviceNameLength as i32) as ULONG
    } else {
        ((*Point).SymbolicLinkNameLength as i32 + (*Point).SymbolicLinkNameOffset as i32) as ULONG
    } ;

    if MaxLength > (*Stack).Parameters.DeviceIoControl.InputBufferLength {
        return STATUS_INVALID_PARAMETER;
    }

    SymbolicName.Length = (*Point).SymbolicLinkNameLength;
    SymbolicName.MaximumLength = (*Point).SymbolicLinkNameLength;
    DeviceName.Length = (*Point).DeviceNameLength;
    DeviceName.MaximumLength = (*Point).DeviceNameLength;
    SymbolicName.Buffer = ((Point as *mut u8).offset((*Point).SymbolicLinkNameOffset as isize))
        as *mut u16;
    DeviceName.Buffer = ((Point as *mut u8).offset((*Point).DeviceNameOffset as isize))
        as *mut u16;

    return MountMgrCreatePointWorker(
        DeviceExtension,
        &raw mut SymbolicName,
        &raw mut DeviceName,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrCheckUnprocessedVolumes(
    DeviceExtension: *mut DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
    let mut ArrivalStatus: NTSTATUS = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).OfflineDeviceListHead)) {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
            IO_NO_INCREMENT as i32,
            1,
            0u8,
        );
        return STATUS_SUCCESS;
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
        IO_NO_INCREMENT as i32,
        1,
        0u8,
    );

    while !IsListEmpty( core::ptr::addr_of_mut!((*DeviceExtension).OfflineDeviceListHead)) {
        NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceExtension).OfflineDeviceListHead));
        DeviceInformation = (NextEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;
        ArrivalStatus = MountMgrMountedDeviceArrival(
            DeviceExtension,
            core::ptr::addr_of_mut!((*DeviceInformation).SymbolicName),
            (*DeviceInformation).ManuallyRegistered,
        );
        MountMgrFreeDeadDeviceInfo(DeviceInformation);
        if Status >= 0 {
            Status = ArrivalStatus;
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn IsFtVolume(mut SymbolicName: *mut _UNICODE_STRING) -> BOOLEAN {
    let mut Status: NTSTATUS = 0;
    let mut FileObject: PFILE_OBJECT = ::core::ptr::null_mut();
    let mut PartitionInfo: PARTITION_INFORMATION = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut();
    let mut FileDeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut();

    Status = IoGetDeviceObjectPointer(
        SymbolicName,
        FILE_READ_ATTRIBUTES,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        return 0;
    }

    FileDeviceObject = (*FileObject).DeviceObject;
    DeviceObject = IoGetAttachedDeviceReference(FileDeviceObject);

    if ((*FileDeviceObject).Characteristics & FILE_REMOVABLE_MEDIA) != 0 {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return 0;
    }

    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    Status = MountMgrSendSyncDeviceIoCtl(
        IOCTL_DISK_GET_PARTITION_INFO,
        DeviceObject,
        core::ptr::null_mut(),
        0u32,
        &raw mut PartitionInfo as *mut core::ffi::c_void,
        core::mem::size_of::<PARTITION_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    if !(Status >= 0) {
        return 0;
    }

    return ((PartitionInfo.PartitionType as i32 & PARTITION_NTFT as i32) != 0
        && ((PartitionInfo.PartitionType as i32 & !(VALID_NTFT as i32)) == PARTITION_HUGE as i32
            || (PartitionInfo.PartitionType as i32 & !(VALID_NTFT as i32)) == PARTITION_IFS as i32
            || (PartitionInfo.PartitionType as i32 & !(VALID_NTFT as i32)) == PARTITION_FAT32 as i32
            || (PartitionInfo.PartitionType as i32 & !(VALID_NTFT as i32)) == PARTITION_FAT32_XINT13 as i32)) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessSuggestedDriveLetters(DeviceExtension: *mut DEVICE_EXTENSION) {
    let mut NameBuffer: [u16; 14] = [0; 14];
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();

    if IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead)) {
        return;
    }

    NextEntry = (*DeviceExtension).DeviceListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        DeviceInformation = (NextEntry as *mut u8).offset(-(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry) as isize)) as *mut DEVICE_INFORMATION;

        if (*DeviceInformation).SuggestedDriveLetter == 0xFFu8 {
            if HasDriveLetter(DeviceInformation) == 0
                && HasNoDriveLetterEntry((*DeviceInformation).UniqueId) == 0u8
            {
                CreateNoDriveLetterEntry((*DeviceInformation).UniqueId);
            }
            (*DeviceInformation).SuggestedDriveLetter = 0;
        } else if (*DeviceInformation).SuggestedDriveLetter != 0
            && HasNoDriveLetterEntry((*DeviceInformation).UniqueId) == 0u8
        {
            SymbolicName.Buffer = NameBuffer.as_mut_ptr();
            core::ptr::copy_nonoverlapping(
                DosDevices.Buffer as *const u8,
                NameBuffer.as_mut_ptr() as *mut u8,
                DosDevices.Length as usize,
            );
            NameBuffer[LETTER_POSITION as usize] = (*DeviceInformation).SuggestedDriveLetter as u16;
            NameBuffer[COLON_POSITION as usize] = ':' as u16;
            SymbolicName.MaximumLength = DRIVE_LETTER_LENGTH as u16;
            SymbolicName.Length = SymbolicName.MaximumLength;
            MountMgrCreatePointWorker(
                DeviceExtension,
                core::ptr::addr_of_mut!(SymbolicName),
                core::ptr::addr_of_mut!((*DeviceInformation).DeviceName),
            );
        }
        NextEntry = (*NextEntry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrNextDriveLetterWorker(
    DeviceExtension: PDEVICE_EXTENSION,
    DeviceName: PUNICODE_STRING,
    DriveLetterInfo: PMOUNTMGR_DRIVE_LETTER_INFORMATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut DriveLetter: u8 = 0;
    let mut NextEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut UniqueId: PMOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut Removable: BOOLEAN = 0;
    let mut GptDriveLetter: BOOLEAN = 0;
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut NameBuffer: [u16; 28] = [0; 28];
    let mut SymlinkInformation: PSYMLINK_INFORMATION = core::ptr::null_mut();
    let mut TargetDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut current_block: u64 = 0;

    if (*DeviceExtension).ProcessedSuggestions == 0 {
        ProcessSuggestedDriveLetters(DeviceExtension);
        (*DeviceExtension).ProcessedSuggestions = 1;
    }

    Status = QueryDeviceInformation(
        DeviceName,
        core::ptr::addr_of_mut!(TargetDeviceName),
        core::ptr::null_mut::<PMOUNTDEV_UNIQUE_ID>(),
        core::ptr::addr_of_mut!(Removable),
        core::ptr::addr_of_mut!(GptDriveLetter),
        core::ptr::null_mut::<u8>(),
        core::ptr::null_mut::<GUID>(),
        core::ptr::null_mut::<u8>(),
    );
    if Status < 0 {
        DbgPrint(
            c"[MNTMGR] NextDlWorker: QueryDeviceInformation(%wZ) FAILED 0x%x\n".as_ptr(),
            DeviceName,
            Status as u32,
        );
        return Status;
    }

    NextEntry = (*DeviceExtension).DeviceListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        DeviceInformation = ((NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
        )) as PDEVICE_INFORMATION;
        if RtlCompareUnicodeString(
            core::ptr::addr_of_mut!((*DeviceInformation).DeviceName) as *const _UNICODE_STRING,
            core::ptr::addr_of_mut!(TargetDeviceName) as *const _UNICODE_STRING,
            1,
        ) == 0
        {
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        DbgPrint(
            c"[MNTMGR] NextDlWorker: no DeviceInformation matches %wZ\n".as_ptr(),
            core::ptr::addr_of_mut!(TargetDeviceName),
        );
        ExFreePoolWithTag(TargetDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    (*DeviceInformation).LetterAssigned = 1;
    (*DriveLetterInfo).DriveLetterWasAssigned = (*DeviceInformation).LetterAssigned;

    NextEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead) {
        SymlinkInformation = ((NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
        )) as PSYMLINK_INFORMATION;
        if IsDriveLetter(core::ptr::addr_of_mut!((*SymlinkInformation).Name)) != 0u8
            && (*SymlinkInformation).Online != 0
        {
            (*DriveLetterInfo).DriveLetterWasAssigned = 0;
            (*DriveLetterInfo).CurrentDriveLetter =
                *(*SymlinkInformation).Name.Buffer.offset(LETTER_POSITION as isize) as u8;
            break;
        } else {
            NextEntry = (*NextEntry).Flink;
        }
    }

    if NextEntry == core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead) {
        if GptDriveLetter == 0u8 || HasNoDriveLetterEntry((*DeviceInformation).UniqueId) != 0u8 {
            DbgPrint(
                c"[MNTMGR] NextDlWorker: bail GptDl=%u NoDlEntry path for %wZ\n".as_ptr(),
                GptDriveLetter as u32,
                core::ptr::addr_of_mut!(TargetDeviceName),
            );
            (*DriveLetterInfo).DriveLetterWasAssigned = 0;
            (*DriveLetterInfo).CurrentDriveLetter = 0;
            current_block = 10912197556463997057;
        } else {
            current_block = 14763689060501151050;
        }
    } else {
        current_block = 14763689060501151050;
    }

    match current_block {
        14763689060501151050 => {
            if (*DeviceExtension).NoAutoMount != 0 && Removable == 0u8 {
                if (*DriveLetterInfo).DriveLetterWasAssigned != 0 {
                    (*DriveLetterInfo).DriveLetterWasAssigned = 0;
                    (*DriveLetterInfo).CurrentDriveLetter = 0;
                    current_block = 10912197556463997057;
                } else {
                    current_block = 17281240262373992796;
                }
            } else {
                current_block = 17281240262373992796;
            }
            match current_block {
                10912197556463997057 => {}
                _ => {
                    if !((*DriveLetterInfo).DriveLetterWasAssigned == 0) {
                        if RtlPrefixUnicodeString(
                            core::ptr::addr_of!(DeviceFloppy) as *const _UNICODE_STRING,
                            core::ptr::addr_of!(TargetDeviceName) as *const _UNICODE_STRING,
                            1,
                        ) != 0
                        {
                            DriveLetter = 'A' as u8;
                        } else if RtlPrefixUnicodeString(
                            core::ptr::addr_of!(DeviceCdRom) as *const _UNICODE_STRING,
                            core::ptr::addr_of!(TargetDeviceName) as *const _UNICODE_STRING,
                            1,
                        ) != 0
                        {
                            DriveLetter = 'D' as u8;
                        } else {
                            DriveLetter = 'C' as u8;
                        }
                        if !((*DeviceInformation).SuggestedDriveLetter as i32 != -(1i32) as u8 as i32)
                        {
                            RtlAssert(
                                b"DeviceInformation->SuggestedDriveLetter != (UCHAR)-1\0".as_ptr()
                                    as *const core::ffi::c_void,
                                b"drivers/storage/mountmgr/device.c\0".as_ptr()
                                    as *const core::ffi::c_void,
                                551,
                                core::ptr::null_mut(),
                            );
                        }
                        if (*DeviceInformation).SuggestedDriveLetter == 0
                            && IsFtVolume(core::ptr::addr_of_mut!(
                                (*DeviceInformation).DeviceName
                            )) != 0
                        {
                            (*DriveLetterInfo).DriveLetterWasAssigned = 0;
                            (*DriveLetterInfo).CurrentDriveLetter = 0;
                        } else {
                            core::ptr::copy_nonoverlapping(
                                DosDevices.Buffer as *const core::ffi::c_void,
                                NameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                                DosDevices.Length as usize,
                            );
                            NameBuffer[COLON_POSITION] = ':' as u16;
                            SymbolicName.Buffer = NameBuffer.as_mut_ptr();
                            SymbolicName.MaximumLength = DRIVE_LETTER_LENGTH;
                            SymbolicName.Length = SymbolicName.MaximumLength;
                            if (*DeviceInformation).SuggestedDriveLetter != 0 {
                                (*DriveLetterInfo).CurrentDriveLetter =
                                    (*DeviceInformation).SuggestedDriveLetter;
                                NameBuffer[LETTER_POSITION] =
                                    (*DeviceInformation).SuggestedDriveLetter as u16;
                                Status = MountMgrCreatePointWorker(
                                    DeviceExtension,
                                    core::ptr::addr_of_mut!(SymbolicName),
                                    core::ptr::addr_of_mut!(TargetDeviceName),
                                );
                                if Status >= 0 {
                                    current_block = 10912197556463997057;
                                } else {
                                    current_block = 13460095289871124136;
                                }
                            } else {
                                current_block = 13460095289871124136;
                            }
                            match current_block {
                                10912197556463997057 => {}
                                _ => {
                                    (*DriveLetterInfo).CurrentDriveLetter = DriveLetter;
                                    while (*DriveLetterInfo).CurrentDriveLetter as i32 <= 'Z' as i32
                                    {
                                        NameBuffer[LETTER_POSITION] =
                                            (*DriveLetterInfo).CurrentDriveLetter as u16;
                                        Status = MountMgrCreatePointWorker(
                                            DeviceExtension,
                                            core::ptr::addr_of_mut!(SymbolicName),
                                            core::ptr::addr_of_mut!(TargetDeviceName),
                                        );
                                        if Status >= 0 {
                                            break;
                                        }
                                        (*DriveLetterInfo).CurrentDriveLetter = (*DriveLetterInfo)
                                            .CurrentDriveLetter
                                            .wrapping_add(1);
                                    }
                                    if (*DriveLetterInfo).CurrentDriveLetter as i32 > 'Z' as i32 {
                                        (*DriveLetterInfo).DriveLetterWasAssigned = 0;
                                        (*DriveLetterInfo).CurrentDriveLetter = 0;
                                        Status = QueryDeviceInformation(
                                            core::ptr::addr_of_mut!(TargetDeviceName),
                                            core::ptr::null_mut::<_UNICODE_STRING>(),
                                            core::ptr::addr_of_mut!(UniqueId),
                                            core::ptr::null_mut::<BOOLEAN>(),
                                            core::ptr::null_mut::<BOOLEAN>(),
                                            core::ptr::null_mut::<u8>(),
                                            core::ptr::null_mut::<GUID>(),
                                            core::ptr::null_mut::<u8>(),
                                        );
                                        if Status >= 0 {
                                            CreateNoDriveLetterEntry(UniqueId);
                                            ExFreePoolWithTag(
                                                UniqueId as *mut core::ffi::c_void,
                                                1098149453u32,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    ExFreePoolWithTag(TargetDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrNextDriveLetter(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let Stack: PIO_STACK_LOCATION;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let DriveLetterTarget: PMOUNTMGR_DRIVE_LETTER_TARGET;
    let mut DriveLetterInformation: MOUNTMGR_DRIVE_LETTER_INFORMATION = MOUNTMGR_DRIVE_LETTER_INFORMATION { DriveLetterWasAssigned: 0, CurrentDriveLetter: 0 };

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_TARGET>()
        || ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize)
            < core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_INFORMATION>()
    {
        return STATUS_INVALID_PARAMETER;
    }

    DriveLetterTarget = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_DRIVE_LETTER_TARGET;

    if (2i32 + (*DriveLetterTarget).DeviceNameLength as i32) as u32
        > (*Stack).Parameters.DeviceIoControl.InputBufferLength
    {
        return STATUS_INVALID_PARAMETER;
    }

    DeviceName.Buffer = core::ptr::addr_of_mut!((*DriveLetterTarget).DeviceName) as *mut u16;
    DeviceName.MaximumLength = (*DriveLetterTarget).DeviceNameLength;
    DeviceName.Length = DeviceName.MaximumLength;

    Status = MountMgrNextDriveLetterWorker(
        DeviceExtension,
        &mut DeviceName,
        &mut DriveLetterInformation,
    );

    if NT_SUCCESS(Status) {
        *((*Irp).AssociatedIrp.SystemBuffer as *mut MOUNTMGR_DRIVE_LETTER_INFORMATION) =
            DriveLetterInformation;
        (*Irp).IoStatus.Information =
            core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_INFORMATION>() as _;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrQuerySystemVolumeNameQueryRoutine(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let mut ValueString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<u16>(),
    };
    let mut SystemVolumeName: PUNICODE_STRING = ::core::ptr::null_mut::<_UNICODE_STRING>();

    // UNREFERENCED_PARAMETER substitutions
    let _ = ValueName;
    let _ = ValueLength;
    let _ = EntryContext;

    if ValueType != REG_SZ as u32 {
        return STATUS_SUCCESS;
    }

    RtlInitUnicodeString(&raw mut ValueString, ValueData as *const u16);
    SystemVolumeName = Context as PUNICODE_STRING;

    (*SystemVolumeName).Length = ValueString.Length;
    (*SystemVolumeName).MaximumLength = (ValueString.Length as usize + ::core::mem::size_of::<u16>()) as u16;
    (*SystemVolumeName).Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*SystemVolumeName).MaximumLength as usize,
        1098149453u32,
    ) as *mut u16;

    if !(*SystemVolumeName).Buffer.is_null() {
        ::core::ptr::copy_nonoverlapping(
            ValueData as *const u8,
            (*SystemVolumeName).Buffer as *mut u8,
            ValueString.Length as usize,
        );
        *(*SystemVolumeName).Buffer.offset(
            (ValueString.Length as usize / ::core::mem::size_of::<u16>()) as isize,
        ) = 0u16; // UNICODE_NULL
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrQuerySystemVolumeName(
    SystemVolumeName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    core::ptr::write_bytes(
        QueryTable.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );
    QueryTable[0].QueryRoutine = Some(core::mem::transmute::<
        unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
        _,
    >(MountMgrQuerySystemVolumeNameQueryRoutine));
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED as ULONG;
    let system_partition: [u16; 16] = core::mem::transmute(*b"S\0y\0s\0t\0e\0m\0P\0a\0r\0t\0i\0t\0i\0o\0n\0\0\0");
    QueryTable[0].Name = system_partition.as_ptr() as PCWSTR;
    (*SystemVolumeName).Buffer = core::ptr::null_mut();
    let registry_path: [u16; 31] = core::mem::transmute(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0S\0e\0t\0u\0p\0\0\0",
    );
    RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE as ULONG,
        registry_path.as_ptr() as PCWSTR,
        QueryTable.as_mut_ptr() as PRTL_QUERY_REGISTRY_TABLE,
        SystemVolumeName as PVOID,
        core::ptr::null_mut(),
    );
    if !(*SystemVolumeName).Buffer.is_null() {
        return STATUS_SUCCESS;
    }
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrAssignDriveLetters(DeviceExtension: *mut DEVICE_EXTENSION) {
    let mut Status: i32 = 0;
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut SystemVolumeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
    let mut DriveLetterInformation: MOUNTMGR_DRIVE_LETTER_INFORMATION = MOUNTMGR_DRIVE_LETTER_INFORMATION {
        DriveLetterWasAssigned: 0,
        CurrentDriveLetter: 0,
    };

    Status = MountMgrQuerySystemVolumeName(core::ptr::addr_of_mut!(SystemVolumeName));
    DbgPrint(
        c"[MNTMGR] AssignDriveLetters: sysvol st=0x%x listEmpty=%u\n".as_ptr(),
        Status as u32,
        IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead)) as u32,
    );

    if IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead)) {
        if Status >= 0 {
            ExFreePoolWithTag(SystemVolumeName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        }
        return;
    }

    NextEntry = (*DeviceExtension).DeviceListHead.Flink;

    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        let offset = core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry);
        DeviceInformation = (NextEntry as *mut u8).wrapping_sub(offset) as *mut DEVICE_INFORMATION;

        DbgPrint(
            c"[MNTMGR] AssignDriveLetters: visit %wZ LetterAssigned=%u\n".as_ptr(),
            core::ptr::addr_of_mut!((*DeviceInformation).DeviceName),
            (*DeviceInformation).LetterAssigned as u32,
        );
        if (*DeviceInformation).LetterAssigned == 0 {
            let st = MountMgrNextDriveLetterWorker(
                DeviceExtension,
                core::ptr::addr_of_mut!((*DeviceInformation).DeviceName),
                core::ptr::addr_of_mut!(DriveLetterInformation),
            );
            DbgPrint(
                c"[MNTMGR] AUTO_DL sweep: %wZ -> st=0x%x assigned=%u letter=%c\n".as_ptr(),
                core::ptr::addr_of_mut!((*DeviceInformation).DeviceName),
                st as u32,
                DriveLetterInformation.DriveLetterWasAssigned as u32,
                if DriveLetterInformation.CurrentDriveLetter != 0 { DriveLetterInformation.CurrentDriveLetter as u32 } else { '?' as u32 },
            );
        }

        if Status >= 0 && RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(SystemVolumeName),
            core::ptr::addr_of_mut!((*DeviceInformation).DeviceName),
            1u8, // TRUE (BOOLEAN)
        ) != 0 {
            let size = ((*(*DeviceInformation).UniqueId).UniqueIdLength as usize)
                .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>());
            (*DeviceExtension).DriveLetterData = ExAllocatePoolWithTag(
                PagedPool,
                size,
                1098149453u32,
            ) as *mut MOUNTDEV_UNIQUE_ID;

            if !(*DeviceExtension).DriveLetterData.is_null() {
                RtlCopyMemory(
                    (*DeviceExtension).DriveLetterData as *mut core::ffi::c_void,
                    (*DeviceInformation).UniqueId as *const core::ffi::c_void,
                    size,
                );
            }

            if (*DeviceExtension).NoAutoMount != 0 {
                (*DeviceExtension).NoAutoMount = 0u8; // FALSE
                MountMgrNextDriveLetterWorker(
                    DeviceExtension,
                    core::ptr::addr_of_mut!((*DeviceInformation).DeviceName),
                    core::ptr::addr_of_mut!(DriveLetterInformation),
                );
                (*DeviceExtension).NoAutoMount = 1u8; // TRUE
            }
        }

        NextEntry = (*NextEntry).Flink;
    }

    if Status >= 0 {
        ExFreePoolWithTag(SystemVolumeName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrQueryDosVolumePath(
    DeviceExtension: *mut DEVICE_EXTENSION,
    Irp: *mut IRP,
) -> i32 {
    let mut Status: i32 = 0;
    let mut DevicesFound: u32 = 0;
    let Stack: *mut IO_STACK_LOCATION;
    let mut SymlinksEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let Target: *mut MOUNTMGR_TARGET_NAME;
    let mut Output: *mut MOUNTMGR_VOLUME_PATHS = core::ptr::null_mut();
    let mut DeviceString: *mut u16 = core::ptr::null_mut();
    let mut OldBuffer: *mut u16 = core::ptr::null_mut();
    let mut DeviceLength: u16 = 0;
    let mut OldLength: u16 = 0;
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
    let mut SymlinkInformation: *mut SYMLINK_INFORMATION = core::ptr::null_mut();
    let mut AssociatedDevice: *mut ASSOCIATED_DEVICE_ENTRY = core::ptr::null_mut();
    let mut current_block: u32 = 0;

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<MOUNTMGR_TARGET_NAME>() {
        return STATUS_INVALID_PARAMETER;
    }

    Target = (*Irp).AssociatedIrp.SystemBuffer as *mut MOUNTMGR_TARGET_NAME;
    if ((*Target).DeviceNameLength as i32 & 1) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (core::mem::offset_of!(MOUNTMGR_TARGET_NAME, DeviceName) + (*Target).DeviceNameLength as usize) as u32
        > (*Stack).Parameters.DeviceIoControl.InputBufferLength
    {
        return STATUS_INVALID_PARAMETER;
    }

    if (*Stack).Parameters.DeviceIoControl.OutputBufferLength < core::mem::offset_of!(MOUNTMGR_VOLUME_PATHS, MultiSz) as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    SymbolicName.MaximumLength = (*Target).DeviceNameLength;
    SymbolicName.Length = SymbolicName.MaximumLength;
    SymbolicName.Buffer = &raw mut (*Target).DeviceName as *mut u16;

    Status = FindDeviceInfo(DeviceExtension, &raw mut SymbolicName as *mut _UNICODE_STRING, 0u8, &raw mut DeviceInformation);
    if Status < 0 {
        return Status;
    }

    DeviceLength = 0;
    DeviceString = core::ptr::null_mut();
    DevicesFound = 0;

    'outer: loop {
        SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
        while SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
            SymlinkInformation = (SymlinksEntry as usize)
                .wrapping_sub(core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry))
                as *mut SYMLINK_INFORMATION;

            if (*SymlinkInformation).Name.Length == 28
                && (*(*SymlinkInformation).Name.Buffer.offset(0) as i32 == '\\' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(1) as i32 == 'D' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(2) as i32 == 'o' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(3) as i32 == 's' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(4) as i32 == 'D' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(5) as i32 == 'e' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(6) as i32 == 'v' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(7) as i32 == 'i' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(8) as i32 == 'c' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(9) as i32 == 'e' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(10) as i32 == 's' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(11) as i32 == '\\' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(12) as i32 >= 'A' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(12) as i32 <= 'Z' as i32)
                && (*(*SymlinkInformation).Name.Buffer.offset(13) as i32 == ':' as i32)
                && (*SymlinkInformation).Online != 0
            {
                break;
            }
            SymlinksEntry = (*SymlinksEntry).Flink;
        }

        if SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
            current_block = 1;
            break 'outer;
        }

        if IsListEmpty(&raw mut (*DeviceInformation).AssociatedDevicesHead) {
            current_block = 2;
            break 'outer;
        }

        AssociatedDevice = ((&raw mut (*DeviceInformation).AssociatedDevicesHead) as usize)
            .wrapping_sub(core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry))
            as *mut ASSOCIATED_DEVICE_ENTRY;

        OldLength = DeviceLength;
        OldBuffer = DeviceString;
        DeviceLength = (DeviceLength as u32 + (*AssociatedDevice).String.Length as u32) as u16;
        DeviceString = AllocatePool(DeviceLength as u32) as *mut u16;
        if DeviceString.is_null() {
            if !OldBuffer.is_null() {
                FreePool(OldBuffer as *mut core::ffi::c_void);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        // RtlCopyMemory/RtlMoveMemory count BYTES; copy as u8 (a u16 copy moves 2x).
        core::ptr::copy_nonoverlapping(
            (*AssociatedDevice).String.Buffer as *const u8,
            DeviceString as *mut u8,
            (*AssociatedDevice).String.Length as usize,
        );
        if !OldBuffer.is_null() {
            // RtlCopyMemory/RtlMoveMemory count BYTES; copy as u8 (a u16 copy moves 2x).
            core::ptr::copy_nonoverlapping(
                OldBuffer as *const u8,
                DeviceString.add(((*AssociatedDevice).String.Length as usize) / core::mem::size_of::<u16>()) as *mut u8,
                OldLength as usize,
            );
            FreePool(OldBuffer as *mut core::ffi::c_void);
        }

        DevicesFound += 1;
        DeviceInformation = (*AssociatedDevice).DeviceInformation;

        if DevicesFound > MAX_DEVICES {
            current_block = 2;
            break 'outer;
        }
    }

    if current_block == 1 {
        OldBuffer = DeviceString;
        OldLength = DeviceLength;
        DeviceLength = (DeviceLength as u32 + 2 * core::mem::size_of::<u16>() as u32) as u16;
        DeviceString = AllocatePool(DeviceLength as u32) as *mut u16;
        if DeviceString.is_null() {
            if !OldBuffer.is_null() {
                FreePool(OldBuffer as *mut core::ffi::c_void);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        *DeviceString.offset(0) = *(*SymlinkInformation).Name.Buffer.offset(LETTER_POSITION as isize);
        *DeviceString.offset(1) = ':' as u16;
        if !OldBuffer.is_null() {
            // RtlCopyMemory/RtlMoveMemory count BYTES; copy as u8 (a u16 copy moves 2x).
            core::ptr::copy_nonoverlapping(
                OldBuffer as *const u8,
                DeviceString.offset(2) as *mut u8,
                OldLength as usize,
            );
            FreePool(OldBuffer as *mut core::ffi::c_void);
        }
    }

    // TryWithVolumeName
    if (DeviceLength as usize) < (2 * core::mem::size_of::<u16>()) || *DeviceString.offset(1) != ':' as u16 {
        if !DeviceString.is_null() {
            FreePool(DeviceString as *mut core::ffi::c_void);
            DeviceString = core::ptr::null_mut();
            DeviceLength = 0;
        }

        SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
        while SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
            SymlinkInformation = (SymlinksEntry as usize)
                .wrapping_sub(core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry))
                as *mut SYMLINK_INFORMATION;

            if ((*SymlinkInformation).Name.Length == 96
                || ((*SymlinkInformation).Name.Length == 98
                    && *(*(*SymlinkInformation).Name.Buffer.offset(48) as *const u16) as i32 == '\\' as i32))
                && *(*(*SymlinkInformation).Name.Buffer.offset(0) as *const u16) as i32 == '\\' as i32
                && (*(*(*SymlinkInformation).Name.Buffer.offset(1) as *const u16) as i32 == '?' as i32
                    || *(*(*SymlinkInformation).Name.Buffer.offset(1) as *const u16) as i32 == '\\' as i32)
                && *(*(*SymlinkInformation).Name.Buffer.offset(2) as *const u16) as i32 == '?' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(3) as *const u16) as i32 == '\\' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(4) as *const u16) as i32 == 'V' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(5) as *const u16) as i32 == 'o' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(6) as *const u16) as i32 == 'l' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(7) as *const u16) as i32 == 'u' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(8) as *const u16) as i32 == 'm' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(9) as *const u16) as i32 == 'e' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(10) as *const u16) as i32 == '{' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(19) as *const u16) as i32 == '-' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(24) as *const u16) as i32 == '-' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(29) as *const u16) as i32 == '-' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(34) as *const u16) as i32 == '-' as i32
                && *(*(*SymlinkInformation).Name.Buffer.offset(47) as *const u16) as i32 == '}' as i32
            {
                break;
            }
            SymlinksEntry = (*SymlinksEntry).Flink;
        }

        if SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
            DeviceLength = (*SymlinkInformation).Name.Length;
            DeviceString = AllocatePool(DeviceLength as u32) as *mut u16;
            if DeviceString.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            // RtlCopyMemory/RtlMoveMemory count BYTES; copy as u8 (a u16 copy moves 2x).
            core::ptr::copy_nonoverlapping(
                (*SymlinkInformation).Name.Buffer as *const u8,
                DeviceString as *mut u8,
                DeviceLength as usize,
            );
            *DeviceString.offset(1) = '\\' as u16;
        }
    }

    if DeviceString.is_null() {
        return STATUS_NOT_FOUND;
    }

    Output = (*Irp).AssociatedIrp.SystemBuffer as *mut MOUNTMGR_VOLUME_PATHS;
    (*Output).MultiSzLength = (DeviceLength as usize + 2 * core::mem::size_of::<u16>()) as u32;
    (*Irp).IoStatus.Information = (core::mem::offset_of!(MOUNTMGR_VOLUME_PATHS, MultiSz) + (*Output).MultiSzLength as usize) as _;

    if (*Irp).IoStatus.Information as u32 <= (*Stack).Parameters.DeviceIoControl.OutputBufferLength {
        if DeviceLength != 0 {
            // RtlCopyMemory/RtlMoveMemory count BYTES; copy as u8 (a u16 copy moves 2x).
            core::ptr::copy_nonoverlapping(
                DeviceString as *const u8,
                (&raw mut (*Output).MultiSz) as *mut u8,
                DeviceLength as usize,
            );
        }
        FreePool(DeviceString as *mut core::ffi::c_void);
        *((&raw mut (*Output).MultiSz) as *mut u16).add((DeviceLength as usize) / core::mem::size_of::<u16>()) = UNICODE_NULL;
        *((&raw mut (*Output).MultiSz) as *mut u16).add((DeviceLength as usize) / core::mem::size_of::<u16>() + 1) = UNICODE_NULL;
        return STATUS_SUCCESS;
    } else {
        FreePool(DeviceString as *mut core::ffi::c_void);
        (*Irp).IoStatus.Information = core::mem::offset_of!(MOUNTMGR_VOLUME_PATHS, MultiSz) as _;
        return STATUS_BUFFER_OVERFLOW;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrValidateBackPointer(
    AssociatedDeviceEntry: PASSOCIATED_DEVICE_ENTRY,
    DeviceInformation: PDEVICE_INFORMATION,
    Invalid: PBOOLEAN,
) -> NTSTATUS {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut SymlinksEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = _IO_STATUS_BLOCK {
        __bindgen_anon_1: ntkrnl_sys::ddk::_IO_STATUS_BLOCK__bindgen_ty_1 { Status: 0 },
        Information: 0,
    };
    let mut ReparseData: PREPARSE_DATA_BUFFER = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut FullName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SubstituteName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SymlinkInformation: PSYMLINK_INFORMATION = core::ptr::null_mut();

    FullName.Length = 0;
    FullName.MaximumLength = ((*AssociatedDeviceEntry)
        .String
        .Length
        .wrapping_add((*(*AssociatedDeviceEntry).DeviceInformation).DeviceName.Length) as u32
        + core::mem::size_of::<u16>() as u32
        + core::mem::size_of::<u16>() as u32) as u16;
    FullName.Buffer = AllocatePool(FullName.MaximumLength as u32) as *mut u16;
    if FullName.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeStringToString(
        &raw mut FullName,
        &raw const (*(*AssociatedDeviceEntry).DeviceInformation).DeviceName as *const _UNICODE_STRING,
    );
    *FullName
        .Buffer
        .offset((FullName.Length as usize / core::mem::size_of::<u16>()) as isize) = '\\' as u16;
    RtlAppendUnicodeStringToString(
        &raw mut FullName,
        &raw const (*AssociatedDeviceEntry).String as *const _UNICODE_STRING,
    );
    *FullName
        .Buffer
        .offset((FullName.Length as usize / core::mem::size_of::<u16>()) as isize) = UNICODE_NULL;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut FullName;
    ObjectAttributes.Attributes = (0x40 | 0x200) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenFile(
        &raw mut Handle,
        (SYNCHRONIZE | FILE_READ_ATTRIBUTES) as u32,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u32,
        (FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT) as u32,
    );
    FreePool(FullName.Buffer as *mut core::ffi::c_void);

    if !(Status >= 0) {
        *Invalid = 1;
        return STATUS_SUCCESS;
    }

    ReparseData = AllocatePool(MAXIMUM_REPARSE_DATA_BUFFER_SIZE) as PREPARSE_DATA_BUFFER;
    if ReparseData.is_null() {
        ZwClose(Handle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwFsControlFile(
        Handle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &raw mut IoStatusBlock,
        FSCTL_GET_REPARSE_POINT,
        core::ptr::null_mut(),
        0,
        ReparseData as *mut core::ffi::c_void,
        MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
    );
    ZwClose(Handle);

    if !(Status >= 0) {
        FreePool(ReparseData as *mut core::ffi::c_void);
        *Invalid = 1;
        return STATUS_SUCCESS;
    }

    SubstituteName.Length = (*ReparseData)
        .__bindgen_anon_1
        .SymbolicLinkReparseBuffer
        .SubstituteNameLength;
    SubstituteName.MaximumLength = SubstituteName.Length;
    SubstituteName.Buffer = ((&raw mut (*ReparseData)
        .__bindgen_anon_1
        .SymbolicLinkReparseBuffer
        .PathBuffer as *mut u16 as usize)
        + (*ReparseData)
            .__bindgen_anon_1
            .SymbolicLinkReparseBuffer
            .SubstituteNameOffset as usize) as *mut u16;

    if (SubstituteName.Length as i32 == 96
        || SubstituteName.Length as i32 == 98
            && *SubstituteName
                .Buffer
                .offset(48)
                as i32
                == '\\' as i32)
        && *SubstituteName.Buffer.offset(0) as i32 == '\\' as i32
        && (*SubstituteName.Buffer.offset(1) as i32 == '?' as i32
            || *SubstituteName.Buffer.offset(1) as i32 == '\\' as i32)
        && *SubstituteName.Buffer.offset(2) as i32 == '?' as i32
        && *SubstituteName.Buffer.offset(3) as i32 == '\\' as i32
        && *SubstituteName.Buffer.offset(4) as i32 == 'V' as i32
        && *SubstituteName.Buffer.offset(5) as i32 == 'o' as i32
        && *SubstituteName.Buffer.offset(6) as i32 == 'l' as i32
        && *SubstituteName.Buffer.offset(7) as i32 == 'u' as i32
        && *SubstituteName.Buffer.offset(8) as i32 == 'm' as i32
        && *SubstituteName.Buffer.offset(9) as i32 == 'e' as i32
        && *SubstituteName.Buffer.offset(10) as i32 == '{' as i32
        && *SubstituteName.Buffer.offset(19) as i32 == '-' as i32
        && *SubstituteName.Buffer.offset(24) as i32 == '-' as i32
        && *SubstituteName.Buffer.offset(29) as i32 == '-' as i32
        && *SubstituteName.Buffer.offset(34) as i32 == '-' as i32
        && *SubstituteName.Buffer.offset(47) as i32 == '}' as i32
    {
        if SubstituteName.Length as i32 == 98
            && *SubstituteName.Buffer.offset(1) as i32 == '?' as i32
        {
            SymlinksEntry = (*DeviceInformation).SymbolicLinksListHead.Flink as PLIST_ENTRY;
            while SymlinksEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
                SymlinkInformation = (SymlinksEntry as usize).wrapping_sub(
                    core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
                ) as PSYMLINK_INFORMATION;
                if RtlEqualUnicodeString(
                    &raw mut SubstituteName,
                    &raw mut (*SymlinkInformation).Name,
                    1u8,
                ) != 0
                {
                    FreePool(ReparseData as *mut core::ffi::c_void);
                    return STATUS_SUCCESS;
                }
                SymlinksEntry = (*SymlinksEntry).Flink as PLIST_ENTRY;
            }
        }
    }

    FreePool(ReparseData as *mut core::ffi::c_void);
    *Invalid = 1;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrQueryVolumePaths(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DeviceInformation: *mut DEVICE_INFORMATION,
    DeviceInfoList: *mut LIST_ENTRY,
    VolumePaths: *mut *mut MOUNTMGR_VOLUME_PATHS,
    FailedDevice: *mut *mut DEVICE_INFORMATION,
) -> NTSTATUS {
    let mut Written: u32 = 0;
    let mut Status: NTSTATUS = 0;
    let mut Entry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut SymlinkInformation: *mut SYMLINK_INFORMATION = core::ptr::null_mut();
    let mut DeviceInfoEntry: *mut DEVICE_INFORMATION_ENTRY = core::ptr::null_mut();
    let mut AssociatedDeviceEntry: *mut ASSOCIATED_DEVICE_ENTRY = core::ptr::null_mut();
    let mut Paths: *mut *mut MOUNTMGR_VOLUME_PATHS = core::ptr::null_mut();
    let mut CurrentPath: *mut *mut MOUNTMGR_VOLUME_PATHS = core::ptr::null_mut();
    let mut OutputPathLength: u32 = 0;
    let mut NumberOfPaths: u32 = 0;
    let mut ReturnedPaths: u32 = 0;

    OutputPathLength = core::mem::size_of::<u16>() as u32;

    Entry = (*DeviceInformation).SymbolicLinksListHead.Flink;
    while Entry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
        SymlinkInformation = (Entry as usize - core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry)) as *mut SYMLINK_INFORMATION;
        if MOUNTMGR_IS_DRIVE_LETTER(&raw const (*SymlinkInformation).Name) != FALSE
            && (*SymlinkInformation).Online != 0
        {
            OutputPathLength = (4 * core::mem::size_of::<u16>()) as u32;
            break;
        }
        Entry = (*Entry).Flink;
    }

    if Entry == &raw mut (*DeviceInformation).SymbolicLinksListHead {
        SymlinkInformation = core::ptr::null_mut();
    }

    Entry = (*DeviceInfoList).Flink;
    while Entry != DeviceInfoList {
        DeviceInfoEntry = (Entry as usize - core::mem::offset_of!(DEVICE_INFORMATION_ENTRY, DeviceInformationEntry)) as *mut DEVICE_INFORMATION_ENTRY;
        if (*DeviceInfoEntry).DeviceInformation == DeviceInformation {
            *VolumePaths = AllocatePool(
                (core::mem::size_of::<u32>() + OutputPathLength as usize) as u32,
            ) as *mut MOUNTMGR_VOLUME_PATHS;
            if (*VolumePaths).is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (**VolumePaths).MultiSzLength = OutputPathLength;
            if !SymlinkInformation.is_null() {
                *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(0) =
                    *(*SymlinkInformation).Name.Buffer.offset(LETTER_POSITION as isize);
                *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(1) = ':' as u16;
                *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(2) = UNICODE_NULL;
                *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(3) = UNICODE_NULL;
            } else {
                *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(0) = UNICODE_NULL;
            }

            return STATUS_SUCCESS;
        }
        Entry = (*Entry).Flink;
    }

    DeviceInfoEntry = AllocatePool(core::mem::size_of::<DEVICE_INFORMATION_ENTRY>() as u32) as *mut DEVICE_INFORMATION_ENTRY;
    if DeviceInfoEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceInfoEntry).DeviceInformation = DeviceInformation;
    InsertTailList(
        DeviceInfoList,
        &raw mut (*DeviceInfoEntry).DeviceInformationEntry,
    );

    NumberOfPaths = 0;
    if !IsListEmpty(&raw mut (*DeviceInformation).AssociatedDevicesHead) {
        Entry = (*DeviceInformation).AssociatedDevicesHead.Flink;
        while Entry != &raw mut (*DeviceInformation).AssociatedDevicesHead {
            NumberOfPaths = NumberOfPaths.wrapping_add(1);
            Entry = (*Entry).Flink;
        }
        // ASSERT(NumberOfPaths != 0) is a no-op due to NDEBUG, skip it.

        Paths = AllocatePool(
            ((NumberOfPaths as usize) * core::mem::size_of::<*mut MOUNTMGR_VOLUME_PATHS>()) as u32,
        ) as *mut *mut MOUNTMGR_VOLUME_PATHS;
        if Paths.is_null() {
            RemoveEntryList(&raw mut (*DeviceInfoEntry).DeviceInformationEntry);
            FreePool(DeviceInfoEntry as *mut core::ffi::c_void);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    ReturnedPaths = 0;
    CurrentPath = Paths;
    Entry = (*DeviceInformation).AssociatedDevicesHead.Flink;
    while Entry != &raw mut (*DeviceInformation).AssociatedDevicesHead {
        let mut InnerStrings: u16 = 0;
        let mut Invalid: BOOLEAN = FALSE;
        AssociatedDeviceEntry = (Entry as usize - core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry)) as *mut ASSOCIATED_DEVICE_ENTRY;

        Status = MountMgrValidateBackPointer(
            AssociatedDeviceEntry,
            DeviceInformation,
            &raw mut Invalid,
        );

        if Invalid != FALSE {
            *FailedDevice = (*AssociatedDeviceEntry).DeviceInformation;
            Status = STATUS_UNSUCCESSFUL;
        }

        if !NT_SUCCESS(Status) {
            let mut i: u32 = 0;
            while i < ReturnedPaths {
                FreePool(*Paths.offset(i as isize) as *mut core::ffi::c_void);
                i = i.wrapping_add(1);
            }
            if !Paths.is_null() {
                FreePool(Paths as *mut core::ffi::c_void);
            }
            RemoveEntryList(&raw mut (*DeviceInfoEntry).DeviceInformationEntry);
            FreePool(DeviceInfoEntry as *mut core::ffi::c_void);
            return Status;
        }

        Status = MountMgrQueryVolumePaths(
            DeviceExtension,
            (*AssociatedDeviceEntry).DeviceInformation,
            DeviceInfoList,
            CurrentPath,
            FailedDevice,
        );

        if !NT_SUCCESS(Status) {
            let mut i: u32 = 0;
            while i < ReturnedPaths {
                FreePool(*Paths.offset(i as isize) as *mut core::ffi::c_void);
                i = i.wrapping_add(1);
            }
            if !Paths.is_null() {
                FreePool(Paths as *mut core::ffi::c_void);
            }
            RemoveEntryList(&raw mut (*DeviceInfoEntry).DeviceInformationEntry);
            FreePool(DeviceInfoEntry as *mut core::ffi::c_void);
            return Status;
        }

        InnerStrings = 0;
        if (**CurrentPath).MultiSzLength as usize != core::mem::size_of::<u16>() {
            let mut i: u32 = 0;
            let mut MultiSz = (**CurrentPath).MultiSz.as_mut_ptr() as *mut u16;
            while (i as usize) < ((**CurrentPath).MultiSzLength as usize) / core::mem::size_of::<u16>() {
                if *MultiSz == UNICODE_NULL {
                    InnerStrings = InnerStrings.wrapping_add(1);
                }
                i = i.wrapping_add(1);
                MultiSz = MultiSz.offset(1);
            }
        }

        ReturnedPaths = ReturnedPaths.wrapping_add(1);
        CurrentPath = CurrentPath.offset(1);

        OutputPathLength = (OutputPathLength as u64).wrapping_add(
            ((**CurrentPath).MultiSzLength as u32).wrapping_add(
                (InnerStrings as i32 * (*AssociatedDeviceEntry).String.Length as i32) as u32,
            ) as u64 - core::mem::size_of::<u16>() as u64,
        ) as u32;

        Entry = (*Entry).Flink;
    }

    *VolumePaths = AllocatePool(
        (core::mem::size_of::<u32>() + OutputPathLength as usize) as u32,
    ) as *mut MOUNTMGR_VOLUME_PATHS;
    if (*VolumePaths).is_null() {
        let mut i: u32 = 0;
        while i < ReturnedPaths {
            FreePool(*Paths.offset(i as isize) as *mut core::ffi::c_void);
            i = i.wrapping_add(1);
        }
        if !Paths.is_null() {
            FreePool(Paths as *mut core::ffi::c_void);
        }
        RemoveEntryList(&raw mut (*DeviceInfoEntry).DeviceInformationEntry);
        FreePool(DeviceInfoEntry as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Written = 0;
    (**VolumePaths).MultiSzLength = OutputPathLength;
    if !SymlinkInformation.is_null() {
        *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(0) =
            *(*SymlinkInformation).Name.Buffer.offset(LETTER_POSITION as isize);
        *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(1) = ':' as u16;
        *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(2) = UNICODE_NULL;
        Written = 3;
    }

    CurrentPath = Paths;
    Entry = (*DeviceInformation).AssociatedDevicesHead.Flink;
    while Entry != &raw mut (*DeviceInformation).AssociatedDevicesHead {
        AssociatedDeviceEntry = (Entry as usize - core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry)) as *mut ASSOCIATED_DEVICE_ENTRY;

        if (**CurrentPath).MultiSzLength as usize != core::mem::size_of::<u16>() {
            let mut i: u32 = 0;
            let mut Offset: u32 = 0;
            let mut MultiSz: *mut u16;
            Offset = core::mem::size_of::<u32>() as u32;
            while (i as usize) < ((**CurrentPath).MultiSzLength as usize) / core::mem::size_of::<u16>() - 1 {
                MultiSz = (*CurrentPath as usize).wrapping_add(Offset as usize) as *mut u16;
                if *MultiSz != UNICODE_NULL {
                    *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(Written as isize) = *MultiSz;
                } else {
                    RtlCopyMemory(
                        ((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(Written as isize) as *mut core::ffi::c_void,
                        (*AssociatedDeviceEntry).String.Buffer as *const core::ffi::c_void,
                        (*AssociatedDeviceEntry).String.Length as usize,
                    );
                    Written = (Written as u32).wrapping_add(
                        (((*AssociatedDeviceEntry).String.Length as usize) / core::mem::size_of::<u16>()) as u32,
                    );
                    *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(Written as isize) = UNICODE_NULL;
                }

                Written = Written.wrapping_add(1);
                Offset = (Offset as u32).wrapping_add(core::mem::size_of::<u16>() as u32);
                i = i.wrapping_add(1);
            }
        }

        FreePool(*CurrentPath as *mut core::ffi::c_void);
        CurrentPath = CurrentPath.offset(1);
        Entry = (*Entry).Flink;
    }

    *((**VolumePaths).MultiSz.as_mut_ptr() as *mut u16).offset(Written as isize) = UNICODE_NULL;

    if !Paths.is_null() {
        FreePool(Paths as *mut core::ffi::c_void);
    }
    RemoveEntryList(&raw mut (*DeviceInfoEntry).DeviceInformationEntry);
    FreePool(DeviceInfoEntry as *mut core::ffi::c_void);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn MountMgrQueryDosVolumePaths(
    DeviceExtension: *mut DEVICE_EXTENSION,
    Irp: *mut IRP,
) -> NTSTATUS {
    const MOUNTMGR_TARGET_NAME_DEVICENAME_OFFSET: u32 = 2;
    const MOUNTMGR_VOLUME_PATHS_MULTISZ_OFFSET: u32 = 4;

    let mut Status: NTSTATUS = 0;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Devices: LIST_ENTRY = LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
    let mut NeedNotification: BOOLEAN = 0u8;
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut Attempts: ULONG = 0;
    let mut OutputLength: ULONG = 0;
    let mut Target: PMOUNTMGR_TARGET_NAME = core::ptr::null_mut();
    let mut Paths: PMOUNTMGR_VOLUME_PATHS = core::ptr::null_mut();
    let mut Output: PMOUNTMGR_VOLUME_PATHS = core::ptr::null_mut();
    let mut ReconcileContext: RECONCILE_WORK_ITEM_CONTEXT = unsafe { core::mem::zeroed() };
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut ListDeviceInfo: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut FailedDevice: PDEVICE_INFORMATION = core::ptr::null_mut();

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_TARGET_NAME>()
    {
        return STATUS_INVALID_PARAMETER;
    }

    Target = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_TARGET_NAME;
    if (*Target).DeviceNameLength as i32 & 1 != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (MOUNTMGR_TARGET_NAME_DEVICENAME_OFFSET as u32 + (*Target).DeviceNameLength as u32)
        > (*Stack).Parameters.DeviceIoControl.InputBufferLength
    {
        return STATUS_INVALID_PARAMETER;
    }

    if (*Stack).Parameters.DeviceIoControl.OutputBufferLength
        < MOUNTMGR_VOLUME_PATHS_MULTISZ_OFFSET
    {
        return STATUS_INVALID_PARAMETER;
    }

    SymbolicName.MaximumLength = (*Target).DeviceNameLength;
    SymbolicName.Length = SymbolicName.MaximumLength;
    SymbolicName.Buffer = core::ptr::addr_of_mut!((*Target).DeviceName) as *mut u16;

    Status = FindDeviceInfo(
        DeviceExtension,
        core::ptr::addr_of_mut!(SymbolicName),
        FALSE,
        core::ptr::addr_of_mut!(DeviceInformation),
    );
    if !(Status >= 0) {
        return Status;
    }

    NeedNotification = FALSE;
    Attempts = 0;
    loop {
        FailedDevice = core::ptr::null_mut();
        InitializeListHead(core::ptr::addr_of_mut!(Devices));
        Status = MountMgrQueryVolumePaths(
            DeviceExtension,
            DeviceInformation,
            core::ptr::addr_of_mut!(Devices),
            core::ptr::addr_of_mut!(Paths),
            core::ptr::addr_of_mut!(FailedDevice),
        );
        if Status >= 0 {
            break;
        }
        if FailedDevice.is_null() {
            return Status;
        }
        if (*DeviceInformation).ManuallyRegistered == 0 {
            NeedNotification = TRUE;
        }
        ReconcileContext.DeviceExtension = DeviceExtension;
        ReconcileContext.DeviceInformation = FailedDevice;
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
            IO_NO_INCREMENT as i32,
            1,
            FALSE,
        );
        ReconcileThisDatabaseWithMasterWorker(
            &mut ReconcileContext as *mut RECONCILE_WORK_ITEM_CONTEXT
                as *mut core::ffi::c_void,
        );
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock) as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
        Entry = (*DeviceExtension).DeviceListHead.Flink;
        let offset = core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry);
        while Entry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
            ListDeviceInfo =
                (Entry as usize).wrapping_sub(offset) as PDEVICE_INFORMATION;
            if ListDeviceInfo == DeviceInformation {
                break;
            }
            Entry = (*Entry).Flink;
        }
        if Entry == core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
            return STATUS_OBJECT_NAME_NOT_FOUND;
        }
        Attempts = Attempts.wrapping_add(1);
        if Attempts >= 1000 {
            return Status;
        }
    }

    if NeedNotification != 0 {
        MountMgrNotifyNameChange(
            DeviceExtension,
            core::ptr::addr_of_mut!(SymbolicName),
            (0i32) as _,
        );
    }

    Output = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_VOLUME_PATHS;
    (*Output).MultiSzLength = (*Paths).MultiSzLength;
    OutputLength = MOUNTMGR_VOLUME_PATHS_MULTISZ_OFFSET.wrapping_add((*Output).MultiSzLength);
    if OutputLength > (*Stack).Parameters.DeviceIoControl.OutputBufferLength {
        (*Irp).IoStatus.Information = MOUNTMGR_VOLUME_PATHS_MULTISZ_OFFSET as _;
        FreePool(Paths as *mut core::ffi::c_void);
        return STATUS_BUFFER_OVERFLOW;
    }
    (*Irp).IoStatus.Information = OutputLength as _;
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*Output).MultiSz) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*Paths).MultiSz) as *const core::ffi::c_void,
        (*Output).MultiSzLength as usize,
    );
    FreePool(Paths as *mut core::ffi::c_void);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrKeepLinksWhenOffline(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Target: PMOUNTMGR_TARGET_NAME = core::ptr::null_mut();
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();

    Stack = IoGetCurrentIrpStackLocation(Irp);

    // Validate input
    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_TARGET_NAME>()
    {
        return STATUS_INVALID_PARAMETER;
    }

    Target = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_TARGET_NAME;

    if (core::mem::offset_of!(MOUNTMGR_TARGET_NAME, DeviceName) as u32
        + (*Target).DeviceNameLength as u32)
        > (*Stack).Parameters.DeviceIoControl.InputBufferLength
    {
        return STATUS_INVALID_PARAMETER;
    }

    SymbolicName.MaximumLength = (*Target).DeviceNameLength;
    SymbolicName.Length = SymbolicName.MaximumLength;
    SymbolicName.Buffer = core::ptr::addr_of_mut!((*Target).DeviceName) as *mut u16;

    // Find the associated device
    Status = FindDeviceInfo(
        DeviceExtension,
        core::ptr::addr_of_mut!(SymbolicName),
        0u8, // FALSE as BOOLEAN
        core::ptr::addr_of_mut!(DeviceInformation),
    );

    if Status < 0 {
        return Status;
    }

    // Mark we want to keep links
    (*DeviceInformation).KeepLinks = 1u8; // TRUE as BOOLEAN

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrVolumeArrivalNotification(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    unsafe {
        let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
        if (*Stack).Parameters.DeviceIoControl.InputBufferLength
            < core::mem::size_of::<MOUNTMGR_TARGET_NAME>() as u32
        {
            return STATUS_INVALID_PARAMETER;
        }
        let Target: PMOUNTMGR_TARGET_NAME =
            (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_TARGET_NAME;
        if (core::mem::offset_of!(MOUNTMGR_TARGET_NAME, DeviceName) as u32
            + (*Target).DeviceNameLength as u32)
            > (*Stack).Parameters.DeviceIoControl.InputBufferLength
        {
            return STATUS_INVALID_PARAMETER;
        }
        let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        SymbolicName.MaximumLength = (*Target).DeviceNameLength;
        SymbolicName.Length = SymbolicName.MaximumLength;
        SymbolicName.Buffer = core::ptr::addr_of_mut!((*Target).DeviceName) as *mut u16;
        let OldState: BOOLEAN = PsGetThreadHardErrorsAreDisabled(PsGetCurrentThread());
        PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), 1);
        let Status: NTSTATUS =
            MountMgrMountedDeviceArrival(DeviceExtension, core::ptr::addr_of_mut!(SymbolicName) as *mut _UNICODE_STRING, 1);
        PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), OldState);
        Status
    }
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrQueryPoints(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let Stack: PIO_STACK_LOCATION;
    let mut UniqueId: PMOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let MountPoint: PMOUNTMGR_MOUNT_POINT;
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() {
        return STATUS_INVALID_PARAMETER;
    }

    MountPoint = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_MOUNT_POINT;

    if (*MountPoint).SymbolicLinkNameLength == 0 {
        (*MountPoint).SymbolicLinkNameOffset = 0u32;
    }
    if (*MountPoint).UniqueIdLength == 0 {
        (*MountPoint).UniqueIdOffset = 0u32;
    }
    if (*MountPoint).DeviceNameLength == 0 {
        (*MountPoint).DeviceNameOffset = 0u32;
    }

    if (*MountPoint).SymbolicLinkNameOffset as u32 & 1u32 != 0 ||
       (*MountPoint).SymbolicLinkNameLength as i32 & 1i32 != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if (*MountPoint).UniqueIdOffset as u32 & 1u32 != 0 ||
       (*MountPoint).UniqueIdLength as i32 & 1i32 != 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if (*MountPoint).DeviceNameOffset as u32 & 1u32 != 0 ||
       (*MountPoint).DeviceNameLength as i32 & 1i32 != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*MountPoint).SymbolicLinkNameLength as u32)
        .wrapping_add((*MountPoint).UniqueIdLength as u32)
        .wrapping_add((*MountPoint).DeviceNameLength as u32)
        > (*Stack).Parameters.DeviceIoControl.InputBufferLength {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize) < core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*MountPoint).SymbolicLinkNameLength != 0 {
        if (*MountPoint).SymbolicLinkNameLength > MAXSHORT {
            return STATUS_INVALID_PARAMETER;
        }

        SymbolicName.Length = (*MountPoint).SymbolicLinkNameLength;
        SymbolicName.MaximumLength = ((*MountPoint).SymbolicLinkNameLength as usize)
            .wrapping_add(core::mem::size_of::<u16>()) as u16;
        SymbolicName.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            SymbolicName.MaximumLength as usize,
            1098149453u32,
        ) as *mut u16;
        if SymbolicName.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyMemory(
            SymbolicName.Buffer as *mut core::ffi::c_void,
            (MountPoint as *const u8)
                .offset((*MountPoint).SymbolicLinkNameOffset as isize)
                as *const core::ffi::c_void,
            SymbolicName.Length as usize,
        );
        *SymbolicName.Buffer.offset(
            (SymbolicName.Length as usize).wrapping_div(core::mem::size_of::<u16>()) as isize,
        ) = UNICODE_NULL;

        Status = QueryPointsFromSymbolicLinkName(DeviceExtension, &mut SymbolicName, Irp);
        ExFreePoolWithTag(SymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    } else if (*MountPoint).UniqueIdLength != 0 {
        UniqueId = ExAllocatePoolWithTag(
            PagedPool,
            ((*MountPoint).UniqueIdLength as usize)
                .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>()),
            1098149453u32,
        ) as PMOUNTDEV_UNIQUE_ID;
        if UniqueId.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*UniqueId).UniqueIdLength = (*MountPoint).UniqueIdLength;
        RtlCopyMemory(
            &raw mut (*UniqueId).UniqueId as *mut core::ffi::c_void,
            (MountPoint as *const u8)
                .offset((*MountPoint).UniqueIdOffset as isize)
                as *const core::ffi::c_void,
            (*MountPoint).UniqueIdLength as usize,
        );

        Status = QueryPointsFromMemory(
            DeviceExtension,
            Irp,
            UniqueId,
            core::ptr::null_mut::<_UNICODE_STRING>(),
        );
        ExFreePoolWithTag(UniqueId as *mut core::ffi::c_void, 1098149453u32);
    } else if (*MountPoint).DeviceNameLength != 0 {
        if (*MountPoint).DeviceNameLength > MAXSHORT {
            return STATUS_INVALID_PARAMETER;
        }

        DeviceName.Length = (*MountPoint).DeviceNameLength;
        DeviceName.MaximumLength = ((*MountPoint).DeviceNameLength as usize)
            .wrapping_add(core::mem::size_of::<u16>()) as u16;
        DeviceName.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            DeviceName.MaximumLength as usize,
            1098149453u32,
        ) as *mut u16;
        if DeviceName.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyMemory(
            DeviceName.Buffer as *mut core::ffi::c_void,
            (MountPoint as *const u8)
                .offset((*MountPoint).DeviceNameOffset as isize)
                as *const core::ffi::c_void,
            DeviceName.Length as usize,
        );
        *DeviceName.Buffer.offset(
            (DeviceName.Length as usize).wrapping_div(core::mem::size_of::<u16>()) as isize,
        ) = UNICODE_NULL;

        Status = QueryPointsFromMemory(
            DeviceExtension,
            Irp,
            core::ptr::null_mut::<MOUNTDEV_UNIQUE_ID>(),
            &mut DeviceName,
        );
        ExFreePoolWithTag(DeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    } else {
        Status = QueryPointsFromMemory(
            DeviceExtension,
            Irp,
            core::ptr::null_mut::<MOUNTDEV_UNIQUE_ID>(),
            core::ptr::null_mut::<_UNICODE_STRING>(),
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrDeletePoints(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Link: u32 = 0;
    let mut Status: NTSTATUS = 0;
    let mut CreateNoDrive: u8 = 0;
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut UniqueId: PMOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut MountPoint: PMOUNTMGR_MOUNT_POINT = core::ptr::null_mut();
    let mut MountPoints: PMOUNTMGR_MOUNT_POINTS = core::ptr::null_mut();
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    Stack = IoGetCurrentIrpStackLocation(Irp);

    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_MOUNT_POINT>()
    {
        return STATUS_INVALID_PARAMETER;
    }

    MountPoint = (*Irp).AssociatedIrp.SystemBuffer as *mut MOUNTMGR_MOUNT_POINT;

    CreateNoDrive = ((*MountPoint).SymbolicLinkNameOffset != 0
        && (*MountPoint).SymbolicLinkNameLength != 0) as u8;

    Status = MountMgrQueryPoints(DeviceExtension, Irp);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    MountPoints = (*Irp).AssociatedIrp.SystemBuffer as *mut MOUNTMGR_MOUNT_POINTS;

    for Link in 0..(*MountPoints).NumberOfMountPoints {
        let mount_point_ptr =
            &raw mut (*MountPoints).MountPoints as *mut MOUNTMGR_MOUNT_POINT;
        let current_mp = mount_point_ptr.add(Link as usize);

        SymbolicName.Length = (*current_mp).SymbolicLinkNameLength;
        SymbolicName.MaximumLength = SymbolicName.Length
            .wrapping_add(core::mem::size_of::<u16>() as u16);
        SymbolicName.Buffer =
            AllocatePool(SymbolicName.MaximumLength as u32) as *mut u16;
        if SymbolicName.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyMemory(
            SymbolicName.Buffer as *mut core::ffi::c_void,
            (MountPoints as usize + (*current_mp).SymbolicLinkNameOffset as usize)
                as *const core::ffi::c_void,
            SymbolicName.Length as usize,
        );
        *SymbolicName.Buffer.add(
            SymbolicName.Length as usize / core::mem::size_of::<u16>(),
        ) = UNICODE_NULL;

        if CreateNoDrive != 0 && IsDriveLetter(&raw mut SymbolicName) != 0u8 {
            UniqueId = AllocatePool(
                (*current_mp).UniqueIdLength as u32
                    + core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as u32,
            ) as *mut MOUNTDEV_UNIQUE_ID;
            if !UniqueId.is_null() {
                (*UniqueId).UniqueIdLength = (*current_mp).UniqueIdLength;
                RtlCopyMemory(
                    &raw mut (*UniqueId).UniqueId as *mut u8
                        as *mut core::ffi::c_void,
                    (MountPoints as usize + (*current_mp).UniqueIdOffset as usize)
                        as *const core::ffi::c_void,
                    (*current_mp).UniqueIdLength as usize,
                );
                CreateNoDriveLetterEntry(UniqueId);
                FreePool(UniqueId as *mut core::ffi::c_void);
            }
        }

        if Link == 0 && CreateNoDrive == 0 {
            UniqueId = AllocatePool((*current_mp).UniqueIdLength as u32)
                as *mut MOUNTDEV_UNIQUE_ID;
            if !UniqueId.is_null() {
                RtlCopyMemory(
                    UniqueId as *mut core::ffi::c_void,
                    (MountPoints as usize + (*current_mp).UniqueIdOffset as usize)
                        as *const core::ffi::c_void,
                    (*current_mp).UniqueIdLength as usize,
                );
                DeleteNoDriveLetterEntry(UniqueId);
                FreePool(UniqueId as *mut core::ffi::c_void);
            }
        }

        GlobalDeleteSymbolicLink(&raw mut SymbolicName);
        DeleteSymbolicLinkNameFromMemory(
            DeviceExtension,
            &raw mut SymbolicName,
            (0i32) as _, // FALSE
        );
        RtlDeleteRegistryValue(
            RTL_REGISTRY_ABSOLUTE as u32,
            DatabasePath as *const u16,
            SymbolicName.Buffer as *const u16,
        );
        FreePool(SymbolicName.Buffer as *mut core::ffi::c_void);

        DeviceName.MaximumLength = (*current_mp).DeviceNameLength;
        DeviceName.Length = DeviceName.MaximumLength;
        DeviceName.Buffer = (MountPoints as usize
            + (*current_mp).DeviceNameOffset as usize) as *mut u16;
        MountMgrNotifyNameChange(
            DeviceExtension,
            &raw mut DeviceName,
            (1i32) as _, // TRUE
        );
    }

    MountMgrNotify(DeviceExtension);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrDeletePointsDbOnly(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Link: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut SymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UniqueId: PMOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut MountPoints: PMOUNTMGR_MOUNT_POINTS = core::ptr::null_mut();

    Status = MountMgrQueryPoints(DeviceExtension, Irp);
    if !(Status >= 0) {
        return Status;
    }

    MountPoints = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_MOUNT_POINTS;
    if (*MountPoints).NumberOfMountPoints == 0 {
        return Status;
    }

    while Link < (*MountPoints).NumberOfMountPoints {
        let mp = (core::ptr::addr_of_mut!((*MountPoints).MountPoints) as *mut MOUNTMGR_MOUNT_POINT)
            .add(Link as usize);

        SymbolicName.Length = (*mp).SymbolicLinkNameLength;
        SymbolicName.MaximumLength = (SymbolicName.Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>()) as u16;

        SymbolicName.Buffer = AllocatePool(SymbolicName.MaximumLength as u32) as *mut WCHAR;
        if SymbolicName.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            (MountPoints as *const u8).add((*mp).SymbolicLinkNameOffset as usize) as *const WCHAR,
            SymbolicName.Buffer,
            SymbolicName.Length as usize / core::mem::size_of::<WCHAR>(),
        );
        *SymbolicName.Buffer.add(
            (SymbolicName.Length as usize) / core::mem::size_of::<WCHAR>(),
        ) = 0;

        if (*MountPoints).NumberOfMountPoints == 1
            && IsDriveLetter(&raw mut SymbolicName) != 0u8
        {
            let alloc_len = (((*mp).UniqueIdLength as usize)
                .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>())) as u32;
            UniqueId = AllocatePool(alloc_len) as PMOUNTDEV_UNIQUE_ID;
            if !UniqueId.is_null() {
                (*UniqueId).UniqueIdLength = (*mp).UniqueIdLength;
                core::ptr::copy_nonoverlapping(
                    (MountPoints as *const u8).add((*mp).UniqueIdOffset as usize) as *const u8,
                    core::ptr::addr_of_mut!((*UniqueId).UniqueId) as *mut u8,
                    (*mp).UniqueIdLength as usize,
                );
                CreateNoDriveLetterEntry(UniqueId);
                FreePool(UniqueId as *mut core::ffi::c_void);
            }
        }

        DeleteSymbolicLinkNameFromMemory(
            DeviceExtension,
            &raw mut SymbolicName,
            (1i32) as _, // TRUE
        );
        RtlDeleteRegistryValue(
            RTL_REGISTRY_ABSOLUTE,
            DatabasePath,
            SymbolicName.Buffer,
        );
        FreePool(SymbolicName.Buffer as *mut core::ffi::c_void);

        Link = Link.wrapping_add(1);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrVolumeMountPointChanged(
    mut DeviceExtension: PDEVICE_EXTENSION,
    mut Irp: PIRP,
    mut LockStatus: NTSTATUS,
    mut SourceDeviceName: PUNICODE_STRING,
    mut SourceSymbolicName: PUNICODE_STRING,
    mut TargetVolumeName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut current_block: u64 = 0;
    let mut Handle: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Status: NTSTATUS = 0;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut::<_FILE_OBJECT>();
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut::<_IO_STACK_LOCATION>();
    let mut Length: ULONG = 0;
    let mut SavedLength: ULONG = 0;
    let mut FOReferenced: BOOLEAN = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = _IO_STATUS_BLOCK {
        __bindgen_anon_1: ntkrnl_sys::ddk::_IO_STATUS_BLOCK__bindgen_ty_1 { Status: 0 },
        Information: 0,
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut::<_DEVICE_INFORMATION>();
    let mut ObjectNameInfo: OBJECT_NAME_INFORMATION = OBJECT_NAME_INFORMATION {
        Name: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null::<u16>() as *mut u16,
        },
    };
    let mut FsDeviceInfo: FILE_FS_DEVICE_INFORMATION = FILE_FS_DEVICE_INFORMATION {
        DeviceType: 0,
        Characteristics: 0,
    };
    let mut FileNameInfo: PFILE_NAME_INFORMATION = core::ptr::null_mut::<_FILE_NAME_INFORMATION>();
    let mut VolumeMountPoint: PMOUNTMGR_VOLUME_MOUNT_POINT = core::ptr::null_mut::<_MOUNTMGR_VOLUME_MOUNT_POINT>();
    let mut ObjectNameInfoPtr: POBJECT_NAME_INFORMATION = core::ptr::null_mut::<_OBJECT_NAME_INFORMATION>();
    let mut SourceVolumeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null::<u16>() as *mut u16,
    };
    let mut TargetDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null::<u16>() as *mut u16,
    };

    Stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<MOUNTMGR_VOLUME_MOUNT_POINT>() as usize
    {
        return STATUS_INVALID_PARAMETER;
    }
    VolumeMountPoint = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTMGR_VOLUME_MOUNT_POINT;
    if ((*VolumeMountPoint).SourceVolumeNameLength as u32)
        .wrapping_add((*VolumeMountPoint).TargetVolumeNameLength as u32)
        < (*Stack).Parameters.DeviceIoControl.InputBufferLength
    {
        return STATUS_INVALID_PARAMETER;
    }
    SourceVolumeName.MaximumLength = (*VolumeMountPoint).SourceVolumeNameLength;
    SourceVolumeName.Length = SourceVolumeName.MaximumLength;
    SourceVolumeName.Buffer = (VolumeMountPoint as usize)
        .wrapping_add((*VolumeMountPoint).SourceVolumeNameOffset as usize) as *mut u16;
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(SourceVolumeName) as PUNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

    Status = ZwOpenFile(
        core::ptr::addr_of_mut!(Handle),
        SYNCHRONIZE | FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        (FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT) as ULONG,
    );
    if Status < 0 {
        return Status;
    }
    TargetDeviceName.Buffer = core::ptr::null_mut::<u16>();

    Status = ZwQueryVolumeInformationFile(
        Handle,
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::addr_of_mut!(FsDeviceInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
        FileFsDeviceInformation,
    );
    if Status >= 0 {
        if FsDeviceInfo.DeviceType != FILE_DEVICE_DISK as u32
            && FsDeviceInfo.DeviceType != FILE_DEVICE_VIRTUAL_DISK as u32
        {
            Status = STATUS_INVALID_PARAMETER;
        } else if FsDeviceInfo.Characteristics as u32
            & (FILE_REMOTE_DEVICE | FILE_REMOVABLE_MEDIA) as u32
            != 0
        {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            Status = ObReferenceObjectByHandle(
                Handle,
                0,
                IoFileObjectType,
                0i8 as KPROCESSOR_MODE,
                core::ptr::addr_of_mut!(FileObject) as *mut *mut core::ffi::c_void,
                core::ptr::null_mut::<_OBJECT_HANDLE_INFORMATION>(),
            );
            if Status >= 0 {
                FOReferenced = 1;
                FileNameInfo = ExAllocatePoolWithTag(
                    PagedPool,
                    core::mem::size_of::<FILE_NAME_INFORMATION>(),
                    1098149453i32 as u32,
                ) as PFILE_NAME_INFORMATION;
                if FileNameInfo.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    Status = ZwQueryInformationFile(
                        Handle,
                        core::ptr::addr_of_mut!(IoStatusBlock),
                        FileNameInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<FILE_NAME_INFORMATION>() as ULONG,
                        FileNameInformation,
                    );
                    if Status == STATUS_BUFFER_OVERFLOW {
                        Length = (*FileNameInfo).FileNameLength;
                        ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, 1098149453i32 as u32);
                        FileNameInfo = ExAllocatePoolWithTag(
                            PagedPool,
                            (core::mem::size_of::<FILE_NAME_INFORMATION>())
                                .wrapping_add(Length as usize),
                            1098149453i32 as u32,
                        ) as PFILE_NAME_INFORMATION;
                        if FileNameInfo.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                            current_block = 1005405192262894193;
                        } else {
                            Status = ZwQueryInformationFile(
                                Handle,
                                core::ptr::addr_of_mut!(IoStatusBlock),
                                FileNameInfo as *mut core::ffi::c_void,
                                (core::mem::size_of::<FILE_NAME_INFORMATION>())
                                    .wrapping_add(Length as usize) as ULONG,
                                FileNameInformation,
                            );
                            current_block = 4090602189656566074;
                        }
                    } else {
                        current_block = 4090602189656566074;
                    }
                    match current_block {
                        1005405192262894193 => {}
                        _ => {
                            if Status >= 0 {
                                ObjectNameInfoPtr =
                                    core::ptr::addr_of_mut!(ObjectNameInfo) as POBJECT_NAME_INFORMATION;
                                SavedLength = core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG;
                                Status = ObQueryNameString(
                                    (*FileObject).DeviceObject as *mut core::ffi::c_void,
                                    ObjectNameInfoPtr,
                                    core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG,
                                    core::ptr::addr_of_mut!(Length),
                                );
                                if Status == STATUS_INFO_LENGTH_MISMATCH {
                                    ObjectNameInfoPtr = ExAllocatePoolWithTag(
                                        PagedPool,
                                        Length as usize,
                                        1098149453i32 as u32,
                                    ) as POBJECT_NAME_INFORMATION;
                                    if ObjectNameInfoPtr.is_null() {
                                        Status = STATUS_INSUFFICIENT_RESOURCES;
                                        current_block = 1005405192262894193;
                                    } else {
                                        SavedLength = Length;
                                        Status = ObQueryNameString(
                                            (*FileObject).DeviceObject as *mut core::ffi::c_void,
                                            ObjectNameInfoPtr,
                                            SavedLength,
                                            core::ptr::addr_of_mut!(Length),
                                        );
                                        current_block = 1434579379687443766;
                                    }
                                } else {
                                    current_block = 1434579379687443766;
                                }
                                match current_block {
                                    1005405192262894193 => {}
                                    _ => {
                                        if Status >= 0 {
                                            Status = QueryDeviceInformation(
                                                core::ptr::addr_of_mut!((*ObjectNameInfoPtr).Name),
                                                SourceDeviceName,
                                                core::ptr::null_mut::<PMOUNTDEV_UNIQUE_ID>(),
                                                core::ptr::null_mut::<BOOLEAN>(),
                                                core::ptr::null_mut::<BOOLEAN>(),
                                                core::ptr::null_mut::<u8>(),
                                                core::ptr::null_mut::<GUID>(),
                                                core::ptr::null_mut::<u8>(),
                                            );
                                            if Status >= 0 {
                                                (*TargetVolumeName).MaximumLength =
                                                    (*VolumeMountPoint).TargetVolumeNameLength;
                                                (*TargetVolumeName).Length =
                                                    (*TargetVolumeName).MaximumLength;
                                                (*TargetVolumeName).Buffer = (VolumeMountPoint as usize)
                                                    .wrapping_add(
                                                        (*VolumeMountPoint).TargetVolumeNameOffset as usize,
                                                    ) as *mut u16;
                                                Status = QueryDeviceInformation(
                                                    TargetVolumeName,
                                                    core::ptr::addr_of_mut!(TargetDeviceName),
                                                    core::ptr::null_mut::<PMOUNTDEV_UNIQUE_ID>(),
                                                    core::ptr::null_mut::<BOOLEAN>(),
                                                    core::ptr::null_mut::<BOOLEAN>(),
                                                    core::ptr::null_mut::<u8>(),
                                                    core::ptr::null_mut::<GUID>(),
                                                    core::ptr::null_mut::<u8>(),
                                                );
                                                if Status >= 0 {
                                                    (*SourceSymbolicName).MaximumLength =
                                                        (*FileNameInfo).FileNameLength as USHORT;
                                                    (*SourceSymbolicName).Length =
                                                        (*SourceSymbolicName).MaximumLength;
                                                    (*SourceSymbolicName).Buffer = FileNameInfo as *mut u16;
                                                    memmove(
                                                        (*SourceSymbolicName).Buffer as *mut core::ffi::c_void,
                                                        core::ptr::addr_of_mut!((*FileNameInfo).FileName)
                                                            as *const u16 as *const core::ffi::c_void,
                                                        (*SourceSymbolicName).Length as usize,
                                                    );
                                                    FileNameInfo = core::ptr::null_mut::<_FILE_NAME_INFORMATION>();
                                                    MountMgrNotify(DeviceExtension);
                                                    MountMgrNotifyNameChange(
                                                        DeviceExtension,
                                                        core::ptr::addr_of_mut!(TargetDeviceName),
                                                        (1i32) as _,
                                                    );
                                                    if LockStatus >= 0 {
                                                        Status = FindDeviceInfo(
                                                            DeviceExtension,
                                                            SourceDeviceName,
                                                            0u8,
                                                            core::ptr::addr_of_mut!(DeviceInformation),
                                                        );
                                                        if Status >= 0 {
                                                            ReconcileThisDatabaseWithMaster(
                                                                DeviceExtension,
                                                                DeviceInformation,
                                                            );
                                                        } else {
                                                            Status = STATUS_PENDING;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !TargetDeviceName.Buffer.is_null() {
        ExFreePoolWithTag(TargetDeviceName.Buffer as *mut core::ffi::c_void, 1098149453i32 as u32);
    }
    if !ObjectNameInfoPtr.is_null() && ObjectNameInfoPtr != core::ptr::addr_of_mut!(ObjectNameInfo) {
        ExFreePoolWithTag(ObjectNameInfoPtr as *mut core::ffi::c_void, 1098149453i32 as u32);
    }
    if !FileNameInfo.is_null() {
        ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, 1098149453i32 as u32);
    }
    if FOReferenced != 0 {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrVolumeMountPointCreated(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
    LockStatus: NTSTATUS,
) -> NTSTATUS {
    let mut Offset: i32 = 0;
    let mut Found: u8 = 0;
    let mut Status: i32 = 0;
    let mut RemoteDatabase: HANDLE = core::ptr::null_mut();
    let mut UniqueId: PMOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut DatabaseEntry: PDATABASE_ENTRY = core::ptr::null_mut();
    let mut AssociatedEntry: PASSOCIATED_DEVICE_ENTRY = core::ptr::null_mut();
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut TargetDeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut LinkTarget: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SourceDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SourceSymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut TargetVolumeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VolumeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DbName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    LinkTarget.Length = 0u16;
    LinkTarget.MaximumLength = 0xC8u16;
    LinkTarget.Buffer = AllocatePool(LinkTarget.MaximumLength as u32) as *mut u16;
    if LinkTarget.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = MountMgrVolumeMountPointChanged(
        DeviceExtension,
        Irp,
        LockStatus,
        &raw mut SourceDeviceName,
        &raw mut SourceSymbolicName,
        &raw mut TargetVolumeName,
    );
    if Status == STATUS_PENDING {
        FreePool(LinkTarget.Buffer as *mut core::ffi::c_void);
        FreePool(SourceDeviceName.Buffer as *mut core::ffi::c_void);
        FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
        return STATUS_SUCCESS;
    } else if Status < 0 {
        FreePool(LinkTarget.Buffer as *mut core::ffi::c_void);
        return Status;
    }

    Status = FindDeviceInfo(
        DeviceExtension,
        &raw mut SourceDeviceName,
        0u8,
        &raw mut DeviceInformation,
    );
    if Status < 0 {
        Status = QueryVolumeName(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut SourceDeviceName,
            &raw mut LinkTarget,
            &raw mut VolumeName,
        );
        if Status < 0 {
            Status = MountMgrQuerySymbolicLink(
                &raw mut SourceDeviceName,
                &raw mut LinkTarget,
            );
            if Status < 0 {
                FreePool(LinkTarget.Buffer as *mut core::ffi::c_void);
                FreePool(SourceDeviceName.Buffer as *mut core::ffi::c_void);
                FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
                return Status;
            }
        } else {
            FreePool(VolumeName.Buffer as *mut core::ffi::c_void);
        }
        FreePool(SourceDeviceName.Buffer as *mut core::ffi::c_void);
        SourceDeviceName.Length = LinkTarget.Length;
        SourceDeviceName.MaximumLength = LinkTarget.MaximumLength;
        SourceDeviceName.Buffer = LinkTarget.Buffer;
        Status = FindDeviceInfo(
            DeviceExtension,
            &raw mut SourceDeviceName,
            0u8,
            &raw mut DeviceInformation,
        );
        if Status < 0 {
            FreePool(SourceDeviceName.Buffer as *mut core::ffi::c_void);
            FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
            return Status;
        }
    }
    FreePool(SourceDeviceName.Buffer as *mut core::ffi::c_void);

    Status = FindDeviceInfo(
        DeviceExtension,
        &raw mut TargetVolumeName,
        0u8,
        &raw mut TargetDeviceInformation,
    );
    if Status < 0 {
        FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
        return Status;
    }

    if (*TargetDeviceInformation).SkipNotifications == 0 {
        PostOnlineNotification(
            DeviceExtension,
            &raw mut (*TargetDeviceInformation).SymbolicName,
        );
    }

    RemoteDatabase = OpenRemoteDatabase(DeviceInformation, 1u8);
    if RemoteDatabase.is_null() {
        FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Offset = 0i32;
    Found = 0u8;
    loop {
        DatabaseEntry = GetRemoteDatabaseEntry(RemoteDatabase, Offset);
        if DatabaseEntry.is_null() {
            break;
        }
        DbName.MaximumLength = (*DatabaseEntry).SymbolicNameLength;
        DbName.Length = DbName.MaximumLength;
        DbName.Buffer = (DatabaseEntry as *mut u8).add(
            (*DatabaseEntry).SymbolicNameOffset as usize,
        ) as *mut u16;
        if RtlEqualUnicodeString(
            &raw mut TargetVolumeName,
            &raw mut DbName,
            1u8,
        ) != 0
        {
            (*DatabaseEntry).EntryReferences =
                (*DatabaseEntry).EntryReferences.wrapping_add(1);
            Status = WriteRemoteDatabaseEntry(
                RemoteDatabase,
                Offset,
                DatabaseEntry,
            );
            FreePool(DatabaseEntry as *mut core::ffi::c_void);
            Found = 1u8;
            break;
        } else {
            Offset = (Offset as u32)
                .wrapping_add((*DatabaseEntry).EntrySize as u32)
                as i32;
            FreePool(DatabaseEntry as *mut core::ffi::c_void);
        }
    }

    if Found == 0 {
        let mut EntrySize: u32 = 0;
        let mut UniqueIdReplicate: PUNIQUE_ID_REPLICATE =
            core::ptr::null_mut();
        Status = QueryDeviceInformation(
            &raw mut TargetVolumeName,
            core::ptr::null_mut::<_UNICODE_STRING>(),
            &raw mut UniqueId,
            core::ptr::null_mut::<BOOLEAN>(),
            core::ptr::null_mut::<BOOLEAN>(),
            core::ptr::null_mut::<u8>(),
            core::ptr::null_mut::<GUID>(),
            core::ptr::null_mut::<u8>(),
        );
        if Status < 0 {
            FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
            CloseRemoteDatabase(RemoteDatabase);
            return Status;
        }
        EntrySize = ((*UniqueId).UniqueIdLength as u32
            + TargetVolumeName.Length as u32)
            + core::mem::size_of::<DATABASE_ENTRY>() as u32;
        DatabaseEntry = AllocatePool(EntrySize) as PDATABASE_ENTRY;
        if DatabaseEntry.is_null() {
            FreePool(UniqueId as *mut core::ffi::c_void);
            FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
            CloseRemoteDatabase(RemoteDatabase);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*DatabaseEntry).EntrySize = EntrySize;
        (*DatabaseEntry).EntryReferences = 1u32;
        (*DatabaseEntry).SymbolicNameOffset =
            core::mem::size_of::<DATABASE_ENTRY>() as u16;
        (*DatabaseEntry).SymbolicNameLength = TargetVolumeName.Length;
        (*DatabaseEntry).UniqueIdOffset = (TargetVolumeName.Length as usize
            + core::mem::size_of::<DATABASE_ENTRY>())
            as u16;
        (*DatabaseEntry).UniqueIdLength = (*UniqueId).UniqueIdLength;
        core::ptr::copy_nonoverlapping(
            TargetVolumeName.Buffer as *const u8,
            (DatabaseEntry as *mut u8)
                .add(core::mem::size_of::<DATABASE_ENTRY>()) as *mut u8,
            (*DatabaseEntry).SymbolicNameLength as usize,
        );
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of_mut!((*UniqueId).UniqueId) as *const u8,
            (DatabaseEntry as *mut u8)
                .add((*DatabaseEntry).UniqueIdOffset as usize) as *mut u8,
            (*UniqueId).UniqueIdLength as usize,
        );
        Status = AddRemoteDatabaseEntry(
            RemoteDatabase,
            DatabaseEntry,
        );
        FreePool(DatabaseEntry as *mut core::ffi::c_void);
        if Status < 0 {
            FreePool(UniqueId as *mut core::ffi::c_void);
            FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
            CloseRemoteDatabase(RemoteDatabase);
            return Status;
        }
        UniqueIdReplicate = AllocatePool(
            core::mem::size_of::<UNIQUE_ID_REPLICATE>() as u32,
        ) as PUNIQUE_ID_REPLICATE;
        if UniqueIdReplicate.is_null() {
            FreePool(UniqueId as *mut core::ffi::c_void);
            FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
            CloseRemoteDatabase(RemoteDatabase);
            return Status;
        }
        (*UniqueIdReplicate).UniqueId = UniqueId;
        InsertTailList(
            &raw mut (*DeviceInformation).ReplicatedUniqueIdsListHead,
            &raw mut (*UniqueIdReplicate).ReplicatedUniqueIdsListEntry,
        );
    }
    CloseRemoteDatabase(RemoteDatabase);
    if Status < 0 {
        FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
        return Status;
    }

    AssociatedEntry = AllocatePool(
        core::mem::size_of::<ASSOCIATED_DEVICE_ENTRY>() as u32,
    ) as PASSOCIATED_DEVICE_ENTRY;
    if AssociatedEntry.is_null() {
        FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*AssociatedEntry).String.Length = SourceSymbolicName.Length;
    (*AssociatedEntry).String.MaximumLength =
        ((*AssociatedEntry).String.Length as usize
            + core::mem::size_of::<u16>()) as u16;
    (*AssociatedEntry).String.Buffer = AllocatePool(
        (*AssociatedEntry).String.MaximumLength as u32,
    ) as *mut u16;
    if (*AssociatedEntry).String.Buffer.is_null() {
        FreePool(AssociatedEntry as *mut core::ffi::c_void);
        FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        SourceSymbolicName.Buffer as *const u8,
        (*AssociatedEntry).String.Buffer as *mut u8,
        SourceSymbolicName.Length as usize,
    );
    *(*AssociatedEntry)
        .String
        .Buffer
        .add(
            (SourceSymbolicName.Length as usize)
                / core::mem::size_of::<u16>(),
        ) = UNICODE_NULL;
    (*AssociatedEntry).DeviceInformation = DeviceInformation;
    InsertTailList(
        &raw mut (*TargetDeviceInformation).AssociatedDevicesHead,
        &raw mut (*AssociatedEntry).AssociatedDevicesEntry,
    );
    FreePool(SourceSymbolicName.Buffer as *mut core::ffi::c_void);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrVolumeMountPointDeleted(
    DeviceExtension: PDEVICE_EXTENSION,
    Irp: PIRP,
    LockStatus: NTSTATUS,
) -> NTSTATUS {
    let mut Offset: i32 = 0;
    let mut Status: NTSTATUS = 0;
    let mut Entry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut RemoteDatabase: HANDLE = core::ptr::null_mut();
    let mut DatabaseEntry: PDATABASE_ENTRY = core::ptr::null_mut();
    let mut UniqueIdReplicate: PUNIQUE_ID_REPLICATE = core::ptr::null_mut();
    let mut AssociatedEntry: PASSOCIATED_DEVICE_ENTRY = core::ptr::null_mut();
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut TargetDeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut LinkTarget: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SourceDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SourceSymbolicName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut TargetVolumeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VolumeName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DbName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    // Initialize LinkTarget
    LinkTarget.Length = 0u16;
    LinkTarget.MaximumLength = 0xc8u16;
    LinkTarget.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        LinkTarget.MaximumLength as usize,
        1098149453u32,
    ) as *mut u16;
    if LinkTarget.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Get mount point change information
    Status = MountMgrVolumeMountPointChanged(
        DeviceExtension,
        Irp,
        LockStatus,
        core::ptr::addr_of_mut!(SourceDeviceName),
        core::ptr::addr_of_mut!(SourceSymbolicName),
        core::ptr::addr_of_mut!(TargetVolumeName),
    );
    if Status == STATUS_PENDING {
        ExFreePoolWithTag(LinkTarget.Buffer as *mut core::ffi::c_void, 1098149453u32);
        ExFreePoolWithTag(SourceDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        return STATUS_SUCCESS;
    } else if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(LinkTarget.Buffer as *mut core::ffi::c_void, 1098149453u32);
        return Status;
    }

    // Query source device info
    Status = FindDeviceInfo(
        DeviceExtension,
        core::ptr::addr_of_mut!(SourceDeviceName),
        0u8,
        core::ptr::addr_of_mut!(DeviceInformation),
    );
    if !NT_SUCCESS(Status) {
        // First try to get volume name
        Status = QueryVolumeName(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(SourceDeviceName),
            core::ptr::addr_of_mut!(LinkTarget),
            core::ptr::addr_of_mut!(VolumeName),
        );
        if !NT_SUCCESS(Status) {
            // Then try to read symbolic link
            Status = MountMgrQuerySymbolicLink(
                core::ptr::addr_of_mut!(SourceDeviceName),
                core::ptr::addr_of_mut!(LinkTarget),
            );
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(LinkTarget.Buffer as *mut core::ffi::c_void, 1098149453u32);
                ExFreePoolWithTag(SourceDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
                ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
                return Status;
            }
        } else {
            ExFreePoolWithTag(VolumeName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        }

        // Free old source name buffer and repoint to LinkTarget
        ExFreePoolWithTag(SourceDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        SourceDeviceName.Length = LinkTarget.Length;
        SourceDeviceName.MaximumLength = LinkTarget.MaximumLength;
        SourceDeviceName.Buffer = LinkTarget.Buffer;

        // Reattempt device info query with corrected name
        Status = FindDeviceInfo(
            DeviceExtension,
            core::ptr::addr_of_mut!(SourceDeviceName),
            0u8,
            core::ptr::addr_of_mut!(DeviceInformation),
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(SourceDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
            return Status;
        }
    }

    // Source name buffer no longer needed
    ExFreePoolWithTag(SourceDeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);

    // Get target device information
    Status = FindDeviceInfo(
        DeviceExtension,
        core::ptr::addr_of_mut!(TargetVolumeName),
        0u8,
        core::ptr::addr_of_mut!(TargetDeviceInformation),
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        return Status;
    }

    // Open remote database
    RemoteDatabase = OpenRemoteDatabase(DeviceInformation, 1u8);
    if RemoteDatabase.is_null() {
        ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Browse database entries looking for the target volume name
    Offset = 0i32;
    loop {
        DatabaseEntry = GetRemoteDatabaseEntry(RemoteDatabase, Offset);
        if DatabaseEntry.is_null() {
            ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
            CloseRemoteDatabase(RemoteDatabase);
            return STATUS_INVALID_PARAMETER;
        }

        DbName.MaximumLength = (*DatabaseEntry).SymbolicNameLength;
        DbName.Length = DbName.MaximumLength;
        DbName.Buffer = ((DatabaseEntry as usize)
            .wrapping_add((*DatabaseEntry).SymbolicNameOffset as usize)) as *mut u16;
        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(TargetVolumeName),
            core::ptr::addr_of_mut!(DbName),
            1u8,
        ) != 0
        {
            break;
        }

        Offset = (Offset as usize)
            .wrapping_add((*DatabaseEntry).EntrySize as usize) as i32;
        ExFreePoolWithTag(DatabaseEntry as *mut core::ffi::c_void, 1098149453u32);
    }

    // Decrement entry references
    (*DatabaseEntry).EntryReferences = (*DatabaseEntry).EntryReferences.wrapping_sub(1);
    if (*DatabaseEntry).EntryReferences == 0u32 {
        // Still referenced, just update
        Status = WriteRemoteDatabaseEntry(RemoteDatabase, Offset, DatabaseEntry);
    } else {
        // No references left, delete entry
        Status = DeleteRemoteDatabaseEntry(RemoteDatabase, Offset);
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(DatabaseEntry as *mut core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
            CloseRemoteDatabase(RemoteDatabase);
            return Status;
        }

        // Also remove the associated unique ID replicated record
        Entry = (*DeviceInformation).ReplicatedUniqueIdsListHead.Flink;
        while Entry != core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead) {
            UniqueIdReplicate = (Entry as usize).wrapping_sub(
                core::mem::offset_of!(UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry),
            ) as PUNIQUE_ID_REPLICATE;

            if (*(*UniqueIdReplicate).UniqueId).UniqueIdLength as i32
                == (*DatabaseEntry).UniqueIdLength as i32
                && RtlCompareMemory(
                    core::ptr::addr_of_mut!((*(*UniqueIdReplicate).UniqueId).UniqueId) as *const core::ffi::c_void,
                    ((DatabaseEntry as usize).wrapping_add((*DatabaseEntry).UniqueIdOffset as usize))
                        as *const core::ffi::c_void,
                    (*DatabaseEntry).UniqueIdLength as usize,
                ) == (*DatabaseEntry).UniqueIdLength as usize
            {
                break;
            }
            Entry = (*Entry).Flink;
        }

        // Must be found
        if Entry == core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead) {
            ExFreePoolWithTag(DatabaseEntry as *mut core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
            CloseRemoteDatabase(RemoteDatabase);
            return STATUS_UNSUCCESSFUL;
        }

        // Remove and free the unique ID replicate
        RemoveEntryList(
            core::ptr::addr_of_mut!((*UniqueIdReplicate).ReplicatedUniqueIdsListEntry),
        );
        ExFreePoolWithTag((*UniqueIdReplicate).UniqueId as *mut core::ffi::c_void, 1098149453u32);
        ExFreePoolWithTag(UniqueIdReplicate as *mut core::ffi::c_void, 1098149453u32);
    }

    // Done with database entry and remote database
    ExFreePoolWithTag(DatabaseEntry as *mut core::ffi::c_void, 1098149453u32);
    CloseRemoteDatabase(RemoteDatabase);

    // Check write/delete operation result
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        return Status;
    }

    // Remove associated device entry from target
    Entry = (*TargetDeviceInformation).AssociatedDevicesHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*TargetDeviceInformation).AssociatedDevicesHead) {
        AssociatedEntry = (Entry as usize).wrapping_sub(
            core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry),
        ) as PASSOCIATED_DEVICE_ENTRY;

        if (*AssociatedEntry).DeviceInformation == DeviceInformation
            && RtlEqualUnicodeString(
                core::ptr::addr_of_mut!((*AssociatedEntry).String),
                core::ptr::addr_of_mut!(SourceSymbolicName),
                1u8,
            ) != 0
        {
            RemoveEntryList(
                core::ptr::addr_of_mut!((*AssociatedEntry).AssociatedDevicesEntry),
            );
            ExFreePoolWithTag(
                (*AssociatedEntry).String.Buffer as *mut core::ffi::c_void,
                1098149453u32,
            );
            ExFreePoolWithTag(AssociatedEntry as *mut core::ffi::c_void, 1098149453u32);
            break;
        }
        Entry = (*Entry).Flink;
    }

    // Final cleanup
    ExFreePoolWithTag(SourceSymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut current_block: u64 = 0;
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    let mut Status: NTSTATUS = STATUS_INVALID_DEVICE_REQUEST;
    let mut LockStatus: NTSTATUS = 0;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i8,
        0u8, /* FALSE */
        core::ptr::null_mut(),
    );

    match (*Stack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_MOUNTMGR_CREATE_POINT => {
            Status = MountMgrCreatePoint(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_DELETE_POINTS => {
            Status = MountMgrDeletePoints(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_QUERY_POINTS => {
            Status = MountMgrQueryPoints(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_DELETE_POINTS_DBONLY => {
            Status = MountMgrDeletePointsDbOnly(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER => {
            Status = MountMgrNextDriveLetter(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_AUTO_DL_ASSIGNMENTS => {
            DbgPrint(c"[MNTMGR] AUTO_DL received; enabling automatic drive letters\n".as_ptr());
            (*DeviceExtension).AutomaticDriveLetter = 1; /* TRUE */
            MountMgrAssignDriveLetters(DeviceExtension);
            ReconcileAllDatabasesWithMaster(DeviceExtension);
            WaitForOnlinesToComplete(DeviceExtension);
            Status = STATUS_SUCCESS;
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_VOLUME_MOUNT_POINT_CREATED => {
            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
                IO_NO_INCREMENT as i32,
                1,
                0u8, /* FALSE */
            );
            LockStatus = WaitForRemoteDatabaseSemaphore(DeviceExtension);
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = MountMgrVolumeMountPointCreated(DeviceExtension, Irp, LockStatus);
            if LockStatus >= 0 {
                ReleaseRemoteDatabaseSemaphore(DeviceExtension);
            }
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_VOLUME_MOUNT_POINT_DELETED => {
            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
                IO_NO_INCREMENT as i32,
                1,
                0u8,
            );
            LockStatus = WaitForRemoteDatabaseSemaphore(DeviceExtension);
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = MountMgrVolumeMountPointDeleted(DeviceExtension, Irp, LockStatus);
            if LockStatus >= 0 {
                ReleaseRemoteDatabaseSemaphore(DeviceExtension);
            }
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_CHANGE_NOTIFY => {
            Status = MountMgrChangeNotify(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_KEEP_LINKS_WHEN_OFFLINE => {
            Status = MountMgrKeepLinksWhenOffline(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_CHECK_UNPROCESSED_VOLUMES => {
            Status = MountMgrCheckUnprocessedVolumes(DeviceExtension, Irp);
            current_block = 1210194547893430006;
        }
        IOCTL_MOUNTMGR_VOLUME_ARRIVAL_NOTIFICATION => {
            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
                IO_NO_INCREMENT as i32,
                1,
                0u8,
            );
            Status = MountMgrVolumeArrivalNotification(DeviceExtension, Irp);
            current_block = 1210194547893430006;
        }
        IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH => {
            Status = MountMgrQueryDosVolumePath(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS => {
            Status = MountMgrQueryDosVolumePaths(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_SCRUB_REGISTRY => {
            Status = MountMgrScrubRegistry(DeviceExtension);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_QUERY_AUTO_MOUNT => {
            Status = MountMgrQueryAutoMount(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        IOCTL_MOUNTMGR_SET_AUTO_MOUNT => {
            Status = MountMgrSetAutoMount(DeviceExtension, Irp);
            current_block = 7226443171521532240;
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
            current_block = 7226443171521532240;
        }
    }

    match current_block {
        7226443171521532240 => {
            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
                IO_NO_INCREMENT as i32,
                1,
                0u8,
            );
            if !(Status != STATUS_PENDING) {
                return Status;
            }
        }
        _ => {}
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
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
//  * FILE:             drivers/filesystem/mountmgr/device.c
//  * PURPOSE:          Mount Manager - Device Control
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #define MAX_DEVICES 0x3E8 /* Matches 1000 devices */
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrChangeNotify(IN PDEVICE_EXTENSION DeviceExtension,
//                      IN PIRP Irp)
// {
//     KIRQL OldIrql;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION Stack;
//     PMOUNTMGR_CHANGE_NOTIFY_INFO ChangeNotify;
//
//     /* Get the I/O buffer */
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     ChangeNotify = (PMOUNTMGR_CHANGE_NOTIFY_INFO)Irp->AssociatedIrp.SystemBuffer;
//
//     /* Validate it */
//     if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(MOUNTMGR_CHANGE_NOTIFY_INFO) ||
//         Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_CHANGE_NOTIFY_INFO))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* If epic number doesn't match, just return now one */
//     if (DeviceExtension->EpicNumber != ChangeNotify->EpicNumber)
//     {
//         ChangeNotify->EpicNumber = DeviceExtension->EpicNumber;
//         Irp->IoStatus.Information = sizeof(MOUNTMGR_CHANGE_NOTIFY_INFO);
//         return STATUS_SUCCESS;
//     }
//
//     /* If IRP is to be canceled, forget about that */
//     IoAcquireCancelSpinLock(&OldIrql);
//     if (Irp->Cancel)
//     {
//         Status = STATUS_CANCELLED;
//     }
//     /* Otherwise queue the IRP to be notified with the next epic number change */
//     else
//     {
//         InsertTailList(&(DeviceExtension->IrpListHead), &(Irp->Tail.Overlay.ListEntry));
//         IoMarkIrpPending(Irp);
//         IoSetCancelRoutine(Irp, MountMgrCancel);
//         Status = STATUS_PENDING;
//     }
//     IoReleaseCancelSpinLock(OldIrql);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountmgrWriteNoAutoMount(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     ULONG Value = DeviceExtension->NoAutoMount;
//
//     return RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                  DeviceExtension->RegistryPath.Buffer,
//                                  L"NoAutoMount",
//                                  REG_DWORD,
//                                  &Value,
//                                  sizeof(Value));
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrSetAutoMount(IN PDEVICE_EXTENSION DeviceExtension,
//                      IN PIRP Irp)
// {
//     PIO_STACK_LOCATION Stack;
//     PMOUNTMGR_SET_AUTO_MOUNT SetState;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_SET_AUTO_MOUNT))
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Change the state only if there is a real difference
//      * with the user-provided NewState (normalized) */
//     SetState = (PMOUNTMGR_SET_AUTO_MOUNT)Irp->AssociatedIrp.SystemBuffer;
//     if ((SetState->NewState != Enabled) == DeviceExtension->NoAutoMount)
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_SUCCESS;
//     }
//
//     /* Set new state */
//     DeviceExtension->NoAutoMount = (SetState->NewState != Enabled);
//     Irp->IoStatus.Information = 0;
//     return MountmgrWriteNoAutoMount(DeviceExtension);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQueryAutoMount(IN PDEVICE_EXTENSION DeviceExtension,
//                        IN PIRP Irp)
// {
//     PIO_STACK_LOCATION Stack;
//     PMOUNTMGR_QUERY_AUTO_MOUNT QueryState;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(MOUNTMGR_QUERY_AUTO_MOUNT))
//     {
//         Irp->IoStatus.Information = 0;
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     QueryState = (PMOUNTMGR_QUERY_AUTO_MOUNT)Irp->AssociatedIrp.SystemBuffer;
//     QueryState->CurrentState = !DeviceExtension->NoAutoMount;
//     Irp->IoStatus.Information = sizeof(MOUNTMGR_QUERY_AUTO_MOUNT);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// ScrubRegistryRoutine(IN PWSTR ValueName,
//                      IN ULONG ValueType,
//                      IN PVOID ValueData,
//                      IN ULONG ValueLength,
//                      IN PVOID Context,
//                      IN PVOID EntryContext)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PDEVICE_INFORMATION DeviceInfo;
//     PBOOLEAN Continue = EntryContext;
//     PDEVICE_EXTENSION DeviceExtension = Context;
//
//     if (ValueType != REG_BINARY)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Delete values for devices that don't have the matching unique ID */
//     if (!IsListEmpty(&(DeviceExtension->DeviceListHead)))
//     {
//         for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//              NextEntry != &(DeviceExtension->DeviceListHead);
//              NextEntry = NextEntry->Flink)
//         {
//             DeviceInfo = CONTAINING_RECORD(NextEntry,
//                                            DEVICE_INFORMATION,
//                                            DeviceListEntry);
//
//              if (!DeviceInfo->UniqueId || DeviceInfo->UniqueId->UniqueIdLength != ValueLength)
//              {
//                  continue;
//              }
//
//              if (RtlCompareMemory(DeviceInfo->UniqueId->UniqueId, ValueData, ValueLength) == ValueLength)
//              {
//                  return STATUS_SUCCESS;
//              }
//         }
//     }
//
//     /* Wrong unique ID, scrub it */
//     Status = RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                     DatabasePath,
//                                     ValueName);
//     if (!NT_SUCCESS(Status))
//     {
//         *Continue = TRUE;
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     *Continue = FALSE;
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrScrubRegistry(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//     BOOLEAN Continue;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     do
//     {
//         RtlZeroMemory(QueryTable, sizeof(QueryTable));
//         QueryTable[0].QueryRoutine = ScrubRegistryRoutine;
//         QueryTable[0].EntryContext = &Continue;
//         Continue = FALSE;
//
//         Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                         DatabasePath,
//                                         QueryTable,
//                                         DeviceExtension,
//                                         NULL);
//     }
//     while (Continue);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrCreatePoint(IN PDEVICE_EXTENSION DeviceExtension,
//                     IN PIRP Irp)
// {
//     ULONG MaxLength;
//     PIO_STACK_LOCATION Stack;
//     PMOUNTMGR_CREATE_POINT_INPUT Point;
//     UNICODE_STRING DeviceName, SymbolicName;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_CREATE_POINT_INPUT))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Point = (PMOUNTMGR_CREATE_POINT_INPUT)Irp->AssociatedIrp.SystemBuffer;
//
//     MaxLength = MAX((Point->DeviceNameOffset + Point->DeviceNameLength),
//                     (Point->SymbolicLinkNameLength + Point->SymbolicLinkNameOffset));
//     if (MaxLength > Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get all the strings and call the worker */
//     SymbolicName.Length = Point->SymbolicLinkNameLength;
//     SymbolicName.MaximumLength = Point->SymbolicLinkNameLength;
//     DeviceName.Length = Point->DeviceNameLength;
//     DeviceName.MaximumLength = Point->DeviceNameLength;
//     SymbolicName.Buffer = (PVOID)((ULONG_PTR)Point + Point->SymbolicLinkNameOffset);
//     DeviceName.Buffer = (PVOID)((ULONG_PTR)Point + Point->DeviceNameOffset);
//
//     return MountMgrCreatePointWorker(DeviceExtension, &SymbolicName, &DeviceName);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrCheckUnprocessedVolumes(IN PDEVICE_EXTENSION DeviceExtension,
//                                 IN PIRP Irp)
// {
//     PLIST_ENTRY NextEntry;
//     PDEVICE_INFORMATION DeviceInformation;
//     NTSTATUS ArrivalStatus, Status = STATUS_SUCCESS;
//
//     UNREFERENCED_PARAMETER(Irp);
//
//     /* No offline volumes, nothing more to do */
//     if (IsListEmpty(&(DeviceExtension->OfflineDeviceListHead)))
//     {
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//         return STATUS_SUCCESS;
//     }
//
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//     /* Reactivate all the offline volumes */
//     while (!IsListEmpty(&(DeviceExtension->OfflineDeviceListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceExtension->OfflineDeviceListHead));
//         DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         ArrivalStatus = MountMgrMountedDeviceArrival(DeviceExtension,
//                                                      &(DeviceInformation->SymbolicName),
//                                                      DeviceInformation->ManuallyRegistered);
//         /* Then, remove them dead information */
//         MountMgrFreeDeadDeviceInfo(DeviceInformation);
//
//         if (NT_SUCCESS(Status))
//         {
//             Status = ArrivalStatus;
//         }
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// IsFtVolume(IN PUNICODE_STRING SymbolicName)
// {
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PARTITION_INFORMATION PartitionInfo;
//     PDEVICE_OBJECT DeviceObject, FileDeviceObject;
//
//     /* Get device object */
//     Status = IoGetDeviceObjectPointer(SymbolicName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//         return FALSE;
//
//     /* Get attached device */
//     FileDeviceObject = FileObject->DeviceObject;
//     DeviceObject = IoGetAttachedDeviceReference(FileDeviceObject);
//
//     /* FT volume can't be removable */
//     if (FileDeviceObject->Characteristics & FILE_REMOVABLE_MEDIA)
//     {
//         ObDereferenceObject(DeviceObject);
//         ObDereferenceObject(FileObject);
//         return FALSE;
//     }
//
//     ObDereferenceObject(FileObject);
//
//     /* Get partition information */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_DISK_GET_PARTITION_INFO,
//                                          DeviceObject,
//                                          NULL,
//                                          0,
//                                          &PartitionInfo,
//                                          sizeof(PartitionInfo),
//                                          NULL);
//
//     ObDereferenceObject(DeviceObject);
//     if (!NT_SUCCESS(Status))
//         return FALSE;
//
//     /* Check if this is a FT volume */
//     return IsFTPartition(PartitionInfo.PartitionType);
// }
//
// /*
//  * @implemented
//  */
// VOID
// ProcessSuggestedDriveLetters(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     WCHAR NameBuffer[DRIVE_LETTER_LENGTH / sizeof(WCHAR)];
//     PLIST_ENTRY NextEntry;
//     UNICODE_STRING SymbolicName;
//     PDEVICE_INFORMATION DeviceInformation;
//
//     /* No devices? Nothing to do! */
//     if (IsListEmpty(&(DeviceExtension->DeviceListHead)))
//     {
//         return;
//     }
//
//     /* For all the devices */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         /* If no drive letter */
//         if (DeviceInformation->SuggestedDriveLetter == (UCHAR)-1)
//         {
//             /* Ensure it has no entry yet */
//             if (!HasDriveLetter(DeviceInformation) &&
//                 !HasNoDriveLetterEntry(DeviceInformation->UniqueId))
//             {
//                 /* And create one */
//                 CreateNoDriveLetterEntry(DeviceInformation->UniqueId);
//             }
//
//             DeviceInformation->SuggestedDriveLetter = 0;
//         }
//         /* Suggested letter & no entry */
//         else if (DeviceInformation->SuggestedDriveLetter &&
//                  !HasNoDriveLetterEntry(DeviceInformation->UniqueId))
//         {
//             /* Just create a mount point */
//             SymbolicName.Buffer = NameBuffer;
//             RtlCopyMemory(NameBuffer, DosDevices.Buffer, DosDevices.Length);
//             NameBuffer[LETTER_POSITION] = DeviceInformation->SuggestedDriveLetter;
//             NameBuffer[COLON_POSITION] = L':';
//             SymbolicName.Length =
//             SymbolicName.MaximumLength = DRIVE_LETTER_LENGTH;
//
//             MountMgrCreatePointWorker(DeviceExtension, &SymbolicName, &(DeviceInformation->DeviceName));
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrNextDriveLetterWorker(IN PDEVICE_EXTENSION DeviceExtension,
//                               IN PUNICODE_STRING DeviceName,
//                               OUT PMOUNTMGR_DRIVE_LETTER_INFORMATION DriveLetterInfo)
// {
//     NTSTATUS Status;
//     UCHAR DriveLetter;
//     PLIST_ENTRY NextEntry;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     BOOLEAN Removable, GptDriveLetter;
//     PDEVICE_INFORMATION DeviceInformation;
//     WCHAR NameBuffer[DRIVE_LETTER_LENGTH];
//     PSYMLINK_INFORMATION SymlinkInformation;
//     UNICODE_STRING TargetDeviceName, SymbolicName;
//
//     /* First, process suggested letters */
//     if (!DeviceExtension->ProcessedSuggestions)
//     {
//         ProcessSuggestedDriveLetters(DeviceExtension);
//         DeviceExtension->ProcessedSuggestions = TRUE;
//     }
//
//     /* Then, get information about the device */
//     Status = QueryDeviceInformation(DeviceName, &TargetDeviceName, NULL, &Removable, &GptDriveLetter, NULL, NULL, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Ensure we have such device */
//     NextEntry = DeviceExtension->DeviceListHead.Flink;
//     while (NextEntry != &(DeviceExtension->DeviceListHead))
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         if (RtlCompareUnicodeString(&(DeviceInformation->DeviceName), &TargetDeviceName, TRUE) == 0)
//         {
//             break;
//         }
//
//         NextEntry = NextEntry->Flink;
//     }
//
//     if (NextEntry == &(DeviceExtension->DeviceListHead))
//     {
//         FreePool(TargetDeviceName.Buffer);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* Now, assume we will assign a letter */
//     DeviceInformation->LetterAssigned =
//     DriveLetterInfo->DriveLetterWasAssigned = TRUE;
//
//     /* Browse all the symlinks to check if there is already a drive letter */
//     NextEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//     while (NextEntry != &(DeviceInformation->SymbolicLinksListHead))
//     {
//         SymlinkInformation = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         /* If this is a drive letter and it is online, forget about new drive letter */
//         if (IsDriveLetter(&(SymlinkInformation->Name)) && SymlinkInformation->Online)
//         {
//             DriveLetterInfo->DriveLetterWasAssigned = FALSE;
//             DriveLetterInfo->CurrentDriveLetter = (CHAR)SymlinkInformation->Name.Buffer[LETTER_POSITION];
//             break;
//         }
//
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* If we didn't find a drive letter online, ensure this is not
//      * a no-drive entry by querying GPT attributes & database */
//     if (NextEntry == &(DeviceInformation->SymbolicLinksListHead))
//     {
//         if (!GptDriveLetter || HasNoDriveLetterEntry(DeviceInformation->UniqueId))
//         {
//             DriveLetterInfo->DriveLetterWasAssigned = FALSE;
//             DriveLetterInfo->CurrentDriveLetter = 0;
//             goto Release;
//         }
//     }
//
//     /* If automount is disabled, and the device is not removable
//      * but needs a drive letter, don't assign one and bail out */
//     if (DeviceExtension->NoAutoMount && !Removable)
//     {
//         if (DriveLetterInfo->DriveLetterWasAssigned)
//         {
//             DriveLetterInfo->DriveLetterWasAssigned = FALSE;
//             DriveLetterInfo->CurrentDriveLetter = 0;
//             goto Release;
//         }
//     }
//
//     /* Stop now if we don't need to assign the drive a letter */
//     if (!DriveLetterInfo->DriveLetterWasAssigned)
//         goto Release;
//
//     /* Now everything is fine, begin drive letter assignment */
//
//     if (RtlPrefixUnicodeString(&DeviceFloppy, &TargetDeviceName, TRUE))
//     {
//         /* If the device is a floppy, start with letter A */
//         DriveLetter = 'A';
//     }
//     else if (RtlPrefixUnicodeString(&DeviceCdRom, &TargetDeviceName, TRUE))
//     {
//         /* If the device is a CD-ROM, start with letter D */
//         DriveLetter = 'D';
//     }
//     else
//     {
//         /* Finally, if it's a disk, use C */
//         DriveLetter = 'C';
//     }
//
//     /* We cannot set NO drive letter */
//     ASSERT(DeviceInformation->SuggestedDriveLetter != (UCHAR)-1);
//
//     /* If we don't have suggested letter but it's a FT volume, fail */
//     if (!DeviceInformation->SuggestedDriveLetter && IsFtVolume(&(DeviceInformation->DeviceName)))
//     {
//         DriveLetterInfo->DriveLetterWasAssigned = FALSE;
//         DriveLetterInfo->CurrentDriveLetter = 0;
//         goto Release;
//     }
//
//     /* Prepare buffer */
//     RtlCopyMemory(NameBuffer, DosDevices.Buffer, DosDevices.Length);
//     NameBuffer[COLON_POSITION] = L':';
//     SymbolicName.Buffer = NameBuffer;
//     SymbolicName.Length =
//     SymbolicName.MaximumLength = DRIVE_LETTER_LENGTH;
//
//     /* It's all prepared, create mount point */
//     if (DeviceInformation->SuggestedDriveLetter)
//     {
//         DriveLetterInfo->CurrentDriveLetter = DeviceInformation->SuggestedDriveLetter;
//         NameBuffer[LETTER_POSITION] = DeviceInformation->SuggestedDriveLetter;
//
//         Status = MountMgrCreatePointWorker(DeviceExtension, &SymbolicName, &TargetDeviceName);
//         if (NT_SUCCESS(Status))
//         {
//             goto Release;
//         }
//     }
//
//     /* It failed with this letter... Try another one! */
//     for (DriveLetterInfo->CurrentDriveLetter = DriveLetter;
//          DriveLetterInfo->CurrentDriveLetter <= L'Z';
//          DriveLetterInfo->CurrentDriveLetter++)
//     {
//         NameBuffer[LETTER_POSITION] = DriveLetterInfo->CurrentDriveLetter;
//
//         Status = MountMgrCreatePointWorker(DeviceExtension, &SymbolicName, &TargetDeviceName);
//         if (NT_SUCCESS(Status))
//         {
//             break;
//         }
//     }
//
//     /* We failed setting a letter */
//     if (DriveLetterInfo->CurrentDriveLetter > L'Z')
//     {
//         DriveLetterInfo->DriveLetterWasAssigned = FALSE;
//         DriveLetterInfo->CurrentDriveLetter = 0;
//
//         /* Try at least to add a no drive letter entry */
//         Status = QueryDeviceInformation(&TargetDeviceName, NULL, &UniqueId, NULL, NULL, NULL, NULL, NULL);
//         if (NT_SUCCESS(Status))
//         {
//             CreateNoDriveLetterEntry(UniqueId);
//             FreePool(UniqueId);
//         }
//     }
//
// Release:
//     FreePool(TargetDeviceName.Buffer);
//
//     return STATUS_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrNextDriveLetter(IN PDEVICE_EXTENSION DeviceExtension,
//                         IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION Stack;
//     UNICODE_STRING DeviceName;
//     PMOUNTMGR_DRIVE_LETTER_TARGET DriveLetterTarget;
//     MOUNTMGR_DRIVE_LETTER_INFORMATION DriveLetterInformation;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_DRIVE_LETTER_TARGET) ||
//         Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(MOUNTMGR_DRIVE_LETTER_INFORMATION))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     DriveLetterTarget = (PMOUNTMGR_DRIVE_LETTER_TARGET)Irp->AssociatedIrp.SystemBuffer;
//     if (FIELD_OFFSET(MOUNTMGR_DRIVE_LETTER_TARGET, DeviceName) + DriveLetterTarget->DeviceNameLength >
//         Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Call the worker */
//     DeviceName.Buffer = DriveLetterTarget->DeviceName;
//     DeviceName.Length =
//     DeviceName.MaximumLength = DriveLetterTarget->DeviceNameLength;
//
//     Status = MountMgrNextDriveLetterWorker(DeviceExtension, &DeviceName,
//                                            &DriveLetterInformation);
//     if (NT_SUCCESS(Status))
//     {
//         *(PMOUNTMGR_DRIVE_LETTER_INFORMATION)Irp->AssociatedIrp.SystemBuffer =
//             DriveLetterInformation;
//         Irp->IoStatus.Information = sizeof(MOUNTMGR_DRIVE_LETTER_INFORMATION);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MountMgrQuerySystemVolumeNameQueryRoutine(IN PWSTR ValueName,
//                                           IN ULONG ValueType,
//                                           IN PVOID ValueData,
//                                           IN ULONG ValueLength,
//                                           IN PVOID Context,
//                                           IN PVOID EntryContext)
// {
//     UNICODE_STRING ValueString;
//     PUNICODE_STRING SystemVolumeName;
//
//     UNREFERENCED_PARAMETER(ValueName);
//     UNREFERENCED_PARAMETER(ValueLength);
//     UNREFERENCED_PARAMETER(EntryContext);
//
//     if (ValueType != REG_SZ)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     RtlInitUnicodeString(&ValueString, ValueData);
//     SystemVolumeName = Context;
//
//     /* Return a string containing system volume name */
//     SystemVolumeName->Length = ValueString.Length;
//     SystemVolumeName->MaximumLength = ValueString.Length + sizeof(WCHAR);
//     SystemVolumeName->Buffer = AllocatePool(SystemVolumeName->MaximumLength);
//     if (SystemVolumeName->Buffer)
//     {
//         RtlCopyMemory(SystemVolumeName->Buffer, ValueData, ValueString.Length);
//         SystemVolumeName->Buffer[ValueString.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     return STATUS_SUCCESS;
//
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQuerySystemVolumeName(OUT PUNICODE_STRING SystemVolumeName)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = MountMgrQuerySystemVolumeNameQueryRoutine;
//     QueryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED;
//     QueryTable[0].Name = L"SystemPartition";
//
//     SystemVolumeName->Buffer = NULL;
//
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            L"\\Registry\\Machine\\System\\Setup",
//                            QueryTable,
//                            SystemVolumeName,
//                            NULL);
//
//     if (SystemVolumeName->Buffer)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrAssignDriveLetters(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     UNICODE_STRING SystemVolumeName;
//     PDEVICE_INFORMATION DeviceInformation;
//     MOUNTMGR_DRIVE_LETTER_INFORMATION DriveLetterInformation;
//
//     /* First, get system volume name */
//     Status = MountMgrQuerySystemVolumeName(&SystemVolumeName);
//
//     /* If there are no device, it's all done */
//     if (IsListEmpty(&(DeviceExtension->DeviceListHead)))
//     {
//         if (NT_SUCCESS(Status))
//         {
//             FreePool(SystemVolumeName.Buffer);
//         }
//
//         return;
//     }
//
//     /* Now, for all the devices... */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         /* If the device doesn't have a letter assigned, do it! */
//         if (!DeviceInformation->LetterAssigned)
//         {
//             MountMgrNextDriveLetterWorker(DeviceExtension,
//                                           &(DeviceInformation->DeviceName),
//                                           &DriveLetterInformation);
//         }
//
//         /* If it's the system volume */
//         if (NT_SUCCESS(Status) && RtlEqualUnicodeString(&SystemVolumeName, &(DeviceInformation->DeviceName), TRUE))
//         {
//             /* Keep track of it */
//             DeviceExtension->DriveLetterData = AllocatePool(DeviceInformation->UniqueId->UniqueIdLength +
//                                                             sizeof(MOUNTDEV_UNIQUE_ID));
//             if (DeviceExtension->DriveLetterData)
//             {
//                 RtlCopyMemory(DeviceExtension->DriveLetterData,
//                               DeviceInformation->UniqueId,
//                               DeviceInformation->UniqueId->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//             }
//
//             /* Ensure it gets mounted if automount is disabled */
//             if (DeviceExtension->NoAutoMount)
//             {
//                 /* Temporarily re-enable automount for the
//                  * worker to mount and set a drive letter */
//                 DeviceExtension->NoAutoMount = FALSE;
//
//                 MountMgrNextDriveLetterWorker(DeviceExtension,
//                                               &(DeviceInformation->DeviceName),
//                                               &DriveLetterInformation);
//
//                 /* And re-disable automount */
//                 DeviceExtension->NoAutoMount = TRUE;
//             }
//         }
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         FreePool(SystemVolumeName.Buffer);
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQueryDosVolumePath(IN PDEVICE_EXTENSION DeviceExtension,
//                            IN PIRP Irp)
// {
//     NTSTATUS Status;
//     ULONG DevicesFound;
//     PIO_STACK_LOCATION Stack;
//     PLIST_ENTRY SymlinksEntry;
//     UNICODE_STRING SymbolicName;
//     PMOUNTMGR_TARGET_NAME Target;
//     PMOUNTMGR_VOLUME_PATHS Output;
//     PWSTR DeviceString, OldBuffer;
//     USHORT DeviceLength, OldLength;
//     PDEVICE_INFORMATION DeviceInformation;
//     PSYMLINK_INFORMATION SymlinkInformation;
//     PASSOCIATED_DEVICE_ENTRY AssociatedDevice;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input size */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_TARGET_NAME))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Ensure we have received UNICODE_STRING */
//     Target = (PMOUNTMGR_TARGET_NAME)Irp->AssociatedIrp.SystemBuffer;
//     if (Target->DeviceNameLength & 1)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate the entry structure size */
//     if (FIELD_OFFSET(MOUNTMGR_TARGET_NAME, DeviceName) + Target->DeviceNameLength >
//         Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Ensure we can at least return needed size */
//     if (Stack->Parameters.DeviceIoControl.OutputBufferLength < FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Construct string for query */
//     SymbolicName.Length =
//     SymbolicName.MaximumLength = Target->DeviceNameLength;
//     SymbolicName.Buffer = Target->DeviceName;
//
//     /* Find device with our info */
//     Status = FindDeviceInfo(DeviceExtension, &SymbolicName, FALSE, &DeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DeviceLength = 0;
//     DeviceString = NULL;
//     DevicesFound = 0;
//
//     /* Try to find associated device info */
//     while (TRUE)
//     {
//         for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//              SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//              SymlinksEntry = SymlinksEntry->Flink)
//         {
//             SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             /* Try to find with drive letter */
//             if (MOUNTMGR_IS_DRIVE_LETTER(&SymlinkInformation->Name) && SymlinkInformation->Online)
//             {
//                 break;
//             }
//         }
//
//         /* If we've found a device via drive letter, do default processing */
//         if (SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead))
//             break;
//
//         /* If it doesn't have an associated device, go to fallback method */
//         if (IsListEmpty(&DeviceInformation->AssociatedDevicesHead))
//             goto TryWithVolumeName;
//
//         /* Create a string with the information about the device */
//         AssociatedDevice = CONTAINING_RECORD(&(DeviceInformation->AssociatedDevicesHead), ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//         OldLength = DeviceLength;
//         OldBuffer = DeviceString;
//         DeviceLength += AssociatedDevice->String.Length;
//         DeviceString = AllocatePool(DeviceLength);
//         if (!DeviceString)
//         {
//             if (OldBuffer)
//             {
//                 FreePool(OldBuffer);
//             }
//
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Store our info and previous if any */
//         RtlCopyMemory(DeviceString, AssociatedDevice->String.Buffer, AssociatedDevice->String.Length);
//         if (OldBuffer)
//         {
//             RtlCopyMemory(&DeviceString[AssociatedDevice->String.Length / sizeof(WCHAR)], OldBuffer, OldLength);
//             FreePool(OldBuffer);
//         }
//
//         /* Count and continue looking */
//         ++DevicesFound;
//         DeviceInformation = AssociatedDevice->DeviceInformation;
//
//         /* If too many devices, try another way */
//         if (DevicesFound > MAX_DEVICES) /*  1000 */
//         {
//             goto TryWithVolumeName;
//         }
//     }
//
//     /* Reallocate our string, so that we can prepend disk letter */
//     OldBuffer = DeviceString;
//     OldLength = DeviceLength;
//     DeviceLength += 2 * sizeof(WCHAR);
//     DeviceString = AllocatePool(DeviceLength);
//     if (!DeviceString)
//     {
//         if (OldBuffer)
//         {
//             FreePool(OldBuffer);
//         }
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Get the letter */
//     DeviceString[0] = SymlinkInformation->Name.Buffer[LETTER_POSITION];
//     DeviceString[1] = L':';
//
//     /* And copy the rest */
//     if (OldBuffer)
//     {
//         RtlCopyMemory(&DeviceString[2], OldBuffer, OldLength);
//         FreePool(OldBuffer);
//     }
//
// TryWithVolumeName:
//     /* If we didn't find anything, try differently */
//     if (DeviceLength < 2 * sizeof(WCHAR) || DeviceString[1] != L':')
//     {
//         if (DeviceString)
//         {
//             FreePool(DeviceString);
//             DeviceString = NULL;
//             DeviceLength = 0;
//         }
//
//         /* Try to find a volume name matching */
//         for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//              SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//              SymlinksEntry = SymlinksEntry->Flink)
//         {
//             SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             if (MOUNTMGR_IS_VOLUME_NAME(&SymlinkInformation->Name))
//             {
//                 break;
//             }
//         }
//
//         /* If found copy */
//         if (SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead))
//         {
//             DeviceLength = SymlinkInformation->Name.Length;
//             DeviceString = AllocatePool(DeviceLength);
//             if (!DeviceString)
//             {
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             RtlCopyMemory(DeviceString, SymlinkInformation->Name.Buffer, DeviceLength);
//             /* Ensure we are in the Win32 namespace; [1] can be '?' */
//             DeviceString[1] = L'\\';
//         }
//     }
//
//     /* If we didn't find something, fail */
//     if (!DeviceString)
//         return STATUS_NOT_FOUND;
//
//     /* Get the output buffer */
//     Output = (PMOUNTMGR_VOLUME_PATHS)Irp->AssociatedIrp.SystemBuffer;
//
//     /* At least, we will return our length */
//     Output->MultiSzLength = DeviceLength + 2 * sizeof(UNICODE_NULL);
//     Irp->IoStatus.Information = FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz) + Output->MultiSzLength;
//
//     /* If we have enough room for copying the string */
//     if (Irp->IoStatus.Information <= Stack->Parameters.DeviceIoControl.OutputBufferLength)
//     {
//         /* Copy it */
//         if (DeviceLength)
//         {
//             RtlCopyMemory(Output->MultiSz, DeviceString, DeviceLength);
//         }
//
//         /* And double-NUL at its end - this is needed in case of
//          * multiple paths which are separated by a single NUL */
//         FreePool(DeviceString);
//         Output->MultiSz[DeviceLength / sizeof(WCHAR)] = UNICODE_NULL;
//         Output->MultiSz[DeviceLength / sizeof(WCHAR) + 1] = UNICODE_NULL;
//
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Just return the size needed and leave */
//         FreePool(DeviceString);
//         Irp->IoStatus.Information = FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz);
//         return STATUS_BUFFER_OVERFLOW;
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrValidateBackPointer(IN PASSOCIATED_DEVICE_ENTRY AssociatedDeviceEntry,
//                             IN PDEVICE_INFORMATION DeviceInformation,
//                             OUT PBOOLEAN Invalid)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     PLIST_ENTRY SymlinksEntry;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PREPARSE_DATA_BUFFER ReparseData;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING FullName, SubstituteName;
//     PSYMLINK_INFORMATION SymlinkInformation;
//
//     /* Initialize & allocate a string big enough to contain our complete mount point name */
//     FullName.Length = 0;
//     FullName.MaximumLength = AssociatedDeviceEntry->String.Length
//                              + AssociatedDeviceEntry->DeviceInformation->DeviceName.Length
//                              + sizeof(WCHAR)
//                              + sizeof(UNICODE_NULL);
//     FullName.Buffer = AllocatePool(FullName.MaximumLength);
//     if (!FullName.Buffer)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Create the path */
//     RtlAppendUnicodeStringToString(&FullName, &AssociatedDeviceEntry->DeviceInformation->DeviceName);
//     FullName.Buffer[FullName.Length / sizeof(WCHAR)] = L'\\';
//     RtlAppendUnicodeStringToString(&FullName, &AssociatedDeviceEntry->String);
//     FullName.Buffer[FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Open it to query the reparse point */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FullName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenFile(&Handle,
//                         SYNCHRONIZE | FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT);
//     FreePool(FullName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         *Invalid = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     /* Allocate a buffer big enough to read reparse data */
//     ReparseData = AllocatePool(MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     if (ReparseData == NULL)
//     {
//         ZwClose(Handle);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Query reparse data */
//     Status = ZwFsControlFile(Handle,
//                              NULL, NULL, NULL,
//                              &IoStatusBlock,
//                              FSCTL_GET_REPARSE_POINT,
//                              NULL, 0,
//                              ReparseData, MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     ZwClose(Handle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(ReparseData);
//         *Invalid = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     /* Create a string with the substitute name */
//     SubstituteName.Length = ReparseData->SymbolicLinkReparseBuffer.SubstituteNameLength;
//     SubstituteName.MaximumLength = SubstituteName.Length;
//     SubstituteName.Buffer = (PWSTR)((ULONG_PTR)ReparseData->SymbolicLinkReparseBuffer.PathBuffer + ReparseData->SymbolicLinkReparseBuffer.SubstituteNameOffset);
//
//     /* If that's a volume name that matches our associated device, that's a success! */
//     if (MOUNTMGR_IS_VOLUME_NAME(&SubstituteName))
//     {
//         if (SubstituteName.Length == 98 && SubstituteName.Buffer[1] == L'?')
//         {
//             for (SymlinksEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//                  SymlinksEntry != &(DeviceInformation->SymbolicLinksListHead);
//                  SymlinksEntry = SymlinksEntry->Flink)
//             {
//                 SymlinkInformation = CONTAINING_RECORD(SymlinksEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//                 if (RtlEqualUnicodeString(&SubstituteName, &SymlinkInformation->Name, TRUE))
//                 {
//                     FreePool(ReparseData);
//                     return STATUS_SUCCESS;
//                 }
//             }
//         }
//     }
//
//     FreePool(ReparseData);
//     *Invalid = TRUE;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQueryVolumePaths(IN PDEVICE_EXTENSION DeviceExtension,
//                          IN PDEVICE_INFORMATION DeviceInformation,
//                          IN PLIST_ENTRY DeviceInfoList,
//                          OUT PMOUNTMGR_VOLUME_PATHS * VolumePaths,
//                          OUT PDEVICE_INFORMATION *FailedDevice)
// {
//     ULONG Written;
//     NTSTATUS Status;
//     PLIST_ENTRY Entry;
//     PSYMLINK_INFORMATION SymlinkInformation;
//     PDEVICE_INFORMATION_ENTRY DeviceInfoEntry;
//     PASSOCIATED_DEVICE_ENTRY AssociatedDeviceEntry;
//     PMOUNTMGR_VOLUME_PATHS * Paths = NULL, * CurrentPath;
//     ULONG OutputPathLength, NumberOfPaths, ReturnedPaths;
//
//     /* We return at least null char */
//     OutputPathLength = sizeof(UNICODE_NULL);
//
//     for (Entry = DeviceInformation->SymbolicLinksListHead.Flink;
//          Entry != &(DeviceInformation->SymbolicLinksListHead);
//          Entry = Entry->Flink)
//     {
//         SymlinkInformation = CONTAINING_RECORD(Entry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         /* Try to find the drive letter (ie, DOS device) */
//         if (MOUNTMGR_IS_DRIVE_LETTER(&SymlinkInformation->Name) && SymlinkInformation->Online)
//         {
//             /* We'll return the letter */
//             OutputPathLength = 4 * sizeof(WCHAR);
//             break;
//         }
//     }
//
//     /* We didn't find any */
//     if (Entry == &(DeviceInformation->SymbolicLinksListHead))
//     {
//         SymlinkInformation = NULL;
//     }
//
//     /* Do we have any device info to return? */
//     for (Entry = DeviceInfoList->Flink; Entry != DeviceInfoList; Entry = Entry->Flink)
//     {
//         DeviceInfoEntry = CONTAINING_RECORD(Entry, DEVICE_INFORMATION_ENTRY, DeviceInformationEntry);
//
//         /* Matching current device */
//         if (DeviceInfoEntry->DeviceInformation == DeviceInformation)
//         {
//             /* Allocate the output buffer */
//             *VolumePaths = AllocatePool(sizeof(ULONG) + OutputPathLength);
//             if (*VolumePaths == NULL)
//             {
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Set size */
//             (*VolumePaths)->MultiSzLength = OutputPathLength;
//             /* If we have a drive letter, return it */
//             if (SymlinkInformation != NULL)
//             {
//                 (*VolumePaths)->MultiSz[0] = SymlinkInformation->Name.Buffer[LETTER_POSITION];
//                 (*VolumePaths)->MultiSz[1] = L':';
//                 (*VolumePaths)->MultiSz[2] = UNICODE_NULL;
//                 (*VolumePaths)->MultiSz[3] = UNICODE_NULL;
//             }
//             else
//             {
//                 (*VolumePaths)->MultiSz[0] = UNICODE_NULL;
//             }
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Allocate a new device entry */
//     DeviceInfoEntry = AllocatePool(sizeof(DEVICE_INFORMATION_ENTRY));
//     if (DeviceInfoEntry == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Add it to the list */
//     DeviceInfoEntry->DeviceInformation = DeviceInformation;
//     InsertTailList(DeviceInfoList, &DeviceInfoEntry->DeviceInformationEntry);
//
//     NumberOfPaths = 0;
//     /* Count the amount of devices we will have to handle */
//     if (!IsListEmpty(&DeviceInformation->AssociatedDevicesHead))
//     {
//         for (Entry = DeviceInformation->AssociatedDevicesHead.Flink;
//                      Entry != &DeviceInformation->AssociatedDevicesHead;
//                      Entry = Entry->Flink)
//         {
//             ++NumberOfPaths;
//         }
//
//         ASSERT(NumberOfPaths != 0);
//         /* And allocate a big enough buffer */
//         Paths = AllocatePool(NumberOfPaths * sizeof(PMOUNTMGR_VOLUME_PATHS));
//         if (Paths == NULL)
//         {
//             RemoveEntryList(&DeviceInfoEntry->DeviceInformationEntry);
//             FreePool(DeviceInfoEntry);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     /* Start the hot loop to gather all the paths and be able to compute total output length! */
//     ReturnedPaths = 0;
//     CurrentPath = Paths;
//     for (Entry = DeviceInformation->AssociatedDevicesHead.Flink;
//          Entry != &DeviceInformation->AssociatedDevicesHead;
//          Entry = Entry->Flink)
//     {
//         USHORT InnerStrings;
//         BOOLEAN Invalid = FALSE;
//
//         AssociatedDeviceEntry = CONTAINING_RECORD(Entry, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//
//         /* Validate the fact its a mount point by query reparse data */
//         Status = MountMgrValidateBackPointer(AssociatedDeviceEntry, DeviceInformation, &Invalid);
//
//         /* If we found an invalid device, that's a failure */
//         if (Invalid)
//         {
//             *FailedDevice = AssociatedDeviceEntry->DeviceInformation;
//             Status = STATUS_UNSUCCESSFUL;
//         }
//
//         /* Check whether we failed, if so, bail out */
//         if (!NT_SUCCESS(Status))
//         {
//             ULONG i;
//
//             for (i = 0; i < ReturnedPaths; ++i)
//             {
//                 FreePool(Paths[i]);
//             }
//
//             if (Paths != NULL)
//             {
//                 FreePool(Paths);
//             }
//             RemoveEntryList(&DeviceInfoEntry->DeviceInformationEntry);
//             FreePool(DeviceInfoEntry);
//             return Status;
//         }
//
//         /* Query associated paths (hello ourselves :-)) */
//         Status = MountMgrQueryVolumePaths(DeviceExtension,
//                                           AssociatedDeviceEntry->DeviceInformation,
//                                           DeviceInfoList,
//                                           CurrentPath,
//                                           FailedDevice);
//         if (!NT_SUCCESS(Status))
//         {
//             ULONG i;
//
//             for (i = 0; i < ReturnedPaths; ++i)
//             {
//                 FreePool(Paths[i]);
//             }
//
//             if (Paths != NULL)
//             {
//                 FreePool(Paths);
//             }
//             RemoveEntryList(&DeviceInfoEntry->DeviceInformationEntry);
//             FreePool(DeviceInfoEntry);
//             return Status;
//         }
//
//         /* Count the number of strings we have in the multi string buffer */
//         InnerStrings = 0;
//         if ((*CurrentPath)->MultiSzLength != sizeof(UNICODE_NULL))
//         {
//             ULONG i;
//             PWSTR MultiSz = (*CurrentPath)->MultiSz;
//
//             for (i = 0; i < (*CurrentPath)->MultiSzLength / sizeof(WCHAR); ++i, ++MultiSz)
//             {
//                 if (*MultiSz == UNICODE_NULL)
//                 {
//                     ++InnerStrings;
//                 }
//             }
//         }
//
//         /* We returned one more path (ie, one more allocated buffer) */
//         ++ReturnedPaths;
//         /* Move the next pointer to use in the array */
//         ++CurrentPath;
//         /* Multiply String.Length by the number of found paths, we always add it after a path */
//         OutputPathLength += (*CurrentPath)->MultiSzLength + InnerStrings * AssociatedDeviceEntry->String.Length - sizeof(UNICODE_NULL);
//     }
//
//     /* Allocate the output buffer */
//     *VolumePaths = AllocatePool(sizeof(ULONG) + OutputPathLength);
//     if (*VolumePaths == NULL)
//     {
//         ULONG i;
//
//         for (i = 0; i < ReturnedPaths; ++i)
//         {
//             FreePool(Paths[i]);
//         }
//
//         if (Paths != NULL)
//         {
//             FreePool(Paths);
//         }
//         RemoveEntryList(&DeviceInfoEntry->DeviceInformationEntry);
//         FreePool(DeviceInfoEntry);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Written = 0;
//     /* If we had found a DOS letter, that's the first thing we return */
//     (*VolumePaths)->MultiSzLength = OutputPathLength;
//     if (SymlinkInformation != NULL)
//     {
//         (*VolumePaths)->MultiSz[0] = SymlinkInformation->Name.Buffer[LETTER_POSITION];
//         (*VolumePaths)->MultiSz[1] = L':';
//         (*VolumePaths)->MultiSz[2] = UNICODE_NULL;
//         Written = 3;
//     }
//
//     /* Now, browse again all our paths to return them */
//     CurrentPath = Paths;
//     for (Entry = DeviceInformation->AssociatedDevicesHead.Flink;
//          Entry != &DeviceInformation->AssociatedDevicesHead;
//          Entry = Entry->Flink)
//     {
//         AssociatedDeviceEntry = CONTAINING_RECORD(Entry, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//
//         /* If we had a path... */
//         if ((*CurrentPath)->MultiSzLength != sizeof(UNICODE_NULL))
//         {
//             ULONG i, Offset;
//             PWSTR MultiSz;
//
//             /* This offset is used to "jump" into MultiSz, so, start with the string begin (ie, skip MultiSzLength) */
//             Offset = sizeof(ULONG);
//             /* Browse every single letter, and skip last UNICODE_NULL */
//             for (i = 0; i < (*CurrentPath)->MultiSzLength / sizeof(WCHAR) - 1; ++i)
//             {
//                 /* Get the letter */
//                 MultiSz = (PWSTR)((ULONG_PTR)(*CurrentPath) + Offset);
//                 /* If it was part of the path, just return it */
//                 if (*MultiSz != UNICODE_NULL)
//                 {
//                     (*VolumePaths)->MultiSz[Written] = *MultiSz;
//                 }
//                 else
//                 {
//                     /* Otherwise, as planed, return our whole associated device name */
//                     RtlCopyMemory(&(*VolumePaths)->MultiSz[Written],
//                                   AssociatedDeviceEntry->String.Buffer,
//                                   AssociatedDeviceEntry->String.Length);
//                     Written += AssociatedDeviceEntry->String.Length / sizeof(WCHAR);
//                     /* And don't forget to nullify */
//                     (*VolumePaths)->MultiSz[Written] = UNICODE_NULL;
//                 }
//
//                 /* We at least return a letter or a null char */
//                 ++Written;
//                 /* Move to the next letter */
//                 Offset += sizeof(WCHAR);
//             }
//         }
//
//         FreePool(*CurrentPath);
//         ++CurrentPath;
//     }
//
//     /* MultiSz: don't forget last null char */
//     (*VolumePaths)->MultiSz[Written] = UNICODE_NULL;
//     /* Cleanup everything and return success! */
//     if (Paths != NULL)
//     {
//         FreePool(Paths);
//     }
//     RemoveEntryList(&DeviceInfoEntry->DeviceInformationEntry);
//     FreePool(DeviceInfoEntry);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQueryDosVolumePaths(IN PDEVICE_EXTENSION DeviceExtension,
//                             IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY Entry;
//     LIST_ENTRY Devices;
//     BOOLEAN NeedNotification;
//     PIO_STACK_LOCATION Stack;
//     UNICODE_STRING SymbolicName;
//     ULONG Attempts, OutputLength;
//     PMOUNTMGR_TARGET_NAME Target;
//     PMOUNTMGR_VOLUME_PATHS Paths, Output;
//     RECONCILE_WORK_ITEM_CONTEXT ReconcileContext;
//     PDEVICE_INFORMATION DeviceInformation, ListDeviceInfo, FailedDevice;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input size */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_TARGET_NAME))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Ensure we have received UNICODE_STRING */
//     Target = (PMOUNTMGR_TARGET_NAME)Irp->AssociatedIrp.SystemBuffer;
//     if (Target->DeviceNameLength & 1)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate the entry structure size */
//     if (FIELD_OFFSET(MOUNTMGR_TARGET_NAME, DeviceName) + Target->DeviceNameLength >
//         Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Ensure we can at least return needed size */
//     if (Stack->Parameters.DeviceIoControl.OutputBufferLength < FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Construct string for query */
//     SymbolicName.Length =
//     SymbolicName.MaximumLength = Target->DeviceNameLength;
//     SymbolicName.Buffer = Target->DeviceName;
//
//     /* Find device with our info */
//     Status = FindDeviceInfo(DeviceExtension, &SymbolicName, FALSE, &DeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     NeedNotification = FALSE;
//     Attempts = 0;
//     for (;;)
//     {
//         FailedDevice = NULL;
//         InitializeListHead(&Devices);
//
//         /* Query paths */
//         Status = MountMgrQueryVolumePaths(DeviceExtension, DeviceInformation, &Devices, &Paths, &FailedDevice);
//         if (NT_SUCCESS(Status))
//         {
//             break;
//         }
//
//         /* If it failed for generic reason (memory, whatever), bail out (ie, FailedDevice not set) */
//         if (FailedDevice == NULL)
//         {
//             return Status;
//         }
//
//         /* If PnP, let's notify in case of success */
//         if (!DeviceInformation->ManuallyRegistered)
//         {
//             NeedNotification = TRUE;
//         }
//
//         /* Reconcile database */
//         ReconcileContext.DeviceExtension = DeviceExtension;
//         ReconcileContext.DeviceInformation = FailedDevice;
//         KeReleaseSemaphore(&DeviceExtension->DeviceLock, IO_NO_INCREMENT, 1, FALSE);
//         ReconcileThisDatabaseWithMasterWorker(&ReconcileContext);
//         KeWaitForSingleObject(&DeviceExtension->DeviceLock, Executive, KernelMode, FALSE, NULL);
//
//         /* Look for our device, to check it's online */
//         for (Entry = DeviceExtension->DeviceListHead.Flink;
//              Entry != &DeviceExtension->DeviceListHead;
//              Entry = Entry->Flink)
//         {
//             ListDeviceInfo = CONTAINING_RECORD(Entry, DEVICE_INFORMATION, DeviceListEntry);
//             /* It's online, it's OK! */
//             if (ListDeviceInfo == DeviceInformation)
//             {
//                 break;
//             }
//         }
//
//         /* It's not online, it's not good */
//         if (Entry == &DeviceExtension->DeviceListHead)
//         {
//             return STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//
//         /* Increase attempts count */
//         ++Attempts;
//         /* Don't look forever and fail if we get out of attempts */
//         if (Attempts >= 1000)
//         {
//             return Status;
//         }
//     }
//
//     /* We need to notify? Go ahead */
//     if (NeedNotification)
//     {
//         MountMgrNotifyNameChange(DeviceExtension, &SymbolicName, FALSE);
//     }
//
//     /* Get the output buffer */
//     Output = (PMOUNTMGR_VOLUME_PATHS)Irp->AssociatedIrp.SystemBuffer;
//
//     /* Set required size */
//     Output->MultiSzLength = Paths->MultiSzLength;
//
//     /* Compute total length */
//     OutputLength = FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz) + Output->MultiSzLength;
//
//     /* If it cannot fit, just return the size needed and leave */
//     if (OutputLength > Stack->Parameters.DeviceIoControl.OutputBufferLength)
//     {
//         Irp->IoStatus.Information = FIELD_OFFSET(MOUNTMGR_VOLUME_PATHS, MultiSz);
//         FreePool(Paths);
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Copy data and quit */
//     Irp->IoStatus.Information = OutputLength;
//     RtlCopyMemory(Output->MultiSz, Paths->MultiSz, Output->MultiSzLength);
//     FreePool(Paths);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrKeepLinksWhenOffline(IN PDEVICE_EXTENSION DeviceExtension,
//                              IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION Stack;
//     UNICODE_STRING SymbolicName;
//     PMOUNTMGR_TARGET_NAME Target;
//     PDEVICE_INFORMATION DeviceInformation;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_TARGET_NAME))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Target = (PMOUNTMGR_TARGET_NAME)Irp->AssociatedIrp.SystemBuffer;
//     if (FIELD_OFFSET(MOUNTMGR_TARGET_NAME, DeviceName) + Target->DeviceNameLength >
//         Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     SymbolicName.Length =
//     SymbolicName.MaximumLength = Target->DeviceNameLength;
//     SymbolicName.Buffer = Target->DeviceName;
//
//     /* Find the associated device */
//     Status = FindDeviceInfo(DeviceExtension, &SymbolicName, FALSE, &DeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Mark we want to keep links */
//     DeviceInformation->KeepLinks = TRUE;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrVolumeArrivalNotification(IN PDEVICE_EXTENSION DeviceExtension,
//                                   IN PIRP Irp)
// {
//     NTSTATUS Status;
//     BOOLEAN OldState;
//     PIO_STACK_LOCATION Stack;
//     UNICODE_STRING SymbolicName;
//     PMOUNTMGR_TARGET_NAME Target;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_TARGET_NAME))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Target = (PMOUNTMGR_TARGET_NAME)Irp->AssociatedIrp.SystemBuffer;
//     if (FIELD_OFFSET(MOUNTMGR_TARGET_NAME, DeviceName) + Target->DeviceNameLength >
//         Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     SymbolicName.Length =
//     SymbolicName.MaximumLength = Target->DeviceNameLength;
//     SymbolicName.Buffer = Target->DeviceName;
//
//     /* Disable hard errors */
//     OldState = PsGetThreadHardErrorsAreDisabled(PsGetCurrentThread());
//     PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), TRUE);
//
//     /* Call real worker */
//     Status = MountMgrMountedDeviceArrival(DeviceExtension, &SymbolicName, TRUE);
//
//     PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), OldState);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQueryPoints(IN PDEVICE_EXTENSION DeviceExtension,
//                     IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION Stack;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     PMOUNTMGR_MOUNT_POINT MountPoint;
//     UNICODE_STRING SymbolicName, DeviceName;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input... */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_MOUNT_POINT))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     MountPoint = (PMOUNTMGR_MOUNT_POINT)Irp->AssociatedIrp.SystemBuffer;
//     if (!MountPoint->SymbolicLinkNameLength)
//     {
//         MountPoint->SymbolicLinkNameOffset = 0;
//     }
//
//     if (!MountPoint->UniqueIdLength)
//     {
//         MountPoint->UniqueIdOffset = 0;
//     }
//
//     if (!MountPoint->DeviceNameLength)
//     {
//         MountPoint->DeviceNameOffset = 0;
//     }
//
//     /* Addresses can't be odd */
//     if ((MountPoint->SymbolicLinkNameOffset & 1) ||
//         (MountPoint->SymbolicLinkNameLength & 1))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((MountPoint->UniqueIdOffset & 1) ||
//         (MountPoint->UniqueIdLength & 1))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((MountPoint->DeviceNameOffset & 1) ||
//         (MountPoint->DeviceNameLength & 1))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* We can't go beyond */
//     if (((ULONG)MountPoint->SymbolicLinkNameLength + MountPoint->UniqueIdLength +
//         MountPoint->DeviceNameLength) > Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(MOUNTMGR_MOUNT_POINTS))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* If caller provided a Symlink, use it */
//     if (MountPoint->SymbolicLinkNameLength != 0)
//     {
//         if (MountPoint->SymbolicLinkNameLength > MAXSHORT)
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         SymbolicName.Length = MountPoint->SymbolicLinkNameLength;
//         SymbolicName.MaximumLength = MountPoint->SymbolicLinkNameLength + sizeof(WCHAR);
//         SymbolicName.Buffer = AllocatePool(SymbolicName.MaximumLength);
//         if (!SymbolicName.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(SymbolicName.Buffer,
//                       (PWSTR)((ULONG_PTR)MountPoint + MountPoint->SymbolicLinkNameOffset),
//                       SymbolicName.Length);
//         SymbolicName.Buffer[SymbolicName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* Query links using it */
//         Status = QueryPointsFromSymbolicLinkName(DeviceExtension, &SymbolicName, Irp);
//         FreePool(SymbolicName.Buffer);
//     }
//     /* If user provided an unique ID */
//     else if (MountPoint->UniqueIdLength != 0)
//     {
//         UniqueId = AllocatePool(MountPoint->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//         if (!UniqueId)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         UniqueId->UniqueIdLength = MountPoint->UniqueIdLength;
//         RtlCopyMemory(UniqueId->UniqueId,
//                       (PVOID)((ULONG_PTR)MountPoint + MountPoint->UniqueIdOffset),
//                       MountPoint->UniqueIdLength);
//
//          /* Query links using it */
//          Status = QueryPointsFromMemory(DeviceExtension, Irp, UniqueId, NULL);
//          FreePool(UniqueId);
//     }
//     /* If caller provided a device name */
//     else if (MountPoint->DeviceNameLength != 0)
//     {
//         if (MountPoint->DeviceNameLength > MAXSHORT)
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         DeviceName.Length = MountPoint->DeviceNameLength;
//         DeviceName.MaximumLength = MountPoint->DeviceNameLength + sizeof(WCHAR);
//         DeviceName.Buffer = AllocatePool(DeviceName.MaximumLength);
//         if (!DeviceName.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(DeviceName.Buffer,
//                       (PWSTR)((ULONG_PTR)MountPoint + MountPoint->DeviceNameOffset),
//                       DeviceName.Length);
//         DeviceName.Buffer[DeviceName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//          /* Query links using it */
//         Status = QueryPointsFromMemory(DeviceExtension, Irp, NULL, &DeviceName);
//         FreePool(DeviceName.Buffer);
//     }
//     else
//     {
//         /* Otherwise, query all links */
//         Status = QueryPointsFromMemory(DeviceExtension, Irp, NULL, NULL);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrDeletePoints(IN PDEVICE_EXTENSION DeviceExtension,
//                      IN PIRP Irp)
// {
//     ULONG Link;
//     NTSTATUS Status;
//     BOOLEAN CreateNoDrive;
//     PIO_STACK_LOCATION Stack;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     PMOUNTMGR_MOUNT_POINT MountPoint;
//     PMOUNTMGR_MOUNT_POINTS MountPoints;
//     UNICODE_STRING SymbolicName, DeviceName;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_MOUNT_POINT))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Query points */
//     MountPoint = (PMOUNTMGR_MOUNT_POINT)Irp->AssociatedIrp.SystemBuffer;
//     CreateNoDrive = (MountPoint->SymbolicLinkNameOffset && MountPoint->SymbolicLinkNameLength);
//
//     Status = MountMgrQueryPoints(DeviceExtension, Irp);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* For all the points matching the request */
//     MountPoints = (PMOUNTMGR_MOUNT_POINTS)Irp->AssociatedIrp.SystemBuffer;
//     for (Link = 0; Link < MountPoints->NumberOfMountPoints; Link++)
//     {
//         SymbolicName.Length = MountPoints->MountPoints[Link].SymbolicLinkNameLength;
//         SymbolicName.MaximumLength = SymbolicName.Length + sizeof(WCHAR);
//         SymbolicName.Buffer = AllocatePool(SymbolicName.MaximumLength);
//         if (!SymbolicName.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(SymbolicName.Buffer,
//                       (PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[Link].SymbolicLinkNameOffset),
//                       SymbolicName.Length);
//         SymbolicName.Buffer[SymbolicName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* Create a no drive entry for the drive letters */
//         if (CreateNoDrive && IsDriveLetter(&SymbolicName))
//         {
//             UniqueId = AllocatePool(MountPoints->MountPoints[Link].UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//             if (UniqueId)
//             {
//                 UniqueId->UniqueIdLength = MountPoints->MountPoints[Link].UniqueIdLength;
//                 RtlCopyMemory(UniqueId->UniqueId,
//                               (PMOUNTDEV_UNIQUE_ID)((ULONG_PTR)MountPoints + MountPoints->MountPoints[Link].UniqueIdOffset),
//                               MountPoints->MountPoints[Link].UniqueIdLength);
//
//                 CreateNoDriveLetterEntry(UniqueId);
//                 FreePool(UniqueId);
//             }
//         }
//
//         /* If there are no link any more, and no need to create a no drive entry */
//         if (Link == 0 && !CreateNoDrive)
//         {
//             /* Then, delete everything */
//             UniqueId = AllocatePool(MountPoints->MountPoints[Link].UniqueIdLength);
//             if (UniqueId)
//             {
//                 RtlCopyMemory(UniqueId,
//                               (PMOUNTDEV_UNIQUE_ID)((ULONG_PTR)MountPoints + MountPoints->MountPoints[Link].UniqueIdOffset),
//                               MountPoints->MountPoints[Link].UniqueIdLength);
//
//                 DeleteNoDriveLetterEntry(UniqueId);
//                 FreePool(UniqueId);
//             }
//         }
//
//         /* Delete all the information about the mount point */
//         GlobalDeleteSymbolicLink(&SymbolicName);
//         DeleteSymbolicLinkNameFromMemory(DeviceExtension, &SymbolicName, FALSE);
//         RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE, DatabasePath, SymbolicName.Buffer);
//         FreePool(SymbolicName.Buffer);
//
//         /* Notify the change */
//         DeviceName.Length = DeviceName.MaximumLength =
//         MountPoints->MountPoints[Link].DeviceNameLength;
//         DeviceName.Buffer = (PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[Link].DeviceNameOffset);
//         MountMgrNotifyNameChange(DeviceExtension, &DeviceName, TRUE);
//     }
//
//     MountMgrNotify(DeviceExtension);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrDeletePointsDbOnly(IN PDEVICE_EXTENSION DeviceExtension,
//                            IN PIRP Irp)
// {
//     ULONG Link;
//     NTSTATUS Status;
//     UNICODE_STRING SymbolicName;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     PMOUNTMGR_MOUNT_POINTS MountPoints;
//
//     /* Query points */
//     Status = MountMgrQueryPoints(DeviceExtension, Irp);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     MountPoints = (PMOUNTMGR_MOUNT_POINTS)Irp->AssociatedIrp.SystemBuffer;
//     if (MountPoints->NumberOfMountPoints == 0)
//     {
//         return Status;
//     }
//
//     /* For all the mount points */
//     for (Link = 0; Link < MountPoints->NumberOfMountPoints; Link++)
//     {
//         SymbolicName.Length = MountPoints->MountPoints[Link].SymbolicLinkNameLength;
//         SymbolicName.MaximumLength = SymbolicName.Length + sizeof(WCHAR);
//         SymbolicName.Buffer = AllocatePool(SymbolicName.MaximumLength);
//         if (!SymbolicName.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(SymbolicName.Buffer,
//                       (PWSTR)((ULONG_PTR)MountPoints + MountPoints->MountPoints[Link].SymbolicLinkNameOffset),
//                       SymbolicName.Length);
//         SymbolicName.Buffer[SymbolicName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* If the only mount point is a drive letter, then create a no letter drive entry */
//         if (MountPoints->NumberOfMountPoints == 1 && IsDriveLetter(&SymbolicName))
//         {
//             UniqueId = AllocatePool(MountPoints->MountPoints[Link].UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//             if (UniqueId)
//             {
//                 UniqueId->UniqueIdLength = MountPoints->MountPoints[Link].UniqueIdLength;
//                 RtlCopyMemory(UniqueId->UniqueId,
//                               (PMOUNTDEV_UNIQUE_ID)((ULONG_PTR)MountPoints + MountPoints->MountPoints[Link].UniqueIdOffset),
//                               MountPoints->MountPoints[Link].UniqueIdLength);
//
//                 CreateNoDriveLetterEntry(UniqueId);
//                 FreePool(UniqueId);
//             }
//         }
//
//         /* Simply delete mount point from DB */
//         DeleteSymbolicLinkNameFromMemory(DeviceExtension, &SymbolicName, TRUE);
//         RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE, DatabasePath, SymbolicName.Buffer);
//         FreePool(SymbolicName.Buffer);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrVolumeMountPointChanged(IN PDEVICE_EXTENSION DeviceExtension,
//                                 IN PIRP Irp,
//                                 IN NTSTATUS LockStatus,
//                                 OUT PUNICODE_STRING SourceDeviceName,
//                                 OUT PUNICODE_STRING SourceSymbolicName,
//                                 OUT PUNICODE_STRING TargetVolumeName)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PIO_STACK_LOCATION Stack;
//     ULONG Length, SavedLength;
//     BOOLEAN FOReferenced = FALSE;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDEVICE_INFORMATION DeviceInformation;
//     OBJECT_NAME_INFORMATION ObjectNameInfo;
//     FILE_FS_DEVICE_INFORMATION FsDeviceInfo;
//     PFILE_NAME_INFORMATION FileNameInfo = NULL;
//     PMOUNTMGR_VOLUME_MOUNT_POINT VolumeMountPoint;
//     POBJECT_NAME_INFORMATION ObjectNameInfoPtr = NULL;
//     UNICODE_STRING SourceVolumeName, TargetDeviceName;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Validate input */
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(MOUNTMGR_VOLUME_MOUNT_POINT))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     VolumeMountPoint = (PMOUNTMGR_VOLUME_MOUNT_POINT)Irp->AssociatedIrp.SystemBuffer;
//
//     if (((ULONG)VolumeMountPoint->SourceVolumeNameLength + VolumeMountPoint->TargetVolumeNameLength) <
//         Stack->Parameters.DeviceIoControl.InputBufferLength)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get source volume name */
//     SourceVolumeName.Length =
//     SourceVolumeName.MaximumLength = VolumeMountPoint->SourceVolumeNameLength;
//     SourceVolumeName.Buffer = (PWSTR)((ULONG_PTR)VolumeMountPoint + VolumeMountPoint->SourceVolumeNameOffset);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SourceVolumeName,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open it */
//     Status = ZwOpenFile(&Handle,
//                         SYNCHRONIZE | FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     TargetDeviceName.Buffer = NULL;
//
//     /* Query its attributes */
//     Status = ZwQueryVolumeInformationFile(Handle,
//                                           &IoStatusBlock,
//                                           &FsDeviceInfo,
//                                           sizeof(FsDeviceInfo),
//                                           FileFsDeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* Mount points can be stored only on storage disks */
//     if (FsDeviceInfo.DeviceType != FILE_DEVICE_DISK && FsDeviceInfo.DeviceType != FILE_DEVICE_VIRTUAL_DISK)
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     /* And they can be on local fixed disks only */
//     if (FsDeviceInfo.Characteristics & (FILE_REMOTE_DEVICE | FILE_REMOVABLE_MEDIA))
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     /* Reference it */
//     Status = ObReferenceObjectByHandle(Handle, 0, *IoFileObjectType, KernelMode, (PVOID *)&FileObject, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//     FOReferenced = TRUE;
//
//     /* Get file name */
//     FileNameInfo = AllocatePool(sizeof(FILE_NAME_INFORMATION));
//     if (!FileNameInfo)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     Status = ZwQueryInformationFile(Handle, &IoStatusBlock, FileNameInfo,
//                                     sizeof(FILE_NAME_INFORMATION),
//                                     FileNameInformation);
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         /* Now we have real length, use it */
//         Length = FileNameInfo->FileNameLength;
//         FreePool(FileNameInfo);
//
//         FileNameInfo = AllocatePool(sizeof(FILE_NAME_INFORMATION) + Length);
//         if (!FileNameInfo)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         /* Really query file name */
//         Status = ZwQueryInformationFile(Handle, &IoStatusBlock, FileNameInfo,
//                                         sizeof(FILE_NAME_INFORMATION) + Length,
//                                         FileNameInformation);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* Get symbolic name */
//     ObjectNameInfoPtr = &ObjectNameInfo;
//     SavedLength = sizeof(OBJECT_NAME_INFORMATION);
//     Status = ObQueryNameString(FileObject->DeviceObject, ObjectNameInfoPtr, sizeof(OBJECT_NAME_INFORMATION), &Length);
//     if (Status == STATUS_INFO_LENGTH_MISMATCH)
//     {
//         /* Once again, with proper size, it works better */
//         ObjectNameInfoPtr = AllocatePool(Length);
//         if (!ObjectNameInfoPtr)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         SavedLength = Length;
//         Status = ObQueryNameString(FileObject->DeviceObject, ObjectNameInfoPtr, SavedLength, &Length);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* Now, query the device name */
//     Status = QueryDeviceInformation(&ObjectNameInfoPtr->Name, SourceDeviceName,
//                                     NULL, NULL, NULL, NULL, NULL, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* For target volume name, use input */
//     TargetVolumeName->Length =
//     TargetVolumeName->MaximumLength = VolumeMountPoint->TargetVolumeNameLength;
//     TargetVolumeName->Buffer = (PWSTR)((ULONG_PTR)VolumeMountPoint + VolumeMountPoint->TargetVolumeNameOffset);
//
//     /* Query its device name */
//     Status = QueryDeviceInformation(TargetVolumeName, &TargetDeviceName,
//                                     NULL, NULL, NULL, NULL, NULL, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* Return symbolic name */
//     SourceSymbolicName->Length =
//     SourceSymbolicName->MaximumLength = (USHORT)FileNameInfo->FileNameLength;
//     SourceSymbolicName->Buffer = (PWSTR)FileNameInfo;
//     /* memmove allows memory overlap */
//     RtlMoveMemory(SourceSymbolicName->Buffer, FileNameInfo->FileName, SourceSymbolicName->Length);
//     FileNameInfo = NULL;
//
//     /* Notify the change */
//     MountMgrNotify(DeviceExtension);
//     MountMgrNotifyNameChange(DeviceExtension, &TargetDeviceName, TRUE);
//
//     /* If we are locked, sync databases if possible */
//     if (NT_SUCCESS(LockStatus))
//     {
//         Status = FindDeviceInfo(DeviceExtension, SourceDeviceName, FALSE, &DeviceInformation);
//         if (NT_SUCCESS(Status))
//         {
//             ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
//         }
//         else
//         {
//             Status = STATUS_PENDING;
//         }
//     }
//
// Cleanup:
//     if (TargetDeviceName.Buffer)
//     {
//         FreePool(TargetDeviceName.Buffer);
//     }
//
//     if (ObjectNameInfoPtr && ObjectNameInfoPtr != &ObjectNameInfo)
//     {
//         FreePool(ObjectNameInfoPtr);
//     }
//
//     if (FileNameInfo)
//     {
//         FreePool(FileNameInfo);
//     }
//
//     if (FOReferenced)
//     {
//         ObDereferenceObject(FileObject);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrVolumeMountPointCreated(IN PDEVICE_EXTENSION DeviceExtension,
//                                 IN PIRP Irp,
//                                 IN NTSTATUS LockStatus)
// {
//     LONG Offset;
//     BOOLEAN Found;
//     NTSTATUS Status;
//     HANDLE RemoteDatabase;
//     PMOUNTDEV_UNIQUE_ID UniqueId;
//     PDATABASE_ENTRY DatabaseEntry;
//     PASSOCIATED_DEVICE_ENTRY AssociatedEntry;
//     PDEVICE_INFORMATION DeviceInformation, TargetDeviceInformation;
//     UNICODE_STRING LinkTarget, SourceDeviceName, SourceSymbolicName, TargetVolumeName, VolumeName, DbName;
//
//     /* Initialize string */
//     LinkTarget.Length = 0;
//     LinkTarget.MaximumLength = 0xC8;
//     LinkTarget.Buffer = AllocatePool(LinkTarget.MaximumLength);
//     if (LinkTarget.Buffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* If the mount point was created, then, it changed!
//      * Also use it to query some information
//      */
//     Status = MountMgrVolumeMountPointChanged(DeviceExtension, Irp, LockStatus, &SourceDeviceName, &SourceSymbolicName, &TargetVolumeName);
//     /* Pending means DB are under synchronization, bail out */
//     if (Status == STATUS_PENDING)
//     {
//         FreePool(LinkTarget.Buffer);
//         FreePool(SourceDeviceName.Buffer);
//         FreePool(SourceSymbolicName.Buffer);
//         return STATUS_SUCCESS;
//     }
//     else if (!NT_SUCCESS(Status))
//     {
//         FreePool(LinkTarget.Buffer);
//         return Status;
//     }
//
//     /* Query the device information */
//     Status = FindDeviceInfo(DeviceExtension, &SourceDeviceName, FALSE, &DeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         /* If it failed, first try to get volume name */
//         Status = QueryVolumeName(0, NULL, &SourceDeviceName, &LinkTarget, &VolumeName);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Then, try to read the symlink */
//             Status = MountMgrQuerySymbolicLink(&SourceDeviceName, &LinkTarget);
//             if (!NT_SUCCESS(Status))
//             {
//                 FreePool(LinkTarget.Buffer);
//                 FreePool(SourceDeviceName.Buffer);
//                 FreePool(SourceSymbolicName.Buffer);
//                 return Status;
//             }
//         }
//         else
//         {
//             FreePool(VolumeName.Buffer);
//         }
//
//         FreePool(SourceDeviceName.Buffer);
//
//         SourceDeviceName.Length = LinkTarget.Length;
//         SourceDeviceName.MaximumLength = LinkTarget.MaximumLength;
//         SourceDeviceName.Buffer = LinkTarget.Buffer;
//
//         /* Now that we have the correct source, reattempt to query information */
//         Status = FindDeviceInfo(DeviceExtension, &SourceDeviceName, FALSE, &DeviceInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(SourceDeviceName.Buffer);
//             FreePool(SourceSymbolicName.Buffer);
//             return Status;
//         }
//     }
//
//     FreePool(SourceDeviceName.Buffer);
//
//     /* Get information about target device */
//     Status = FindDeviceInfo(DeviceExtension, &TargetVolumeName, FALSE, &TargetDeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return Status;
//     }
//
//     /* Notify if not disabled */
//     if (!TargetDeviceInformation->SkipNotifications)
//     {
//         PostOnlineNotification(DeviceExtension, &TargetDeviceInformation->SymbolicName);
//     }
//
//     /* Open the remote database */
//     RemoteDatabase = OpenRemoteDatabase(DeviceInformation, TRUE);
//     if (RemoteDatabase == 0)
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Browse all the entries */
//     Offset = 0;
//     Found = FALSE;
//     for (;;)
//     {
//         DatabaseEntry = GetRemoteDatabaseEntry(RemoteDatabase, Offset);
//         if (DatabaseEntry == NULL)
//         {
//             break;
//         }
//
//         /* Try to find ourselves */
//         DbName.MaximumLength = DatabaseEntry->SymbolicNameLength;
//         DbName.Length = DbName.MaximumLength;
//         DbName.Buffer = (PWSTR)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset);
//         if (RtlEqualUnicodeString(&TargetVolumeName, &DbName, TRUE))
//         {
//             /* Reference ourselves and update the entry */
//             ++DatabaseEntry->EntryReferences;
//             Status = WriteRemoteDatabaseEntry(RemoteDatabase, Offset, DatabaseEntry);
//             FreePool(DatabaseEntry);
//             Found = TRUE;
//             break;
//         }
//
//         Offset += DatabaseEntry->EntrySize;
//         FreePool(DatabaseEntry);
//     }
//
//     /* We couldn't find ourselves, we'll have to add ourselves */
//     if (!Found)
//     {
//         ULONG EntrySize;
//         PUNIQUE_ID_REPLICATE UniqueIdReplicate;
//
//         /* Query the device unique ID */
//         Status = QueryDeviceInformation(&TargetVolumeName, NULL, &UniqueId, NULL, NULL, NULL, NULL, NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return Status;
//         }
//
//         /* Allocate a database entry */
//         EntrySize = UniqueId->UniqueIdLength + TargetVolumeName.Length + sizeof(DATABASE_ENTRY);
//         DatabaseEntry = AllocatePool(EntrySize);
//         if (DatabaseEntry == NULL)
//         {
//             FreePool(UniqueId);
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Fill it in */
//         DatabaseEntry->EntrySize = EntrySize;
//         DatabaseEntry->EntryReferences = 1;
//         DatabaseEntry->SymbolicNameOffset = sizeof(DATABASE_ENTRY);
//         DatabaseEntry->SymbolicNameLength = TargetVolumeName.Length;
//         DatabaseEntry->UniqueIdOffset = TargetVolumeName.Length + sizeof(DATABASE_ENTRY);
//         DatabaseEntry->UniqueIdLength = UniqueId->UniqueIdLength;
//         RtlCopyMemory((PVOID)((ULONG_PTR)DatabaseEntry + sizeof(DATABASE_ENTRY)), TargetVolumeName.Buffer, DatabaseEntry->SymbolicNameLength);
//         RtlCopyMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset), UniqueId->UniqueId, UniqueId->UniqueIdLength);
//
//         /* And write it down */
//         Status = AddRemoteDatabaseEntry(RemoteDatabase, DatabaseEntry);
//         FreePool(DatabaseEntry);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(UniqueId);
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return Status;
//         }
//
//         /* And now, allocate an Unique ID item */
//         UniqueIdReplicate = AllocatePool(sizeof(UNIQUE_ID_REPLICATE));
//         if (UniqueIdReplicate == NULL)
//         {
//             FreePool(UniqueId);
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return Status;
//         }
//
//         /* To associate it with the device */
//         UniqueIdReplicate->UniqueId = UniqueId;
//         InsertTailList(&DeviceInformation->ReplicatedUniqueIdsListHead, &UniqueIdReplicate->ReplicatedUniqueIdsListEntry);
//     }
//
//     /* We're done with the remote database */
//     CloseRemoteDatabase(RemoteDatabase);
//
//     /* Check we were find writing the entry */
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return Status;
//     }
//
//     /* This is the end, allocate an associated entry */
//     AssociatedEntry = AllocatePool(sizeof(ASSOCIATED_DEVICE_ENTRY));
//     if (AssociatedEntry == NULL)
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize its source name string */
//     AssociatedEntry->String.Length = SourceSymbolicName.Length;
//     AssociatedEntry->String.MaximumLength = AssociatedEntry->String.Length + sizeof(UNICODE_NULL);
//     AssociatedEntry->String.Buffer = AllocatePool(AssociatedEntry->String.MaximumLength);
//     if (AssociatedEntry->String.Buffer == NULL)
//     {
//         FreePool(AssociatedEntry);
//         FreePool(SourceSymbolicName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy data & insert in list */
//     RtlCopyMemory(AssociatedEntry->String.Buffer, SourceSymbolicName.Buffer, SourceSymbolicName.Length);
//     AssociatedEntry->String.Buffer[SourceSymbolicName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     AssociatedEntry->DeviceInformation = DeviceInformation;
//     InsertTailList(&TargetDeviceInformation->AssociatedDevicesHead, &AssociatedEntry->AssociatedDevicesEntry);
//
//     /* We're done! */
//     FreePool(SourceSymbolicName.Buffer);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrVolumeMountPointDeleted(IN PDEVICE_EXTENSION DeviceExtension,
//                                 IN PIRP Irp,
//                                 IN NTSTATUS LockStatus)
// {
//     LONG Offset;
//     NTSTATUS Status;
//     PLIST_ENTRY Entry;
//     HANDLE RemoteDatabase;
//     PDATABASE_ENTRY DatabaseEntry;
//     PUNIQUE_ID_REPLICATE UniqueIdReplicate;
//     PASSOCIATED_DEVICE_ENTRY AssociatedEntry;
//     PDEVICE_INFORMATION DeviceInformation, TargetDeviceInformation;
//     UNICODE_STRING LinkTarget, SourceDeviceName, SourceSymbolicName, TargetVolumeName, VolumeName, DbName;
//
//     /* Initialize string */
//     LinkTarget.Length = 0;
//     LinkTarget.MaximumLength = 0xC8;
//     LinkTarget.Buffer = AllocatePool(LinkTarget.MaximumLength);
//     if (LinkTarget.Buffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* If the mount point was deleted, then, it changed!
//      * Also use it to query some information
//      */
//     Status = MountMgrVolumeMountPointChanged(DeviceExtension, Irp, LockStatus, &SourceDeviceName, &SourceSymbolicName, &TargetVolumeName);
//     /* Pending means DB are under synchronization, bail out */
//     if (Status == STATUS_PENDING)
//     {
//         FreePool(LinkTarget.Buffer);
//         FreePool(SourceDeviceName.Buffer);
//         FreePool(SourceSymbolicName.Buffer);
//         return STATUS_SUCCESS;
//     }
//     else if (!NT_SUCCESS(Status))
//     {
//         FreePool(LinkTarget.Buffer);
//         return Status;
//     }
//
//     /* Query the device information */
//     Status = FindDeviceInfo(DeviceExtension, &SourceDeviceName, FALSE, &DeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         /* If it failed, first try to get volume name */
//         Status = QueryVolumeName(0, NULL, &SourceDeviceName, &LinkTarget, &VolumeName);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Then, try to read the symlink */
//             Status = MountMgrQuerySymbolicLink(&SourceDeviceName, &LinkTarget);
//             if (!NT_SUCCESS(Status))
//             {
//                 FreePool(LinkTarget.Buffer);
//                 FreePool(SourceDeviceName.Buffer);
//                 FreePool(SourceSymbolicName.Buffer);
//                 return Status;
//             }
//         }
//         else
//         {
//             FreePool(VolumeName.Buffer);
//         }
//
//         FreePool(SourceDeviceName.Buffer);
//
//         SourceDeviceName.Length = LinkTarget.Length;
//         SourceDeviceName.MaximumLength = LinkTarget.MaximumLength;
//         SourceDeviceName.Buffer = LinkTarget.Buffer;
//
//         /* Now that we have the correct source, reattempt to query information */
//         Status = FindDeviceInfo(DeviceExtension, &SourceDeviceName, FALSE, &DeviceInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(SourceDeviceName.Buffer);
//             FreePool(SourceSymbolicName.Buffer);
//             return Status;
//         }
//     }
//
//     FreePool(SourceDeviceName.Buffer);
//
//     /* Get information about target device */
//     Status = FindDeviceInfo(DeviceExtension, &TargetVolumeName, FALSE, &TargetDeviceInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return Status;
//     }
//
//     /* Open the remote database */
//     RemoteDatabase = OpenRemoteDatabase(DeviceInformation, TRUE);
//     if (RemoteDatabase == 0)
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Browse all the entries */
//     Offset = 0;
//     for (;;)
//     {
//         DatabaseEntry = GetRemoteDatabaseEntry(RemoteDatabase, Offset);
//         if (DatabaseEntry == NULL)
//         {
//             /* We didn't find ourselves, that's infortunate! */
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Try to find ourselves */
//         DbName.MaximumLength = DatabaseEntry->SymbolicNameLength;
//         DbName.Length = DbName.MaximumLength;
//         DbName.Buffer = (PWSTR)((ULONG_PTR)DatabaseEntry + DatabaseEntry->SymbolicNameOffset);
//         if (RtlEqualUnicodeString(&TargetVolumeName, &DbName, TRUE))
//         {
//             break;
//         }
//
//         Offset += DatabaseEntry->EntrySize;
//         FreePool(DatabaseEntry);
//     }
//
//     /* Dereference ourselves */
//     DatabaseEntry->EntryReferences--;
//     if (DatabaseEntry->EntryReferences == 0)
//     {
//         /* If we're still referenced, just update the entry */
//         Status = WriteRemoteDatabaseEntry(RemoteDatabase, Offset, DatabaseEntry);
//     }
//     else
//     {
//         /* Otherwise, delete the entry */
//         Status = DeleteRemoteDatabaseEntry(RemoteDatabase, Offset);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(DatabaseEntry);
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return Status;
//         }
//
//         /* Also, delete our unique ID replicated record */
//         for (Entry = DeviceInformation->ReplicatedUniqueIdsListHead.Flink;
//              Entry != &DeviceInformation->ReplicatedUniqueIdsListHead;
//              Entry = Entry->Flink)
//         {
//             UniqueIdReplicate = CONTAINING_RECORD(Entry, UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry);
//
//             if (UniqueIdReplicate->UniqueId->UniqueIdLength == DatabaseEntry->UniqueIdLength &&
//                 RtlCompareMemory(UniqueIdReplicate->UniqueId->UniqueId,
//                                  (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                                  DatabaseEntry->UniqueIdLength) == DatabaseEntry->UniqueIdLength)
//             {
//                 break;
//             }
//         }
//
//         /* It has to exist! */
//         if (Entry == &DeviceInformation->ReplicatedUniqueIdsListHead)
//         {
//             FreePool(DatabaseEntry);
//             FreePool(SourceSymbolicName.Buffer);
//             CloseRemoteDatabase(RemoteDatabase);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* Remove it and free it */
//         RemoveEntryList(&UniqueIdReplicate->ReplicatedUniqueIdsListEntry);
//         FreePool(UniqueIdReplicate->UniqueId);
//         FreePool(UniqueIdReplicate);
//     }
//
//     /* We're done with the remote database */
//     FreePool(DatabaseEntry);
//     CloseRemoteDatabase(RemoteDatabase);
//
//     /* Check write operation succeed */
//     if (!NT_SUCCESS(Status))
//     {
//         FreePool(SourceSymbolicName.Buffer);
//         return Status;
//     }
//
//     /* Try to find our associated device entry */
//     for (Entry = TargetDeviceInformation->AssociatedDevicesHead.Flink;
//          Entry != &TargetDeviceInformation->AssociatedDevicesHead;
//          Entry = Entry->Flink)
//     {
//         AssociatedEntry = CONTAINING_RECORD(Entry, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//
//         /* If found, delete it */
//         if (AssociatedEntry->DeviceInformation == DeviceInformation &&
//             RtlEqualUnicodeString(&AssociatedEntry->String, &SourceSymbolicName, TRUE))
//         {
//             RemoveEntryList(&AssociatedEntry->AssociatedDevicesEntry);
//             FreePool(AssociatedEntry->String.Buffer);
//             FreePool(AssociatedEntry);
//             break;
//         }
//     }
//
//     /* We're done! */
//     FreePool(SourceSymbolicName.Buffer);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI

// NTAPI cross-module decls — extern "system" to match the defs (i386 stdcall).

// NTAPI exports moved to extern "system" (i386 stdcall).
unsafe extern "system" {
    fn ExFreePoolWithTag(p: *mut core::ffi::c_void, tag: u32);
    fn RtlAssert(condition: *const core::ffi::c_void, file: *const core::ffi::c_void, line: u32, msg: *mut i8);
}

// CRT cdecl moved to extern "C".

// driver-internal cdecl decls (types_*.rs authority = extern "C").

// ddk_shims-provided fns — extern "system" (match the shim def).
unsafe extern "system" {
    fn AllocatePool(Size: u32) -> *mut core::ffi::c_void;
    fn FreePool(Buffer: *mut core::ffi::c_void);
}

// ddk_shims-provided cdecl fns.
unsafe extern "C" {
    fn InsertTailList( ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY, );
}
unsafe extern "C" {
    fn MountMgrMountedDeviceArrival( DeviceExtension: *mut DEVICE_EXTENSION, SymbolicName: *mut _UNICODE_STRING, ManuallyRegistered: BOOLEAN, ) -> NTSTATUS;
    fn MountMgrFreeDeadDeviceInfo( DeviceInformation: *mut DEVICE_INFORMATION, );
    fn HasDriveLetter(DeviceInformation: *mut DEVICE_INFORMATION) -> u8;
    fn FindDeviceInfo( DeviceExtension: PDEVICE_EXTENSION, Name: PUNICODE_STRING, flag: BOOLEAN, Info: *mut PDEVICE_INFORMATION, ) -> NTSTATUS;
    fn ReconcileThisDatabaseWithMaster(DeviceExtension: PDEVICE_EXTENSION, DeviceInformation: PDEVICE_INFORMATION);
    fn QueryVolumeName( hFile: HANDLE, ReparsePoint: *mut core::ffi::c_void, SourceDeviceName: *mut _UNICODE_STRING, LinkTarget: *mut _UNICODE_STRING, VolumeName: *mut _UNICODE_STRING, ) -> NTSTATUS;
    fn MountMgrQuerySymbolicLink( SourceDeviceName: *mut _UNICODE_STRING, LinkTarget: *mut _UNICODE_STRING, ) -> NTSTATUS;
    fn PostOnlineNotification( DeviceExtension: PDEVICE_EXTENSION, SymbolicName: *mut _UNICODE_STRING, );
    fn OpenRemoteDatabase( DeviceInformation: PDEVICE_INFORMATION, Create: BOOLEAN, ) -> HANDLE;
    fn GetRemoteDatabaseEntry( RemoteDatabase: HANDLE, Offset: LONG, ) -> PDATABASE_ENTRY;
    fn WriteRemoteDatabaseEntry( RemoteDatabase: HANDLE, Offset: LONG, DatabaseEntry: PDATABASE_ENTRY, ) -> NTSTATUS;
    fn AddRemoteDatabaseEntry( RemoteDatabase: HANDLE, DatabaseEntry: PDATABASE_ENTRY, ) -> NTSTATUS;
    fn CloseRemoteDatabase(RemoteDatabase: HANDLE) -> u32;
    fn DeleteRemoteDatabaseEntry(Handle: HANDLE, Offset: i32) -> NTSTATUS;
    fn QueryDeviceInformation( DeviceName: PUNICODE_STRING, TargetDeviceName: *mut _UNICODE_STRING, UniqueId: *mut PMOUNTDEV_UNIQUE_ID, Removable: *mut BOOLEAN, GptDriveLetter: *mut BOOLEAN, param5: *mut u8, param6: *mut GUID, param7: *mut u8, ) -> NTSTATUS;
    fn IsDriveLetter(Name: *mut _UNICODE_STRING) -> BOOLEAN;
    fn HasNoDriveLetterEntry(UniqueId: PMOUNTDEV_UNIQUE_ID) -> BOOLEAN;
    fn CreateNoDriveLetterEntry(UniqueId: PMOUNTDEV_UNIQUE_ID);
    fn DeleteNoDriveLetterEntry(UniqueId: *mut MOUNTDEV_UNIQUE_ID);
    fn GlobalDeleteSymbolicLink(SymbolicName: *mut _UNICODE_STRING) -> u32;
    fn MountMgrNotify(DeviceExtension: PDEVICE_EXTENSION);
    fn MountMgrNotifyNameChange(DeviceExtension: PDEVICE_EXTENSION, DeviceName: *mut _UNICODE_STRING, b: u8);
    fn ReleaseRemoteDatabaseSemaphore(DeviceExtension: PDEVICE_EXTENSION);
    fn WaitForRemoteDatabaseSemaphore(DeviceExtension: PDEVICE_EXTENSION) -> NTSTATUS;
}
unsafe extern "C" {
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}
unsafe extern "system" {
}
// MountMgrDeviceControl(IN PDEVICE_OBJECT DeviceObject,
//                       IN PIRP Irp)
// {
//     PIO_STACK_LOCATION Stack;
//     NTSTATUS Status, LockStatus;
//     PDEVICE_EXTENSION DeviceExtension;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//     switch (Stack->Parameters.DeviceIoControl.IoControlCode)
//     {
//         case IOCTL_MOUNTMGR_CREATE_POINT:
//             Status = MountMgrCreatePoint(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_DELETE_POINTS:
//             Status = MountMgrDeletePoints(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_QUERY_POINTS:
//             Status = MountMgrQueryPoints(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_DELETE_POINTS_DBONLY:
//             Status = MountMgrDeletePointsDbOnly(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER:
//             Status = MountMgrNextDriveLetter(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_AUTO_DL_ASSIGNMENTS:
//         // NOTE: On Win7+, this is handled during driver re-initialization.
//             DeviceExtension->AutomaticDriveLetter = TRUE;
//             MountMgrAssignDriveLetters(DeviceExtension);
//             ReconcileAllDatabasesWithMaster(DeviceExtension);
//             WaitForOnlinesToComplete(DeviceExtension);
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IOCTL_MOUNTMGR_VOLUME_MOUNT_POINT_CREATED:
//             KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//             LockStatus = WaitForRemoteDatabaseSemaphore(DeviceExtension);
//             KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//             Status = MountMgrVolumeMountPointCreated(DeviceExtension, Irp, LockStatus);
//             if (NT_SUCCESS(LockStatus))
//             {
//                 ReleaseRemoteDatabaseSemaphore(DeviceExtension);
//             }
//
//             break;
//
//         case IOCTL_MOUNTMGR_VOLUME_MOUNT_POINT_DELETED:
//             KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//             LockStatus = WaitForRemoteDatabaseSemaphore(DeviceExtension);
//             KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//             Status = MountMgrVolumeMountPointDeleted(DeviceExtension, Irp, LockStatus);
//             if (NT_SUCCESS(LockStatus))
//             {
//                 ReleaseRemoteDatabaseSemaphore(DeviceExtension);
//             }
//
//             break;
//
//         case IOCTL_MOUNTMGR_CHANGE_NOTIFY:
//             Status = MountMgrChangeNotify(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_KEEP_LINKS_WHEN_OFFLINE:
//             Status = MountMgrKeepLinksWhenOffline(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_CHECK_UNPROCESSED_VOLUMES:
//             Status = MountMgrCheckUnprocessedVolumes(DeviceExtension, Irp);
//             goto Complete;
//
//         case IOCTL_MOUNTMGR_VOLUME_ARRIVAL_NOTIFICATION:
//             KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//             Status = MountMgrVolumeArrivalNotification(DeviceExtension, Irp);
//             goto Complete;
//
//         case IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH:
//             Status = MountMgrQueryDosVolumePath(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS:
//             Status = MountMgrQueryDosVolumePaths(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_SCRUB_REGISTRY:
//             Status = MountMgrScrubRegistry(DeviceExtension);
//             break;
//
//         case IOCTL_MOUNTMGR_QUERY_AUTO_MOUNT:
//             Status = MountMgrQueryAutoMount(DeviceExtension, Irp);
//             break;
//
//         case IOCTL_MOUNTMGR_SET_AUTO_MOUNT:
//             Status = MountMgrSetAutoMount(DeviceExtension, Irp);
//             break;
//
//         default:
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//     if (Status != STATUS_PENDING)
//     {
//         goto Complete;
//     }
//
//     return Status;
//
// Complete:
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
