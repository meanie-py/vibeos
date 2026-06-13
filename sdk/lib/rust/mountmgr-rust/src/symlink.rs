use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExUuidCreate, GUID, IoCreateSymbolicLink, IoDeleteSymbolicLink, IoGetAttachedDeviceReference, IoGetDeviceObjectPointer, LPGUID, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PFILE_OBJECT, PGUID, PMOUNTDEV_NAME, PUNICODE_STRING, RTL_QUERY_REGISTRY_TABLE, RtlAppendUnicodeStringToString, RtlCompareUnicodeString, RtlCopyUnicodeString, RtlEqualUnicodeString, RtlInitUnicodeString, RtlPrefixUnicodeString, RtlQueryRegistryValues, RtlStringFromGUID, RtlWriteRegistryValue, ZwClose, ZwOpenSymbolicLinkObject, ZwQuerySymbolicLinkObject};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_OBJECT, HANDLE, MOUNTDEV_NAME, PCUNICODE_STRING, _UNICODE_STRING};

// NTAPI kernel exports — extern "system" (stdcall on i386, _Name@N).
unsafe extern "system" {
    fn ExAllocatePoolWithTag( PoolType: ULONG, NumberOfBytes: SIZE_T, Tag: ULONG, ) -> PVOID;
}

extern "C" {
    static mut DatabasePath: *mut u16; // PWSTR (C: PWSTR DatabasePath = L"...")
}
extern "system" {
    fn ExFreePoolWithTag(P: *mut core::ffi::c_void, Tag: u32);
    fn FreePool(p: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static DEVICEMOUNT_WSTR: [u16; 26] = [0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x004D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0050, 0x006F, 0x0069, 0x006E, 0x0074, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000];
#[no_mangle]
pub static mut DeviceMount: _UNICODE_STRING = _UNICODE_STRING {
    Length: 50,
    MaximumLength: 52,
    Buffer: &DEVICEMOUNT_WSTR as *const [u16; 26] as *const u16 as *mut u16,
};

static DOSDEVICESMOUNT_WSTR: [u16; 30] = [0x005C, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x005C, 0x004D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0050, 0x006F, 0x0069, 0x006E, 0x0074, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000];
#[no_mangle]
pub static mut DosDevicesMount: _UNICODE_STRING = _UNICODE_STRING {
    Length: 58,
    MaximumLength: 60,
    Buffer: &DOSDEVICESMOUNT_WSTR as *const [u16; 30] as *const u16 as *mut u16,
};

static DOSDEVICES_WSTR: [u16; 13] = [0x005C, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x005C, 0x0000];
#[no_mangle]
pub static mut DosDevices: _UNICODE_STRING = _UNICODE_STRING {
    Length: 24,
    MaximumLength: 26,
    Buffer: &DOSDEVICES_WSTR as *const [u16; 13] as *const u16 as *mut u16,
};

static DEVICEFLOPPY_WSTR: [u16; 15] = [0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C, 0x006F, 0x0070, 0x0070, 0x0079, 0x0000];
#[no_mangle]
pub static mut DeviceFloppy: _UNICODE_STRING = _UNICODE_STRING {
    Length: 28,
    MaximumLength: 30,
    Buffer: &DEVICEFLOPPY_WSTR as *const [u16; 15] as *const u16 as *mut u16,
};

static DEVICECDROM_WSTR: [u16; 14] = [0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0043, 0x0064, 0x0052, 0x006F, 0x006D, 0x0000];
#[no_mangle]
pub static mut DeviceCdRom: _UNICODE_STRING = _UNICODE_STRING {
    Length: 26,
    MaximumLength: 28,
    Buffer: &DEVICECDROM_WSTR as *const [u16; 14] as *const u16 as *mut u16,
};

static DOSGLOBAL_WSTR: [u16; 11] = [0x005C, 0x0047, 0x004C, 0x004F, 0x0042, 0x0041, 0x004C, 0x003F, 0x003F, 0x005C, 0x0000];
#[no_mangle]
pub static mut DosGlobal: _UNICODE_STRING = _UNICODE_STRING {
    Length: 20,
    MaximumLength: 22,
    Buffer: &DOSGLOBAL_WSTR as *const [u16; 11] as *const u16 as *mut u16,
};

static GLOBAL_WSTR: [u16; 5] = [0x005C, 0x003F, 0x003F, 0x005C, 0x0000];
#[no_mangle]
pub static mut Global: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: &GLOBAL_WSTR as *const [u16; 5] as *const u16 as *mut u16,
};

static SAFEVOLUMES_WSTR: [u16; 34] = [0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0073, 0x0053, 0x0061, 0x0066, 0x0065, 0x0046, 0x006F, 0x0072, 0x0057, 0x0072, 0x0069, 0x0074, 0x0065, 0x0041, 0x0063, 0x0063, 0x0065, 0x0073, 0x0073, 0x0000];
#[no_mangle]
pub static mut SafeVolumes: _UNICODE_STRING = _UNICODE_STRING {
    Length: 66,
    MaximumLength: 68,
    Buffer: &SAFEVOLUMES_WSTR as *const [u16; 34] as *const u16 as *mut u16,
};

static VOLUME_WSTR: [u16; 11] = [0x005C, 0x003F, 0x003F, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0000];
#[no_mangle]
pub static mut Volume: _UNICODE_STRING = _UNICODE_STRING {
    Length: 20,
    MaximumLength: 22,
    Buffer: &VOLUME_WSTR as *const [u16; 11] as *const u16 as *mut u16,
};

static REPARSEINDEX_WSTR: [u16; 39] = [0x005C, 0x0024, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0064, 0x005C, 0x0024, 0x0052, 0x0065, 0x0070, 0x0061, 0x0072, 0x0073, 0x0065, 0x003A, 0x0024, 0x0052, 0x003A, 0x0024, 0x0049, 0x004E, 0x0044, 0x0045, 0x0058, 0x005F, 0x0041, 0x004C, 0x004C, 0x004F, 0x0043, 0x0041, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000];
#[no_mangle]
pub static mut ReparseIndex: _UNICODE_STRING = _UNICODE_STRING {
    Length: 76,
    MaximumLength: 78,
    Buffer: &REPARSEINDEX_WSTR as *const [u16; 39] as *const u16 as *mut u16,
};

pub const IOCTL_MOUNTDEV_LINK_CREATED_UNSECURE_DEPRECATED: u32 = 0x006D0010;
pub const IOCTL_MOUNTDEV_LINK_DELETED_UNSECURE_DEPRECATED: u32 = 0x006D0014;

// Wide string data for \\Device\\MountPointManager
static WIDE_MOUNTMGR_DEVICE_NAME: &[u16] = &[
    0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
    0x004D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0050, 0x006F, 0x0069,
    0x006E, 0x0074, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065,
    0x0072, 0x0000,
];

// Wide string for \\DosDevices\\MountPointManager
static WIDE_DOSDEVICES_MOUNT: &[u16] = &[
    0x005C, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x004D, 0x006F, 0x0075, 0x006E,
    0x0074, 0x0050, 0x006F, 0x0069, 0x006E, 0x0074, 0x004D, 0x0061,
    0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];

// Wide string for \\DosDevices\\
static WIDE_DOSDEVICES: &[u16] = &[
    0x005C, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0000,
];

// Wide string for \\Device\\Floppy
static WIDE_DEVICE_FLOPPY: &[u16] = &[
    0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
    0x0046, 0x006C, 0x006F, 0x0070, 0x0070, 0x0079, 0x0000,
];

// Wide string for \\Device\\CdRom
static WIDE_DEVICE_CDROM: &[u16] = &[
    0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
    0x0043, 0x0064, 0x0052, 0x006F, 0x006D, 0x0000,
];

// Wide string for \\GLOBAL??
static WIDE_DOS_GLOBAL: &[u16] = &[
    0x005C, 0x0047, 0x004C, 0x004F, 0x0042, 0x0041, 0x004C, 0x003F,
    0x003F, 0x005C, 0x0000,
];

// Wide string for \\??
static WIDE_GLOBAL: &[u16] = &[
    0x005C, 0x003F, 0x003F, 0x005C, 0x0000,
];

// Wide string for \\Device\\VolumesSafeForWriteAccess
static WIDE_SAFE_VOLUMES: &[u16] = &[
    0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
    0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0073, 0x0053,
    0x0061, 0x0066, 0x0065, 0x0046, 0x006F, 0x0072, 0x0057, 0x0072,
    0x0069, 0x0074, 0x0065, 0x0041, 0x0063, 0x0063, 0x0065, 0x0073,
    0x0073, 0x0000,
];

// Wide string for \\??\\Volume
static WIDE_VOLUME: &[u16] = &[
    0x005C, 0x003F, 0x003F, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075,
    0x006D, 0x0065, 0x0000,
];

// Wide string for \\$Extend\\$Reparse:$R:$INDEX_ALLOCATION
static WIDE_REPARSE_INDEX: &[u16] = &[
    0x005C, 0x0024, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0064,
    0x005C, 0x0024, 0x0052, 0x0065, 0x0070, 0x0061, 0x0072, 0x0073,
    0x0065, 0x003A, 0x0024, 0x0052, 0x003A, 0x0024, 0x0049, 0x004E,
    0x0044, 0x0045, 0x0058, 0x005F, 0x0041, 0x004C, 0x004C, 0x004F,
    0x0043, 0x0041, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000,
];

const PagedPool: u32 = 1;
const UNICODE_NULL: u16 = 0;
const IOCTL_MOUNTDEV_LINK_CREATED: u32 = 0x006D0010;
const IOCTL_MOUNTDEV_LINK_DELETED: u32 = 0x004D000C;

unsafe extern "system" {
    fn RtlCompareMemory(
        Source1: *const core::ffi::c_void,
        Source2: *const core::ffi::c_void,
        Length: usize,
    ) -> usize;
}
#[no_mangle]
pub unsafe extern "C" fn CreateStringWithGlobal(
    DosName: PUNICODE_STRING,
    GlobalString: PUNICODE_STRING,
) -> NTSTATUS {
    let mut IntGlobal: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if RtlPrefixUnicodeString(
        &raw mut DosDevices as PCUNICODE_STRING,
        DosName as PCUNICODE_STRING,
        1,
    ) != 0
    {
        IntGlobal.Length = (((*DosName).Length as i32) - (DosDevices.Length as i32) + (DosGlobal.Length as i32)) as u16;
        IntGlobal.MaximumLength = (IntGlobal.Length as usize).wrapping_add(core::mem::size_of::<WCHAR>()) as u16;
        IntGlobal.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            IntGlobal.MaximumLength as usize,
            1098149453u32,
        ) as *mut WCHAR;
        if IntGlobal.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping(
            DosGlobal.Buffer as *const u8,
            IntGlobal.Buffer as *mut u8,
            DosGlobal.Length as usize,
        );
        core::ptr::copy_nonoverlapping(
            ((*DosName).Buffer.offset(
                (DosDevices.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize
            )) as *const u8,
            (IntGlobal.Buffer.offset(
                (DosGlobal.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize
            )) as *mut u8,
            (((*DosName).Length as i32) - (DosDevices.Length as i32)) as usize,
        );
        *IntGlobal.Buffer.offset(
            (IntGlobal.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
        ) = UNICODE_NULL;
    } else if RtlPrefixUnicodeString(
        &raw mut Global as PCUNICODE_STRING,
        DosName as PCUNICODE_STRING,
        1,
    ) != 0
    {
        IntGlobal.Length = (((*DosName).Length as i32) - (Global.Length as i32) + (DosGlobal.Length as i32)) as u16;
        IntGlobal.MaximumLength = (IntGlobal.Length as usize).wrapping_add(core::mem::size_of::<WCHAR>()) as u16;
        IntGlobal.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            IntGlobal.MaximumLength as usize,
            1098149453u32,
        ) as *mut WCHAR;
        if IntGlobal.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping(
            DosGlobal.Buffer as *const u8,
            IntGlobal.Buffer as *mut u8,
            DosGlobal.Length as usize,
        );
        core::ptr::copy_nonoverlapping(
            ((*DosName).Buffer.offset(
                (Global.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize
            )) as *const u8,
            (IntGlobal.Buffer.offset(
                (DosGlobal.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize
            )) as *mut u8,
            (((*DosName).Length as i32) - (Global.Length as i32)) as usize,
        );
        *IntGlobal.Buffer.offset(
            (IntGlobal.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
        ) = UNICODE_NULL;
    } else {
        IntGlobal.Length = (*DosName).Length;
        IntGlobal.MaximumLength = (*DosName).MaximumLength;
        IntGlobal.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            IntGlobal.MaximumLength as usize,
            1098149453u32,
        ) as *mut WCHAR;
        if IntGlobal.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping(
            (*DosName).Buffer as *const u8,
            IntGlobal.Buffer as *mut u8,
            IntGlobal.MaximumLength as usize,
        );
    }

    (*GlobalString).Length = IntGlobal.Length;
    (*GlobalString).MaximumLength = IntGlobal.MaximumLength;
    (*GlobalString).Buffer = IntGlobal.Buffer;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn GlobalCreateSymbolicLink(
    DosName: *mut _UNICODE_STRING,
    DeviceName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut GlobalName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    Status = CreateStringWithGlobal(DosName, &raw mut GlobalName);
    if !(Status >= 0) {
        return Status;
    }
    Status = IoCreateSymbolicLink(&raw mut GlobalName, DeviceName);
    ExFreePoolWithTag(GlobalName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn GlobalDeleteSymbolicLink(DosName: *mut _UNICODE_STRING) -> i32 {
    let mut Status: i32 = 0;
    let mut GlobalName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    Status = CreateStringWithGlobal(DosName, &raw mut GlobalName);
    if !(Status >= 0) {
        return Status;
    }
    Status = IoDeleteSymbolicLink(&raw mut GlobalName);
    ExFreePoolWithTag(GlobalName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn SendLinkCreated(mut SymbolicName: PUNICODE_STRING) {
    let mut Status: NTSTATUS = 0;
    let mut NameSize: ULONG = 0;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Name: PMOUNTDEV_NAME = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    Status = IoGetDeviceObjectPointer(
        SymbolicName,
        FILE_READ_ATTRIBUTES as u32,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        return;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    NameSize = (core::mem::size_of::<u16>() as u32).wrapping_add((*SymbolicName).Length as u32);

    Name = ExAllocatePoolWithTag(PagedPool, NameSize as usize, 1098149453u32) as PMOUNTDEV_NAME;
    if !Name.is_null() {
        (*Name).NameLength = (*SymbolicName).Length;
        core::ptr::copy_nonoverlapping::<u8>(
            (*SymbolicName).Buffer as *const u8,
            &raw mut (*Name).Name as *mut u8,
            (*SymbolicName).Length as usize,
        );

        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_LINK_CREATED,
            DeviceObject,
            Name as *mut core::ffi::c_void,
            NameSize,
            core::ptr::null_mut(),
            0u32,
            FileObject,
        );

        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_LINK_CREATED_UNSECURE_DEPRECATED,
            DeviceObject,
            Name as *mut core::ffi::c_void,
            NameSize,
            core::ptr::null_mut(),
            0u32,
            FileObject,
        );
    }

    if !Name.is_null() {
        ExFreePoolWithTag(Name as *mut core::ffi::c_void, 1098149453u32);
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn SendLinkDeleted(
    DeviceName: *mut _UNICODE_STRING,
    SymbolicName: *mut _UNICODE_STRING,
) {
    let mut Status: NTSTATUS;
    let mut NameSize: u32;
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut Name: *mut MOUNTDEV_NAME = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();

    // Get the device associated with the name
    Status = IoGetDeviceObjectPointer(
        DeviceName,
        FILE_READ_ATTRIBUTES as u32,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if Status < 0 {
        return;
    }

    // Get attached device (will notify it)
    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    // NameSize is the size of the whole MOUNTDEV_NAME structure
    NameSize = (core::mem::size_of::<u16>() as u32).wrapping_add((*SymbolicName).Length as u32);
    Name = ExAllocatePoolWithTag(PagedPool, NameSize as usize, 1098149453u32) as *mut MOUNTDEV_NAME;

    'cleanup: {
        if Name.is_null() {
            break 'cleanup;
        }

        // Initialize structure
        (*Name).NameLength = (*SymbolicName).Length;
        RtlCopyMemory(
            &raw mut (*Name).Name as *mut u16 as *mut core::ffi::c_void,
            (*SymbolicName).Buffer as *const core::ffi::c_void,
            (*SymbolicName).Length as usize,
        );

        // Send the notification, using the new (Windows 2003+) IOCTL definition
        // (with limited access) first. If this fails, the called driver may be
        // for an older NT version, and so, we send again the notification using
        // the old (Windows 2000) IOCTL definition (with any access).
        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_LINK_DELETED,
            DeviceObject,
            Name as *mut core::ffi::c_void,
            NameSize,
            core::ptr::null_mut(),
            0,
            FileObject,
        );
        // This one can fail, no one matters
        let _ = Status;

        // Then, second one
        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_LINK_DELETED_UNSECURE_DEPRECATED,
            DeviceObject,
            Name as *mut core::ffi::c_void,
            NameSize,
            core::ptr::null_mut(),
            0,
            FileObject,
        );
        let _ = Status;
    }

    if !Name.is_null() {
        ExFreePoolWithTag(Name as *mut core::ffi::c_void, 1098149453u32);
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "system" fn SymbolicLinkNamesFromUniqueIdCount(
    ValueName: PWSTR,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let mut ValueNameString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let UniqueId = Context as *mut MOUNTDEV_UNIQUE_ID;

    if *ValueName != 0x0023_u16 || ValueType != REG_BINARY || (*UniqueId).UniqueIdLength as u32 != ValueLength {
        return STATUS_SUCCESS;
    }

    if RtlCompareMemory(
        (&raw const (*UniqueId).UniqueId).cast::<core::ffi::c_void>(),
        ValueData,
        ValueLength as usize,
    ) != ValueLength as usize
    {
        return STATUS_SUCCESS;
    }

    RtlInitUnicodeString(&raw mut ValueNameString, ValueName as *const u16);

    if ValueNameString.Length != 0 {
        let count_ptr = EntryContext as *mut u32;
        *count_ptr = (*count_ptr).wrapping_add(1);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn SymbolicLinkNamesFromUniqueIdQuery(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let mut ValueNameString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let UniqueId: *mut MOUNTDEV_UNIQUE_ID = Context as *mut MOUNTDEV_UNIQUE_ID;
    let mut ReturnString: *mut _UNICODE_STRING = EntryContext as *mut _UNICODE_STRING;
    if *ValueName.offset(0) as u32 != '#' as u32
        || ValueType != REG_BINARY
        || (*UniqueId).UniqueIdLength as ULONG != ValueLength
    {
        return STATUS_SUCCESS;
    }
    if RtlCompareMemory(
        &raw mut (*UniqueId).UniqueId as *mut u8 as *const core::ffi::c_void,
        ValueData as *const core::ffi::c_void,
        ValueLength as usize,
    ) != ValueLength as usize
    {
        return STATUS_SUCCESS;
    }
    RtlInitUnicodeString(&raw mut ValueNameString, ValueName as *const u16);
    if ValueNameString.Length == 0 {
        return STATUS_SUCCESS;
    }
    ValueNameString.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        ValueNameString.MaximumLength as usize,
        1098149453_u32,
    ) as *mut u16;
    if ValueNameString.Buffer.is_null() {
        return STATUS_SUCCESS;
    }
    RtlCopyMemory(
        ValueNameString.Buffer as *mut core::ffi::c_void,
        ValueName as *const core::ffi::c_void,
        ValueNameString.Length as usize,
    );
    *ValueNameString.Buffer.offset(
        (ValueNameString.Length as usize).wrapping_div(core::mem::size_of::<u16>()) as isize,
    ) = 0;
    while (*ReturnString).Length != 0 {
        ReturnString = ReturnString.add(1);
    }
    *ReturnString = ValueNameString;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CreateNewVolumeName(
    VolumeName: *mut _UNICODE_STRING,
    VolumeGuid: *const GUID,
) -> NTSTATUS {
    let mut Guid: GUID = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    if VolumeGuid.is_null() {
        Status = ExUuidCreate(&raw mut Guid);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    } else {
        core::ptr::copy_nonoverlapping(
            VolumeGuid as *const core::ffi::c_void,
            &raw mut Guid as *mut core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
    }
    Status = RtlStringFromGUID(&raw mut Guid, &raw mut GuidString);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    (*VolumeName).MaximumLength =
        ((0x14u32 + 0x4cu32) as usize + core::mem::size_of::<u16>()) as u16;
    (*VolumeName).Buffer = ExAllocatePoolWithTag(
        PagedPool,
        ((0x14u32 + 0x4cu32) as usize + core::mem::size_of::<u16>()),
        1098149453i32 as u32,
    ) as *mut u16;
    if (*VolumeName).Buffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        RtlCopyUnicodeString(VolumeName, &raw const Volume as *const _UNICODE_STRING);
        RtlAppendUnicodeStringToString(VolumeName, &raw mut GuidString as *const _UNICODE_STRING);
        *(*VolumeName).Buffer.add(
            ((*VolumeName).Length as usize) / core::mem::size_of::<u16>(),
        ) = UNICODE_NULL;
        Status = STATUS_SUCCESS;
    }
    ExFreePoolWithTag(GuidString.Buffer as *mut core::ffi::c_void, 0u32);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn QuerySymbolicLinkNamesFromStorage(
    DeviceExtension: PDEVICE_EXTENSION,
    DeviceInformation: PDEVICE_INFORMATION,
    SuggestedLinkName: PUNICODE_STRING,
    UseOnlyIfThereAreNoOtherLinks: BOOLEAN,
    SymLinks: *mut PUNICODE_STRING,
    SymLinkCount: PULONG,
    HasGuid: BOOLEAN,
    Guid: LPGUID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut WriteNew: BOOLEAN;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: 0,
            Name: core::ptr::null::<u16>(),
            EntryContext: core::ptr::null_mut::<core::ffi::c_void>(),
            DefaultType: 0,
            DefaultData: core::ptr::null_mut::<core::ffi::c_void>(),
            DefaultLength: 0,
        },
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: 0,
            Name: core::ptr::null::<u16>(),
            EntryContext: core::ptr::null_mut::<core::ffi::c_void>(),
            DefaultType: 0,
            DefaultData: core::ptr::null_mut::<core::ffi::c_void>(),
            DefaultLength: 0,
        },
    ];

    // RtlZeroMemory(QueryTable, sizeof(QueryTable));
    core::ptr::write_bytes(
        &raw mut QueryTable as *mut core::ffi::c_void,
        0u8,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );

    QueryTable[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(SymbolicLinkNamesFromUniqueIdCount));
    QueryTable[0].EntryContext = SymLinkCount as *mut core::ffi::c_void;
    *SymLinkCount = 0;

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE as u32,
        DatabasePath as *const u16,
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        (*DeviceInformation).UniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut::<core::ffi::c_void>(),
    );

    if !(Status >= 0) {
        *SymLinkCount = 0;
    }

    if !SuggestedLinkName.is_null()
        && IsDriveLetter(SuggestedLinkName) == 0
        && UseOnlyIfThereAreNoOtherLinks != 0
        && *SymLinkCount == 0
    {
        WriteNew = 1u8;
    } else {
        WriteNew = 0u8;
    }

    if HasGuid != 0 {
        *SymLinkCount += 1;
    }

    if WriteNew != 0 {
        RtlWriteRegistryValue(
            RTL_REGISTRY_ABSOLUTE as u32,
            DatabasePath as *const u16,
            (*SuggestedLinkName).Buffer as *const u16,
            REG_BINARY as u32,
            &raw mut (*(*DeviceInformation).UniqueId).UniqueId as *mut u8 as *mut core::ffi::c_void,
            (*(*DeviceInformation).UniqueId).UniqueIdLength as u32,
        );

        // recount
        core::ptr::write_bytes(
            &raw mut QueryTable as *mut core::ffi::c_void,
            0u8,
            core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
        );
        QueryTable[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(SymbolicLinkNamesFromUniqueIdCount));
        QueryTable[0].EntryContext = SymLinkCount as *mut core::ffi::c_void;
        *SymLinkCount = 0;

        Status = RtlQueryRegistryValues(
            RTL_REGISTRY_ABSOLUTE as u32,
            DatabasePath as *const u16,
            &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
            (*DeviceInformation).UniqueId as *mut core::ffi::c_void,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );

        if !(Status >= 0) {
            return STATUS_NOT_FOUND;
        }
    }

    if *SymLinkCount == 0 {
        return STATUS_NOT_FOUND;
    }

    *SymLinks = ExAllocatePoolWithTag(
        PagedPool,
        (*SymLinkCount as usize).wrapping_mul(core::mem::size_of::<_UNICODE_STRING>()),
        0x416C6C6Fu32,
    ) as PUNICODE_STRING;

    if (*SymLinks).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(
        *SymLinks as *mut core::ffi::c_void,
        0u8,
        (*SymLinkCount as usize).wrapping_mul(core::mem::size_of::<_UNICODE_STRING>()),
    );

    core::ptr::write_bytes(
        &raw mut QueryTable as *mut core::ffi::c_void,
        0u8,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );
    QueryTable[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> i32, _>(SymbolicLinkNamesFromUniqueIdQuery));

    if HasGuid == 0 {
        QueryTable[0].EntryContext = *SymLinks as *mut core::ffi::c_void;
    } else {
        Status = CreateNewVolumeName(
            *SymLinks,
            Guid as PGUID,
        );
        if !(Status >= 0) {
            ExFreePoolWithTag(*SymLinks as *mut core::ffi::c_void, 0x416C6C6Fu32);
            return Status;
        }
        QueryTable[0].EntryContext =
            (*SymLinks).add(1) as *mut core::ffi::c_void;
    }

    RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE as u32,
        DatabasePath as *const u16,
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        (*DeviceInformation).UniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut::<core::ffi::c_void>(),
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn RemoveSavedLinks(
    DeviceExtension: *mut DEVICE_EXTENSION,
    UniqueId: *mut MOUNTDEV_UNIQUE_ID,
) -> *mut SAVED_LINK_INFORMATION {
    let mut NextEntry: *mut LIST_ENTRY;
    let mut SavedLinkInformation: *mut SAVED_LINK_INFORMATION;

    if IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).SavedLinksListHead)) {
        return core::ptr::null_mut::<SAVED_LINK_INFORMATION>();
    }

    NextEntry = (*DeviceExtension).SavedLinksListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).SavedLinksListHead) {
        SavedLinkInformation = ((NextEntry as usize)
            .wrapping_sub(core::mem::offset_of!(SAVED_LINK_INFORMATION, SavedLinksListEntry)))
            as *mut SAVED_LINK_INFORMATION;

        if (*(*SavedLinkInformation).UniqueId).UniqueIdLength as i32
            == (*UniqueId).UniqueIdLength as i32
        {
            if RtlCompareMemory(
                core::ptr::addr_of!((*(*SavedLinkInformation).UniqueId).UniqueId)
                    as *const core::ffi::c_void,
                core::ptr::addr_of!((*UniqueId).UniqueId) as *const core::ffi::c_void,
                (*UniqueId).UniqueIdLength as usize,
            ) == (*UniqueId).UniqueIdLength as usize
            {
                RemoveEntryList(core::ptr::addr_of_mut!(
                    (*SavedLinkInformation).SavedLinksListEntry
                ));
                return SavedLinkInformation;
            }
        }
        NextEntry = (*NextEntry).Flink;
    }

    core::ptr::null_mut::<SAVED_LINK_INFORMATION>()
}
#[no_mangle]
pub unsafe extern "C" fn QuerySuggestedLinkName(
    SymbolicName: *mut _UNICODE_STRING,
    SuggestedLinkName: *mut _UNICODE_STRING,
    UseOnlyIfThereAreNoOtherLinks: *mut u8,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut NameLength: u16;
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();
    let mut IoCtlSuggested: *mut MOUNTDEV_SUGGESTED_LINK_NAME;

    Status = IoGetDeviceObjectPointer(
        SymbolicName,
        FILE_READ_ATTRIBUTES,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        return Status;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    IoCtlSuggested = ExAllocatePoolWithTag(
        0u32,
        core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>(),
        1098149453u32,
    ) as *mut MOUNTDEV_SUGGESTED_LINK_NAME;
    if IoCtlSuggested.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        let mut current_block: u64;

        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME,
            DeviceObject,
            core::ptr::null_mut(),
            0u32,
            IoCtlSuggested as *mut core::ffi::c_void,
            core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>() as u32,
            FileObject,
        );
        if Status == STATUS_BUFFER_OVERFLOW {
            NameLength = ((*IoCtlSuggested).NameLength as usize)
                .wrapping_add(core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>())
                as u16;
            ExFreePoolWithTag(
                IoCtlSuggested as *mut core::ffi::c_void,
                1098149453u32,
            );
            IoCtlSuggested = ExAllocatePoolWithTag(
                0u32,
                NameLength as usize,
                1098149453u32,
            ) as *mut MOUNTDEV_SUGGESTED_LINK_NAME;
            if IoCtlSuggested.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                current_block = 3513003631995291863u64;
            } else {
                Status = MountMgrSendSyncDeviceIoCtl(
                    IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME,
                    DeviceObject,
                    core::ptr::null_mut(),
                    0u32,
                    IoCtlSuggested as *mut core::ffi::c_void,
                    NameLength as u32,
                    FileObject,
                );
                current_block = 3512920355445576850u64;
            }
        } else {
            current_block = 3512920355445576850u64;
        }

        match current_block {
            3513003631995291863u64 => {}
            _ => {
                if Status >= 0 {
                    (*SuggestedLinkName).Length = (*IoCtlSuggested).NameLength;
                    (*SuggestedLinkName).MaximumLength = ((*IoCtlSuggested).NameLength as usize)
                        .wrapping_add(core::mem::size_of::<u16>())
                        as u16;
                    (*SuggestedLinkName).Buffer = ExAllocatePoolWithTag(
                        0u32,
                        ((*IoCtlSuggested).NameLength as usize)
                            .wrapping_add(core::mem::size_of::<u16>()),
                        1098149453u32,
                    ) as *mut u16;
                    if (*SuggestedLinkName).Buffer.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    } else {
                        core::ptr::copy_nonoverlapping::<u8>(
                            &raw mut (*IoCtlSuggested).Name as *mut u8,
                            (*SuggestedLinkName).Buffer as *mut u8,
                            (*IoCtlSuggested).NameLength as usize,
                        );
                        *(*SuggestedLinkName)
                            .Buffer
                            .add(
                                ((*SuggestedLinkName).Length as usize)
                                    / core::mem::size_of::<u16>(),
                            ) = 0u16;
                    }
                    *UseOnlyIfThereAreNoOtherLinks =
                        (*IoCtlSuggested).UseOnlyIfThereAreNoOtherLinks as u8;
                }
                ExFreePoolWithTag(
                    IoCtlSuggested as *mut core::ffi::c_void,
                    1098149453u32,
                );
            }
        }
    }

    ntkrnl_sys::ob::ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    ntkrnl_sys::ob::ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    Status
}

#[no_mangle]
pub unsafe extern "C" fn RedirectSavedLink(
    SavedLinkInformation: *mut SAVED_LINK_INFORMATION,
    DosName: PUNICODE_STRING,
    NewLink: PUNICODE_STRING,
) -> BOOLEAN {
    let mut NextEntry: *mut LIST_ENTRY;
    let mut SymlinkInformation: *mut SYMLINK_INFORMATION;

    NextEntry = (*SavedLinkInformation).SymbolicLinksListHead.Flink;
    while NextEntry != &raw mut (*SavedLinkInformation).SymbolicLinksListHead {
        SymlinkInformation = (NextEntry as usize).wrapping_sub(
            &raw mut (*::core::ptr::null_mut::<SYMLINK_INFORMATION>()).SymbolicLinksListEntry
                as usize,
        ) as *mut SYMLINK_INFORMATION;

        if RtlEqualUnicodeString(
            DosName as *const _UNICODE_STRING,
            &raw mut (*SymlinkInformation).Name,
            1u8,
        ) == 0
        {
            GlobalDeleteSymbolicLink(DosName);
            GlobalCreateSymbolicLink(DosName, NewLink);
            RemoveEntryList(&raw mut (*SymlinkInformation).SymbolicLinksListEntry);
            ExFreePoolWithTag(
                (*SymlinkInformation).Name.Buffer as *mut core::ffi::c_void,
                1098149453u32,
            );
            ExFreePoolWithTag(SymlinkInformation as *mut core::ffi::c_void, 1098149453u32);
            return 1;
        }
        NextEntry = (*NextEntry).Flink;
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteSymbolicLinkNameFromMemory(
    DeviceExtension: PDEVICE_EXTENSION,
    SymbolicLink: PUNICODE_STRING,
    MarkOffline: BOOLEAN,
) {
    if IsListEmpty(core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead)) {
        return;
    }
    let mut DeviceEntry = (*DeviceExtension).DeviceListHead.Flink;
    while DeviceEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        let DeviceInformation = ((DeviceEntry as *mut u8)
            .sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry)))
            as *mut DEVICE_INFORMATION;
        let mut SymbolEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
        while SymbolEntry != core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead) {
            let SymlinkInformation = ((SymbolEntry as *mut u8)
                .sub(core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry)))
                as *mut SYMLINK_INFORMATION;
            if RtlCompareUnicodeString(
                SymbolicLink as *const _UNICODE_STRING,
                core::ptr::addr_of!((*SymlinkInformation).Name) as *const _UNICODE_STRING,
                1,
            ) == 0
            {
                if MarkOffline != 0 {
                    (*SymlinkInformation).Online = 0;
                } else {
                    SendLinkDeleted(
                        core::ptr::addr_of_mut!((*DeviceInformation).SymbolicName),
                        SymbolicLink,
                    );
                    RemoveEntryList(core::ptr::addr_of_mut!(
                        (*SymlinkInformation).SymbolicLinksListEntry
                    ));
                    FreePool((*SymlinkInformation).Name.Buffer as *mut core::ffi::c_void);
                    FreePool(SymlinkInformation as *mut core::ffi::c_void);
                }
                return;
            }
            SymbolEntry = (*SymbolEntry).Flink;
        }
        DeviceEntry = (*DeviceEntry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn IsDriveLetter(SymbolicName: PUNICODE_STRING) -> BOOLEAN {
    let mut Letter: WCHAR = 0;
    let mut Colon: WCHAR = 0;
    if (*SymbolicName).Length as usize
        != (DosDevices.Length as usize)
            .wrapping_add(
                (2 as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            )
    {
        return 0;
    }
    if RtlPrefixUnicodeString(
        &raw const DosDevices as PCUNICODE_STRING,
        SymbolicName as PCUNICODE_STRING,
        1,
    ) == 0
    {
        return 0;
    }
    Letter = *(*SymbolicName).Buffer.offset(
        (DosDevices.Length as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as isize,
    );
    if (Letter < 0x41u16 || Letter > 0x5Au16) && Letter != 0xFFFFu16 {
        return 0;
    }
    Colon = *(*SymbolicName).Buffer.offset(
        (DosDevices.Length as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_add(1usize) as isize,
    );
    if Colon != 0x3Au16 {
        return 0;
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrQuerySymbolicLink(
    SymbolicName: PUNICODE_STRING,
    LinkTarget: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut LinkHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    // InitializeObjectAttributes inline
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = SymbolicName;
    ObjectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenSymbolicLinkObject(
        &raw mut LinkHandle,
        GENERIC_READ,
        &raw mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ZwQuerySymbolicLinkObject(
        LinkHandle,
        LinkTarget,
        core::ptr::null_mut(),
    );

    ZwClose(LinkHandle);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    // If the target is only a single null terminator, nothing to do
    if (*LinkTarget).Length as usize <= core::mem::size_of::<u16>() {
        return Status;
    }

    let len_chars = ((*LinkTarget).Length as usize) / core::mem::size_of::<u16>();
    let last_char_ptr = (*LinkTarget).Buffer.offset((len_chars - 1) as isize);

    // If it doesn't end with a backslash, return as-is
    if *last_char_ptr != 0x5Cu16 {
        return Status;
    }

    // Remove the trailing backslash
    (*LinkTarget).Length = ((*LinkTarget).Length as u64)
        .wrapping_sub(core::mem::size_of::<u16>() as u64) as u16;
    let null_pos = ((*LinkTarget).Length as usize) / core::mem::size_of::<u16>();
    *(*LinkTarget).Buffer.offset(null_pos as isize) = UNICODE_NULL;

    Status
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
//  * FILE:             drivers/filesystem/mountmgr/symlink.c
//  * PURPOSE:          Mount Manager - Symbolic links functions
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* Deprecated Windows 2000/XP versions of IOCTL_MOUNTDEV_LINK_[CREATED|DELETED]
//  * without access protection, that were updated in Windows 2003.
//  * They are sent to MountMgr clients if they do not recognize the new IOCTLs
//  * (e.g. they are for an older NT version). */
// #if (NTDDI_VERSION >= NTDDI_WS03)
// #define IOCTL_MOUNTDEV_LINK_CREATED_UNSECURE_DEPRECATED     CTL_CODE(MOUNTDEVCONTROLTYPE, 4, METHOD_BUFFERED, FILE_ANY_ACCESS)
// #define IOCTL_MOUNTDEV_LINK_DELETED_UNSECURE_DEPRECATED     CTL_CODE(MOUNTDEVCONTROLTYPE, 5, METHOD_BUFFERED, FILE_ANY_ACCESS)
// #endif
//
// UNICODE_STRING DeviceMount = RTL_CONSTANT_STRING(MOUNTMGR_DEVICE_NAME);
// UNICODE_STRING DosDevicesMount = RTL_CONSTANT_STRING(L"\\DosDevices\\MountPointManager");
// UNICODE_STRING DosDevices = RTL_CONSTANT_STRING(L"\\DosDevices\\");
// UNICODE_STRING DeviceFloppy = RTL_CONSTANT_STRING(L"\\Device\\Floppy");
// UNICODE_STRING DeviceCdRom = RTL_CONSTANT_STRING(L"\\Device\\CdRom");
// UNICODE_STRING DosGlobal = RTL_CONSTANT_STRING(L"\\GLOBAL??\\");
// UNICODE_STRING Global = RTL_CONSTANT_STRING(L"\\??\\");
// UNICODE_STRING SafeVolumes = RTL_CONSTANT_STRING(L"\\Device\\VolumesSafeForWriteAccess");
// UNICODE_STRING Volume = RTL_CONSTANT_STRING(L"\\??\\Volume");
// UNICODE_STRING ReparseIndex = RTL_CONSTANT_STRING(L"\\$Extend\\$Reparse:$R:$INDEX_ALLOCATION");
//
// /*
//  * @implemented
//  */
// NTSTATUS
// CreateStringWithGlobal(IN PUNICODE_STRING DosName,
//                        OUT PUNICODE_STRING GlobalString)
// {
//     UNICODE_STRING IntGlobal;
//
//     if (RtlPrefixUnicodeString(&DosDevices, DosName, TRUE))
//     {
//         /* DOS device - use DOS global */
//         IntGlobal.Length = DosName->Length - DosDevices.Length + DosGlobal.Length;
//         IntGlobal.MaximumLength = IntGlobal.Length + sizeof(WCHAR);
//         IntGlobal.Buffer = AllocatePool(IntGlobal.MaximumLength);
//         if (!IntGlobal.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(IntGlobal.Buffer, DosGlobal.Buffer, DosGlobal.Length);
//         RtlCopyMemory(IntGlobal.Buffer + (DosGlobal.Length / sizeof(WCHAR)),
//                       DosName->Buffer + (DosDevices.Length / sizeof(WCHAR)),
//                       DosName->Length - DosDevices.Length);
//         IntGlobal.Buffer[IntGlobal.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     else if (RtlPrefixUnicodeString(&Global, DosName, TRUE))
//     {
//         /* Switch to DOS global */
//         IntGlobal.Length = DosName->Length - Global.Length + DosGlobal.Length;
//         IntGlobal.MaximumLength = IntGlobal.Length + sizeof(WCHAR);
//         IntGlobal.Buffer = AllocatePool(IntGlobal.MaximumLength);
//         if (!IntGlobal.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(IntGlobal.Buffer, DosGlobal.Buffer, DosGlobal.Length);
//         RtlCopyMemory(IntGlobal.Buffer + (DosGlobal.Length / sizeof(WCHAR)),
//                       DosName->Buffer + (Global.Length / sizeof(WCHAR)),
//                       DosName->Length - Global.Length);
//         IntGlobal.Buffer[IntGlobal.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     else
//     {
//         /* Simply duplicate string */
//         IntGlobal.Length = DosName->Length;
//         IntGlobal.MaximumLength = DosName->MaximumLength;
//         IntGlobal.Buffer = AllocatePool(IntGlobal.MaximumLength);
//         if (!IntGlobal.Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlCopyMemory(IntGlobal.Buffer, DosName->Buffer, IntGlobal.MaximumLength);
//     }
//
//     /* Return string */
//     GlobalString->Length = IntGlobal.Length;
//     GlobalString->MaximumLength = IntGlobal.MaximumLength;
//     GlobalString->Buffer = IntGlobal.Buffer;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// GlobalCreateSymbolicLink(IN PUNICODE_STRING DosName,
//                          IN PUNICODE_STRING DeviceName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING GlobalName;
//
//     /* First create the global string */
//     Status = CreateStringWithGlobal(DosName, &GlobalName);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Then, create the symlink */
//     Status = IoCreateSymbolicLink(&GlobalName, DeviceName);
//
//     FreePool(GlobalName.Buffer);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// GlobalDeleteSymbolicLink(IN PUNICODE_STRING DosName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING GlobalName;
//
//     /* Recreate the string (to find the link) */
//     Status = CreateStringWithGlobal(DosName, &GlobalName);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* And delete the link */
//     Status = IoDeleteSymbolicLink(&GlobalName);
//
//     FreePool(GlobalName.Buffer);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// SendLinkCreated(IN PUNICODE_STRING SymbolicName)
// {
//     NTSTATUS Status;
//     ULONG NameSize;
//     PFILE_OBJECT FileObject;
//     PMOUNTDEV_NAME Name = NULL;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Get the device associated with the name */
//     Status = IoGetDeviceObjectPointer(SymbolicName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//         return;
//
//     /* Get attached device (will notify it) */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     /* NameSize is the size of the whole MOUNTDEV_NAME structure */
//     NameSize = sizeof(USHORT) + SymbolicName->Length;
//     Name = AllocatePool(NameSize);
//     if (!Name)
//         goto Cleanup;
//
//     /* Initialize structure */
//     Name->NameLength = SymbolicName->Length;
//     RtlCopyMemory(Name->Name, SymbolicName->Buffer, SymbolicName->Length);
//
//     /* Send the notification, using the new (Windows 2003+) IOCTL definition
//      * (with limited access) first. If this fails, the called driver may be
//      * for an older NT version, and so, we send again the notification using
//      * the old (Windows 2000) IOCTL definition (with any access). */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_LINK_CREATED,
//                                          DeviceObject,
//                                          Name,
//                                          NameSize,
//                                          NULL,
//                                          0,
//                                          FileObject);
//     /* This one can fail, no one matters */
//     UNREFERENCED_PARAMETER(Status);
// #if (NTDDI_VERSION >= NTDDI_WS03)
//     /* Then, second one */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_LINK_CREATED_UNSECURE_DEPRECATED,
//                                          DeviceObject,
//                                          Name,
//                                          NameSize,
//                                          NULL,
//                                          0,
//                                          FileObject);
//     UNREFERENCED_PARAMETER(Status);
// #endif // (NTDDI_VERSION >= NTDDI_WS03)
//
// Cleanup:
//     if (Name)
//         FreePool(Name);
//
//     ObDereferenceObject(DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// SendLinkDeleted(IN PUNICODE_STRING DeviceName,
//                 IN PUNICODE_STRING SymbolicName)
// {
//     NTSTATUS Status;
//     ULONG NameSize;
//     PFILE_OBJECT FileObject;
//     PMOUNTDEV_NAME Name = NULL;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Get the device associated with the name */
//     Status = IoGetDeviceObjectPointer(DeviceName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//         return;
//
//     /* Get attached device (will notify it) */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     /* NameSize is the size of the whole MOUNTDEV_NAME structure */
//     NameSize = sizeof(USHORT) + SymbolicName->Length;
//     Name = AllocatePool(NameSize);
//     if (!Name)
//         goto Cleanup;
//
//     /* Initialize structure */
//     Name->NameLength = SymbolicName->Length;
//     RtlCopyMemory(Name->Name, SymbolicName->Buffer, SymbolicName->Length);
//
//     /* Send the notification, using the new (Windows 2003+) IOCTL definition
//      * (with limited access) first. If this fails, the called driver may be
//      * for an older NT version, and so, we send again the notification using
//      * the old (Windows 2000) IOCTL definition (with any access). */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_LINK_DELETED,
//                                          DeviceObject,
//                                          Name,
//                                          NameSize,
//                                          NULL,
//                                          0,
//                                          FileObject);
//     /* This one can fail, no one matters */
//     UNREFERENCED_PARAMETER(Status);
// #if (NTDDI_VERSION >= NTDDI_WS03)
//     /* Then, second one */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_LINK_DELETED_UNSECURE_DEPRECATED,
//                                          DeviceObject,
//                                          Name,
//                                          NameSize,
//                                          NULL,
//                                          0,
//                                          FileObject);
//     UNREFERENCED_PARAMETER(Status);
// #endif // (NTDDI_VERSION >= NTDDI_WS03)
//
// Cleanup:
//     if (Name)
//         FreePool(Name);
//
//     ObDereferenceObject(DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// SymbolicLinkNamesFromUniqueIdCount(IN PWSTR ValueName,
//                                    IN ULONG ValueType,
//                                    IN PVOID ValueData,
//                                    IN ULONG ValueLength,
//                                    IN PVOID Context,
//                                    IN PVOID EntryContext)
// {
//     UNICODE_STRING ValueNameString;
//     PMOUNTDEV_UNIQUE_ID UniqueId = Context;
//
//     if (ValueName[0] != L'#' || ValueType != REG_BINARY ||
//         (UniqueId->UniqueIdLength != ValueLength))
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (RtlCompareMemory(UniqueId->UniqueId, ValueData, ValueLength) != ValueLength)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* That one matched, increase count */
//     RtlInitUnicodeString(&ValueNameString, ValueName);
//     if (ValueNameString.Length)
//     {
//         (*((PULONG)EntryContext))++;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI

// driver-internal cdecl decls (types_*.rs authority = extern "C").
unsafe extern "C" {
    fn MountMgrSendSyncDeviceIoCtl( IoControlCode: u32, DeviceObject: *mut DEVICE_OBJECT, InputBuffer: *mut core::ffi::c_void, InputBufferLength: u32, OutputBuffer: *mut core::ffi::c_void, OutputBufferLength: u32, FileObject: *mut FILE_OBJECT, ) -> NTSTATUS;
}
// SymbolicLinkNamesFromUniqueIdQuery(IN PWSTR ValueName,
//                                    IN ULONG ValueType,
//                                    IN PVOID ValueData,
//                                    IN ULONG ValueLength,
//                                    IN PVOID Context,
//                                    IN PVOID EntryContext)
// {
//     UNICODE_STRING ValueNameString;
//     PMOUNTDEV_UNIQUE_ID UniqueId = Context;
//     /* Unicode strings table */
//     PUNICODE_STRING ReturnString = EntryContext;
//
//     if (ValueName[0] != L'#' || ValueType != REG_BINARY ||
//         (UniqueId->UniqueIdLength != ValueLength))
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (RtlCompareMemory(UniqueId->UniqueId, ValueData, ValueLength) != ValueLength)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Unique ID matches, let's put the symlink */
//     RtlInitUnicodeString(&ValueNameString, ValueName);
//     if (!ValueNameString.Length)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Allocate string to copy */
//     ValueNameString.Buffer = AllocatePool(ValueNameString.MaximumLength);
//     if (!ValueNameString.Buffer)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Copy */
//     RtlCopyMemory(ValueNameString.Buffer, ValueName, ValueNameString.Length);
//     ValueNameString.Buffer[ValueNameString.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     while (ReturnString->Length)
//     {
//         ReturnString++;
//     }
//
//     /* And return that string */
//     *ReturnString = ValueNameString;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// CreateNewVolumeName(OUT PUNICODE_STRING VolumeName,
//                     IN PGUID VolumeGuid OPTIONAL)
// {
//     GUID Guid;
//     NTSTATUS Status;
//     UNICODE_STRING GuidString;
//
//     /* If no GUID was provided, then create one */
//     if (!VolumeGuid)
//     {
//         Status = ExUuidCreate(&Guid);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//     else
//     {
//         RtlCopyMemory(&Guid, VolumeGuid, sizeof(GUID));
//     }
//
//     /* Convert GUID to string */
//     Status = RtlStringFromGUID(&Guid, &GuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Size for volume namespace, literal GUID, and null char */
//     VolumeName->MaximumLength = 0x14 + 0x4C + sizeof(UNICODE_NULL);
//     VolumeName->Buffer = AllocatePool(0x14 + 0x4C + sizeof(UNICODE_NULL));
//     if (!VolumeName->Buffer)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         RtlCopyUnicodeString(VolumeName, &Volume);
//         RtlAppendUnicodeStringToString(VolumeName, &GuidString);
//         VolumeName->Buffer[VolumeName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//         Status = STATUS_SUCCESS;
//     }
//
//     ExFreePoolWithTag(GuidString.Buffer, 0);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QuerySymbolicLinkNamesFromStorage(IN PDEVICE_EXTENSION DeviceExtension,
//                                   IN PDEVICE_INFORMATION DeviceInformation,
//                                   IN PUNICODE_STRING SuggestedLinkName,
//                                   IN BOOLEAN UseOnlyIfThereAreNoOtherLinks,
//                                   OUT PUNICODE_STRING * SymLinks,
//                                   OUT PULONG SymLinkCount,
//                                   IN BOOLEAN HasGuid,
//                                   IN LPGUID Guid)
// {
//     NTSTATUS Status;
//     BOOLEAN WriteNew;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     UNREFERENCED_PARAMETER(DeviceExtension);
//
//     /* First of all, count links */
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = SymbolicLinkNamesFromUniqueIdCount;
//     QueryTable[0].EntryContext = SymLinkCount;
//     *SymLinkCount = 0;
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                     DatabasePath,
//                                     QueryTable,
//                                     DeviceInformation->UniqueId,
//                                     NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         *SymLinkCount = 0;
//     }
//
//     /* Check if we have to write a new one first */
//     if (SuggestedLinkName && !IsDriveLetter(SuggestedLinkName) &&
//         UseOnlyIfThereAreNoOtherLinks && *SymLinkCount == 0)
//     {
//         WriteNew = TRUE;
//     }
//     else
//     {
//         WriteNew = FALSE;
//     }
//
//     /* If has GUID, it makes one more link */
//     if (HasGuid)
//     {
//         (*SymLinkCount)++;
//     }
//
//     if (WriteNew)
//     {
//         /* Write link */
//         RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                               DatabasePath,
//                               SuggestedLinkName->Buffer,
//                               REG_BINARY,
//                               DeviceInformation->UniqueId->UniqueId,
//                               DeviceInformation->UniqueId->UniqueIdLength);
//
//         /* And recount all the needed links */
//         RtlZeroMemory(QueryTable, sizeof(QueryTable));
//         QueryTable[0].QueryRoutine = SymbolicLinkNamesFromUniqueIdCount;
//         QueryTable[0].EntryContext = SymLinkCount;
//         *SymLinkCount = 0;
//
//         Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                         DatabasePath,
//                                         QueryTable,
//                                         DeviceInformation->UniqueId,
//                                         NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             return STATUS_NOT_FOUND;
//         }
//     }
//
//     /* Not links found? */
//     if (!*SymLinkCount)
//     {
//         return STATUS_NOT_FOUND;
//     }
//
//     /* Allocate a buffer big enough to hold symlinks (table of unicode strings) */
//     *SymLinks = AllocatePool(*SymLinkCount * sizeof(UNICODE_STRING));
//     if (!*SymLinks)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Prepare to query links */
//     RtlZeroMemory(*SymLinks, *SymLinkCount * sizeof(UNICODE_STRING));
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = SymbolicLinkNamesFromUniqueIdQuery;
//
//     /* No GUID? Keep it that way */
//     if (!HasGuid)
//     {
//         QueryTable[0].EntryContext = *SymLinks;
//     }
//     /* Otherwise, first create volume name */
//     else
//     {
//         Status = CreateNewVolumeName(SymLinks[0], Guid);
//         if (!NT_SUCCESS(Status))
//         {
//             FreePool(*SymLinks);
//             return Status;
//         }
//
//         /* Skip first link (ours) */
//         QueryTable[0].EntryContext = *SymLinks + 1;
//     }
//
//     /* Now, query */
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            DeviceInformation->UniqueId,
//                            NULL);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// PSAVED_LINK_INFORMATION
// RemoveSavedLinks(IN PDEVICE_EXTENSION DeviceExtension,
//                  IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     PLIST_ENTRY NextEntry;
//     PSAVED_LINK_INFORMATION SavedLinkInformation;
//
//     /* No saved links? Easy! */
//     if (IsListEmpty(&(DeviceExtension->SavedLinksListHead)))
//     {
//         return NULL;
//     }
//
//     /* Now, browse saved links */
//     for (NextEntry = DeviceExtension->SavedLinksListHead.Flink;
//          NextEntry != &(DeviceExtension->SavedLinksListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         SavedLinkInformation = CONTAINING_RECORD(NextEntry,
//                                                  SAVED_LINK_INFORMATION,
//                                                  SavedLinksListEntry);
//
//         /* Find the one that matches */
//         if (SavedLinkInformation->UniqueId->UniqueIdLength == UniqueId->UniqueIdLength)
//         {
//             if (RtlCompareMemory(SavedLinkInformation->UniqueId->UniqueId,
//                                  UniqueId->UniqueId,
//                                  UniqueId->UniqueIdLength) ==
//                 UniqueId->UniqueIdLength)
//             {
//                 /* Remove it and return it */
//                 RemoveEntryList(&(SavedLinkInformation->SavedLinksListEntry));
//                 return SavedLinkInformation;
//             }
//         }
//     }
//
//     /* None found (none removed) */
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QuerySuggestedLinkName(IN PUNICODE_STRING SymbolicName,
//                        OUT PUNICODE_STRING SuggestedLinkName,
//                        OUT PBOOLEAN UseOnlyIfThereAreNoOtherLinks)
// {
//     NTSTATUS Status;
//     USHORT NameLength;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     PMOUNTDEV_SUGGESTED_LINK_NAME IoCtlSuggested;
//
//     /* First, get device */
//     Status = IoGetDeviceObjectPointer(SymbolicName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Then, get attached device */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     /* Then, prepare buffer to query suggested name */
//     IoCtlSuggested = AllocatePool(sizeof(MOUNTDEV_SUGGESTED_LINK_NAME));
//     if (!IoCtlSuggested)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Dereference;
//     }
//
//     /* Prepare request */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME,
//                                          DeviceObject,
//                                          NULL,
//                                          0,
//                                          IoCtlSuggested,
//                                          sizeof(MOUNTDEV_SUGGESTED_LINK_NAME),
//                                          FileObject);
//     /* Retry with appropriate length */
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         /* Reallocate big enough buffer */
//         NameLength = IoCtlSuggested->NameLength + sizeof(MOUNTDEV_SUGGESTED_LINK_NAME);
//         FreePool(IoCtlSuggested);
//
//         IoCtlSuggested = AllocatePool(NameLength);
//         if (!IoCtlSuggested)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Dereference;
//         }
//
//         /* And re-ask */
//         Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME,
//                                              DeviceObject,
//                                              NULL,
//                                              0,
//                                              IoCtlSuggested,
//                                              NameLength,
//                                              FileObject);
//     }
//     if (!NT_SUCCESS(Status))
//         goto Release;
//
//     /* Now we have suggested name, copy it */
//     SuggestedLinkName->Length = IoCtlSuggested->NameLength;
//     SuggestedLinkName->MaximumLength = IoCtlSuggested->NameLength + sizeof(UNICODE_NULL);
//     SuggestedLinkName->Buffer = AllocatePool(IoCtlSuggested->NameLength + sizeof(UNICODE_NULL));
//     if (!SuggestedLinkName->Buffer)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         RtlCopyMemory(SuggestedLinkName->Buffer, IoCtlSuggested->Name, IoCtlSuggested->NameLength);
//         SuggestedLinkName->Buffer[SuggestedLinkName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* Also return its priority */
//     *UseOnlyIfThereAreNoOtherLinks = IoCtlSuggested->UseOnlyIfThereAreNoOtherLinks;
//
// Release:
//     FreePool(IoCtlSuggested);
//
// Dereference:
//     ObDereferenceObject(DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// RedirectSavedLink(IN PSAVED_LINK_INFORMATION SavedLinkInformation,
//                   IN PUNICODE_STRING DosName,
//                   IN PUNICODE_STRING NewLink)
// {
//     PLIST_ENTRY NextEntry;
//     PSYMLINK_INFORMATION SymlinkInformation;
//
//     /* Find the link */
//     for (NextEntry = SavedLinkInformation->SymbolicLinksListHead.Flink;
//          NextEntry != &(SavedLinkInformation->SymbolicLinksListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         SymlinkInformation = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         if (!RtlEqualUnicodeString(DosName, &(SymlinkInformation->Name), TRUE))
//         {
//             /* Delete old link */
//             GlobalDeleteSymbolicLink(DosName);
//             /* Set its new location */
//             GlobalCreateSymbolicLink(DosName, NewLink);
//
//             /* And remove it from the list (not valid any more) */
//             RemoveEntryList(&(SymlinkInformation->SymbolicLinksListEntry));
//             FreePool(SymlinkInformation->Name.Buffer);
//             FreePool(SymlinkInformation);
//
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// DeleteSymbolicLinkNameFromMemory(IN PDEVICE_EXTENSION DeviceExtension,
//                                  IN PUNICODE_STRING SymbolicLink,
//                                  IN BOOLEAN MarkOffline)
// {
//     PLIST_ENTRY DeviceEntry, SymbolEntry;
//     PDEVICE_INFORMATION DeviceInformation;
//     PSYMLINK_INFORMATION SymlinkInformation;
//
//     /* First of all, ensure we have devices */
//     if (IsListEmpty(&(DeviceExtension->DeviceListHead)))
//     {
//         return;
//     }
//
//     /* Then, look for the symbolic name */
//     for (DeviceEntry = DeviceExtension->DeviceListHead.Flink;
//          DeviceEntry != &(DeviceExtension->DeviceListHead);
//          DeviceEntry = DeviceEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(DeviceEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         for (SymbolEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//              SymbolEntry != &(DeviceInformation->SymbolicLinksListHead);
//              SymbolEntry = SymbolEntry->Flink)
//         {
//             SymlinkInformation = CONTAINING_RECORD(SymbolEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             /* One we have found it */
//             if (RtlCompareUnicodeString(SymbolicLink, &(SymlinkInformation->Name), TRUE) == 0)
//             {
//                 /* Check if caller just want it to be offline */
//                 if (MarkOffline)
//                 {
//                     SymlinkInformation->Online = FALSE;
//                 }
//                 else
//                 {
//                     /* If not, delete it & notify */
//                     SendLinkDeleted(&(DeviceInformation->SymbolicName), SymbolicLink);
//                     RemoveEntryList(&(SymlinkInformation->SymbolicLinksListEntry));
//
//                     FreePool(SymlinkInformation->Name.Buffer);
//                     FreePool(SymlinkInformation);
//                 }
//
//                 /* No need to go farther */
//                 return;
//             }
//         }
//     }
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// IsDriveLetter(PUNICODE_STRING SymbolicName)
// {
//     WCHAR Letter, Colon;
//
//     /* We must have a precise length */
//     if (SymbolicName->Length != DosDevices.Length + 2 * sizeof(WCHAR))
//     {
//         return FALSE;
//     }
//
//     /* Must start with the DosDevices prefix */
//     if (!RtlPrefixUnicodeString(&DosDevices, SymbolicName, TRUE))
//     {
//         return FALSE;
//     }
//
//     /* Check if letter is correct */
//     Letter = SymbolicName->Buffer[DosDevices.Length / sizeof(WCHAR)];
//     if ((Letter < L'A' || Letter > L'Z') && Letter != (WCHAR)-1)
//     {
//         return FALSE;
//     }
//
//     /* And finally it must end with a colon */
//     Colon = SymbolicName->Buffer[DosDevices.Length / sizeof(WCHAR) + 1];
//     if (Colon != L':')
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrQuerySymbolicLink(IN PUNICODE_STRING SymbolicName,
//                           IN OUT PUNICODE_STRING LinkTarget)
// {
//     NTSTATUS Status;
//     HANDLE LinkHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Open the symbolic link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                SymbolicName,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = ZwOpenSymbolicLinkObject(&LinkHandle,
//                                       GENERIC_READ,
//                                       &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Query its target */
//     Status = ZwQuerySymbolicLinkObject(LinkHandle,
//                                        LinkTarget,
//                                        NULL);
//
//     ZwClose(LinkHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (LinkTarget->Length <= sizeof(WCHAR))
//     {
//         return Status;
//     }
//
//     /* If it's not finished by \, just return */
//     if (LinkTarget->Buffer[LinkTarget->Length / sizeof(WCHAR) - 1] != L'\\')
//     {
//         return Status;
//     }
//
//     /* Otherwise, ensure to drop the tailing \ */
//     LinkTarget->Length -= sizeof(WCHAR);
//     LinkTarget->Buffer[LinkTarget->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
