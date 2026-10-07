use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use crate::glue::*;
use crate::glue::{IRP_MN_EJECT, IRP_MN_QUERY_BUS_INFORMATION, IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_DEVICE_TEXT, IRP_MN_QUERY_ID, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_RESOURCE_REQUIREMENTS}; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
use crate::types::*;
// Explicit import shadows the `globals::*`/`glue::*` globs so bare `KeLoaderBlock`
// resolves to glue's raw-dylib data import (i386/arm64 cross-DLL data-import thunk).
use crate::glue::KeLoaderBlock;
use ntkrnl_sys::ddk::{DEVICE_CAPABILITIES, DEVICE_RELATIONS, DISK_GEOMETRY, ExAcquireFastMutex, ExFreePool, ExReleaseFastMutex, _KWAIT_REASON_Executive, GET_LENGTH_INFORMATION, GUID, IoAllocateWorkItem, IoAttachDeviceToDeviceStack, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoDetachDevice, IoFreeWorkItem, IoGetDeviceProperty, IoInvalidateDeviceRelations, IoQueueWorkItem, IoRegisterDeviceInterface, IoReportDetectedDevice, IoSetDeviceInterfaceState, KEVENT, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, KeWaitForSingleObject, LARGE_INTEGER, MmMapIoSpace, MmMapLockedPagesSpecifyCache, MmUnmapIoSpace, _EVENT_TYPE_NotificationEvent, PARTITION_INFORMATION, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDISK_GEOMETRY, PDRIVER_OBJECT, PGET_LENGTH_INFORMATION, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIO_WORKITEM, PIO_WORKITEM_ROUTINE, PIRP, PKEVENT, PMDL, PPARTITION_INFORMATION, PPNP_BUS_INFORMATION, PUNICODE_STRING, RTL_QUERY_REGISTRY_TABLE, RtlFreeUnicodeString, RtlStringFromGUID};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoCopyCurrentIrpStackLocationToNext, IoMarkIrpPending, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation};
use core::ffi::c_void;

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


// NTAPI (stdcall on i386 -> _Name@N; cdecl on amd64/arm64). MUST be extern "system",
// not "C": a cdecl decl links on amd64/arm64 but emits the undecorated `_Rtl...` on i386.
extern "system" {
    fn RtlQueryRegistryValues( RelativeTo: u32, Path: PWSTR, QueryTable: *mut RTL_QUERY_REGISTRY_TABLE, Context: *mut core::ffi::c_void, Environment: *mut core::ffi::c_void, ) -> i32;
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: PWSTR);
    fn RtlCopyUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PUNICODE_STRING);
}
// C runtime helpers exported cdecl by ntoskrnl (no @N decoration on i386).
extern "C" {
    fn wcsncpy(dest: PWSTR, src: *const u16, count: usize) -> PWSTR;
    fn wcsncat(dest: PWSTR, src: *const u16, count: usize) -> PWSTR;
    fn _snwprintf(buffer: PWSTR, count: usize, format: PCWSTR, ...) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn _strnicmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
}

// ---- file-scope preamble (translated) ----
pub const DO_XIP: u32 = 0x00020000;

pub const TOC_DATA_TRACK: u32 = 0x04;

pub const RamdiskBus: RAMDISK_DEVICE_TYPE = 0;
pub const RamdiskDrive: RAMDISK_DEVICE_TYPE = 1;
pub type RAMDISK_DEVICE_TYPE = i32;

pub const RamdiskStateUninitialized: RAMDISK_DEVICE_STATE = 0;
pub const RamdiskStateStarted: RAMDISK_DEVICE_STATE = 1;
pub const RamdiskStatePaused: RAMDISK_DEVICE_STATE = 2;
pub const RamdiskStateStopped: RAMDISK_DEVICE_STATE = 3;
pub const RamdiskStateRemoved: RAMDISK_DEVICE_STATE = 4;
pub const RamdiskStateBusRemoved: RAMDISK_DEVICE_STATE = 5;
pub const RamdiskStateEnumerated: RAMDISK_DEVICE_STATE = 6;
pub type RAMDISK_DEVICE_STATE = i32;

pub static RamdiskBusInterface: GUID = GUID {
    Data1: 0x5DC52DF0,
    Data2: 0x2F8A,
    Data3: 0x410F,
    Data4: [0x80, 0xE4, 0x05, 0xF8, 0x10, 0xE7, 0xA8, 0x8A],
};

pub static RamdiskDiskInterface: GUID = GUID {
    Data1: 0x31D909F0,
    Data2: 0x2CDF,
    Data3: 0x4A20,
    Data4: [0x9E, 0xD4, 0x7D, 0x65, 0x47, 0x6C, 0xA7, 0x68],
};

#[repr(C)]
pub struct _RAMDISK_EXTENSION {
    pub Type: RAMDISK_DEVICE_TYPE,
    pub State: RAMDISK_DEVICE_STATE,
    pub DeviceObject: PDEVICE_OBJECT,
    pub PhysicalDeviceObject: PDEVICE_OBJECT,
    pub AttachedDevice: PDEVICE_OBJECT,
    pub RemoveLock: IO_REMOVE_LOCK,
    pub DriveDeviceName: _UNICODE_STRING,
    pub BusDeviceName: _UNICODE_STRING,
    pub DiskListLock: FAST_MUTEX,
    pub DiskList: LIST_ENTRY,
}
pub type RAMDISK_EXTENSION = _RAMDISK_EXTENSION;
pub type PRAMDISK_EXTENSION = *mut _RAMDISK_EXTENSION;

#[repr(C)]
pub struct _RAMDISK_BUS_EXTENSION {
    pub Type: RAMDISK_DEVICE_TYPE,
    pub State: RAMDISK_DEVICE_STATE,
    pub DeviceObject: PDEVICE_OBJECT,
    pub PhysicalDeviceObject: PDEVICE_OBJECT,
    pub AttachedDevice: PDEVICE_OBJECT,
    pub RemoveLock: IO_REMOVE_LOCK,
    pub DriveDeviceName: _UNICODE_STRING,
    pub BusDeviceName: _UNICODE_STRING,
    pub DiskListLock: FAST_MUTEX,
    pub DiskList: LIST_ENTRY,
}
pub type RAMDISK_BUS_EXTENSION = _RAMDISK_BUS_EXTENSION;
pub type PRAMDISK_BUS_EXTENSION = *mut _RAMDISK_BUS_EXTENSION;

#[repr(C)]
pub struct _RAMDISK_DRIVE_EXTENSION {
    pub Type: RAMDISK_DEVICE_TYPE,
    pub State: RAMDISK_DEVICE_STATE,
    pub DeviceObject: PDEVICE_OBJECT,
    pub PhysicalDeviceObject: PDEVICE_OBJECT,
    pub AttachedDevice: PDEVICE_OBJECT,
    pub RemoveLock: IO_REMOVE_LOCK,
    pub DriveDeviceName: _UNICODE_STRING,
    pub BusDeviceName: _UNICODE_STRING,
    pub DiskListLock: FAST_MUTEX,
    pub DiskList: LIST_ENTRY,
    pub DiskGuid: GUID,
    pub GuidString: _UNICODE_STRING,
    pub SymbolicLinkName: _UNICODE_STRING,
    pub DiskType: ULONG,
    pub DiskOptions: RAMDISK_CREATE_OPTIONS,
    pub DiskLength: LARGE_INTEGER,
    pub DiskOffset: LONG,
    pub DriveLetter: WCHAR,
    pub BasePage: ULONG,
    pub BytesPerSector: ULONG,
    pub SectorsPerTrack: ULONG,
    pub NumberOfHeads: ULONG,
    pub Cylinders: ULONG,
    pub HiddenSectors: ULONG,
}
pub type RAMDISK_DRIVE_EXTENSION = _RAMDISK_DRIVE_EXTENSION;
pub type PRAMDISK_DRIVE_EXTENSION = *mut _RAMDISK_DRIVE_EXTENSION;

pub static mut MaximumViewLength: ULONG = 0;
pub static mut MaximumPerDiskViewLength: ULONG = 0;
pub static mut ReportDetectedDevice: ULONG = 0;
pub static mut MarkRamdisksAsRemovable: ULONG = 0;
pub static mut MinimumViewCount: ULONG = 0;
pub static mut DefaultViewCount: ULONG = 0;
pub static mut MaximumViewCount: ULONG = 0;
pub static mut MinimumViewLength: ULONG = 0;
pub static mut DefaultViewLength: ULONG = 0;
pub static mut DriverRegistryPath: _UNICODE_STRING = unsafe { core::mem::zeroed() };
pub static mut ExportBootDiskAsCd: BOOLEAN = 0;
pub static mut IsWinPEBoot: BOOLEAN = 0;
pub static mut RamdiskBusFdo: PDEVICE_OBJECT = core::ptr::null_mut();

const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;

const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn QueryParameters(RegistryPath: PUNICODE_STRING) {

    let mut MinView: ULONG;
    let mut DefView: ULONG;
    let mut MinViewLength: ULONG;
    let mut DefViewLength: ULONG;
    let mut MaxViewLength: ULONG;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 11] = core::mem::zeroed();

    /* Set defaults */
    MaximumViewLength = 0x10000000u32;
    MaximumPerDiskViewLength = 0x10000000u32;
    ReportDetectedDevice = 0;
    MarkRamdisksAsRemovable = 0;
    MinimumViewCount = 2;
    DefaultViewCount = 16;
    MaximumViewCount = 64;
    MinimumViewLength = 0x10000u32;
    DefaultViewLength = 0x100000u32;

    /* Setup the query table and query the registry */
    RtlZeroMemory(
        QueryTable.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 11]>() as _,
    );
    QueryTable[0].Flags = 1;
    QueryTable[0].Name = crate::wstr!("Parameters") as PWSTR;
    QueryTable[1].Flags = 32;
    QueryTable[1].Name = crate::wstr!("ReportDetectedDevice") as PWSTR;
    QueryTable[1].EntryContext = core::ptr::addr_of_mut!(ReportDetectedDevice) as *mut core::ffi::c_void;
    QueryTable[2].Flags = 32;
    QueryTable[2].Name = crate::wstr!("MarkRamdisksAsRemovable") as PWSTR;
    QueryTable[2].EntryContext = core::ptr::addr_of_mut!(MarkRamdisksAsRemovable) as *mut core::ffi::c_void;
    QueryTable[3].Flags = 32;
    QueryTable[3].Name = crate::wstr!("MinimumViewCount") as PWSTR;
    QueryTable[3].EntryContext = core::ptr::addr_of_mut!(MinimumViewCount) as *mut core::ffi::c_void;
    QueryTable[4].Flags = 32;
    QueryTable[4].Name = crate::wstr!("DefaultViewCount") as PWSTR;
    QueryTable[4].EntryContext = core::ptr::addr_of_mut!(DefaultViewCount) as *mut core::ffi::c_void;
    QueryTable[5].Flags = 32;
    QueryTable[5].Name = crate::wstr!("MaximumViewCount") as PWSTR;
    QueryTable[5].EntryContext = core::ptr::addr_of_mut!(MaximumViewCount) as *mut core::ffi::c_void;
    QueryTable[6].Flags = 32;
    QueryTable[6].Name = crate::wstr!("MinimumViewLength") as PWSTR;
    QueryTable[6].EntryContext = core::ptr::addr_of_mut!(MinimumViewLength) as *mut core::ffi::c_void;
    QueryTable[7].Flags = 32;
    QueryTable[7].Name = crate::wstr!("DefaultViewLength") as PWSTR;
    QueryTable[7].EntryContext = core::ptr::addr_of_mut!(DefaultViewLength) as *mut core::ffi::c_void;
    QueryTable[8].Flags = 32;
    QueryTable[8].Name = crate::wstr!("MaximumViewLength") as PWSTR;
    QueryTable[8].EntryContext = core::ptr::addr_of_mut!(MaximumViewLength) as *mut core::ffi::c_void;
    QueryTable[9].Flags = 32;
    QueryTable[9].Name = crate::wstr!("MaximumPerDiskViewLength") as PWSTR;
    QueryTable[9].EntryContext = core::ptr::addr_of_mut!(MaximumPerDiskViewLength) as *mut core::ffi::c_void;
    RtlQueryRegistryValues(
        RTL_REGISTRY_OPTIONAL,
        (*RegistryPath).Buffer,
        QueryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    /* Parse minimum view count, cannot be bigger than 256 or smaller than 2 */
    MinView = MinimumViewCount;
    if MinimumViewCount >= 2 {
        if MinimumViewCount > 256 {
            MinView = 256;
        }
    } else {
        MinView = 2;
    }
    MinimumViewCount = MinView;

    /* Parse default view count, cannot be bigger than 256 or smaller than minimum */
    DefView = DefaultViewCount;
    if DefaultViewCount >= MinView {
        if DefaultViewCount > 256 {
            DefView = 256;
        }
    } else {
        DefView = MinView;
    }
    DefaultViewCount = DefView;

    /* Parse maximum view count, cannot be bigger than 256 or smaller than default */
    if MaximumViewCount >= DefView {
        if MaximumViewCount > 256 {
            MaximumViewCount = 256;
        }
    } else {
        MaximumViewCount = DefView;
    }

    /* Parse minimum view length, cannot be bigger than 1GB or smaller than 64KB */
    MinViewLength = MinimumViewLength;
    if MinimumViewLength >= 0x10000 {
        if MinimumViewLength > 0x40000000 {
            MinViewLength = 0x40000000u32;
        }
    } else {
        MinViewLength = 0x10000u32;
    }
    MinimumViewLength = MinViewLength;

    /* Parse default view length, cannot be bigger than 1GB or smaller than minimum */
    DefViewLength = DefaultViewLength;
    if DefaultViewLength >= MinViewLength {
        if DefaultViewLength > 0x40000000 {
            DefViewLength = 0x40000000u32;
        }
    } else {
        DefViewLength = MinViewLength;
    }
    DefaultViewLength = DefViewLength;

    /* Parse maximum view length, cannot be bigger than 1GB or smaller than default */
    MaxViewLength = MaximumViewLength;
    if MaximumViewLength >= DefViewLength {
        if MaximumViewLength > 0x40000000 {
            MaxViewLength = 0x40000000u32;
        }
    } else {
        MaxViewLength = DefViewLength;
    }
    MaximumViewLength = MaxViewLength;

    /* Parse maximum view length per disk, cannot be smaller than 16MB */
    if MaximumPerDiskViewLength >= 0x1000000 {
        if (MaxViewLength as u64) > 0xFFFFFFFFu64 {
            MaximumPerDiskViewLength = -1i32 as u32;
        }
    } else {
        MaximumPerDiskViewLength = 0x1000000u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskMapPages(
    DeviceExtension: PRAMDISK_DRIVE_EXTENSION,
    Offset: LARGE_INTEGER,
    Length: ULONG,
    OutputLength: PULONG,
) -> PVOID {
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut MappedBase: PVOID;
    let PageOffset: ULONG;
    let mut ActualLength: SIZE_T;
    let mut ActualOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ActualPages: LARGE_INTEGER = core::mem::zeroed();

    /* We only support boot disks for now */
    /* ASSERT(DeviceExtension->DiskType == RAMDISK_BOOT_DISK); */

    /* Calculate the actual offset in the drive */
    ActualOffset.QuadPart = (*DeviceExtension).DiskOffset as i64 + Offset.QuadPart;

    /* Convert to pages */
    ActualPages.QuadPart = ActualOffset.QuadPart >> PAGE_SHIFT;

    /* Now add the base page */
    ActualPages.QuadPart = (*DeviceExtension).BasePage as i64 + ActualPages.QuadPart;

    /* Calculate final amount of bytes */
    PhysicalAddress.QuadPart = ActualPages.QuadPart << PAGE_SHIFT;

    /* Calculate pages spanned for the mapping */
    ActualLength = ADDRESS_AND_SIZE_TO_SPAN_PAGES(ActualOffset.QuadPart as PVOID, Length as SIZE_T) as SIZE_T;

    /* And convert this back to bytes */
    ActualLength <<= PAGE_SHIFT;

    /* Get the offset within the page */
    PageOffset = BYTE_OFFSET(ActualOffset.QuadPart);

    /* Map the I/O Space from the loader */
    MappedBase = MmMapIoSpace(PhysicalAddress, ActualLength as _, MmCached);

    /* Return actual offset within the page as well as the length */
    if !MappedBase.is_null() {
        MappedBase = (MappedBase as ULONG_PTR + PageOffset as ULONG_PTR) as PVOID;
    }
    *OutputLength = Length;
    return MappedBase;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskUnmapPages(
    DeviceExtension: PRAMDISK_DRIVE_EXTENSION,
    mut BaseAddress: PVOID,
    Offset: LARGE_INTEGER,
    Length: ULONG,
) {
    let mut ActualOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ActualLength: SIZE_T;
    let PageOffset: ULONG;

    /* We only support boot disks for now */
    /* ASSERT(DeviceExtension->DiskType == RAMDISK_BOOT_DISK); */

    /* Calculate the actual offset in the drive */
    ActualOffset.QuadPart = (*DeviceExtension).DiskOffset as i64 + Offset.QuadPart;

    /* Calculate pages spanned for the mapping */
    ActualLength = ADDRESS_AND_SIZE_TO_SPAN_PAGES(ActualOffset.QuadPart as PVOID, Length as SIZE_T) as SIZE_T;

    /* And convert this back to bytes */
    ActualLength <<= PAGE_SHIFT;

    /* Get the offset within the page */
    PageOffset = BYTE_OFFSET(ActualOffset.QuadPart as i64);

    /* Calculate actual base address where we mapped this */
    BaseAddress = ((BaseAddress as ULONG_PTR) - PageOffset as ULONG_PTR) as PVOID;

    /* Unmap the I/O space we got from the loader */
    MmUnmapIoSpace(BaseAddress, ActualLength as _);
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskCreateDiskDevice(
    DeviceExtension: PRAMDISK_BUS_EXTENSION,
    Input: PRAMDISK_CREATE_INPUT,
    ValidateOnly: BOOLEAN,
    NewDriveExtension: *mut PRAMDISK_DRIVE_EXTENSION,
) -> NTSTATUS {

    const RAMDISK_BOOT_DISK: ULONG = 3; // ntddrdsk.h RAMDISK_BOOT_DISK
    const RAMDISK_WIM_DISK: ULONG = 4; // ntddrdsk.h RAMDISK_WIM_DISK
    const DO_XIP: ULONG = 0x00020000;

    let BasePage: ULONG;
    let DiskType: ULONG;
    let Length: ULONG;
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let DriveExtension: PRAMDISK_DRIVE_EXTENSION;
    let mut Buffer: PVOID;
    let mut LocalBuffer: [WCHAR; 16] = [0; 16];
    let mut SymbolicLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut DriveString: _UNICODE_STRING = core::mem::zeroed();
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let BootSector: PPACKED_BOOT_SECTOR;
    let mut BiosBlock: BIOS_PARAMETER_BLOCK = core::mem::zeroed();
    let BytesPerSector: ULONG;
    let SectorsPerTrack: ULONG;
    let Heads: ULONG;
    let mut BytesRead: ULONG = 0;
    let BaseAddress: PVOID;
    let mut CurrentOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut CylinderSize: LARGE_INTEGER = core::mem::zeroed();
    let DiskLength: LARGE_INTEGER;
    let CylinderCount: ULONG;
    let SizeByCylinders: ULONG;

    let _ = DeviceObject;

    let mut _device_object: PDEVICE_OBJECT = core::ptr::null_mut();

    'FailCreate: loop {
        /* Check if we're a boot RAM disk */
        DiskType = (*Input).DiskType;
        if DiskType >= RAMDISK_BOOT_DISK {
            /* Check if we're an ISO */
            if DiskType == RAMDISK_BOOT_DISK {
                /* NTLDR mounted us somewhere */
                BasePage = (*Input).__bindgen_anon_1.__bindgen_anon_2.as_ref().BasePage as _;
                if BasePage == 0 {
                    return STATUS_INVALID_PARAMETER;
                }

                /* Sanitize disk options */
                (*Input).Options.set_Fixed(1);
                (*Input).Options.set_Readonly(
                    (*Input).Options.ExportAsCd() | (*Input).Options.Readonly(),
                );
                (*Input).Options.set_Hidden(0);
                (*Input).Options.set_NoDosDevice(0);
                (*Input).Options.set_NoDriveLetter(if IsWinPEBoot != 0 { 1 } else { 0 });
            } else {
                /* The only other possibility is a WIM disk */
                if DiskType != RAMDISK_WIM_DISK {
                    /* Fail */
                    return STATUS_INVALID_PARAMETER;
                }

                /* Read the view count instead */
                // ViewCount = Input->ViewCount;

                /* Sanitize disk options */
                (*Input).Options.set_Hidden(0);
                (*Input).Options.set_NoDosDevice(0);
                (*Input).Options.set_Readonly(0);
                (*Input).Options.set_NoDriveLetter(1);
                (*Input).Options.set_Fixed(1);
            }

            /* Are we just validating and returning to the user? */
            if ValidateOnly != 0 {
                return STATUS_SUCCESS;
            }

            /* Build the GUID string */
            Status = RtlStringFromGUID(core::ptr::addr_of_mut!((*Input).DiskGuid), core::ptr::addr_of_mut!(GuidString));
            if !NT_SUCCESS(Status) || GuidString.Buffer.is_null() {
                /* Fail */
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'FailCreate;
            }

            /* Allocate our device name */
            Length = (GuidString.Length as ULONG) + 32;
            Buffer = ExAllocatePoolWithTag(NonPagedPool, Length as usize, 0x646D6152);
            if Buffer.is_null() {
                /* Fail */
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'FailCreate;
            }

            /* Build the device name string */
            DeviceName.Buffer = Buffer as PWSTR;
            DeviceName.Length = (Length - 2) as u16;
            DeviceName.MaximumLength = Length as u16;
            wcsncpy(
                Buffer as PWSTR,
                crate::wstr!("\\Device\\Ramdisk"),
                (Length as usize) / core::mem::size_of::<WCHAR>(),
            );
            wcsncat(
                Buffer as PWSTR,
                GuidString.Buffer,
                (Length as usize) / core::mem::size_of::<WCHAR>(),
            );

            /* Create the drive device */
            Status = IoCreateDevice(
                (*(*DeviceExtension).DeviceObject).DriverObject,
                core::mem::size_of::<RAMDISK_DRIVE_EXTENSION>() as ULONG,
                core::ptr::addr_of_mut!(DeviceName),
                if (*Input).Options.ExportAsCd() != 0 {
                    FILE_DEVICE_CD_ROM
                } else {
                    FILE_DEVICE_DISK
                },
                0,
                0,
                core::ptr::addr_of_mut!(_device_object),
            );
            DeviceObject = _device_object;
            if !NT_SUCCESS(Status) {
                break 'FailCreate;
            }

            /* Grab the drive extension */
            DriveExtension = (*DeviceObject).DeviceExtension as PRAMDISK_DRIVE_EXTENSION;

            /* Check if we need a DOS device */
            if (*Input).Options.NoDosDevice() == 0 {
                /* Build the symbolic link name */
                SymbolicLinkName.MaximumLength = (GuidString.Length + 36) as u16;
                SymbolicLinkName.Length = (GuidString.Length + 34) as u16;
                Buffer = ExAllocatePoolWithTag(
                    NonPagedPool,
                    SymbolicLinkName.MaximumLength as usize,
                    0x646D6152,
                );
                SymbolicLinkName.Buffer = Buffer as PWSTR;
                if !Buffer.is_null() {
                    /* Create it */
                    wcsncpy(
                        Buffer as PWSTR,
                        crate::wstr!("\\GLOBAL??\\Ramdisk"),
                        (SymbolicLinkName.MaximumLength as usize) / core::mem::size_of::<WCHAR>(),
                    );
                    wcsncat(
                        Buffer as PWSTR,
                        GuidString.Buffer,
                        (SymbolicLinkName.MaximumLength as usize) / core::mem::size_of::<WCHAR>(),
                    );
                    Status = IoCreateSymbolicLink(
                        core::ptr::addr_of_mut!(SymbolicLinkName),
                        core::ptr::addr_of_mut!(DeviceName),
                    );
                    if !NT_SUCCESS(Status) {
                        /* Nevermind... */
                        (*Input).Options.set_NoDosDevice(1);
                        ExFreePool(Buffer);
                        SymbolicLinkName.Buffer = core::ptr::null_mut();
                    }
                } else {
                    /* No DOS device */
                    (*Input).Options.set_NoDosDevice(1);
                }

                /* Is this an ISO boot ramdisk? */
                if (*Input).DiskType == RAMDISK_BOOT_DISK {
                    /* Does it need a drive letter? */
                    if (*Input).Options.NoDriveLetter() == 0 {
                        /* Build it and take over the existing symbolic link */
                        _snwprintf(
                            LocalBuffer.as_mut_ptr(),
                            30,
                            crate::wstr!("\\DosDevices\\%wc:"),
                            (*Input).__bindgen_anon_1.__bindgen_anon_2.as_ref().DriveLetter as i32,
                        );
                        RtlInitUnicodeString(core::ptr::addr_of_mut!(DriveString), LocalBuffer.as_ptr() as PWSTR);
                        IoDeleteSymbolicLink(core::ptr::addr_of_mut!(DriveString));
                        IoCreateSymbolicLink(
                            core::ptr::addr_of_mut!(DriveString),
                            core::ptr::addr_of_mut!(DeviceName),
                        );

                        /* Save the drive letter */
                        (*DriveExtension).DriveLetter = (*Input).__bindgen_anon_1.__bindgen_anon_2.as_ref().DriveLetter;
                    }
                }
            }

            /* Setup the device object flags */
            (*DeviceObject).Flags |= DO_XIP | DO_POWER_PAGABLE | DO_DIRECT_IO;
            (*DeviceObject).AlignmentRequirement = 1;

            /* Build the drive FDO */
            *NewDriveExtension = DriveExtension;
            (*DriveExtension).Type = RamdiskDrive;
            DiskLength = (*Input).DiskLength;
            ExInitializeFastMutex(core::ptr::addr_of_mut!((*DriveExtension).DiskListLock));
            IoInitializeRemoveLock(
                core::ptr::addr_of_mut!((*DriveExtension).RemoveLock) as *mut core::ffi::c_void,
                0x646D6152,
                1,
                0,
            );
            (*DriveExtension).DriveDeviceName = DeviceName;
            (*DriveExtension).SymbolicLinkName = SymbolicLinkName;
            (*DriveExtension).GuidString = GuidString;
            (*DriveExtension).DiskGuid = (*Input).DiskGuid;
            (*DriveExtension).PhysicalDeviceObject = DeviceObject;
            (*DriveExtension).DeviceObject = RamdiskBusFdo;
            (*DriveExtension).AttachedDevice = RamdiskBusFdo;
            (*DriveExtension).DiskType = (*Input).DiskType;
            (*DriveExtension).DiskOptions = core::ptr::read(core::ptr::addr_of!((*Input).Options));
            (*DriveExtension).DiskLength = DiskLength;
            (*DriveExtension).DiskOffset = (*Input).DiskOffset;
            (*DriveExtension).BasePage = (*Input).__bindgen_anon_1.__bindgen_anon_2.as_ref().BasePage as _;
            (*DriveExtension).BytesPerSector = 0;
            (*DriveExtension).SectorsPerTrack = 0;
            (*DriveExtension).NumberOfHeads = 0;

            /* Make sure we don't free it later */
            DeviceName.Buffer = core::ptr::null_mut();
            SymbolicLinkName.Buffer = core::ptr::null_mut();
            GuidString.Buffer = core::ptr::null_mut();

            /* Check if this is a boot disk, or a registry ram drive */
            if (*Input).Options.ExportAsCd() == 0 && (*Input).DiskType == RAMDISK_BOOT_DISK {
                /* Not an ISO boot, but it's a boot FS -- map it to figure out the
                 * drive settings */
                CurrentOffset.QuadPart = 0;
                BaseAddress = RamdiskMapPages(
                    DriveExtension,
                    CurrentOffset,
                    PAGE_SIZE as ULONG,
                    core::ptr::addr_of_mut!(BytesRead),
                );
                if !BaseAddress.is_null() {
                    /* Get the data */
                    BootSector = BaseAddress as PPACKED_BOOT_SECTOR;
                    FatUnpackBios(
                        core::ptr::addr_of_mut!(BiosBlock),
                        core::ptr::addr_of_mut!((*BootSector).PackedBpb),
                    );
                    BytesPerSector = BiosBlock.BytesPerSector as ULONG;
                    SectorsPerTrack = BiosBlock.SectorsPerTrack as ULONG;
                    Heads = BiosBlock.Heads as ULONG;

                    /* Save it */
                    (*DriveExtension).BytesPerSector = BytesPerSector;
                    (*DriveExtension).SectorsPerTrack = SectorsPerTrack;
                    (*DriveExtension).NumberOfHeads = Heads;

                    /* Unmap now */
                    CurrentOffset.QuadPart = 0;
                    RamdiskUnmapPages(DriveExtension, BaseAddress, CurrentOffset, BytesRead);
                } else {
                    /* Fail */
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'FailCreate;
                }
            }

            /* Check if the drive settings haven't been set yet */
            if (*DriveExtension).BytesPerSector == 0
                || (*DriveExtension).SectorsPerTrack == 0
                || (*DriveExtension).NumberOfHeads == 0
            {
                /* Check if this is a CD */
                if (*Input).Options.ExportAsCd() != 0 {
                    /* Setup partition parameters default for ISO 9660 */
                    (*DriveExtension).BytesPerSector = 2048;
                    (*DriveExtension).SectorsPerTrack = 32;
                    (*DriveExtension).NumberOfHeads = 64;
                } else {
                    /* Setup partition parameters default for FAT */
                    (*DriveExtension).BytesPerSector = 512;
                    (*DriveExtension).SectorsPerTrack = 128;
                    (*DriveExtension).NumberOfHeads = 16;
                }
            }

            /* Calculate the cylinder size */
            CylinderSize.QuadPart = ((*DriveExtension).BytesPerSector
                * (*DriveExtension).SectorsPerTrack
                * (*DriveExtension).NumberOfHeads) as i64;
            CylinderCount = (DiskLength.QuadPart / CylinderSize.QuadPart) as ULONG;
            SizeByCylinders = (CylinderSize.QuadPart * CylinderCount as i64) as ULONG;
            (*DriveExtension).Cylinders = CylinderCount;
            if DiskLength.u.HighPart > 0 || SizeByCylinders < DiskLength.u.LowPart {
                /* Align cylinder size up */
                (*DriveExtension).Cylinders += 1;
            }

            /* Acquire the disk lock */
            KeEnterCriticalRegion();
            ExAcquireFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));

            /* Insert us */
            InsertTailList(
                core::ptr::addr_of_mut!((*DeviceExtension).DiskList),
                core::ptr::addr_of_mut!((*DriveExtension).DiskList),
            );

            /* Release the lock */
            ExReleaseFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));
            KeLeaveCriticalRegion();

            /* Clear init flag */
            (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
            return STATUS_SUCCESS;
        }

        break 'FailCreate;
    }

    crate::UNIMPLEMENTED_DBGBREAK!();
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskCreateRamdisk(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    ValidateOnly: BOOLEAN,
) -> NTSTATUS {
    let Input: PRAMDISK_CREATE_INPUT;
    let Length: ULONG;
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;
    let mut DriveExtension: PRAMDISK_DRIVE_EXTENSION = core::ptr::null_mut();
    let DiskType: ULONG;
    let FileNameStart: PWCHAR;
    let mut FileNameEnd: PWCHAR;
    let Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    /* Get the device extension and our input data */
    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;
    Length = (*IoStackLocation).Parameters.DeviceIoControl.InputBufferLength;
    Input = (*Irp).AssociatedIrp.SystemBuffer as PRAMDISK_CREATE_INPUT;

    /* Validate input parameters */
    if (Length < core::mem::size_of::<RAMDISK_CREATE_INPUT>() as ULONG)
        || ((*Input).Version != core::mem::size_of::<RAMDISK_CREATE_INPUT>() as ULONG)
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Validate the disk type */
    DiskType = (*Input).DiskType;
    if DiskType == RAMDISK_WIM_DISK {
        return STATUS_INVALID_PARAMETER;
    }

    /* Look at the disk type */
    if DiskType == RAMDISK_BOOT_DISK {
        /* We only allow this as an early-init boot */
        if KeLoaderBlock.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        /* Save command-line flags */
        if ExportBootDiskAsCd != 0 {
            (*Input).Options.set_ExportAsCd(1);
        }
        if IsWinPEBoot != 0 {
            (*Input).Options.set_NoDriveLetter(1);
        }
    }

    /* Validate the disk type */
    if ((*Input).Options.ExportAsCd() != 0) && (DiskType != RAMDISK_BOOT_DISK) {
        /* If the type isn't CDFS, it has to at least be raw CD */
        if DiskType != RAMDISK_MEMORY_MAPPED_DISK {
            return STATUS_INVALID_PARAMETER;
        }
    }

    /* Check if this is an actual file */
    if DiskType <= RAMDISK_MEMORY_MAPPED_DISK {
        /* Validate the file name */
        FileNameStart = ((Input as ULONG_PTR) + Length as ULONG_PTR) as PWCHAR;
        FileNameEnd = (*Input).__bindgen_anon_1.__bindgen_anon_1.as_ref().FileName.as_ptr().offset(1) as PWCHAR;
        while (FileNameEnd < FileNameStart) && (*FileNameEnd != 0) {
            FileNameEnd = FileNameEnd.offset(1);
        }
        if FileNameEnd == FileNameStart {
            return STATUS_INVALID_PARAMETER;
        }
    }

    /* Create the actual device */
    Status = RamdiskCreateDiskDevice(
        DeviceExtension,
        Input,
        ValidateOnly,
        core::ptr::addr_of_mut!(DriveExtension),
    );
    if NT_SUCCESS(Status) {
        /* Invalidate and set success */
        IoInvalidateDeviceRelations((*DeviceExtension).PhysicalDeviceObject, 0);
        (*Irp).IoStatus.Information = STATUS_SUCCESS as _;
    }

    /* We are done */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskGetPartitionInfo(
    Irp: PIRP,
    DeviceExtension: PRAMDISK_DRIVE_EXTENSION,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let PartitionInfo: PPARTITION_INFORMATION;
    let BaseAddress: PVOID;
    let mut Zero: LARGE_INTEGER = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let IoStackLocation: PIO_STACK_LOCATION;

    /* Validate the length */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    if (*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength
        < core::mem::size_of::<PARTITION_INFORMATION>() as ULONG
    {
        /* Invalid length */
        Status = STATUS_BUFFER_TOO_SMALL;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        return Status;
    }

    /* Map the partition table */
    BaseAddress = RamdiskMapPages(
        DeviceExtension,
        Zero,
        PAGE_SIZE as ULONG,
        core::ptr::addr_of_mut!(Length),
    );
    if BaseAddress.is_null() {
        /* No memory */
        Status = STATUS_INSUFFICIENT_RESOURCES;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        return Status;
    }

    /* Fill out the information */
    PartitionInfo = (*Irp).AssociatedIrp.SystemBuffer as PPARTITION_INFORMATION;
    (*PartitionInfo).StartingOffset.QuadPart = (*DeviceExtension).BytesPerSector as i64;
    (*PartitionInfo).PartitionLength.QuadPart = ((*DeviceExtension).BytesPerSector
        * (*DeviceExtension).SectorsPerTrack
        * (*DeviceExtension).NumberOfHeads
        * (*DeviceExtension).Cylinders) as i64;
    (*PartitionInfo).HiddenSectors = (*DeviceExtension).HiddenSectors;
    (*PartitionInfo).PartitionNumber = 0;
    (*PartitionInfo).PartitionType = *((BaseAddress as *mut CHAR).offset(450)) as u8;
    (*PartitionInfo).BootIndicator = if (*DeviceExtension).DiskType == RAMDISK_BOOT_DISK {
        1
    } else {
        0
    };
    (*PartitionInfo).RecognizedPartition =
        IsRecognizedPartition((*PartitionInfo).PartitionType);
    (*PartitionInfo).RewritePartition = 0;

    /* Unmap the partition table */
    RamdiskUnmapPages(DeviceExtension, BaseAddress, Zero, Length);

    /* Done */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = core::mem::size_of::<PARTITION_INFORMATION>() as _;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskSetPartitionInfo(
    Irp: PIRP,
    DeviceExtension: PRAMDISK_DRIVE_EXTENSION,
) -> NTSTATUS {
    let mut BytesRead: ULONG = 0;
    let Status: NTSTATUS;
    let BaseAddress: PVOID;
    let Stack: PIO_STACK_LOCATION;
    let mut Zero: LARGE_INTEGER = core::mem::zeroed();
    let PartitionInfo: PPARTITION_INFORMATION;

    /* First validate input */
    Stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<PARTITION_INFORMATION>()
    {
        Status = STATUS_INVALID_PARAMETER;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        return Status;
    }

    /* Map to get MBR */
    BaseAddress = RamdiskMapPages(
        DeviceExtension,
        Zero,
        PAGE_SIZE as ULONG,
        core::ptr::addr_of_mut!(BytesRead),
    );
    if BaseAddress.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        return Status;
    }

    /* Set the new partition type on partition 0, field system indicator */
    PartitionInfo = (*Irp).AssociatedIrp.SystemBuffer as PPARTITION_INFORMATION;
    *((BaseAddress as PCHAR).offset(450)) = (*PartitionInfo).PartitionType as i8;

    /* And unmap */
    RamdiskUnmapPages(DeviceExtension, BaseAddress, Zero, BytesRead);
    Status = STATUS_SUCCESS;

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    return Status;
}

const FSCTL_CREATE_RAM_DISK: u32 = 0x240000; // CTL_CODE(FILE_DEVICE_VIRTUAL_DISK=0x24, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
const IOCTL_DISK_SET_PARTITION_INFO: u32 = 0x7C008; // IOCTL_DISK_SET_PARTITION_INFO
const IOCTL_DISK_GET_DRIVE_LAYOUT: u32 = 0x7400c; // CTL_CODE(IOCTL_DISK_BASE=7, 3, METHOD_BUFFERED, FILE_READ_ACCESS)
const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x74004; // IOCTL_DISK_GET_PARTITION_INFO

#[no_mangle]
pub unsafe extern "system" fn SendIrpToThread(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let WorkItem: PIO_WORKITEM;

    /* Mark the IRP pending */
    IoMarkIrpPending(Irp);

    /* Allocate a work item */
    WorkItem = IoAllocateWorkItem(DeviceObject);
    if !WorkItem.is_null() {
        /* Queue it up */
        (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = WorkItem as *mut core::ffi::c_void;
        IoQueueWorkItem(WorkItem, core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PVOID), PIO_WORKITEM_ROUTINE>(RamdiskWorkerThread), DelayedWorkQueue, Irp as *mut core::ffi::c_void);
        return STATUS_PENDING;
    } else {
        /* Fail */
        return STATUS_INSUFFICIENT_RESOURCES;
    }
}

pub unsafe extern "system" fn RamdiskWorkerThread(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    const IRP_MJ_DEVICE_CONTROL: u32 = 0x0e;
    const IRP_MJ_READ: u32 = 0x03;
    const IRP_MJ_WRITE: u32 = 0x04;
    const IRP_MJ_INTERNAL_DEVICE_CONTROL: u32 = 0x0f;
    const IRP_MJ_FLUSH_BUFFERS: u32 = 0x09;
    const FSCTL_CREATE_RAM_DISK: u32 = 0x240000; // CTL_CODE(FILE_DEVICE_VIRTUAL_DISK=0x24, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
    const IOCTL_DISK_SET_PARTITION_INFO: u32 = 0x7c008;
    const IOCTL_DISK_GET_DRIVE_LAYOUT: u32 = 0x7400c; // CTL_CODE(IOCTL_DISK_BASE=7, 3, METHOD_BUFFERED, FILE_READ_ACCESS)
    const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x74004;
    const IO_DISK_INCREMENT: i8 = 1;

    let DeviceExtension: PRAMDISK_BUS_EXTENSION;
    let mut Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION;
    let Irp: PIRP = Context as PIRP;

    /* Get the stack location */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    /* Free the work item */
    IoFreeWorkItem((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as PIO_WORKITEM);

    /* Grab the device extension and lock it */
    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;
    Status = IoAcquireRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );
    if NT_SUCCESS(Status) {
        /* Discriminate by major code */
        match (*IoStackLocation).MajorFunction as u32 {
            /* Device control */
            IRP_MJ_DEVICE_CONTROL => {
                /* Let's take a look at the IOCTL */
                match (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode {
                    /* Ramdisk create request */
                    FSCTL_CREATE_RAM_DISK => {
                        /* This time we'll do it for real */
                        Status = RamdiskCreateRamdisk(DeviceObject, Irp, 0);
                    }

                    IOCTL_DISK_SET_PARTITION_INFO => {
                        Status = RamdiskSetPartitionInfo(
                            Irp,
                            DeviceExtension as PRAMDISK_DRIVE_EXTENSION,
                        );
                    }

                    IOCTL_DISK_GET_DRIVE_LAYOUT => {
                        crate::UNIMPLEMENTED_DBGBREAK!(
                            b"Get drive layout request\n\0".as_ptr() as *const i8
                        );
                    }

                    IOCTL_DISK_GET_PARTITION_INFO => {
                        Status = RamdiskGetPartitionInfo(
                            Irp,
                            DeviceExtension as PRAMDISK_DRIVE_EXTENSION,
                        );
                    }

                    _ => {
                        crate::UNIMPLEMENTED_DBGBREAK!(b"Invalid request\n\0".as_ptr() as *const i8);
                    }
                }
            }

            /* Read or write request */
            IRP_MJ_READ | IRP_MJ_WRITE => {
                crate::UNIMPLEMENTED_DBGBREAK!(b"Read/Write request\n\0".as_ptr() as *const i8);
            }

            /* Internal request (SCSI?) */
            IRP_MJ_INTERNAL_DEVICE_CONTROL => {
                crate::UNIMPLEMENTED_DBGBREAK!(b"SCSI request\n\0".as_ptr() as *const i8);
            }

            /* Flush request */
            IRP_MJ_FLUSH_BUFFERS => {
                crate::UNIMPLEMENTED_DBGBREAK!(b"Flush request\n\0".as_ptr() as *const i8);
            }

            /* Anything else */
            _ => {
                crate::UNIMPLEMENTED_DBGBREAK!(
                    b"Invalid request: %lx\n\0".as_ptr() as *const i8,
                    (*IoStackLocation).MajorFunction as u32
                );
            }
        }

        /* Complete the I/O */
        IoReleaseRemoveLock(
            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
            Irp as PVOID,
        );
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return;
    }

    /* Fail the I/O */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskReadWriteReal(
    Irp: PIRP,
    DeviceExtension: PRAMDISK_DRIVE_EXTENSION,
) -> NTSTATUS {
    let Mdl: PMDL;
    let mut CurrentBase: PVOID;
    let SystemVa: PVOID;
    let mut BaseAddress: PVOID;
    let IoStackLocation: PIO_STACK_LOCATION;
    let mut CurrentOffset: LARGE_INTEGER;
    let mut BytesRead: ULONG = 0;
    let mut BytesLeft: ULONG;
    let mut CopyLength: ULONG;
    let mut Source: PVOID;
    let mut Destination: PVOID;
    let mut Status: NTSTATUS;

    /* Get the MDL and check if it's mapped */
    Mdl = (*Irp).MdlAddress;
    if (*Mdl).MdlFlags as u32 & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
        /* Use the mapped address */
        SystemVa = (*Mdl).MappedSystemVa;
    } else {
        /* Map it ourselves */
        SystemVa = MmMapLockedPagesSpecifyCache(
            Mdl,
            0,
            MmCached,
            core::ptr::null_mut(),
            0,
            NormalPagePriority as u32,
        );
    }

    /* Make sure we were able to map it */
    CurrentBase = SystemVa;
    if SystemVa.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Initialize default */
    (*Irp).IoStatus.Information = 0;

    /* Get the I/O Stack Location and capture the data */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    CurrentOffset = (*IoStackLocation).Parameters.Read.ByteOffset;
    BytesLeft = (*IoStackLocation).Parameters.Read.Length;
    if BytesLeft == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /* Do the copy loop */
    loop {
        /* Map the pages */
        BaseAddress = RamdiskMapPages(
            DeviceExtension,
            CurrentOffset,
            BytesLeft,
            core::ptr::addr_of_mut!(BytesRead),
        );
        if BaseAddress.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        /* Update our lengths */
        (*Irp).IoStatus.Information = (*Irp).IoStatus.Information.wrapping_add(BytesRead as _);
        CopyLength = BytesRead;

        /* Check if this was a read or write */
        Status = STATUS_SUCCESS;
        if (*IoStackLocation).MajorFunction as u32 == (IRP_MJ_READ as u32) {
            /* Set our copy parameters */
            Destination = CurrentBase;
            Source = BaseAddress;
            /* Copy the data */
            RtlCopyMemory(Destination, Source, CopyLength as usize);
        } else if (*IoStackLocation).MajorFunction as u32 == (IRP_MJ_WRITE as u32) {
            /* Set our copy parameters */
            Destination = BaseAddress;
            Source = CurrentBase;
            /* Copy the data */
            RtlCopyMemory(Destination, Source, CopyLength as usize);
        } else {
            /* Prepare us for failure */
            BytesLeft = CopyLength;
            Status = STATUS_INVALID_PARAMETER;
        }

        /* Unmap the pages */
        RamdiskUnmapPages(DeviceExtension, BaseAddress, CurrentOffset, BytesRead);

        /* Update offset and bytes left */
        BytesLeft -= BytesRead;
        CurrentOffset.QuadPart += BytesRead as i64;
        CurrentBase = (CurrentBase as ULONG_PTR + BytesRead as ULONG_PTR) as PVOID;

        /* Check if we are done */
        if BytesLeft == 0 {
            return Status;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskOpenClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // RD_TRACE("[RAMDISK-DRV] OpenClose\r\n");
    /* Complete the IRP */
    (*Irp).IoStatus.Information = 1;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskReadWrite(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PRAMDISK_DRIVE_EXTENSION;
    let IoStackLocation: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut ReturnStatus: NTSTATUS = 0;

    crate::RD_TRACE!(b"[RAMDISK-DRV] ReadWrite\r\n\0".as_ptr() as *const i8);

    /* Get the device extension and make sure this isn't a bus */
    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_DRIVE_EXTENSION;
    if (*DeviceExtension).Type == RamdiskBus {
        /* Fail */
        Status = STATUS_INVALID_DEVICE_REQUEST;
        /* Complete the IRP */
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT as i8);
        ReturnStatus = Status;
        return ReturnStatus;
    }

    /* Capture parameters */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    /* Validate write */
    if (*IoStackLocation).MajorFunction as u32 == (IRP_MJ_WRITE as u32)
        && (*DeviceExtension).DiskOptions.Readonly() != 0
    {
        /* Fail, this is read-only */
        Status = STATUS_MEDIA_WRITE_PROTECTED;
        /* Complete the IRP */
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT as i8);
        ReturnStatus = Status;
        return ReturnStatus;
    }

    /* See if we want to do this sync or async */
    if (*DeviceExtension).DiskType > RAMDISK_MEMORY_MAPPED_DISK {
        /* Do it sync */
        Status = RamdiskReadWriteReal(Irp, DeviceExtension);
        /* Complete the IRP */
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT as i8);
        ReturnStatus = Status;
        return ReturnStatus;
    }

    /* Queue it to the worker */
    Status = SendIrpToThread(DeviceObject, Irp);
    ReturnStatus = STATUS_PENDING;

    /* Check if we're pending or not */
    if Status != STATUS_PENDING {
        /* Complete the IRP */
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT as i8);
        ReturnStatus = Status;
    }

    /* Return to caller */
    ReturnStatus
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const FSCTL_CREATE_RAM_DISK: u32 = 0x240000; // CTL_CODE(FILE_DEVICE_VIRTUAL_DISK=0x24, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
    const IOCTL_DISK_CHECK_VERIFY: u32 = 0x74800; // CTL_CODE(IOCTL_DISK_BASE=7, 0x200, METHOD_BUFFERED, FILE_READ_ACCESS) — was 0x70000, collided with GET_DRIVE_GEOMETRY
    const IOCTL_STORAGE_CHECK_VERIFY: u32 = 0x2d4800;
    const IOCTL_STORAGE_CHECK_VERIFY2: u32 = 0x2d0800;
    const IOCTL_CDROM_CHECK_VERIFY: u32 = 0x24800; // CTL_CODE(IOCTL_CDROM_BASE=2, 0x200, METHOD_BUFFERED, FILE_READ_ACCESS)
    const IOCTL_STORAGE_GET_MEDIA_TYPES: u32 = 0x2d0c00;
    const IOCTL_DISK_GET_MEDIA_TYPES: u32 = 0x70c00;
    const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x70000;
    const IOCTL_CDROM_GET_DRIVE_GEOMETRY: u32 = 0x2404c; // CTL_CODE(IOCTL_CDROM_BASE=2, 0x13, METHOD_BUFFERED, FILE_READ_ACCESS)
    const IOCTL_CDROM_READ_TOC: u32 = 0x24000;
    const IOCTL_DISK_SET_PARTITION_INFO: u32 = 0x7c008;
    const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x74004;
    const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x7405c;
    const IOCTL_VOLUME_GET_GPT_ATTRIBUTES: u32 = 0x560038;
    const IOCTL_DISK_GET_DRIVE_LAYOUT: u32 = 0x7400c; // CTL_CODE(IOCTL_DISK_BASE=7, 3, METHOD_BUFFERED, FILE_READ_ACCESS)
    const IOCTL_DISK_IS_WRITABLE: u32 = 0x70024;
    const IOCTL_SCSI_MINIPORT: u32 = 0x4d008;
    const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x2d1400;
    const IOCTL_MOUNTDEV_QUERY_UNIQUE_ID: u32 = 0x4d0000;
    const IOCTL_MOUNTDEV_QUERY_STABLE_GUID: u32 = 0x4d0008;
    const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x560000;
    const IOCTL_VOLUME_SET_GPT_ATTRIBUTES: u32 = 0x56003c;
    const IOCTL_VOLUME_OFFLINE: u32 = 0x56c008;

    const RemovableMedia: u32 = 11;
    const GPT_BASIC_DATA_ATTRIBUTE_READ_ONLY: u64 = 0x1000000000000000;
    const GPT_BASIC_DATA_ATTRIBUTE_HIDDEN: u64 = 0x4000000000000000;
    const GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER: u64 = 0x8000000000000000;
    const RAMDISK_MEMORY_MAPPED_DISK: u32 = 2; // ntddrdsk.h RAMDISK_MEMORY_MAPPED_DISK

    let RAMDISK_TOC_SIZE: usize =
        core::mem::offset_of!(CDROM_TOC, TrackData) + 2 * core::mem::size_of::<TRACK_DATA>();

    let Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PRAMDISK_BUS_EXTENSION =
        (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;
    let DriveExtension: PRAMDISK_DRIVE_EXTENSION =
        DeviceExtension as PVOID as PRAMDISK_DRIVE_EXTENSION;
    let mut Information: ULONG;
    let Toc: PCDROM_TOC;
    let DiskGeometry: PDISK_GEOMETRY;

    /* Grab the remove lock */
    let mut Status = IoAcquireRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );
    if !NT_SUCCESS(Status) {
        /* Fail the IRP */
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return Status;
    }

    /* Setup some defaults */
    Status = STATUS_INVALID_DEVICE_REQUEST;
    Information = 0;

    'outer: loop {
        /* Check if this is an bus device or the drive */
        if (*DeviceExtension).Type == RamdiskBus {
            /* Check what the request is */
            match (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode {
                /* Request to create a ramdisk */
                FSCTL_CREATE_RAM_DISK => {
                    /* Do it */
                    Status = RamdiskCreateRamdisk(DeviceObject, Irp, 1);
                    if !NT_SUCCESS(Status) {
                        break 'outer;
                    }
                }
                _ => {
                    /* We don't handle anything else yet */
                    crate::UNIMPLEMENTED_DBGBREAK!(
                        b"FSCTL: 0x%lx is UNSUPPORTED!\n\0".as_ptr() as *const i8,
                        (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode,
                    );
                }
            }
        } else {
            /* Check what the request is */
            match (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode {
                IOCTL_DISK_CHECK_VERIFY
                | IOCTL_STORAGE_CHECK_VERIFY
                | IOCTL_STORAGE_CHECK_VERIFY2
                | IOCTL_CDROM_CHECK_VERIFY => {
                    /* Just pretend it's OK, don't do more */
                    Status = STATUS_SUCCESS;
                }

                IOCTL_STORAGE_GET_MEDIA_TYPES
                | IOCTL_DISK_GET_MEDIA_TYPES
                | IOCTL_DISK_GET_DRIVE_GEOMETRY
                | IOCTL_CDROM_GET_DRIVE_GEOMETRY => {
                    /* Validate the length */
                    if ((*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<DISK_GEOMETRY>()
                    {
                        /* Invalid length */
                        Status = STATUS_BUFFER_TOO_SMALL;
                    } else {
                        /* Fill it out */
                        let DiskGeometry =
                            (*Irp).AssociatedIrp.SystemBuffer as PDISK_GEOMETRY;
                        (*DiskGeometry).Cylinders.QuadPart =
                            (*DriveExtension).Cylinders as i64;
                        (*DiskGeometry).BytesPerSector = (*DriveExtension).BytesPerSector;
                        (*DiskGeometry).SectorsPerTrack = (*DriveExtension).SectorsPerTrack;
                        (*DiskGeometry).TracksPerCylinder = (*DriveExtension).NumberOfHeads;
                        (*DiskGeometry).MediaType =
                            if (*DriveExtension).DiskOptions.Fixed() != 0 {
                                FixedMedia
                            } else {
                                RemovableMedia
                            };

                        /* We are done */
                        Status = STATUS_SUCCESS;
                        Information = core::mem::size_of::<DISK_GEOMETRY>() as ULONG;
                    }
                }

                IOCTL_CDROM_READ_TOC => {
                    /* Validate the length */
                    if ((*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<CDROM_TOC>()
                    {
                        /* Invalid length */
                        Status = STATUS_BUFFER_TOO_SMALL;
                    } else {
                        /* Clear the TOC */
                        let Toc = (*Irp).AssociatedIrp.SystemBuffer as PCDROM_TOC;
                        RtlZeroMemory(Toc as PVOID, core::mem::size_of::<CDROM_TOC>());

                        /* Fill it out */
                        (*Toc).Length[0] = 0;
                        (*Toc).Length[1] =
                            (RAMDISK_TOC_SIZE - core::mem::size_of_val(&(*Toc).Length)) as u8;
                        (*Toc).FirstTrack = 1;
                        (*Toc).LastTrack = 1;
                        (*Toc).TrackData[0].set_Adr(1);
                        (*Toc).TrackData[0].set_Control(TOC_DATA_TRACK);
                        (*Toc).TrackData[0].TrackNumber = 1;

                        /* We are done */
                        Status = STATUS_SUCCESS;
                        Information = RAMDISK_TOC_SIZE as ULONG;
                    }
                }

                IOCTL_DISK_SET_PARTITION_INFO => {
                    Status = RamdiskSetPartitionInfo(Irp, DriveExtension);
                }

                IOCTL_DISK_GET_PARTITION_INFO => {
                    /* Validate the length */
                    if ((*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<PARTITION_INFORMATION>()
                    {
                        /* Invalid length */
                        Status = STATUS_BUFFER_TOO_SMALL;
                    } else {
                        /* Check if we need to do this sync or async */
                        if (*DriveExtension).DiskType > RAMDISK_MEMORY_MAPPED_DISK {
                            /* Call the helper function */
                            Status = RamdiskGetPartitionInfo(Irp, DriveExtension);
                        } else {
                            /* Do it asynchronously later */
                            Status = SendIrpToThread(DeviceObject, Irp);
                            break 'outer;
                        }

                        /* We are done */
                        Information = (*Irp).IoStatus.Information as ULONG;
                    }
                }

                IOCTL_DISK_GET_LENGTH_INFO => {
                    let LengthInformation =
                        (*Irp).AssociatedIrp.SystemBuffer as PGET_LENGTH_INFORMATION;

                    /* Validate the length */
                    if ((*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<GET_LENGTH_INFORMATION>()
                    {
                        /* Invalid length */
                        Status = STATUS_BUFFER_TOO_SMALL;
                    } else {
                        /* Fill it out */
                        (*LengthInformation).Length = (*DriveExtension).DiskLength;

                        /* We are done */
                        Status = STATUS_SUCCESS;
                        Information = core::mem::size_of::<GET_LENGTH_INFORMATION>() as ULONG;
                    }
                }

                IOCTL_VOLUME_GET_GPT_ATTRIBUTES => {
                    let GptInformation: PVOLUME_GET_GPT_ATTRIBUTES_INFORMATION;

                    /* Validate the length */
                    if ((*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<VOLUME_GET_GPT_ATTRIBUTES_INFORMATION>()
                    {
                        /* Invalid length */
                        Status = STATUS_BUFFER_TOO_SMALL;
                    } else {
                        /* Fill it out */
                        let GptInformation = (*Irp).AssociatedIrp.SystemBuffer
                            as PVOLUME_GET_GPT_ATTRIBUTES_INFORMATION;
                        (*GptInformation).GptAttributes = 0;

                        /* Translate the Attributes */
                        if (*DriveExtension).DiskOptions.Readonly() != 0 {
                            (*GptInformation).GptAttributes |=
                                GPT_BASIC_DATA_ATTRIBUTE_READ_ONLY;
                        }
                        if (*DriveExtension).DiskOptions.Hidden() != 0 {
                            (*GptInformation).GptAttributes |=
                                GPT_BASIC_DATA_ATTRIBUTE_HIDDEN;
                        }
                        if (*DriveExtension).DiskOptions.NoDriveLetter() != 0 {
                            (*GptInformation).GptAttributes |=
                                GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER;
                        }

                        /* We are done */
                        Status = STATUS_SUCCESS;
                        Information =
                            core::mem::size_of::<VOLUME_GET_GPT_ATTRIBUTES_INFORMATION>()
                                as ULONG;
                    }
                }

                IOCTL_DISK_GET_DRIVE_LAYOUT
                | IOCTL_DISK_IS_WRITABLE
                | IOCTL_SCSI_MINIPORT
                | IOCTL_STORAGE_QUERY_PROPERTY
                | IOCTL_MOUNTDEV_QUERY_UNIQUE_ID
                | IOCTL_MOUNTDEV_QUERY_STABLE_GUID
                | IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS
                | IOCTL_VOLUME_SET_GPT_ATTRIBUTES
                | IOCTL_VOLUME_OFFLINE => {
                    crate::UNIMPLEMENTED_DBGBREAK!(
                        b"IOCTL: 0x%lx is unimplemented!()\n\0".as_ptr() as *const i8,
                        (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode,
                    );
                }

                _ => {
                    /* Drive code not emulated */
                    // DPRINT1("IOCTL: 0x%lx is UNSUPPORTED!\n", ...)
                }
            }

            /* If requests drop down here, we just return them complete them */
            break 'outer;
        }

        /* Queue the request to our worker thread */
        Status = SendIrpToThread(DeviceObject, Irp);
        break 'outer;
    }

    /* Release the lock */
    IoReleaseRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );
    if Status != STATUS_PENDING {
        /* Complete the request */
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = Information as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    /* Return status */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskQueryDeviceRelations(
    Type: DEVICE_RELATION_TYPE,
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;
    let mut DriveExtension: PRAMDISK_DRIVE_EXTENSION;
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let OurDeviceRelations: PDEVICE_RELATIONS;
    let mut Count: ULONG;
    let mut DiskCount: ULONG;
    let FinalCount: ULONG;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut DriveDeviceObject: *mut PDEVICE_OBJECT;
    let mut State: RAMDISK_DEVICE_STATE;

    /* Get the device extension and check if this is a drive */
    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;
    if (*DeviceExtension).Type == RamdiskDrive {
        let Status: NTSTATUS;
        let DeviceRelations: PDEVICE_RELATIONS;

        /* We're a child device, only handle target device relations */
        if Type != TargetDeviceRelation {
            let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return Status;
        }

        /* Allocate a buffer big enough to contain only one DO */
        DeviceRelations = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<DEVICE_RELATIONS>(),
            0x646D6152,
        ) as PDEVICE_RELATIONS;
        if DeviceRelations != core::ptr::null_mut() {
            /* Reference the DO and add it to the buffer */
            ObReferenceObject(DeviceObject as *mut core::ffi::c_void);
            *(*DeviceRelations).Objects.as_mut_ptr().offset(0) = DeviceObject;
            (*DeviceRelations).Count = 1;
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }

        /* Return our processing & complete */
        (*Irp).IoStatus.Information = DeviceRelations as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return Status;
    }

    /* We don't handle anything but bus relations */
    if Type != BusRelations {
        /* Pass to the next driver */
        IoCopyCurrentIrpStackLocationToNext(Irp);
        return IoCallDriver((*DeviceExtension).AttachedDevice, Irp);
    }

    /* Acquire the disk list lock */
    KeEnterCriticalRegion();
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));

    /* Did a device already fill relations? */
    DeviceRelations = (*Irp).IoStatus.Information as PDEVICE_RELATIONS;
    if !DeviceRelations.is_null() {
        /* Use the data */
        Count = (*DeviceRelations).Count;
    } else {
        /* We're the first */
        Count = 0;
    }

    /* Now loop our drives */
    DiskCount = 0;
    ListHead = core::ptr::addr_of_mut!((*DeviceExtension).DiskList);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        /* As long as it wasn't removed, count it in */
        DriveExtension = ((NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(RAMDISK_DRIVE_EXTENSION, DiskList) as isize)))
            as PRAMDISK_DRIVE_EXTENSION;
        if (*DriveExtension).State < RamdiskStateBusRemoved {
            DiskCount += 1;
        }

        /* Move to the next one */
        NextEntry = (*NextEntry).Flink;
    }

    /* Now we know our final count */
    FinalCount = Count + DiskCount;

    /* Allocate the structure */
    OurDeviceRelations = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::offset_of!(DEVICE_RELATIONS, Objects)
            + (FinalCount as usize) * core::mem::size_of::<PDEVICE_OBJECT>(),
        0x646D6152,
    ) as PDEVICE_RELATIONS;
    if OurDeviceRelations.is_null() {
        /* Fail */
        ExReleaseFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));
        KeLeaveCriticalRegion();
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Check if we already had some relations */
    if Count != 0 {
        /* Copy them in */
        RtlCopyMemory(
            (*OurDeviceRelations).Objects.as_mut_ptr() as *mut core::ffi::c_void,
            (*DeviceRelations).Objects.as_mut_ptr() as *const core::ffi::c_void,
            (Count as usize) * core::mem::size_of::<PDEVICE_OBJECT>(),
        );
    }

    /* Save the count */
    (*OurDeviceRelations).Count = FinalCount;

    /* Now loop our drives again */
    ListHead = core::ptr::addr_of_mut!((*DeviceExtension).DiskList);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        /* Go to the end of the list */
        DriveDeviceObject = (*OurDeviceRelations).Objects.as_mut_ptr().offset(Count as isize);

        /* Get the drive state */
        DriveExtension = ((NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(RAMDISK_DRIVE_EXTENSION, DiskList) as isize)))
            as PRAMDISK_DRIVE_EXTENSION;
        State = (*DriveExtension).State;

        /* If it was removed or enumerated, we don't touch the device object */
        if State >= RamdiskStateBusRemoved {
            /* If it was removed, we still have to keep track of this though */
            if State == RamdiskStateBusRemoved {
                /* Mark it as enumerated now, but don't actually reference it */
                (*DriveExtension).State = RamdiskStateEnumerated;
            }
        } else {
            /* First time it's enumerated, reference the device object */
            ObReferenceObject((*DriveExtension).DeviceObject as *mut core::ffi::c_void);

            /* Save the object pointer and move on */
            *DriveDeviceObject = (*DriveExtension).PhysicalDeviceObject;
            DriveDeviceObject = DriveDeviceObject.offset(1);
        }

        if (*DriveExtension).State < RamdiskStateBusRemoved {
            DiskCount += 1;
        }

        /* Move to the next one */
        NextEntry = (*NextEntry).Flink;
    }

    /* Release the lock */
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));
    KeLeaveCriticalRegion();

    /* Cleanup old relations */
    if !DeviceRelations.is_null() {
        ExFreePool(DeviceRelations as *mut core::ffi::c_void);
    }

    /* Complete our IRP */
    (*Irp).IoStatus.Information = OurDeviceRelations as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    /* Pass to the next driver */
    IoCopyCurrentIrpStackLocationToNext(Irp);
    return IoCallDriver((*DeviceExtension).AttachedDevice, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskDeleteDiskDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    crate::UNIMPLEMENTED_DBGBREAK!();
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskRemoveBusDevice(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;
    let mut DriveExtension: PRAMDISK_DRIVE_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;

    /* Acquire disks list lock */
    KeEnterCriticalRegion();
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));

    /* Loop over drives */
    ListHead = core::ptr::addr_of_mut!((*DeviceExtension).DiskList);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        DriveExtension = (NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(RAMDISK_DRIVE_EXTENSION, DiskList) as isize))
            as PRAMDISK_DRIVE_EXTENSION;

        /* Delete the disk */
        IoAcquireRemoveLock(
            core::ptr::addr_of_mut!((*DriveExtension).RemoveLock),
            core::ptr::null_mut(),
        );
        RamdiskDeleteDiskDevice(
            (*DriveExtension).PhysicalDeviceObject,
            core::ptr::null_mut(),
        );

        /* RamdiskDeleteDiskDevice releases list lock, so reacquire it */
        KeEnterCriticalRegion();
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));
    }

    /* Release disks list lock */
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));
    KeLeaveCriticalRegion();

    /* Prepare to pass to the lower driver */
    IoSkipCurrentIrpStackLocation(Irp);
    /* Here everything went fine */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    /* Call lower driver */
    Status = IoCallDriver((*DeviceExtension).AttachedDevice, Irp);

    /* Update state */
    (*DeviceExtension).State = RamdiskStateBusRemoved;

    /* Release the lock and ensure that everyone has finished its job before
     * we continue. The lock has been acquired by the dispatcher */
    IoReleaseRemoveLockAndWait(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );

    /* If there's a drive name */
    if !(*DeviceExtension).DriveDeviceName.Buffer.is_null() {
        /* Inform it's going to be disabled and free the drive name */
        IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!((*DeviceExtension).DriveDeviceName),
            0,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*DeviceExtension).DriveDeviceName));
    }

    /* Part from the stack, detach from lower device */
    IoDetachDevice((*DeviceExtension).AttachedDevice);

    /* Finally, delete device */
    RamdiskBusFdo = core::ptr::null_mut();
    IoDeleteDevice(DeviceObject);

    /* Return status from lower driver */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskQueryId(
    DriveExtension: PRAMDISK_DRIVE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    const RAMDISK_REGISTRY_DISK: u32 = 0x00000001;
    const UNICODE_NULL: u16 = 0;

    let mut Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION;
    let mut OutputString: PWSTR = core::ptr::null_mut();
    let mut StringLength: ULONG;

    Status = STATUS_SUCCESS;
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    /* Get what is being queried */
    match (*IoStackLocation).Parameters.QueryId.IdType {
        BusQueryDeviceID => {
            /* Allocate a buffer long enough to receive Ramdisk\RamDisk in any case */
            StringLength = 4 * ((*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK) as ULONG
                + core::mem::size_of::<[u16; 16]>() as ULONG;
            OutputString = ExAllocatePoolWithTag(PagedPool, StringLength as usize, 0x646D6152) as PWSTR;
            if OutputString.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                wcsncpy(
                    OutputString,
                    crate::wstr!("Ramdisk\\"),
                    (StringLength as usize) / core::mem::size_of::<u16>(),
                );
                if (*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK {
                    wcsncat(
                        OutputString,
                        crate::wstr!("RamVolume"),
                        (StringLength as usize) / core::mem::size_of::<u16>(),
                    );
                } else {
                    wcsncat(
                        OutputString,
                        crate::wstr!("RamDisk"),
                        (StringLength as usize) / core::mem::size_of::<u16>(),
                    );
                }
            }
        }

        BusQueryHardwareIDs => {
            /* Allocate a buffer long enough, with an extra char for double null-term */
            StringLength = 4 * ((*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK) as ULONG
                + core::mem::size_of::<u16>() as ULONG
                + core::mem::size_of::<[u16; 16]>() as ULONG;
            OutputString = ExAllocatePoolWithTag(PagedPool, StringLength as usize, 0x646D6152) as PWSTR;
            if OutputString.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                wcsncpy(
                    OutputString,
                    crate::wstr!("Ramdisk\\"),
                    (StringLength as usize) / core::mem::size_of::<u16>(),
                );
                if (*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK {
                    wcsncat(
                        OutputString,
                        crate::wstr!("RamVolume"),
                        (StringLength as usize) / core::mem::size_of::<u16>(),
                    );
                } else {
                    wcsncat(
                        OutputString,
                        crate::wstr!("RamDisk"),
                        (StringLength as usize) / core::mem::size_of::<u16>(),
                    );
                }
                *OutputString.add((StringLength as usize) / core::mem::size_of::<u16>() - 1) = UNICODE_NULL;
            }
        }

        BusQueryCompatibleIDs => {
            if (*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK {
                Status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                StringLength = core::mem::size_of::<[u16; 8]>() as ULONG;
                OutputString = ExAllocatePoolWithTag(PagedPool, StringLength as usize, 0x646D6152) as PWSTR;
                if OutputString.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    wcsncpy(
                        OutputString,
                        crate::wstr!("GenDisk"),
                        (StringLength as usize) / core::mem::size_of::<u16>(),
                    );
                    *OutputString.add((StringLength as usize) / core::mem::size_of::<u16>() - 1) = UNICODE_NULL;
                }
            }
        }

        BusQueryInstanceID => {
            OutputString = ExAllocatePoolWithTag(
                PagedPool,
                (*DriveExtension).GuidString.MaximumLength as usize,
                0x646D6152,
            ) as PWSTR;
            if OutputString.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                wcsncpy(
                    OutputString,
                    (*DriveExtension).GuidString.Buffer,
                    ((*DriveExtension).GuidString.MaximumLength as usize) / core::mem::size_of::<u16>(),
                );
            }
        }

        BusQueryDeviceSerialNumber | BusQueryContainerID => {
            /* Nothing to do */
        }

        _ => {}
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = OutputString as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskQueryCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION;
    let DeviceCapabilities: PDEVICE_CAPABILITIES;
    let DriveExtension: PRAMDISK_DRIVE_EXTENSION;

    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    DeviceCapabilities = (*IoStackLocation).Parameters.DeviceCapabilities.Capabilities;
    DriveExtension = (*DeviceObject).DeviceExtension as PRAMDISK_DRIVE_EXTENSION;

    /* Validate our input buffer */
    if (*DeviceCapabilities).Version != 1
        || ((*DeviceCapabilities).Size as usize) < core::mem::size_of::<DEVICE_CAPABILITIES>()
    {
        Status = STATUS_UNSUCCESSFUL;
    } else {
        /* And set everything we know about our capabilities */
        (*DeviceCapabilities).set_Removable(MarkRamdisksAsRemovable);
        (*DeviceCapabilities).set_UniqueID(1);
        (*DeviceCapabilities).set_SilentInstall(1);
        (*DeviceCapabilities).set_RawDeviceOK(1);
        (*DeviceCapabilities)
            .set_SurpriseRemovalOK(((*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK) as u32);
        (*DeviceCapabilities).set_NoDisplayInUI(1);
        Status = STATUS_SUCCESS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskQueryDeviceText(
    DriveExtension: PRAMDISK_DRIVE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION;
    let DeviceTextType: DEVICE_TEXT_TYPE;
    let mut OutputString: PWSTR = core::ptr::null_mut();

    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    DeviceTextType = (*IoStackLocation).Parameters.QueryDeviceText.DeviceTextType;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    /* Just copy our constants, according to the input */
    match DeviceTextType {
        x if x == DeviceTextDescription => {
            const RAMDISK_DESC: &[u16; 8] = &[
                'R' as u16, 'a' as u16, 'm' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'k' as u16,
                0,
            ];
            OutputString = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<[u16; 8]>(),
                u32::from_le_bytes(*b"Ramd"),
            ) as PWSTR;
            if OutputString == core::ptr::null_mut() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                wcsncpy(
                    OutputString,
                    RAMDISK_DESC.as_ptr(),
                    core::mem::size_of::<[u16; 8]>() / core::mem::size_of::<u16>(),
                );
            }
        }
        x if x == DeviceTextLocationInformation => {
            const RAMDISK_LOC: &[u16; 10] = &[
                'R' as u16, 'a' as u16, 'm' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'k' as u16,
                '\\' as u16, '0' as u16, 0,
            ];
            OutputString = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<[u16; 10]>(),
                u32::from_le_bytes(*b"Ramd"),
            ) as PWSTR;
            if OutputString == core::ptr::null_mut() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                wcsncpy(
                    OutputString,
                    RAMDISK_LOC.as_ptr(),
                    core::mem::size_of::<[u16; 10]>() / core::mem::size_of::<u16>(),
                );
            }
        }
        _ => {}
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = OutputString as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskQueryBusInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut PnpBusInfo: PPNP_BUS_INFORMATION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    /* Allocate output memory */
    PnpBusInfo = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<PNP_BUS_INFORMATION>(),
        0x52616d64,
    ) as PPNP_BUS_INFORMATION;
    if PnpBusInfo.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        /* Copy our bus GUID and set our legacy type */
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*PnpBusInfo).BusTypeGuid) as *mut _,
            core::ptr::addr_of!(GUID_BUS_TYPE_RAMDISK) as *const _,
            core::mem::size_of::<GUID>(),
        );
        (*PnpBusInfo).LegacyBusType = PNPBus;
        (*PnpBusInfo).BusNumber = 0;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = PnpBusInfo as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskIoCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    /* Just set the event to unlock caller */
    KeSetEvent(Context as PKEVENT as _, 0, 0);

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;
    let mut Status: NTSTATUS;
    let Minor: UCHAR;
    let mut Event: KEVENT = core::mem::zeroed();

    /* Get the device extension and stack location */
    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    Minor = (*IoStackLocation).MinorFunction;

    /* Check if the bus is removed */
    if (*DeviceExtension).State == RamdiskStateBusRemoved {
        /* Only remove-device and query-id are allowed */
        if (Minor as u32 != (IRP_MN_REMOVE_DEVICE as u32)) && (Minor as u32 != (IRP_MN_QUERY_ID as u32)) {
            /* Fail anything else */
            Status = STATUS_NO_SUCH_DEVICE;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return Status;
        }
    }

    /* Acquire the remove lock */
    Status = IoAcquireRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );
    if !NT_SUCCESS(Status) {
        /* Fail the IRP */
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return Status;
    }

    /* Query the IRP type */
    loop {
        match Minor as u8 {
            IRP_MN_START_DEVICE => {
                if (*DeviceExtension).Type == RamdiskDrive {
                    let mut ResultLength: ULONG = 0;
                    let mut InstallState: DEVICE_INSTALL_STATE = core::mem::zeroed();
                    let DriveExtension: PRAMDISK_DRIVE_EXTENSION =
                        DeviceExtension as PRAMDISK_DRIVE_EXTENSION;

                    /* If we already have a drive name, free it */
                    if !(*DriveExtension).DriveDeviceName.Buffer.is_null() {
                        ExFreePool((*DriveExtension).DriveDeviceName.Buffer as PVOID);
                    }

                    /* Register our device interface */
                    if (*DriveExtension).DiskType != RAMDISK_REGISTRY_DISK {
                        Status = IoRegisterDeviceInterface(
                            DeviceObject,
                            core::ptr::addr_of!(GUID_DEVINTERFACE_VOLUME),
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!((*DriveExtension).DriveDeviceName),
                        );
                    } else {
                        Status = IoRegisterDeviceInterface(
                            DeviceObject,
                            core::ptr::addr_of!(RamdiskDiskInterface),
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!((*DriveExtension).DriveDeviceName),
                        );
                    }

                    /* If we were asked not to assign a drive letter or if getting
                     * a name failed, just return saying we're now started */
                    if (*DriveExtension).DiskOptions.NoDriveLetter() != 0
                        || (*DriveExtension).DriveDeviceName.Buffer.is_null()
                    {
                        (*DriveExtension).State = RamdiskStateStarted;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                        break;
                    }

                    /* Now get our installation state */
                    Status = IoGetDeviceProperty(
                        DeviceObject,
                        DevicePropertyInstallState,
                        core::mem::size_of::<DEVICE_INSTALL_STATE>() as ULONG,
                        core::ptr::addr_of_mut!(InstallState) as PVOID,
                        core::ptr::addr_of_mut!(ResultLength),
                    );
                    /* If querying the information failed, assume success */
                    if !NT_SUCCESS(Status) {
                        InstallState = InstallStateInstalled;
                    }

                    /* If we were properly installed, then, enable the interface */
                    if InstallState == InstallStateInstalled {
                        Status = IoSetDeviceInterfaceState(
                            core::ptr::addr_of_mut!((*DriveExtension).DriveDeviceName),
                            1,
                        );
                    }

                    /* We're fine & up */
                    (*DriveExtension).State = RamdiskStateStarted;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    break;
                }

                /* Prepare next stack to pass it down */
                IoCopyCurrentIrpStackLocationToNext(Irp);

                /* Initialize our notification event & our completion routine */
                KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);
                IoSetCompletionRoutine(
                    Irp,
                    crate::glue::as_completion(RamdiskIoCompletionRoutine),
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    1,
                    1,
                    1,
                );

                /* Call lower driver */
                Status = IoCallDriver((*DeviceExtension).AttachedDevice, Irp);
                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of_mut!(Event) as PVOID,
                        _KWAIT_REASON_Executive,
                        0i32 as i8,
                        0,
                        core::ptr::null_mut(),
                    );
                    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                }

                /* If it succeed to start then enable ourselves and we're up! */
                if NT_SUCCESS(Status) {
                    Status = IoSetDeviceInterfaceState(
                        core::ptr::addr_of_mut!((*DeviceExtension).DriveDeviceName),
                        1,
                    );
                    (*DeviceExtension).State = RamdiskStateStarted;
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                break;
            }

            IRP_MN_QUERY_STOP_DEVICE
            | IRP_MN_CANCEL_STOP_DEVICE
            | IRP_MN_STOP_DEVICE
            | IRP_MN_QUERY_REMOVE_DEVICE
            | IRP_MN_CANCEL_REMOVE_DEVICE => {
                crate::UNIMPLEMENTED_DBGBREAK!(
                    b"PnP IRP: %lx\n\0".as_ptr() as *const i8,
                    Minor as u32,
                );
                break;
            }

            IRP_MN_REMOVE_DEVICE => {
                /* Remove the proper device */
                if (*DeviceExtension).Type == RamdiskBus {
                    Status = RamdiskRemoveBusDevice(DeviceObject, Irp);

                    /* Return here, lower device has already been called
                     * And remove lock released. This is needed by the function. */
                    return Status;
                } else {
                    Status = RamdiskDeleteDiskDevice(DeviceObject, Irp);

                    /* Complete the IRP here and return
                     * Here again we don't have to release remove lock
                     * This has already been done by the function. */
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                    return Status;
                }
            }

            IRP_MN_SURPRISE_REMOVAL => {
                crate::UNIMPLEMENTED_DBGBREAK!(
                    b"PnP IRP: %lx\n\0".as_ptr() as *const i8,
                    Minor as u32,
                );
                break;
            }

            IRP_MN_QUERY_ID => {
                /* Are we a drive? */
                if (*DeviceExtension).Type == RamdiskDrive {
                    Status =
                        RamdiskQueryId(DeviceExtension as PRAMDISK_DRIVE_EXTENSION, Irp);
                }
                break;
            }

            IRP_MN_QUERY_BUS_INFORMATION => {
                /* Are we a drive? */
                if (*DeviceExtension).Type == RamdiskDrive {
                    Status = RamdiskQueryBusInformation(DeviceObject, Irp);
                }
                break;
            }

            IRP_MN_EJECT => {
                crate::UNIMPLEMENTED_DBGBREAK!(
                    b"PnP IRP: %lx\n\0".as_ptr() as *const i8,
                    Minor as u32,
                );
                break;
            }

            IRP_MN_QUERY_DEVICE_TEXT => {
                /* Are we a drive? */
                if (*DeviceExtension).Type == RamdiskDrive {
                    Status = RamdiskQueryDeviceText(
                        DeviceExtension as PRAMDISK_DRIVE_EXTENSION,
                        Irp,
                    );
                }
                break;
            }

            IRP_MN_QUERY_DEVICE_RELATIONS => {
                /* Call our main routine */
                Status = RamdiskQueryDeviceRelations(
                    (*IoStackLocation).Parameters.QueryDeviceRelations.Type,
                    DeviceObject,
                    Irp,
                );
                IoReleaseRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
                    Irp as PVOID,
                );
                return Status;
            }

            IRP_MN_QUERY_CAPABILITIES => {
                /* Are we a drive? */
                if (*DeviceExtension).Type == RamdiskDrive {
                    Status = RamdiskQueryCapabilities(DeviceObject, Irp);
                }
                break;
            }

            IRP_MN_QUERY_RESOURCES | IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
                /* Complete immediately without touching it */
                IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                IoReleaseRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
                    Irp as PVOID,
                );
                return Status;
            }

            _ => {
                // DPRINT removed
                break;
            }
        }
    }

    /* Are we the bus? */
    if (*DeviceExtension).Type == RamdiskBus {
        /* Do we have an attached device? */
        if !(*DeviceExtension).AttachedDevice.is_null() {
            /* Forward the IRP */
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*DeviceExtension).AttachedDevice, Irp);
        }
    }

    /* Release the lock and return status */
    IoReleaseRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let IoStackLocation: PIO_STACK_LOCATION;
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;

    /* If we have a device extension, take extra caution with the lower driver */
    if !DeviceExtension.is_null() {
        PoStartNextPowerIrp(Irp);

        /* Device has not been removed yet, so pass to the attached/lower driver */
        if (*DeviceExtension).State < RamdiskStateBusRemoved {
            IoSkipCurrentIrpStackLocation(Irp);
            return PoCallDriver((*DeviceExtension).AttachedDevice, Irp);
        }
        /* Otherwise, simply complete the IRP notifying that deletion is pending */
        else {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DELETE_PENDING;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_DELETE_PENDING;
        }
    }

    /* Get stack and deal with minor functions */
    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStackLocation).MinorFunction as u8 {
        IRP_MN_SET_POWER => {
            /* If setting device power state it's all fine and return success */
            if DevicePowerState != 0 {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            }

            /* Get appropriate status for return */
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            PoStartNextPowerIrp(Irp);
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }

        IRP_MN_QUERY_POWER => {
            /* We can obviously accept all states so just return success */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = STATUS_SUCCESS;
            PoStartNextPowerIrp(Irp);
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }

        _ => {
            /* Just complete and save status for return */
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            PoStartNextPowerIrp(Irp);
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;

    /* If we have a device extension, forward the IRP to the attached device */
    if !DeviceExtension.is_null() {
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*DeviceExtension).AttachedDevice, Irp);
    }
    /* Otherwise just complete the request
     * And return the status with which we complete it */
    else {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskScsi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let DeviceExtension: PRAMDISK_BUS_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;

    /* Having a proper device is mandatory */
    if (*DeviceExtension).State > RamdiskStateStopped {
        Status = STATUS_DEVICE_DOES_NOT_EXIST;

        /* CompleteIRP */
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

        return Status;
    }

    /* Acquire the remove lock */
    Status = IoAcquireRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );
    if !NT_SUCCESS(Status) {
        /* CompleteIRP */
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

        return Status;
    }

    /* Queue the IRP for worker */
    Status = SendIrpToThread(DeviceObject, Irp);
    if Status != STATUS_PENDING {
        /* CompleteIRP */
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

        return Status;
    }

    /* Release the remove lock */
    IoReleaseRemoveLock(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock),
        Irp as PVOID,
    );

    /* Quit */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskFlushBuffers(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let DeviceExtension: PRAMDISK_DRIVE_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_DRIVE_EXTENSION;

    if (*DeviceExtension).Type != RamdiskDrive
        || (*DeviceExtension).DiskType > RAMDISK_MEMORY_MAPPED_DISK
    {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return STATUS_SUCCESS;
    }

    Status = SendIrpToThread(DeviceObject, Irp);
    if Status != STATUS_PENDING {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskUnload(DriverObject: PDRIVER_OBJECT) {
    if !DriverRegistryPath.Buffer.is_null() {
        ExFreePoolWithTag(DriverRegistryPath.Buffer as *mut _, 0x646D6152);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RamdiskAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    const STATUS_DEVICE_ALREADY_ATTACHED: NTSTATUS = 0xC0000038u32 as NTSTATUS;
    const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000Eu32 as NTSTATUS;
    const FILE_DEVICE_BUS_EXTENDER: u32 = 0x0000002A;

    let DeviceExtension: PRAMDISK_BUS_EXTENSION;
    let AttachedDevice: PDEVICE_OBJECT;
    let mut Status: NTSTATUS;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let DeviceObject: PDEVICE_OBJECT;

    /* Only create the bus FDO once */
    if !RamdiskBusFdo.is_null() {
        return STATUS_DEVICE_ALREADY_ATTACHED;
    }

    /* Create the bus FDO */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        w!("\\Device\\Ramdisk").as_ptr() as PWSTR,
    );
    let mut DeviceObject_local: PDEVICE_OBJECT = core::ptr::null_mut();
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<RAMDISK_BUS_EXTENSION>() as u32,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_BUS_EXTENDER,
        FILE_DEVICE_SECURE_OPEN,
        0,
        core::ptr::addr_of_mut!(DeviceObject_local),
    );
    DeviceObject = DeviceObject_local;
    if NT_SUCCESS(Status) {
        /* Initialize the bus FDO extension */
        DeviceExtension = (*DeviceObject).DeviceExtension as PRAMDISK_BUS_EXTENSION;
        RtlZeroMemory(
            DeviceExtension as *mut core::ffi::c_void,
            core::mem::size_of::<RAMDISK_BUS_EXTENSION>(),
        );

        /* Set bus FDO flags */
        (*DeviceObject).Flags |= DO_POWER_PAGABLE | DO_DIRECT_IO;

        /* Setup the bus FDO extension */
        (*DeviceExtension).Type = RamdiskBus;
        ExInitializeFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).DiskListLock));
        IoInitializeRemoveLock(
            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as *mut core::ffi::c_void,
            u32::from_le_bytes([b'R', b'a', b'm', b'd']),
            1,
            0,
        );
        InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).DiskList));
        (*DeviceExtension).PhysicalDeviceObject = PhysicalDeviceObject;
        (*DeviceExtension).DeviceObject = DeviceObject;

        /* Register the RAM disk device interface */
        Status = IoRegisterDeviceInterface(
            PhysicalDeviceObject,
            core::ptr::addr_of!(RamdiskBusInterface),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*DeviceExtension).BusDeviceName),
        );
        if !NT_SUCCESS(Status) {
            /* Fail */
            IoDeleteDevice(DeviceObject);
            return Status;
        }

        /* Attach us to the device stack */
        AttachedDevice = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
        (*DeviceExtension).AttachedDevice = AttachedDevice;
        if AttachedDevice.is_null() {
            /* Fail */
            IoSetDeviceInterfaceState(core::ptr::addr_of_mut!((*DeviceExtension).BusDeviceName), 0);
            RtlFreeUnicodeString(core::ptr::addr_of_mut!((*DeviceExtension).BusDeviceName));
            IoDeleteDevice(DeviceObject);
            return STATUS_NO_SUCH_DEVICE;
        }

        /* Bus FDO is initialized */
        RamdiskBusFdo = DeviceObject;

        /* Loop for loader block */
        if !KeLoaderBlock.is_null() {
            /* Are we being booted from setup? Not yet supported */
            if !(*KeLoaderBlock).SetupLdrBlock.is_null() {
                // DPRINT1("FIXME: RamdiskAddDevice is UNSUPPORTED when being started from SETUPLDR!\n");
            }
            // ASSERT(!KeLoaderBlock->SetupLdrBlock);
        }

        /* All done — clear DO_DEVICE_INITIALIZING so the bus FDO is marked
         * ready. Upstream ReactOS ramdisk.c (and the faithful translation)
         * dropped the `~`, leaving the FDO permanently "initializing"; under a
         * kernel-text layout shift this raced PnP enumeration so the bus
         * QUERY_DEVICE_RELATIONS returned STATUS_INVALID_DEVICE_REQUEST and the
         * boot-disk PDO was never surfaced (arm64 BUGCHECK 0x64 on smss open).
         * crash-value 2026-07-10. */
        (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
        Status = STATUS_SUCCESS;
    }

    /* Return status */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {

    let BootDeviceName: PCHAR;
    let CommandLine: PCHAR;
    let mut PhysicalDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    // RD_TRACE("[RAMDISK-DRV] DriverEntry called\r\n");
    // DPRINT("RAM Disk Driver Initialized\n");

    /* Save the registry path */
    DriverRegistryPath.MaximumLength =
        (*RegistryPath).Length + core::mem::size_of::<UNICODE_NULL>() as u16;
    DriverRegistryPath.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        DriverRegistryPath.MaximumLength as usize,
        u32::from_le_bytes(*b"Ramd"),
    ) as PWSTR;
    if DriverRegistryPath.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyUnicodeString(core::ptr::addr_of_mut!(DriverRegistryPath), RegistryPath);

    /* Query ramdisk parameters */
    QueryParameters(core::ptr::addr_of_mut!(DriverRegistryPath));

    /* Set device routines */
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = crate::glue::as_dispatch(RamdiskOpenClose);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = crate::glue::as_dispatch(RamdiskOpenClose);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = crate::glue::as_dispatch(RamdiskReadWrite);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = crate::glue::as_dispatch(RamdiskReadWrite);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = crate::glue::as_dispatch(RamdiskDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = crate::glue::as_dispatch(RamdiskPnp);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = crate::glue::as_dispatch(RamdiskPower);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = crate::glue::as_dispatch(RamdiskSystemControl);
    (*DriverObject).MajorFunction[IRP_MJ_SCSI as usize] = crate::glue::as_dispatch(RamdiskScsi);
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS as usize] = crate::glue::as_dispatch(RamdiskFlushBuffers);
    (*(*DriverObject).DriverExtension).AddDevice = crate::glue::as_add_device(RamdiskAddDevice);
    (*DriverObject).DriverUnload = crate::glue::as_unload(RamdiskUnload);

    /* Check for a loader block */
    if !KeLoaderBlock.is_null() {
        /* Get the boot device name */
        BootDeviceName = (*KeLoaderBlock).ArcBootDeviceName;
        if !BootDeviceName.is_null() {
            /* Check if we're booting from ramdisk */
            if (strlen(BootDeviceName) >= 10)
                && (_strnicmp(
                    BootDeviceName,
                    b"ramdisk(0)\0".as_ptr() as *const i8,
                    10,
                ) == 0)
            {
                /* We'll have to tell the PnP Manager */
                ReportDetectedDevice = 1;

                /* Check for a command line */
                CommandLine = (*KeLoaderBlock).LoadOptions;
                if !CommandLine.is_null() {
                    /* Check if this is an ISO boot */
                    if !strstr(CommandLine, b"RDEXPORTASCD\0".as_ptr() as *const i8).is_null() {
                        /* Remember for later */
                        ExportBootDiskAsCd = 1;
                    }

                    /* Check if this is PE boot */
                    if !strstr(CommandLine, b"MININT\0".as_ptr() as *const i8).is_null() {
                        /* Remember for later */
                        IsWinPEBoot = 1;
                    }
                }
            }
        }
    }

    /* Installing from Ramdisk isn't supported yet */
    if !(*KeLoaderBlock).SetupLdrBlock.is_null() {
        // DPRINT1("FIXME: Installing from RamDisk is UNSUPPORTED!\n");
    }
    // ASSERT(!KeLoaderBlock->SetupLdrBlock);

    /* Are we reporting the device */
    if ReportDetectedDevice != 0 {
        // RD_TRACE("[RAMDISK-DRV] Reporting detected device\r\n");
        /* Do it */
        Status = IoReportDetectedDevice(
            DriverObject,
            InterfaceTypeUndefined,
            0xFFFFFFFF,
            0xFFFFFFFF,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(PhysicalDeviceObject),
        );
        if NT_SUCCESS(Status) {
            /* Create the device object */
            Status = RamdiskAddDevice(DriverObject, PhysicalDeviceObject);
            if NT_SUCCESS(Status) {
                /* We are done */
                (*PhysicalDeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
                Status = STATUS_SUCCESS;
            }
        }
    } else {
        /* Done */
        Status = STATUS_SUCCESS;
    }

    /* Done */
    // RD_TRACE("[RAMDISK-DRV] DriverEntry returning\r\n");
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Ramdisk Class Driver
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            drivers/storage/class/ramdisk/ramdisk.c
//  * PURPOSE:         Main Driver Routines
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <initguid.h>
// #include <ntddk.h>
// #include <ntifs.h>
// #include <ntdddisk.h>
// #include <ntddcdrm.h>
// #include <scsi.h>
// #include <ntddscsi.h>
// #include <ntddvol.h>
// #include <mountdev.h>
// #include <mountmgr.h>
// #include <ketypes.h>
// #include <iotypes.h>
// #include <rtlfuncs.h>
// #include <arc/arc.h>
// #include <reactos/drivers/ntddrdsk.h>
// #include "../../../filesystems/fs_rec/fs_rec.h"
// #include <stdio.h>
// #define NDEBUG
// #include <debug.h>
//
// #define RD_TRACE(s) ((void)0)
//
// #define DO_XIP   0x00020000
//
// /* GLOBALS ********************************************************************/
//
// #define RAMDISK_SESSION_SIZE \
//     FIELD_OFFSET(CDROM_TOC, TrackData) + sizeof(TRACK_DATA)
//
// #define RAMDISK_TOC_SIZE \
//     FIELD_OFFSET(CDROM_TOC, TrackData) + 2 * sizeof(TRACK_DATA)
//
// #define TOC_DATA_TRACK              (0x04)
//
// typedef enum _RAMDISK_DEVICE_TYPE
// {
//     RamdiskBus,
//     RamdiskDrive
// } RAMDISK_DEVICE_TYPE;
//
// typedef enum _RAMDISK_DEVICE_STATE
// {
//     RamdiskStateUninitialized,
//     RamdiskStateStarted,
//     RamdiskStatePaused,
//     RamdiskStateStopped,
//     RamdiskStateRemoved,
//     RamdiskStateBusRemoved,
//     RamdiskStateEnumerated,
// } RAMDISK_DEVICE_STATE;
//
// DEFINE_GUID(RamdiskBusInterface,
//             0x5DC52DF0,
//             0x2F8A,
//             0x410F,
//             0x80, 0xE4, 0x05, 0xF8, 0x10, 0xE7, 0xA8, 0x8A);
//
// DEFINE_GUID(RamdiskDiskInterface,
//             0x31D909F0,
//             0x2CDF,
//             0x4A20,
//             0x9E, 0xD4, 0x7D, 0x65, 0x47, 0x6C, 0xA7, 0x68);
//
// typedef struct _RAMDISK_EXTENSION
// {
//     RAMDISK_DEVICE_TYPE Type;
//     RAMDISK_DEVICE_STATE State;
//     PDEVICE_OBJECT DeviceObject;
//     PDEVICE_OBJECT PhysicalDeviceObject;
//     PDEVICE_OBJECT AttachedDevice;
//     IO_REMOVE_LOCK RemoveLock;
//     UNICODE_STRING DriveDeviceName;
//     UNICODE_STRING BusDeviceName;
//     FAST_MUTEX DiskListLock;
//     LIST_ENTRY DiskList;
// } RAMDISK_EXTENSION, *PRAMDISK_EXTENSION;
//
// typedef struct _RAMDISK_BUS_EXTENSION
// {
//     RAMDISK_EXTENSION;
// } RAMDISK_BUS_EXTENSION, *PRAMDISK_BUS_EXTENSION;
//
// typedef struct _RAMDISK_DRIVE_EXTENSION
// {
//     /* Inherited base class */
//     RAMDISK_EXTENSION;
//
//     /* Data we get from the creator */
//     GUID DiskGuid;
//     UNICODE_STRING GuidString;
//     UNICODE_STRING SymbolicLinkName;
//     ULONG DiskType;
//     RAMDISK_CREATE_OPTIONS DiskOptions;
//     LARGE_INTEGER DiskLength;
//     LONG DiskOffset;
//     WCHAR DriveLetter;
//     ULONG BasePage;
//
//     /* Data we get from the disk */
//     ULONG BytesPerSector;
//     ULONG SectorsPerTrack;
//     ULONG NumberOfHeads;
//     ULONG Cylinders;
//     ULONG HiddenSectors;
// } RAMDISK_DRIVE_EXTENSION, *PRAMDISK_DRIVE_EXTENSION;
//
// ULONG MaximumViewLength;
// ULONG MaximumPerDiskViewLength;
// ULONG ReportDetectedDevice;
// ULONG MarkRamdisksAsRemovable;
// ULONG MinimumViewCount;
// ULONG DefaultViewCount;
// ULONG MaximumViewCount;
// ULONG MinimumViewLength;
// ULONG DefaultViewLength;
// UNICODE_STRING DriverRegistryPath;
// BOOLEAN ExportBootDiskAsCd;
// BOOLEAN IsWinPEBoot;
// PDEVICE_OBJECT RamdiskBusFdo;
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// NTAPI
// QueryParameters(IN PUNICODE_STRING RegistryPath)
// {
//     ULONG MinView, DefView, MinViewLength, DefViewLength, MaxViewLength;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[11];
//
//     /* Set defaults */
//     MaximumViewLength = 0x10000000u;
//     MaximumPerDiskViewLength = 0x10000000u;
//     ReportDetectedDevice = 0;
//     MarkRamdisksAsRemovable = 0;
//     MinimumViewCount = 2;
//     DefaultViewCount = 16;
//     MaximumViewCount = 64;
//     MinimumViewLength = 0x10000u;
//     DefaultViewLength = 0x100000u;
//
//     /* Setup the query table and query the registry */
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].Flags = 1;
//     QueryTable[0].Name = L"Parameters";
//     QueryTable[1].Flags = 32;
//     QueryTable[1].Name = L"ReportDetectedDevice";
//     QueryTable[1].EntryContext = &ReportDetectedDevice;
//     QueryTable[2].Flags = 32;
//     QueryTable[2].Name = L"MarkRamdisksAsRemovable";
//     QueryTable[2].EntryContext = &MarkRamdisksAsRemovable;
//     QueryTable[3].Flags = 32;
//     QueryTable[3].Name = L"MinimumViewCount";
//     QueryTable[3].EntryContext = &MinimumViewCount;
//     QueryTable[4].Flags = 32;
//     QueryTable[4].Name = L"DefaultViewCount";
//     QueryTable[4].EntryContext = &DefaultViewCount;
//     QueryTable[5].Flags = 32;
//     QueryTable[5].Name = L"MaximumViewCount";
//     QueryTable[5].EntryContext = &MaximumViewCount;
//     QueryTable[6].Flags = 32;
//     QueryTable[6].Name = L"MinimumViewLength";
//     QueryTable[6].EntryContext = &MinimumViewLength;
//     QueryTable[7].Flags = 32;
//     QueryTable[7].Name = L"DefaultViewLength";
//     QueryTable[7].EntryContext = &DefaultViewLength;
//     QueryTable[8].Flags = 32;
//     QueryTable[8].Name = L"MaximumViewLength";
//     QueryTable[8].EntryContext = &MaximumViewLength;
//     QueryTable[9].Flags = 32;
//     QueryTable[9].Name = L"MaximumPerDiskViewLength";
//     QueryTable[9].EntryContext = &MaximumPerDiskViewLength;
//     RtlQueryRegistryValues(RTL_REGISTRY_OPTIONAL,
//                            RegistryPath->Buffer,
//                            QueryTable,
//                            NULL,
//                            NULL);
//
//     /* Parse minimum view count, cannot be bigger than 256 or smaller than 2 */
//     MinView = MinimumViewCount;
//     if (MinimumViewCount >= 2)
//     {
//         if (MinimumViewCount > 256) MinView = 256;
//     }
//     else
//     {
//         MinView = 2;
//     }
//     MinimumViewCount = MinView;
//
//     /* Parse default view count, cannot be bigger than 256 or smaller than minimum */
//     DefView = DefaultViewCount;
//     if (DefaultViewCount >= MinView)
//     {
//         if (DefaultViewCount > 256) DefView = 256;
//     }
//     else
//     {
//         DefView = MinView;
//     }
//     DefaultViewCount = DefView;
//
//     /* Parse maximum view count, cannot be bigger than 256 or smaller than default */
//     if (MaximumViewCount >= DefView)
//     {
//         if (MaximumViewCount > 256) MaximumViewCount = 256;
//     }
//     else
//     {
//         MaximumViewCount = DefView;
//     }
//
//     /* Parse minimum view length, cannot be bigger than 1GB or smaller than 64KB */
//     MinViewLength = MinimumViewLength;
//     if (MinimumViewLength >= 0x10000)
//     {
//         if (MinimumViewLength > 0x40000000) MinViewLength = 0x40000000u;
//     }
//     else
//     {
//         MinViewLength = 0x10000u;
//     }
//     MinimumViewLength = MinViewLength;
//
//     /* Parse default view length, cannot be bigger than 1GB or smaller than minimum */
//     DefViewLength = DefaultViewLength;
//     if (DefaultViewLength >= MinViewLength)
//     {
//         if (DefaultViewLength > 0x40000000) DefViewLength = 0x40000000u;
//     }
//     else
//     {
//         DefViewLength = MinViewLength;
//     }
//     DefaultViewLength = DefViewLength;
//
//     /* Parse maximum view length, cannot be bigger than 1GB or smaller than default */
//     MaxViewLength = MaximumViewLength;
//     if (MaximumViewLength >= DefViewLength)
//     {
//         if (MaximumViewLength > 0x40000000) MaxViewLength = 0x40000000u;
//     }
//     else
//     {
//         MaxViewLength = DefViewLength;
//     }
//     MaximumViewLength = MaxViewLength;
//
//     /* Parse maximum view length per disk, cannot be smaller than 16MB */
//     if (MaximumPerDiskViewLength >= 0x1000000)
//     {
//         if (MaxViewLength > 0xFFFFFFFF) MaximumPerDiskViewLength = -1;
//     }
//     else
//     {
//         MaximumPerDiskViewLength = 0x1000000u;
//     }
// }
//
// PVOID
// NTAPI
// RamdiskMapPages(IN PRAMDISK_DRIVE_EXTENSION DeviceExtension,
//                 IN LARGE_INTEGER Offset,
//                 IN ULONG Length,
//                 OUT PULONG OutputLength)
// {
//     PHYSICAL_ADDRESS PhysicalAddress;
//     PVOID MappedBase;
//     ULONG PageOffset;
//     SIZE_T ActualLength;
//     LARGE_INTEGER ActualOffset;
//     LARGE_INTEGER ActualPages;
//
//     /* We only support boot disks for now */
//     ASSERT(DeviceExtension->DiskType == RAMDISK_BOOT_DISK);
//
//     /* Calculate the actual offset in the drive */
//     ActualOffset.QuadPart = DeviceExtension->DiskOffset + Offset.QuadPart;
//
//     /* Convert to pages */
//     ActualPages.QuadPart = ActualOffset.QuadPart >> PAGE_SHIFT;
//
//     /* Now add the base page */
//     ActualPages.QuadPart = DeviceExtension->BasePage + ActualPages.QuadPart;
//
//     /* Calculate final amount of bytes */
//     PhysicalAddress.QuadPart = ActualPages.QuadPart << PAGE_SHIFT;
//
//     /* Calculate pages spanned for the mapping */
//     ActualLength = ADDRESS_AND_SIZE_TO_SPAN_PAGES(ActualOffset.QuadPart, Length);
//
//     /* And convert this back to bytes */
//     ActualLength <<= PAGE_SHIFT;
//
//     /* Get the offset within the page */
//     PageOffset = BYTE_OFFSET(ActualOffset.QuadPart);
//
//     /* Map the I/O Space from the loader */
//     MappedBase = MmMapIoSpace(PhysicalAddress, ActualLength, MmCached);
//
//     /* Return actual offset within the page as well as the length */
//     if (MappedBase) MappedBase = (PVOID)((ULONG_PTR)MappedBase + PageOffset);
//     *OutputLength = Length;
//     return MappedBase;
// }
//
// VOID
// NTAPI
// RamdiskUnmapPages(IN PRAMDISK_DRIVE_EXTENSION DeviceExtension,
//                   IN PVOID BaseAddress,
//                   IN LARGE_INTEGER Offset,
//                   IN ULONG Length)
// {
//     LARGE_INTEGER ActualOffset;
//     SIZE_T ActualLength;
//     ULONG PageOffset;
//
//     /* We only support boot disks for now */
//     ASSERT(DeviceExtension->DiskType == RAMDISK_BOOT_DISK);
//
//     /* Calculate the actual offset in the drive */
//     ActualOffset.QuadPart = DeviceExtension->DiskOffset + Offset.QuadPart;
//
//     /* Calculate pages spanned for the mapping */
//     ActualLength = ADDRESS_AND_SIZE_TO_SPAN_PAGES(ActualOffset.QuadPart, Length);
//
//     /* And convert this back to bytes */
//     ActualLength <<= PAGE_SHIFT;
//
//     /* Get the offset within the page */
//     PageOffset = BYTE_OFFSET(ActualOffset.QuadPart);
//
//     /* Calculate actual base address where we mapped this */
//     BaseAddress = (PVOID)((ULONG_PTR)BaseAddress - PageOffset);
//
//     /* Unmap the I/O space we got from the loader */
//     MmUnmapIoSpace(BaseAddress, ActualLength);
// }
//
// NTSTATUS
// NTAPI
// RamdiskCreateDiskDevice(IN PRAMDISK_BUS_EXTENSION DeviceExtension,
//                         IN PRAMDISK_CREATE_INPUT Input,
//                         IN BOOLEAN ValidateOnly,
//                         OUT PRAMDISK_DRIVE_EXTENSION *NewDriveExtension)
// {
//     ULONG BasePage, DiskType, Length;
//     //ULONG ViewCount;
//     NTSTATUS Status;
//     PDEVICE_OBJECT DeviceObject;
//     PRAMDISK_DRIVE_EXTENSION DriveExtension;
//     PVOID Buffer;
//     WCHAR LocalBuffer[16];
//     UNICODE_STRING SymbolicLinkName, DriveString, GuidString, DeviceName;
//     PPACKED_BOOT_SECTOR BootSector;
//     BIOS_PARAMETER_BLOCK BiosBlock;
//     ULONG BytesPerSector, SectorsPerTrack, Heads, BytesRead;
//     PVOID BaseAddress;
//     LARGE_INTEGER CurrentOffset, CylinderSize, DiskLength;
//     ULONG CylinderCount, SizeByCylinders;
//
//     /* Check if we're a boot RAM disk */
//     DiskType = Input->DiskType;
//     if (DiskType >= RAMDISK_BOOT_DISK)
//     {
//         /* Check if we're an ISO */
//         if (DiskType == RAMDISK_BOOT_DISK)
//         {
//             /* NTLDR mounted us somewhere */
//             BasePage = Input->BasePage;
//             if (!BasePage) return STATUS_INVALID_PARAMETER;
//
//             /* Sanitize disk options */
//             Input->Options.Fixed = TRUE;
//             Input->Options.Readonly = Input->Options.ExportAsCd |
//                                       Input->Options.Readonly;
//             Input->Options.Hidden = FALSE;
//             Input->Options.NoDosDevice = FALSE;
//             Input->Options.NoDriveLetter = IsWinPEBoot ? TRUE : FALSE;
//         }
//         else
//         {
//             /* The only other possibility is a WIM disk */
//             if (DiskType != RAMDISK_WIM_DISK)
//             {
//                 /* Fail */
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Read the view count instead */
//             // ViewCount = Input->ViewCount;
//
//             /* Sanitize disk options */
//             Input->Options.Hidden = FALSE;
//             Input->Options.NoDosDevice = FALSE;
//             Input->Options.Readonly = FALSE;
//             Input->Options.NoDriveLetter = TRUE;
//             Input->Options.Fixed = TRUE;
//         }
//
//         /* Are we just validating and returning to the user? */
//         if (ValidateOnly) return STATUS_SUCCESS;
//
//         /* Build the GUID string */
//         Status = RtlStringFromGUID(&Input->DiskGuid, &GuidString);
//         if (!(NT_SUCCESS(Status)) || !(GuidString.Buffer))
//         {
//             /* Fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto FailCreate;
//         }
//
//         /* Allocate our device name */
//         Length = GuidString.Length + 32;
//         Buffer = ExAllocatePoolWithTag(NonPagedPool, Length, 'dmaR');
//         if (!Buffer)
//         {
//             /* Fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto FailCreate;
//         }
//
//         /* Build the device name string */
//         DeviceName.Buffer = Buffer;
//         DeviceName.Length = Length - 2;
//         DeviceName.MaximumLength = Length;
//         wcsncpy(Buffer, L"\\Device\\Ramdisk", Length / sizeof(WCHAR));
//         wcsncat(Buffer, GuidString.Buffer, Length / sizeof(WCHAR));
//
//         /* Create the drive device */
//         Status = IoCreateDevice(DeviceExtension->DeviceObject->DriverObject,
//                                 sizeof(RAMDISK_DRIVE_EXTENSION),
//                                 &DeviceName,
//                                 (Input->Options.ExportAsCd) ?
//                                 FILE_DEVICE_CD_ROM : FILE_DEVICE_DISK,
//                                 0,
//                                 0,
//                                 &DeviceObject);
//         if (!NT_SUCCESS(Status)) goto FailCreate;
//
//         /* Grab the drive extension */
//         DriveExtension = DeviceObject->DeviceExtension;
//
//         /* Check if we need a DOS device */
//         if (!Input->Options.NoDosDevice)
//         {
//             /* Build the symbolic link name */
//             SymbolicLinkName.MaximumLength = GuidString.Length + 36;
//             SymbolicLinkName.Length = GuidString.Length + 34;
//             Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                            SymbolicLinkName.MaximumLength,
//                                            'dmaR');
//             SymbolicLinkName.Buffer = Buffer;
//             if (Buffer)
//             {
//                 /* Create it */
//                 wcsncpy(Buffer,
//                         L"\\GLOBAL??\\Ramdisk",
//                         SymbolicLinkName.MaximumLength / sizeof(WCHAR));
//                 wcsncat(Buffer,
//                         GuidString.Buffer,
//                         SymbolicLinkName.MaximumLength / sizeof(WCHAR));
//                 Status = IoCreateSymbolicLink(&SymbolicLinkName, &DeviceName);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Nevermind... */
//                     Input->Options.NoDosDevice = TRUE;
//                     ExFreePool(Buffer);
//                     SymbolicLinkName.Buffer = NULL;
//                 }
//             }
//             else
//             {
//                 /* No DOS device */
//                 Input->Options.NoDosDevice = TRUE;
//             }
//
//             /* Is this an ISO boot ramdisk? */
//             if (Input->DiskType == RAMDISK_BOOT_DISK)
//             {
//                 /* Does it need a drive letter? */
//                 if (!Input->Options.NoDriveLetter)
//                 {
//                     /* Build it and take over the existing symbolic link */
//                     _snwprintf(LocalBuffer,
//                                30,
//                                L"\\DosDevices\\%wc:",
//                                Input->DriveLetter);
//                     RtlInitUnicodeString(&DriveString, LocalBuffer);
//                     IoDeleteSymbolicLink(&DriveString);
//                     IoCreateSymbolicLink(&DriveString, &DeviceName);
//
//                     /* Save the drive letter */
//                     DriveExtension->DriveLetter = Input->DriveLetter;
//                 }
//             }
//
//         }
//
//         /* Setup the device object flags */
//         DeviceObject->Flags |= (DO_XIP | DO_POWER_PAGABLE | DO_DIRECT_IO);
//         DeviceObject->AlignmentRequirement = 1;
//
//         /* Build the drive FDO */
//         *NewDriveExtension = DriveExtension;
//         DriveExtension->Type = RamdiskDrive;
//         DiskLength = Input->DiskLength;
//         ExInitializeFastMutex(&DriveExtension->DiskListLock);
//         IoInitializeRemoveLock(&DriveExtension->RemoveLock, 'dmaR', 1, 0);
//         DriveExtension->DriveDeviceName = DeviceName;
//         DriveExtension->SymbolicLinkName = SymbolicLinkName;
//         DriveExtension->GuidString = GuidString;
//         DriveExtension->DiskGuid = Input->DiskGuid;
//         DriveExtension->PhysicalDeviceObject = DeviceObject;
//         DriveExtension->DeviceObject = RamdiskBusFdo;
//         DriveExtension->AttachedDevice = RamdiskBusFdo;
//         DriveExtension->DiskType = Input->DiskType;
//         DriveExtension->DiskOptions = Input->Options;
//         DriveExtension->DiskLength = DiskLength;
//         DriveExtension->DiskOffset = Input->DiskOffset;
//         DriveExtension->BasePage = Input->BasePage;
//         DriveExtension->BytesPerSector = 0;
//         DriveExtension->SectorsPerTrack = 0;
//         DriveExtension->NumberOfHeads = 0;
//
//         /* Make sure we don't free it later */
//         DeviceName.Buffer = NULL;
//         SymbolicLinkName.Buffer = NULL;
//         GuidString.Buffer = NULL;
//
//         /* Check if this is a boot disk, or a registry ram drive */
//         if (!(Input->Options.ExportAsCd) &&
//             (Input->DiskType == RAMDISK_BOOT_DISK))
//         {
//             /* Not an ISO boot, but it's a boot FS -- map it to figure out the
//              * drive settings */
//             CurrentOffset.QuadPart = 0;
//             BaseAddress = RamdiskMapPages(DriveExtension,
//                                           CurrentOffset,
//                                           PAGE_SIZE,
//                                           &BytesRead);
//             if (BaseAddress)
//             {
//                 /* Get the data */
//                 BootSector = (PPACKED_BOOT_SECTOR)BaseAddress;
//                 FatUnpackBios(&BiosBlock, &BootSector->PackedBpb);
//                 BytesPerSector = BiosBlock.BytesPerSector;
//                 SectorsPerTrack = BiosBlock.SectorsPerTrack;
//                 Heads = BiosBlock.Heads;
//
//                 /* Save it */
//                 DriveExtension->BytesPerSector = BytesPerSector;
//                 DriveExtension->SectorsPerTrack = SectorsPerTrack;
//                 DriveExtension->NumberOfHeads = Heads;
//
//                 /* Unmap now */
//                 CurrentOffset.QuadPart = 0;
//                 RamdiskUnmapPages(DriveExtension,
//                                   BaseAddress,
//                                   CurrentOffset,
//                                   BytesRead);
//             }
//             else
//             {
//                 /* Fail */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto FailCreate;
//             }
//         }
//
//         /* Check if the drive settings haven't been set yet */
//         if ((DriveExtension->BytesPerSector == 0) ||
//             (DriveExtension->SectorsPerTrack == 0) ||
//             (DriveExtension->NumberOfHeads == 0))
//         {
//             /* Check if this is a CD */
//             if (Input->Options.ExportAsCd)
//             {
//                 /* Setup partition parameters default for ISO 9660 */
//                 DriveExtension->BytesPerSector = 2048;
//                 DriveExtension->SectorsPerTrack = 32;
//                 DriveExtension->NumberOfHeads = 64;
//             }
//             else
//             {
//                 /* Setup partition parameters default for FAT */
//                 DriveExtension->BytesPerSector = 512;
//                 DriveExtension->SectorsPerTrack = 128;
//                 DriveExtension->NumberOfHeads = 16;
//             }
//         }
//
//         /* Calculate the cylinder size */
//         CylinderSize.QuadPart = DriveExtension->BytesPerSector *
//                                 DriveExtension->SectorsPerTrack *
//                                 DriveExtension->NumberOfHeads;
//         CylinderCount = DiskLength.QuadPart / CylinderSize.QuadPart;
//         SizeByCylinders = CylinderSize.QuadPart * CylinderCount;
//         DriveExtension->Cylinders = CylinderCount;
//         if ((DiskLength.HighPart > 0) || (SizeByCylinders < DiskLength.LowPart))
//         {
//             /* Align cylinder size up */
//             DriveExtension->Cylinders++;
//         }
//
//         /* Acquire the disk lock */
//         KeEnterCriticalRegion();
//         ExAcquireFastMutex(&DeviceExtension->DiskListLock);
//
//         /* Insert us */
//         InsertTailList(&DeviceExtension->DiskList, &DriveExtension->DiskList);
//
//         /* Release the lock */
//         ExReleaseFastMutex(&DeviceExtension->DiskListLock);
//         KeLeaveCriticalRegion();
//
//         /* Clear init flag */
//         DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//         return STATUS_SUCCESS;
//     }
//
// FailCreate:
//     UNIMPLEMENTED_DBGBREAK();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RamdiskCreateRamdisk(IN PDEVICE_OBJECT DeviceObject,
//                      IN PIRP Irp,
//                      IN BOOLEAN ValidateOnly)
// {
//     PRAMDISK_CREATE_INPUT Input;
//     ULONG Length;
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//     PRAMDISK_DRIVE_EXTENSION DriveExtension;
//     ULONG DiskType;
//     PWCHAR FileNameStart, FileNameEnd;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Get the device extension and our input data */
//     DeviceExtension = DeviceObject->DeviceExtension;
//     Length = IoStackLocation->Parameters.DeviceIoControl.InputBufferLength;
//     Input = (PRAMDISK_CREATE_INPUT)Irp->AssociatedIrp.SystemBuffer;
//
//     /* Validate input parameters */
//     if ((Length < sizeof(RAMDISK_CREATE_INPUT)) ||
//         (Input->Version != sizeof(RAMDISK_CREATE_INPUT)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate the disk type */
//     DiskType = Input->DiskType;
//     if (DiskType == RAMDISK_WIM_DISK) return STATUS_INVALID_PARAMETER;
//
//     /* Look at the disk type */
//     if (DiskType == RAMDISK_BOOT_DISK)
//     {
//         /* We only allow this as an early-init boot */
//         if (!KeLoaderBlock) return STATUS_INVALID_PARAMETER;
//
//         /* Save command-line flags */
//         if (ExportBootDiskAsCd) Input->Options.ExportAsCd = TRUE;
//         if (IsWinPEBoot) Input->Options.NoDriveLetter = TRUE;
//     }
//
//     /* Validate the disk type */
//     if ((Input->Options.ExportAsCd) && (DiskType != RAMDISK_BOOT_DISK))
//     {
//         /* If the type isn't CDFS, it has to at least be raw CD */
//         if (DiskType != RAMDISK_MEMORY_MAPPED_DISK) return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if this is an actual file */
//     if (DiskType <= RAMDISK_MEMORY_MAPPED_DISK)
//     {
//         /* Validate the file name */
//         FileNameStart = (PWCHAR)((ULONG_PTR)Input + Length);
//         FileNameEnd = Input->FileName + 1;
//         while ((FileNameEnd < FileNameStart) && *(FileNameEnd)) FileNameEnd++;
//         if (FileNameEnd == FileNameStart) return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the actual device */
//     Status = RamdiskCreateDiskDevice(DeviceExtension,
//                                      Input,
//                                      ValidateOnly,
//                                      &DriveExtension);
//     if (NT_SUCCESS(Status))
//     {
//         /* Invalidate and set success */
//         IoInvalidateDeviceRelations(DeviceExtension->PhysicalDeviceObject, 0);
//         Irp->IoStatus.Information = STATUS_SUCCESS;
//     }
//
//     /* We are done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskGetPartitionInfo(IN PIRP Irp,
//                         IN PRAMDISK_DRIVE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//     PPARTITION_INFORMATION PartitionInfo;
//     PVOID BaseAddress;
//     LARGE_INTEGER Zero = {{0, 0}};
//     ULONG Length;
//     PIO_STACK_LOCATION IoStackLocation;
//
//     /* Validate the length */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     if (IoStackLocation->Parameters.DeviceIoControl.
//         OutputBufferLength < sizeof(PARTITION_INFORMATION))
//     {
//         /* Invalid length */
//         Status = STATUS_BUFFER_TOO_SMALL;
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         return Status;
//     }
//
//     /* Map the partition table */
//     BaseAddress = RamdiskMapPages(DeviceExtension, Zero, PAGE_SIZE, &Length);
//     if (!BaseAddress)
//     {
//         /* No memory */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         return Status;
//     }
//
//     /* Fill out the information */
//     PartitionInfo = Irp->AssociatedIrp.SystemBuffer;
//     PartitionInfo->StartingOffset.QuadPart = DeviceExtension->BytesPerSector;
//     PartitionInfo->PartitionLength.QuadPart = DeviceExtension->BytesPerSector *
//                                               DeviceExtension->SectorsPerTrack *
//                                               DeviceExtension->NumberOfHeads *
//                                               DeviceExtension->Cylinders;
//     PartitionInfo->HiddenSectors = DeviceExtension->HiddenSectors;
//     PartitionInfo->PartitionNumber = 0;
//     PartitionInfo->PartitionType = *((PCHAR)BaseAddress + 450);
//     PartitionInfo->BootIndicator = (DeviceExtension->DiskType ==
//                                     RAMDISK_BOOT_DISK) ? TRUE: FALSE;
//     PartitionInfo->RecognizedPartition = IsRecognizedPartition(PartitionInfo->
//                                                                PartitionType);
//     PartitionInfo->RewritePartition = FALSE;
//
//     /* Unmap the partition table */
//     RamdiskUnmapPages(DeviceExtension, BaseAddress, Zero, Length);
//
//     /* Done */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = sizeof(PARTITION_INFORMATION);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RamdiskSetPartitionInfo(IN PIRP Irp,
//                         IN PRAMDISK_DRIVE_EXTENSION DeviceExtension)
// {
//     ULONG BytesRead;
//     NTSTATUS Status;
//     PVOID BaseAddress;
//     PIO_STACK_LOCATION Stack;
//     LARGE_INTEGER Zero = {{0, 0}};
//     PPARTITION_INFORMATION PartitionInfo;
//
//     /* First validate input */
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(PARTITION_INFORMATION))
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto SetAndQuit;
//     }
//
//     /* Map to get MBR */
//     BaseAddress = RamdiskMapPages(DeviceExtension, Zero, PAGE_SIZE, &BytesRead);
//     if (BaseAddress == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto SetAndQuit;
//     }
//
//     /* Set the new partition type on partition 0, field system indicator */
//     PartitionInfo = (PPARTITION_INFORMATION)Irp->AssociatedIrp.SystemBuffer;
//     *((PCHAR)BaseAddress + 450) = PartitionInfo->PartitionType;
//
//     /* And unmap */
//     RamdiskUnmapPages(DeviceExtension, BaseAddress, Zero, BytesRead);
//     Status = STATUS_SUCCESS;
//
// SetAndQuit:
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = 0;
//     return Status;
// }
//
// VOID
// NTAPI
// RamdiskWorkerThread(IN PDEVICE_OBJECT DeviceObject,
//                     IN PVOID Context)
// {
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation;
//     PIRP Irp = Context;
//
//     /* Get the stack location */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Free the work item */
//     IoFreeWorkItem(Irp->Tail.Overlay.DriverContext[0]);
//
//     /* Grab the device extension and lock it */
//     DeviceExtension = DeviceObject->DeviceExtension;
//     Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     if (NT_SUCCESS(Status))
//     {
//         /* Discriminate by major code */
//         switch (IoStackLocation->MajorFunction)
//         {
//             /* Device control */
//             case IRP_MJ_DEVICE_CONTROL:
//             {
//                 /* Let's take a look at the IOCTL */
//                 switch (IoStackLocation->Parameters.DeviceIoControl.IoControlCode)
//                 {
//                     /* Ramdisk create request */
//                     case FSCTL_CREATE_RAM_DISK:
//                     {
//                         /* This time we'll do it for real */
//                         Status = RamdiskCreateRamdisk(DeviceObject, Irp, FALSE);
//                         break;
//                     }
//
//                     case IOCTL_DISK_SET_PARTITION_INFO:
//                     {
//                         Status = RamdiskSetPartitionInfo(Irp, (PRAMDISK_DRIVE_EXTENSION)DeviceExtension);
//                         break;
//                     }
//
//                     case IOCTL_DISK_GET_DRIVE_LAYOUT:
//                         UNIMPLEMENTED_DBGBREAK("Get drive layout request\n");
//                         break;
//
//                     case IOCTL_DISK_GET_PARTITION_INFO:
//                     {
//                         Status = RamdiskGetPartitionInfo(Irp, (PRAMDISK_DRIVE_EXTENSION)DeviceExtension);
//                         break;
//                     }
//
//                     default:
//                         UNIMPLEMENTED_DBGBREAK("Invalid request\n");
//                         break;
//                 }
//
//                 /* We're here */
//                 break;
//             }
//
//             /* Read or write request */
//             case IRP_MJ_READ:
//             case IRP_MJ_WRITE:
//                 UNIMPLEMENTED_DBGBREAK("Read/Write request\n");
//                 break;
//
//             /* Internal request (SCSI?) */
//             case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//                 UNIMPLEMENTED_DBGBREAK("SCSI request\n");
//                 break;
//
//             /* Flush request */
//             case IRP_MJ_FLUSH_BUFFERS:
//                 UNIMPLEMENTED_DBGBREAK("Flush request\n");
//                 break;
//
//             /* Anything else */
//             default:
//                 UNIMPLEMENTED_DBGBREAK("Invalid request: %lx\n",
//                                        IoStackLocation->MajorFunction);
//                 break;
//         }
//
//         /* Complete the I/O */
//         IoReleaseRemoveLock(&DeviceExtension->RemoveLock, Irp);
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return;
//     }
//
//     /* Fail the I/O */
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// NTSTATUS
// NTAPI
// SendIrpToThread(IN PDEVICE_OBJECT DeviceObject,
//                 IN PIRP Irp)
// {
//     PIO_WORKITEM WorkItem;
//
//     /* Mark the IRP pending */
//     IoMarkIrpPending(Irp);
//
//     /* Allocate a work item */
//     WorkItem = IoAllocateWorkItem(DeviceObject);
//     if (WorkItem)
//     {
//         /* Queue it up */
//         Irp->Tail.Overlay.DriverContext[0] = WorkItem;
//         IoQueueWorkItem(WorkItem, RamdiskWorkerThread, DelayedWorkQueue, Irp);
//         return STATUS_PENDING;
//     }
//     else
//     {
//         /* Fail */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
// }
//
// NTSTATUS
// NTAPI
// RamdiskReadWriteReal(IN PIRP Irp,
//                      IN PRAMDISK_DRIVE_EXTENSION DeviceExtension)
// {
//     PMDL Mdl;
//     PVOID CurrentBase, SystemVa, BaseAddress;
//     PIO_STACK_LOCATION IoStackLocation;
//     LARGE_INTEGER CurrentOffset;
//     ULONG BytesRead, BytesLeft, CopyLength;
//     PVOID Source, Destination;
//     NTSTATUS Status;
//
//     /* Get the MDL and check if it's mapped */
//     Mdl = Irp->MdlAddress;
//     if (Mdl->MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL))
//     {
//         /* Use the mapped address */
//         SystemVa = Mdl->MappedSystemVa;
//     }
//     else
//     {
//         /* Map it ourselves */
//         SystemVa = MmMapLockedPagesSpecifyCache(Mdl,
//                                                 0,
//                                                 MmCached,
//                                                 NULL,
//                                                 0,
//                                                 NormalPagePriority);
//     }
//
//     /* Make sure we were able to map it */
//     CurrentBase = SystemVa;
//     if (!SystemVa) { RD_TRACE("[RAMDISK-DRV] RWReal: no SystemVa\r\n"); return STATUS_INSUFFICIENT_RESOURCES; }
//
//     RD_TRACE("[RAMDISK-DRV] RWReal: start\r\n");
//
//     /* Initialize default */
//     Irp->IoStatus.Information = 0;
//
//     /* Get the I/O Stack Location and capture the data */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     CurrentOffset = IoStackLocation->Parameters.Read.ByteOffset;
//     BytesLeft = IoStackLocation->Parameters.Read.Length;
//     if (!BytesLeft) return STATUS_INVALID_PARAMETER;
//
//     /* Do the copy loop */
//     while (TRUE)
//     {
//         /* Map the pages */
//         BaseAddress = RamdiskMapPages(DeviceExtension,
//                                       CurrentOffset,
//                                       BytesLeft,
//                                       &BytesRead);
//         if (!BaseAddress) { RD_TRACE("[RAMDISK-DRV] RWReal: MapPages failed\r\n"); return STATUS_INSUFFICIENT_RESOURCES; }
//
//         /* Update our lengths */
//         Irp->IoStatus.Information += BytesRead;
//         CopyLength = BytesRead;
//
//         /* Check if this was a read or write */
//         Status = STATUS_SUCCESS;
//         if (IoStackLocation->MajorFunction == IRP_MJ_READ)
//         {
//             /* Set our copy parameters */
//             Destination = CurrentBase;
//             Source = BaseAddress;
//             goto DoCopy;
//         }
//         else if (IoStackLocation->MajorFunction == IRP_MJ_WRITE)
//         {
//             /* Set our copy parameters */
//             Destination = BaseAddress;
//             Source = CurrentBase;
// DoCopy:
//             /* Copy the data */
//             RtlCopyMemory(Destination, Source, CopyLength);
//         }
//         else
//         {
//             /* Prepare us for failure */
//             BytesLeft = CopyLength;
//             Status = STATUS_INVALID_PARAMETER;
//         }
//
//         /* Unmap the pages */
//         RamdiskUnmapPages(DeviceExtension, BaseAddress, CurrentOffset, BytesRead);
//
//         /* Update offset and bytes left */
//         BytesLeft -= BytesRead;
//         CurrentOffset.QuadPart += BytesRead;
//         CurrentBase = (PVOID)((ULONG_PTR)CurrentBase + BytesRead);
//
//         /* Check if we are done */
//         if (!BytesLeft) { RD_TRACE("[RAMDISK-DRV] RWReal: done\r\n"); return Status; }
//     }
// }
//
// NTSTATUS
// NTAPI
// RamdiskOpenClose(IN PDEVICE_OBJECT DeviceObject,
//                  IN PIRP Irp)
// {
//     RD_TRACE("[RAMDISK-DRV] OpenClose\r\n");
//     /* Complete the IRP */
//     Irp->IoStatus.Information = 1;
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RamdiskReadWrite(IN PDEVICE_OBJECT DeviceObject,
//                  IN PIRP Irp)
// {
//     PRAMDISK_DRIVE_EXTENSION DeviceExtension;
//     // ULONG Length;
//     // LARGE_INTEGER ByteOffset;
//     PIO_STACK_LOCATION IoStackLocation;
//     NTSTATUS Status, ReturnStatus;
//
//     RD_TRACE("[RAMDISK-DRV] ReadWrite\r\n");
//
//     /* Get the device extension and make sure this isn't a bus */
//     DeviceExtension = DeviceObject->DeviceExtension;
//     if (DeviceExtension->Type == RamdiskBus)
//     {
//         /* Fail */
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         goto Complete;
//     }
//
//     /* Capture parameters */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     // Length = IoStackLocation->Parameters.Read.Length;
//     // ByteOffset = IoStackLocation->Parameters.Read.ByteOffset;
//
//     /* FIXME: Validate offset */
//
//     /* FIXME: Validate sector */
//
//     /* Validate write */
//     if ((IoStackLocation->MajorFunction == IRP_MJ_WRITE) &&
//         (DeviceExtension->DiskOptions.Readonly))
//     {
//         /* Fail, this is read-only */
//         Status = STATUS_MEDIA_WRITE_PROTECTED;
//         goto Complete;
//     }
//
//     /* See if we want to do this sync or async */
//     if (DeviceExtension->DiskType > RAMDISK_MEMORY_MAPPED_DISK)
//     {
//         /* Do it sync */
//         Status = RamdiskReadWriteReal(Irp, DeviceExtension);
//         goto Complete;
//     }
//
//     /* Queue it to the worker */
//     Status = SendIrpToThread(DeviceObject, Irp);
//     ReturnStatus = STATUS_PENDING;
//
//     /* Check if we're pending or not */
//     if (Status != STATUS_PENDING)
//     {
// Complete:
//         /* Complete the IRP */
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         ReturnStatus = Status;
//     }
//
//     /* Return to caller */
//     return ReturnStatus;
// }
//
// NTSTATUS
// NTAPI
// RamdiskDeviceControl(IN PDEVICE_OBJECT DeviceObject,
//                      IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     PRAMDISK_BUS_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
//     PRAMDISK_DRIVE_EXTENSION DriveExtension = (PVOID)DeviceExtension;
//     ULONG Information;
//     PCDROM_TOC Toc;
//     PDISK_GEOMETRY DiskGeometry;
//
//     /* Grab the remove lock */
//     Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail the IRP */
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     /* Setup some defaults */
//     Status = STATUS_INVALID_DEVICE_REQUEST;
//     Information = 0;
//
//     /* Check if this is an bus device or the drive */
//     if (DeviceExtension->Type == RamdiskBus)
//     {
//         /* Check what the request is */
//         switch (IoStackLocation->Parameters.DeviceIoControl.IoControlCode)
//         {
//             /* Request to create a ramdisk */
//             case FSCTL_CREATE_RAM_DISK:
//             {
//                 /* Do it */
//                 Status = RamdiskCreateRamdisk(DeviceObject, Irp, TRUE);
//                 if (!NT_SUCCESS(Status)) goto CompleteRequest;
//                 break;
//             }
//
//             default:
//             {
//                 /* We don't handle anything else yet */
//                 UNIMPLEMENTED_DBGBREAK("FSCTL: 0x%lx is UNSUPPORTED!\n",
//                                        IoStackLocation->Parameters.DeviceIoControl.IoControlCode);
//             }
//         }
//     }
//     else
//     {
//         /* Check what the request is */
//         switch (IoStackLocation->Parameters.DeviceIoControl.IoControlCode)
//         {
//             case IOCTL_DISK_CHECK_VERIFY:
//             case IOCTL_STORAGE_CHECK_VERIFY:
//             case IOCTL_STORAGE_CHECK_VERIFY2:
//             case IOCTL_CDROM_CHECK_VERIFY:
//             {
//                 /* Just pretend it's OK, don't do more */
//                 Status = STATUS_SUCCESS;
//                 break;
//             }
//
//             case IOCTL_STORAGE_GET_MEDIA_TYPES:
//             case IOCTL_DISK_GET_MEDIA_TYPES:
//             case IOCTL_DISK_GET_DRIVE_GEOMETRY:
//             case IOCTL_CDROM_GET_DRIVE_GEOMETRY:
//             {
//                 /* Validate the length */
//                 if (IoStackLocation->Parameters.DeviceIoControl.
//                     OutputBufferLength < sizeof(DISK_GEOMETRY))
//                 {
//                     /* Invalid length */
//                     Status = STATUS_BUFFER_TOO_SMALL;
//                     break;
//                 }
//
//                 /* Fill it out */
//                 DiskGeometry = Irp->AssociatedIrp.SystemBuffer;
//                 DiskGeometry->Cylinders.QuadPart = DriveExtension->Cylinders;
//                 DiskGeometry->BytesPerSector = DriveExtension->BytesPerSector;
//                 DiskGeometry->SectorsPerTrack = DriveExtension->SectorsPerTrack;
//                 DiskGeometry->TracksPerCylinder = DriveExtension->NumberOfHeads;
//                 DiskGeometry->MediaType = DriveExtension->DiskOptions.Fixed ?
//                                           FixedMedia : RemovableMedia;
//
//                 /* We are done */
//                 Status = STATUS_SUCCESS;
//                 Information = sizeof(DISK_GEOMETRY);
//                 break;
//             }
//
//             case IOCTL_CDROM_READ_TOC:
//             {
//                 /* Validate the length */
//                 if (IoStackLocation->Parameters.DeviceIoControl.
//                     OutputBufferLength < sizeof(CDROM_TOC))
//                 {
//                     /* Invalid length */
//                     Status = STATUS_BUFFER_TOO_SMALL;
//                     break;
//                 }
//
//                 /* Clear the TOC */
//                 Toc = Irp->AssociatedIrp.SystemBuffer;
//                 RtlZeroMemory(Toc, sizeof(CDROM_TOC));
//
//                 /* Fill it out */
//                 Toc->Length[0] = 0;
//                 Toc->Length[1] = RAMDISK_TOC_SIZE - sizeof(Toc->Length);
//                 Toc->FirstTrack = 1;
//                 Toc->LastTrack = 1;
//                 Toc->TrackData[0].Adr = 1;
//                 Toc->TrackData[0].Control = TOC_DATA_TRACK;
//                 Toc->TrackData[0].TrackNumber = 1;
//
//                 /* We are done */
//                 Status = STATUS_SUCCESS;
//                 Information = RAMDISK_TOC_SIZE;
//                 break;
//             }
//
//             case IOCTL_DISK_SET_PARTITION_INFO:
//             {
//                 Status = RamdiskSetPartitionInfo(Irp, DriveExtension);
//                 break;
//             }
//
//             case IOCTL_DISK_GET_PARTITION_INFO:
//             {
//                 /* Validate the length */
//                 if (IoStackLocation->Parameters.DeviceIoControl.
//                     OutputBufferLength < sizeof(PARTITION_INFORMATION))
//                 {
//                     /* Invalid length */
//                     Status = STATUS_BUFFER_TOO_SMALL;
//                     break;
//                 }
//
//                 /* Check if we need to do this sync or async */
//                 if (DriveExtension->DiskType > RAMDISK_MEMORY_MAPPED_DISK)
//                 {
//                     /* Call the helper function */
//                     Status = RamdiskGetPartitionInfo(Irp, DriveExtension);
//                 }
//                 else
//                 {
//                     /* Do it asynchronously later */
//                     goto CallWorker;
//                 }
//
//                 /* We are done */
//                 Information = Irp->IoStatus.Information;
//                 break;
//             }
//
//             case IOCTL_DISK_GET_LENGTH_INFO:
//             {
//                 PGET_LENGTH_INFORMATION LengthInformation = Irp->AssociatedIrp.SystemBuffer;
//
//                 /* Validate the length */
//                 if (IoStackLocation->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GET_LENGTH_INFORMATION))
//                 {
//                     /* Invalid length */
//                     Status = STATUS_BUFFER_TOO_SMALL;
//                     break;
//                 }
//
//                 /* Fill it out */
//                 LengthInformation->Length = DriveExtension->DiskLength;
//
//                 /* We are done */
//                 Status = STATUS_SUCCESS;
//                 Information = sizeof(GET_LENGTH_INFORMATION);
//                 break;
//             }
//             case IOCTL_VOLUME_GET_GPT_ATTRIBUTES:
//             {
//                 PVOLUME_GET_GPT_ATTRIBUTES_INFORMATION GptInformation;
//
//                 /* Validate the length */
//                 if (IoStackLocation->Parameters.DeviceIoControl.OutputBufferLength < sizeof(VOLUME_GET_GPT_ATTRIBUTES_INFORMATION))
//                 {
//                     /* Invalid length */
//                     Status = STATUS_BUFFER_TOO_SMALL;
//                     break;
//                 }
//
//                 /* Fill it out */
//                 GptInformation = Irp->AssociatedIrp.SystemBuffer;
//                 GptInformation->GptAttributes = 0;
//
//                 /* Translate the Attributes */
//                 if (DriveExtension->DiskOptions.Readonly)
//                     GptInformation->GptAttributes |= GPT_BASIC_DATA_ATTRIBUTE_READ_ONLY;
//                 if (DriveExtension->DiskOptions.Hidden)
//                     GptInformation->GptAttributes |= GPT_BASIC_DATA_ATTRIBUTE_HIDDEN;
//                 if (DriveExtension->DiskOptions.NoDriveLetter)
//                     GptInformation->GptAttributes |= GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER;
//
//                 /* We are done */
//                 Status = STATUS_SUCCESS;
//                 Information = sizeof(VOLUME_GET_GPT_ATTRIBUTES_INFORMATION);
//                 break;
//             }
//
//             case IOCTL_DISK_GET_DRIVE_LAYOUT:
//             case IOCTL_DISK_IS_WRITABLE:
//             case IOCTL_SCSI_MINIPORT:
//             case IOCTL_STORAGE_QUERY_PROPERTY:
//             case IOCTL_MOUNTDEV_QUERY_UNIQUE_ID:
//             case IOCTL_MOUNTDEV_QUERY_STABLE_GUID:
//             case IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS:
//             case IOCTL_VOLUME_SET_GPT_ATTRIBUTES:
//             case IOCTL_VOLUME_OFFLINE:
//             {
//                 UNIMPLEMENTED_DBGBREAK("IOCTL: 0x%lx is UNIMPLEMENTED!\n",
//                                        IoStackLocation->Parameters.DeviceIoControl.IoControlCode);
//                 break;
//             }
//
//             default:
//             {
//                 /* Drive code not emulated */
//                 DPRINT1("IOCTL: 0x%lx is UNSUPPORTED!\n",
//                         IoStackLocation->Parameters.DeviceIoControl.IoControlCode);
//                 break;
//             }
//         }
//
//         /* If requests drop down here, we just return them complete them */
//         goto CompleteRequest;
//     }
//
//     /* Queue the request to our worker thread */
// CallWorker:
//     Status = SendIrpToThread(DeviceObject, Irp);
//
// CompleteRequest:
//     /* Release the lock */
//     IoReleaseRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     if (Status != STATUS_PENDING)
//     {
//         /* Complete the request */
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = Information;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskQueryDeviceRelations(IN DEVICE_RELATION_TYPE Type,
//                             IN PDEVICE_OBJECT DeviceObject,
//                             IN PIRP Irp)
// {
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//     PRAMDISK_DRIVE_EXTENSION DriveExtension;
//     PDEVICE_RELATIONS DeviceRelations, OurDeviceRelations;
//     ULONG Count, DiskCount, FinalCount;
//     PLIST_ENTRY ListHead, NextEntry;
//     PDEVICE_OBJECT* DriveDeviceObject;
//     RAMDISK_DEVICE_STATE State;
//
//     /* Get the device extension and check if this is a drive */
//     DeviceExtension = DeviceObject->DeviceExtension;
//     if (DeviceExtension->Type == RamdiskDrive)
//     {
//         NTSTATUS Status;
//         PDEVICE_RELATIONS DeviceRelations;
//
//         /* We're a child device, only handle target device relations */
//         if (Type != TargetDeviceRelation)
//         {
//             Status = Irp->IoStatus.Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//
//         /* Allocate a buffer big enough to contain only one DO */
//         DeviceRelations = ExAllocatePoolWithTag(PagedPool,
//                                                 sizeof(*DeviceRelations),
//                                                 'dmaR');
//         if (DeviceRelations != NULL)
//         {
//             /* Reference the DO and add it to the buffer */
//             ObReferenceObject(DeviceObject);
//             DeviceRelations->Objects[0] = DeviceObject;
//             DeviceRelations->Count = 1;
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Return our processing & complete */
//         Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     /* We don't handle anything but bus relations */
//     if (Type != BusRelations) goto PassToNext;
//
//     /* Acquire the disk list lock */
//     KeEnterCriticalRegion();
//     ExAcquireFastMutex(&DeviceExtension->DiskListLock);
//
//     /* Did a device already fill relations? */
//     DeviceRelations = (PDEVICE_RELATIONS)Irp->IoStatus.Information;
//     if (DeviceRelations)
//     {
//         /* Use the data */
//         Count = DeviceRelations->Count;
//     }
//     else
//     {
//         /* We're the first */
//         Count = 0;
//     }
//
//     /* Now loop our drives */
//     DiskCount = 0;
//     ListHead = &DeviceExtension->DiskList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* As long as it wasn't removed, count it in */
//         DriveExtension = CONTAINING_RECORD(NextEntry,
//                                            RAMDISK_DRIVE_EXTENSION,
//                                            DiskList);
//         if (DriveExtension->State < RamdiskStateBusRemoved) DiskCount++;
//
//         /* Move to the next one */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Now we know our final count */
//     FinalCount = Count + DiskCount;
//
//     /* Allocate the structure */
//     OurDeviceRelations = ExAllocatePoolWithTag(PagedPool,
//                                                FIELD_OFFSET(DEVICE_RELATIONS,
//                                                             Objects) +
//                                                FinalCount *
//                                                sizeof(PDEVICE_OBJECT),
//                                                'dmaR');
//     if (!OurDeviceRelations)
//     {
//         /* Fail */
//         ExReleaseFastMutex(&DeviceExtension->DiskListLock);
//         KeLeaveCriticalRegion();
//         Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Check if we already had some relations */
//     if (Count)
//     {
//         /* Copy them in */
//         RtlCopyMemory(OurDeviceRelations->Objects,
//                       DeviceRelations->Objects,
//                       Count * sizeof(PDEVICE_OBJECT));
//     }
//
//     /* Save the count */
//     OurDeviceRelations->Count = FinalCount;
//
//     /* Now loop our drives again */
//     ListHead = &DeviceExtension->DiskList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* Go to the end of the list */
//         DriveDeviceObject = &OurDeviceRelations->Objects[Count];
//
//         /* Get the drive state */
//         DriveExtension = CONTAINING_RECORD(NextEntry,
//                                            RAMDISK_DRIVE_EXTENSION,
//                                            DiskList);
//         State = DriveExtension->State;
//
//         /* If it was removed or enumerated, we don't touch the device object */
//         if (State >= RamdiskStateBusRemoved)
//         {
//             /* If it was removed, we still have to keep track of this though */
//             if (State == RamdiskStateBusRemoved)
//             {
//                 /* Mark it as enumerated now, but don't actually reference it */
//                 DriveExtension->State = RamdiskStateEnumerated;
//             }
//         }
//         else
//         {
//             /* First time it's enumerated, reference the device object */
//             ObReferenceObject(DriveExtension->DeviceObject);
//
//             /* Save the object pointer and move on */
//             *DriveDeviceObject++ = DriveExtension->PhysicalDeviceObject;
//         }
//
//         if (DriveExtension->State < RamdiskStateBusRemoved) DiskCount++;
//
//         /* Move to the next one */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the lock */
//     ExReleaseFastMutex(&DeviceExtension->DiskListLock);
//     KeLeaveCriticalRegion();
//
//     /* Cleanup old relations */
//     if (DeviceRelations) ExFreePool(DeviceRelations);
//
//     /* Complete our IRP */
//     Irp->IoStatus.Information = (ULONG_PTR)OurDeviceRelations;
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* Pass to the next driver */
// PassToNext:
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//     return IoCallDriver(DeviceExtension->AttachedDevice, Irp);
// }
//
// NTSTATUS
// NTAPI
// RamdiskDeleteDiskDevice(IN PDEVICE_OBJECT DeviceObject,
//                         IN PIRP Irp)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RamdiskRemoveBusDevice(IN PDEVICE_OBJECT DeviceObject,
//                        IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY ListHead, NextEntry;
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//     PRAMDISK_DRIVE_EXTENSION DriveExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* Acquire disks list lock */
//     KeEnterCriticalRegion();
//     ExAcquireFastMutex(&DeviceExtension->DiskListLock);
//
//     /* Loop over drives */
//     ListHead = &DeviceExtension->DiskList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         DriveExtension = CONTAINING_RECORD(NextEntry,
//                                            RAMDISK_DRIVE_EXTENSION,
//                                            DiskList);
//
//         /* Delete the disk */
//         IoAcquireRemoveLock(&DriveExtension->RemoveLock, NULL);
//         RamdiskDeleteDiskDevice(DriveExtension->PhysicalDeviceObject, NULL);
//
//         /* RamdiskDeleteDiskDevice releases list lock, so reacquire it */
//         KeEnterCriticalRegion();
//         ExAcquireFastMutex(&DeviceExtension->DiskListLock);
//     }
//
//     /* Release disks list lock */
//     ExReleaseFastMutex(&DeviceExtension->DiskListLock);
//     KeLeaveCriticalRegion();
//
//     /* Prepare to pass to the lower driver */
//     IoSkipCurrentIrpStackLocation(Irp);
//     /* Here everything went fine */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* Call lower driver */
//     Status = IoCallDriver(DeviceExtension->AttachedDevice, Irp);
//
//     /* Update state */
//     DeviceExtension->State = RamdiskStateBusRemoved;
//
//     /* Release the lock and ensure that everyone has finished its job before
//      * we continue. The lock has been acquired by the dispatcher */
//     IoReleaseRemoveLockAndWait(&DeviceExtension->RemoveLock, Irp);
//
//     /* If there's a drive name */
//     if (DeviceExtension->DriveDeviceName.Buffer)
//     {
//         /* Inform it's going to be disabled and free the drive name */
//         IoSetDeviceInterfaceState(&DeviceExtension->DriveDeviceName, FALSE);
//         RtlFreeUnicodeString(&DeviceExtension->DriveDeviceName);
//     }
//
//     /* Part from the stack, detach from lower device */
//     IoDetachDevice(DeviceExtension->AttachedDevice);
//
//     /* Finally, delete device */
//     RamdiskBusFdo = NULL;
//     IoDeleteDevice(DeviceObject);
//
//     /* Return status from lower driver */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskQueryId(IN PRAMDISK_DRIVE_EXTENSION DriveExtension,
//                IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation;
//     PWSTR OutputString = NULL;
//     ULONG StringLength;
//
//     Status = STATUS_SUCCESS;
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Get what is being queried */
//     switch (IoStackLocation->Parameters.QueryId.IdType)
//     {
//         case BusQueryDeviceID:
//         {
//             /* Allocate a buffer long enough to receive Ramdisk\RamDisk in any case
//              * In case we don't have RAMDISK_REGISTRY_DISK, we then need two more
//              * chars to store Ramdisk\RamVolume instead */
//             StringLength = 4 * (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK) + sizeof(L"Ramdisk\\RamDisk");
//             OutputString = ExAllocatePoolWithTag(PagedPool, StringLength, 'dmaR');
//             if (OutputString == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             wcsncpy(OutputString, L"Ramdisk\\", StringLength / sizeof(WCHAR));
//             if (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK)
//             {
//                 wcsncat(OutputString, L"RamVolume", StringLength / sizeof(WCHAR));
//             }
//             else
//             {
//                 wcsncat(OutputString, L"RamDisk", StringLength / sizeof(WCHAR));
//             }
//
//             break;
//         }
//
//         case BusQueryHardwareIDs:
//         {
//             /* Allocate a buffer long enough to receive Ramdisk\RamDisk in any case
//              * In case we don't have RAMDISK_REGISTRY_DISK, we then need two more
//              * chars to store Ramdisk\RamVolume instead
//              * We also need an extra char, because it is required that the string
//              * is null-terminated twice */
//             StringLength = 4 * (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK) +
//                            sizeof(UNICODE_NULL) + sizeof(L"Ramdisk\\RamDisk");
//             OutputString = ExAllocatePoolWithTag(PagedPool, StringLength, 'dmaR');
//             if (OutputString == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             wcsncpy(OutputString, L"Ramdisk\\", StringLength / sizeof(WCHAR));
//             if (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK)
//             {
//                 wcsncat(OutputString, L"RamVolume", StringLength / sizeof(WCHAR));
//             }
//             else
//             {
//                 wcsncat(OutputString, L"RamDisk", StringLength / sizeof(WCHAR));
//             }
//             OutputString[(StringLength / sizeof(WCHAR)) - 1] = UNICODE_NULL;
//
//             break;
//         }
//
//         case BusQueryCompatibleIDs:
//         {
//             if (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK)
//             {
//                 Status = STATUS_INVALID_DEVICE_REQUEST;
//                 break;
//             }
//
//             StringLength = sizeof(L"GenDisk");
//             OutputString = ExAllocatePoolWithTag(PagedPool, StringLength, 'dmaR');
//             if (OutputString == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             wcsncpy(OutputString, L"GenDisk", StringLength / sizeof(WCHAR));
//             OutputString[(StringLength / sizeof(WCHAR)) - 1] = UNICODE_NULL;
//
//             break;
//         }
//
//         case BusQueryInstanceID:
//         {
//             OutputString = ExAllocatePoolWithTag(PagedPool,
//                                                  DriveExtension->GuidString.MaximumLength,
//                                                  'dmaR');
//             if (OutputString == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             wcsncpy(OutputString,
//                     DriveExtension->GuidString.Buffer,
//                     DriveExtension->GuidString.MaximumLength / sizeof(WCHAR));
//
//             break;
//         }
//
//         case BusQueryDeviceSerialNumber:
//         case BusQueryContainerID:
//         {
//             /* Nothing to do */
//             break;
//         }
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = (ULONG_PTR)OutputString;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskQueryCapabilities(IN PDEVICE_OBJECT DeviceObject,
//                          IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation;
//     PDEVICE_CAPABILITIES DeviceCapabilities;
//     PRAMDISK_DRIVE_EXTENSION DriveExtension;
//
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     DeviceCapabilities = IoStackLocation->Parameters.DeviceCapabilities.Capabilities;
//     DriveExtension = DeviceObject->DeviceExtension;
//
//     /* Validate our input buffer */
//     if (DeviceCapabilities->Version != 1 ||
//         DeviceCapabilities->Size < sizeof(DEVICE_CAPABILITIES))
//     {
//         Status = STATUS_UNSUCCESSFUL;
//     }
//     else
//     {
//         /* And set everything we know about our capabilities */
//         DeviceCapabilities->Removable = MarkRamdisksAsRemovable;
//         DeviceCapabilities->UniqueID = TRUE;
//         DeviceCapabilities->SilentInstall = TRUE;
//         DeviceCapabilities->RawDeviceOK = TRUE;
//         DeviceCapabilities->SurpriseRemovalOK = (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK);
//         DeviceCapabilities->NoDisplayInUI = TRUE;
//         Status = STATUS_SUCCESS;
//     }
//
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskQueryDeviceText(IN PRAMDISK_DRIVE_EXTENSION DriveExtension,
//                        IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation;
//     DEVICE_TEXT_TYPE DeviceTextType;
//     PWSTR OutputString = NULL;
//
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     DeviceTextType = IoStackLocation->Parameters.QueryDeviceText.DeviceTextType;
//     Status = STATUS_SUCCESS;
//
//     /* Just copy our constants, according to the input */
//     switch (DeviceTextType)
//     {
//         case DeviceTextDescription:
//         {
//             OutputString = ExAllocatePoolWithTag(PagedPool, sizeof(L"RamDisk"), 'dmaR');
//             if (OutputString == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             wcsncpy(OutputString, L"RamDisk", sizeof(L"RamDisk") / sizeof(WCHAR));
//
//             break;
//         }
//
//         case DeviceTextLocationInformation:
//         {
//             OutputString = ExAllocatePoolWithTag(PagedPool, sizeof(L"RamDisk\\0"), 'dmaR');
//             if (OutputString == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             wcsncpy(OutputString, L"RamDisk\\0", sizeof(L"RamDisk\\0") / sizeof(WCHAR));
//
//             break;
//         }
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = (ULONG_PTR)OutputString;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskQueryBusInformation(IN PDEVICE_OBJECT DeviceObject,
//                            IN PIRP Irp)
// {
//     PPNP_BUS_INFORMATION PnpBusInfo;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Allocate output memory */
//     PnpBusInfo = ExAllocatePoolWithTag(PagedPool, sizeof(*PnpBusInfo), 'dmaR');
//     if (PnpBusInfo == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         /* Copy our bus GUID and set our legacy type */
//         RtlCopyMemory(&PnpBusInfo->BusTypeGuid, &GUID_BUS_TYPE_RAMDISK, sizeof(GUID));
//         PnpBusInfo->LegacyBusType = PNPBus;
//         PnpBusInfo->BusNumber = 0;
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = (ULONG_PTR)PnpBusInfo;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskIoCompletionRoutine(IN PDEVICE_OBJECT DeviceObject,
//                            IN PIRP Irp,
//                            IN PVOID Context)
//
// {
//     /* Just set the event to unlock caller */
//     KeSetEvent((PKEVENT)Context, 0, FALSE);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// RamdiskPnp(IN PDEVICE_OBJECT DeviceObject,
//            IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStackLocation;
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     UCHAR Minor;
//     KEVENT Event;
//
//     /* Get the device extension and stack location */
//     DeviceExtension = DeviceObject->DeviceExtension;
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     Minor = IoStackLocation->MinorFunction;
//
//     /* Check if the bus is removed */
//     if (DeviceExtension->State == RamdiskStateBusRemoved)
//     {
//         /* Only remove-device and query-id are allowed */
//         if ((Minor != IRP_MN_REMOVE_DEVICE) && (Minor != IRP_MN_QUERY_ID))
//         {
//             /* Fail anything else */
//             Status = STATUS_NO_SUCH_DEVICE;
//             Irp->IoStatus.Status = Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//     }
//
//     /* Acquire the remove lock */
//     Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail the IRP */
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     /* Query the IRP type */
//     switch (Minor)
//     {
//         case IRP_MN_START_DEVICE:
//         {
//             if (DeviceExtension->Type == RamdiskDrive)
//             {
//                 ULONG ResultLength;
//                 DEVICE_INSTALL_STATE InstallState;
//                 PRAMDISK_DRIVE_EXTENSION DriveExtension = (PRAMDISK_DRIVE_EXTENSION)DeviceExtension;
//
//                 /* If we already have a drive name, free it */
//                 if (DriveExtension->DriveDeviceName.Buffer)
//                 {
//                     ExFreePool(DriveExtension->DriveDeviceName.Buffer);
//                 }
//
//                 /* Register our device interface */
//                 if (DriveExtension->DiskType != RAMDISK_REGISTRY_DISK)
//                 {
//                     Status = IoRegisterDeviceInterface(DeviceObject,
//                                                        &GUID_DEVINTERFACE_VOLUME,
//                                                        NULL,
//                                                        &DriveExtension->DriveDeviceName);
//                 }
//                 else
//                 {
//                     Status = IoRegisterDeviceInterface(DeviceObject,
//                                                        &RamdiskDiskInterface,
//                                                        NULL,
//                                                        &DriveExtension->DriveDeviceName);
//                 }
//
//                 /* If we were asked not to assign a drive letter or if getting
//                  * a name failed, just return saying we're now started */
//                 if (DriveExtension->DiskOptions.NoDriveLetter ||
//                     DriveExtension->DriveDeviceName.Buffer == NULL)
//                 {
//                     DriveExtension->State = RamdiskStateStarted;
//                     Irp->IoStatus.Status = Status;
//                     break;
//                 }
//
//                 /* Now get our installation state */
//                 Status = IoGetDeviceProperty(DeviceObject,
//                                              DevicePropertyInstallState,
//                                              sizeof(InstallState),
//                                              &InstallState,
//                                              &ResultLength);
//                 /* If querying the information failed, assume success */
//                 if (!NT_SUCCESS(Status))
//                 {
//                     InstallState = InstallStateInstalled;
//                 }
//
//                 /* If we were properly installed, then, enable the interface */
//                 if (InstallState == InstallStateInstalled)
//                 {
//                     Status = IoSetDeviceInterfaceState(&DriveExtension->DriveDeviceName, TRUE);
//                 }
//
//                 /* We're fine & up */
//                 DriveExtension->State = RamdiskStateStarted;
//                 Irp->IoStatus.Status = Status;
//                 break;
//             }
//
//             /* Prepare next stack to pass it down */
//             IoCopyCurrentIrpStackLocationToNext(Irp);
//
//             /* Initialize our notification event & our completion routine */
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//             IoSetCompletionRoutine(Irp, RamdiskIoCompletionRoutine, &Event, TRUE, TRUE, TRUE);
//
//             /* Call lower driver */
//             Status = IoCallDriver(DeviceExtension->AttachedDevice, Irp);
//             if (Status == STATUS_PENDING)
//             {
//                 KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 Status = Irp->IoStatus.Status;
//             }
//
//             /* If it succeed to start then enable ourselves and we're up! */
//             if (NT_SUCCESS(Status))
//             {
//                 Status = IoSetDeviceInterfaceState(&DeviceExtension->DriveDeviceName, TRUE);
//                 DeviceExtension->State = RamdiskStateStarted;
//             }
//
//             Irp->IoStatus.Status = Status;
//             break;
//         }
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//         case IRP_MN_CANCEL_STOP_DEVICE:
//         case IRP_MN_STOP_DEVICE:
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//         {
//             UNIMPLEMENTED_DBGBREAK("PnP IRP: %lx\n", Minor);
//             break;
//         }
//
//         case IRP_MN_REMOVE_DEVICE:
//         {
//             /* Remove the proper device */
//             if (DeviceExtension->Type == RamdiskBus)
//             {
//                 Status = RamdiskRemoveBusDevice(DeviceObject, Irp);
//
//                 /* Return here, lower device has already been called
//                  * And remove lock released. This is needed by the function. */
//                 return Status;
//             }
//             else
//             {
//                 Status = RamdiskDeleteDiskDevice(DeviceObject, Irp);
//
//                 /* Complete the IRP here and return
//                  * Here again we don't have to release remove lock
//                  * This has already been done by the function. */
//                 Irp->IoStatus.Status = Status;
//                 IoCompleteRequest(Irp, IO_NO_INCREMENT);
//                 return Status;
//             }
//         }
//
//         case IRP_MN_SURPRISE_REMOVAL:
//             UNIMPLEMENTED_DBGBREAK("PnP IRP: %lx\n", Minor);
//             break;
//
//         case IRP_MN_QUERY_ID:
//         {
//             /* Are we a drive? */
//             if (DeviceExtension->Type == RamdiskDrive)
//             {
//                 Status = RamdiskQueryId((PRAMDISK_DRIVE_EXTENSION)DeviceExtension, Irp);
//             }
//             break;
//         }
//
//         case IRP_MN_QUERY_BUS_INFORMATION:
//         {
//             /* Are we a drive? */
//             if (DeviceExtension->Type == RamdiskDrive)
//             {
//                 Status = RamdiskQueryBusInformation(DeviceObject, Irp);
//             }
//             break;
//         }
//
//         case IRP_MN_EJECT:
//             UNIMPLEMENTED_DBGBREAK("PnP IRP: %lx\n", Minor);
//             break;
//
//         case IRP_MN_QUERY_DEVICE_TEXT:
//         {
//             /* Are we a drive? */
//             if (DeviceExtension->Type == RamdiskDrive)
//             {
//                 Status = RamdiskQueryDeviceText((PRAMDISK_DRIVE_EXTENSION)DeviceExtension, Irp);
//             }
//             break;
//         }
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//         {
//             /* Call our main routine */
//             Status = RamdiskQueryDeviceRelations(IoStackLocation->
//                                                  Parameters.
//                                                  QueryDeviceRelations.Type,
//                                                  DeviceObject,
//                                                  Irp);
//             goto ReleaseAndReturn;
//         }
//
//         case IRP_MN_QUERY_CAPABILITIES:
//         {
//             /* Are we a drive? */
//             if (DeviceExtension->Type == RamdiskDrive)
//             {
//                 Status = RamdiskQueryCapabilities(DeviceObject, Irp);
//             }
//             break;
//         }
//
//         case IRP_MN_QUERY_RESOURCES:
//         case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//         {
//             /* Complete immediately without touching it */
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             goto ReleaseAndReturn;
//         }
//
//         default:
//             DPRINT1("Illegal IRP: %lx\n", Minor);
//             break;
//     }
//
//     /* Are we the bus? */
//     if (DeviceExtension->Type == RamdiskBus)
//     {
//         /* Do we have an attached device? */
//         if (DeviceExtension->AttachedDevice)
//         {
//             /* Forward the IRP */
//             IoSkipCurrentIrpStackLocation(Irp);
//             Status = IoCallDriver(DeviceExtension->AttachedDevice, Irp);
//         }
//     }
//
//     /* Release the lock and return status */
// ReleaseAndReturn:
//     IoReleaseRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskPower(IN PDEVICE_OBJECT DeviceObject,
//              IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation;
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* If we have a device extension, take extra caution with the lower driver */
//     if (DeviceExtension != NULL)
//     {
//         PoStartNextPowerIrp(Irp);
//
//         /* Device has not been removed yet, so pass to the attached/lower driver */
//         if (DeviceExtension->State < RamdiskStateBusRemoved)
//         {
//             IoSkipCurrentIrpStackLocation(Irp);
//             return PoCallDriver(DeviceExtension->AttachedDevice, Irp);
//         }
//         /* Otherwise, simply complete the IRP notifying that deletion is pending */
//         else
//         {
//             Irp->IoStatus.Status = STATUS_DELETE_PENDING;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_DELETE_PENDING;
//         }
//     }
//
//     /* Get stack and deal with minor functions */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     switch (IoStackLocation->MinorFunction)
//     {
//         case IRP_MN_SET_POWER:
//         {
//             /* If setting device power state it's all fine and return success */
//             if (DevicePowerState)
//             {
//                 Irp->IoStatus.Status = STATUS_SUCCESS;
//             }
//
//             /* Get appropriate status for return */
//             Status = Irp->IoStatus.Status;
//             PoStartNextPowerIrp(Irp);
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             break;
//         }
//
//         case IRP_MN_QUERY_POWER:
//         {
//             /* We can obviously accept all states so just return success */
//             Status = Irp->IoStatus.Status = STATUS_SUCCESS;
//             PoStartNextPowerIrp(Irp);
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             break;
//         }
//
//         default:
//         {
//             /* Just complete and save status for return */
//             Status = Irp->IoStatus.Status;
//             PoStartNextPowerIrp(Irp);
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             break;
//         }
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskSystemControl(IN PDEVICE_OBJECT DeviceObject,
//                      IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* If we have a device extension, forward the IRP to the attached device */
//     if (DeviceExtension != NULL)
//     {
//         IoSkipCurrentIrpStackLocation(Irp);
//         Status = IoCallDriver(DeviceExtension->AttachedDevice, Irp);
//     }
//     /* Otherwise just complete the request
//      * And return the status with which we complete it */
//     else
//     {
//         Status = Irp->IoStatus.Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskScsi(IN PDEVICE_OBJECT DeviceObject,
//             IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* Having a proper device is mandatory */
//     if (DeviceExtension->State > RamdiskStateStopped)
//     {
//         Status = STATUS_DEVICE_DOES_NOT_EXIST;
//         goto CompleteIRP;
//     }
//
//     /* Acquire the remove lock */
//     Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     if (!NT_SUCCESS(Status))
//     {
//         goto CompleteIRP;
//     }
//
//     /* Queue the IRP for worker */
//     Status = SendIrpToThread(DeviceObject, Irp);
//     if (Status != STATUS_PENDING)
//     {
//         goto CompleteIRP;
//     }
//
//     /* Release the remove lock */
//     IoReleaseRemoveLock(&DeviceExtension->RemoveLock, Irp);
//     goto Quit;
//
// CompleteIRP:
//     Irp->IoStatus.Information = 0;
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
// Quit:
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// RamdiskFlushBuffers(IN PDEVICE_OBJECT DeviceObject,
//                     IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PRAMDISK_DRIVE_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* Ensure we have drive extension
//      * Only perform flush on disks that have been created
//      * from registry entries */
//     if (DeviceExtension->Type != RamdiskDrive ||
//         DeviceExtension->DiskType > RAMDISK_MEMORY_MAPPED_DISK)
//     {
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_SUCCESS;
//     }
//
//     /* Queue the IRP for worker */
//     Status = SendIrpToThread(DeviceObject, Irp);
//     if (Status != STATUS_PENDING)
//     {
//         /* Queuing failed - complete the IRP and return failure */
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// RamdiskUnload(IN PDRIVER_OBJECT DriverObject)
// {
//     /* Just release registry path if previously allocated */
//     if (DriverRegistryPath.Buffer)
//     {
//         ExFreePoolWithTag(DriverRegistryPath.Buffer, 'dmaR');
//     }
// }
//
// NTSTATUS
// NTAPI
// RamdiskAddDevice(IN PDRIVER_OBJECT DriverObject,
//                  IN PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     PRAMDISK_BUS_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDevice;
//     NTSTATUS Status;
//     UNICODE_STRING DeviceName;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Only create the bus FDO once */
//     if (RamdiskBusFdo) return STATUS_DEVICE_ALREADY_ATTACHED;
//
//     /* Create the bus FDO */
//     RtlInitUnicodeString(&DeviceName, L"\\Device\\Ramdisk");
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(RAMDISK_BUS_EXTENSION),
//                             &DeviceName,
//                             FILE_DEVICE_BUS_EXTENDER,
//                             FILE_DEVICE_SECURE_OPEN,
//                             0,
//                             &DeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         /* Initialize the bus FDO extension */
//         DeviceExtension = DeviceObject->DeviceExtension;
//         RtlZeroMemory(DeviceExtension, sizeof(*DeviceExtension));
//
//         /* Set bus FDO flags */
//         DeviceObject->Flags |= DO_POWER_PAGABLE | DO_DIRECT_IO;
//
//         /* Setup the bus FDO extension */
//         DeviceExtension->Type = RamdiskBus;
//         ExInitializeFastMutex(&DeviceExtension->DiskListLock);
//         IoInitializeRemoveLock(&DeviceExtension->RemoveLock, 'dmaR', 1, 0);
//         InitializeListHead(&DeviceExtension->DiskList);
//         DeviceExtension->PhysicalDeviceObject = PhysicalDeviceObject;
//         DeviceExtension->DeviceObject = DeviceObject;
//
//         /* Register the RAM disk device interface */
//         Status = IoRegisterDeviceInterface(PhysicalDeviceObject,
//                                            &RamdiskBusInterface,
//                                            NULL,
//                                            &DeviceExtension->BusDeviceName);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             IoDeleteDevice(DeviceObject);
//             return Status;
//         }
//
//         /* Attach us to the device stack */
//         AttachedDevice = IoAttachDeviceToDeviceStack(DeviceObject,
//                                                      PhysicalDeviceObject);
//         DeviceExtension->AttachedDevice = AttachedDevice;
//         if (!AttachedDevice)
//         {
//             /* Fail */
//             IoSetDeviceInterfaceState(&DeviceExtension->BusDeviceName, 0);
//             RtlFreeUnicodeString(&DeviceExtension->BusDeviceName);
//             IoDeleteDevice(DeviceObject);
//             return STATUS_NO_SUCH_DEVICE;
//         }
//
//         /* Bus FDO is initialized */
//         RamdiskBusFdo = DeviceObject;
//
//         /* Loop for loader block */
//         if (KeLoaderBlock)
//         {
//             /* Are we being booted from setup? Not yet supported */
//             if (KeLoaderBlock->SetupLdrBlock)
//                 DPRINT1("FIXME: RamdiskAddDevice is UNSUPPORTED when being started from SETUPLDR!\n");
//             // ASSERT(!KeLoaderBlock->SetupLdrBlock);
//         }
//
//         /* All done */
//         DeviceObject->Flags &= DO_DEVICE_INITIALIZING;
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     PCHAR BootDeviceName, CommandLine;
//     PDEVICE_OBJECT PhysicalDeviceObject = NULL;
//     NTSTATUS Status;
//     RD_TRACE("[RAMDISK-DRV] DriverEntry called\r\n");
//     DPRINT("RAM Disk Driver Initialized\n");
//
//     /* Save the registry path */
//     DriverRegistryPath.MaximumLength = RegistryPath->Length + sizeof(UNICODE_NULL);
//     DriverRegistryPath.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                       DriverRegistryPath.MaximumLength,
//                                                       'dmaR');
//     if (!DriverRegistryPath.Buffer)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RtlCopyUnicodeString(&DriverRegistryPath, RegistryPath);
//
//     /* Query ramdisk parameters */
//     QueryParameters(&DriverRegistryPath);
//
//     /* Set device routines */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = RamdiskOpenClose;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = RamdiskOpenClose;
//     DriverObject->MajorFunction[IRP_MJ_READ] = RamdiskReadWrite;
//     DriverObject->MajorFunction[IRP_MJ_WRITE] = RamdiskReadWrite;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = RamdiskDeviceControl;
//     DriverObject->MajorFunction[IRP_MJ_PNP] = RamdiskPnp;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = RamdiskPower;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = RamdiskSystemControl;
//     DriverObject->MajorFunction[IRP_MJ_SCSI] = RamdiskScsi;
//     DriverObject->MajorFunction[IRP_MJ_FLUSH_BUFFERS] = RamdiskFlushBuffers;
//     DriverObject->DriverExtension->AddDevice = RamdiskAddDevice;
//     DriverObject->DriverUnload = RamdiskUnload;
//
//     /* Check for a loader block */
//     if (KeLoaderBlock)
//     {
//         /* Get the boot device name */
//         BootDeviceName = KeLoaderBlock->ArcBootDeviceName;
//         if (BootDeviceName)
//         {
//             /* Check if we're booting from ramdisk */
//             if ((strlen(BootDeviceName) >= 10) &&
//                 !(_strnicmp(BootDeviceName, "ramdisk(0)", 10)))
//             {
//                 /* We'll have to tell the PnP Manager */
//                 ReportDetectedDevice = TRUE;
//
//                 /* Check for a command line */
//                 CommandLine = KeLoaderBlock->LoadOptions;
//                 if (CommandLine)
//                 {
//                     /* Check if this is an ISO boot */
//                     if (strstr(CommandLine, "RDEXPORTASCD"))
//                     {
//                         /* Remember for later */
//                         ExportBootDiskAsCd = TRUE;
//                     }
//
//                     /* Check if this is PE boot */
//                     if (strstr(CommandLine, "MININT"))
//                     {
//                         /* Remember for later */
//                         IsWinPEBoot = TRUE;
//                     }
//                 }
//             }
//
//         }
//     }
//
//     /* Installing from Ramdisk isn't supported yet */
//     if (KeLoaderBlock->SetupLdrBlock)
//         DPRINT1("FIXME: Installing from RamDisk is UNSUPPORTED!\n");
//     // ASSERT(!KeLoaderBlock->SetupLdrBlock);
//
//     /* Are we reporting the device */
//     if (ReportDetectedDevice)
//     {
//         RD_TRACE("[RAMDISK-DRV] Reporting detected device\r\n");
//         /* Do it */
//         Status = IoReportDetectedDevice(DriverObject,
//                                         InterfaceTypeUndefined,
//                                         0xFFFFFFFF,
//                                         0xFFFFFFFF,
//                                         NULL,
//                                         NULL,
//                                         0,
//                                         &PhysicalDeviceObject);
//         if (NT_SUCCESS(Status))
//         {
//             /* Create the device object */
//             Status = RamdiskAddDevice(DriverObject, PhysicalDeviceObject);
//             if (NT_SUCCESS(Status))
//             {
//                 /* We are done */
//                 PhysicalDeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//                 Status = STATUS_SUCCESS;
//             }
//         }
//     }
//     else
//     {
//         /* Done */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Done */
//     RD_TRACE("[RAMDISK-DRV] DriverEntry returning\r\n");
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
