use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK, _NT_PRODUCT_TYPE_NtProductWinNt, _OBJECT_ATTRIBUTES, _RTL_BSD_ITEM_TYPE_RtlBsdItemMax, _UNICODE_STRING, BOOLEAN, HANDLE, LARGE_INTEGER, PBOOLEAN, PHANDLE, PRTL_BSD_DATA, PULONG, RTL_BSD_DATA, RTL_BSD_ITEM_TYPE, ZwClose, ZwCreateFile, ZwFlushBuffersFile, ZwOpenFile, ZwReadFile, ZwWriteFile};
use core::ffi::c_void;

// ---- constants from sdk/include/xdk/winnt_old.h ----
// FILE_GENERIC_READ = STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE
pub const FILE_GENERIC_READ: ULONG = STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE;
// FILE_GENERIC_WRITE = STANDARD_RIGHTS_WRITE | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA | SYNCHRONIZE
pub const FILE_GENERIC_WRITE: ULONG = STANDARD_RIGHTS_WRITE | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA | SYNCHRONIZE;
pub const FILE_ATTRIBUTE_SYSTEM: ULONG = 0x00000004;
pub const FILE_CREATE: ULONG = 0x00000002;
pub const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;

// Convenience re-exports with the short names used in the original C
pub const NtProductWinNt: ULONG = _NT_PRODUCT_TYPE_NtProductWinNt as ULONG;
pub const RtlBsdItemMax: ULONG = _RTL_BSD_ITEM_TYPE_RtlBsdItemMax as ULONG;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _RTL_BSD_ITEM_TABLE_ENTRY {
    pub Offset: UCHAR,
    pub Size: UCHAR,
}
pub type RTL_BSD_ITEM_TABLE_ENTRY = _RTL_BSD_ITEM_TABLE_ENTRY;

pub static mut DummyBsd: PRTL_BSD_DATA = core::ptr::null_mut();

pub static BsdItemTable: [RTL_BSD_ITEM_TABLE_ENTRY; RtlBsdItemMax as usize] = [
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, Version) as UCHAR,
        // Faithful to C: sizeof(&DummyBsd->Version) == sizeof(pointer), like
        // every other entry (8 on 64-bit, not size_of::<ULONG>()==4).
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, ProductType) as UCHAR,
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, AabEnabled) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, AabTimeout) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootSucceeded) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootShutdown) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, SleepInProgress) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, PowerTransition) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, BootAttemptCount) as UCHAR,
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootCheckpoint) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootId) as UCHAR,
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, LastSuccessfulShutdownBootId) as UCHAR,
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, LastReportedAbnormalShutdownBootId) as UCHAR,
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, ErrorInfo) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, PowerButtonPressInfo) as UCHAR,
        Size: core::mem::size_of::<*mut u8>() as UCHAR,
    },
    RTL_BSD_ITEM_TABLE_ENTRY {
        Offset: core::mem::offset_of!(RTL_BSD_DATA, Checksum) as UCHAR,
        Size: core::mem::size_of::<*mut u32>() as UCHAR,
    },
];

#[no_mangle]
pub unsafe extern "system" fn RtlCreateBootStatusDataFile() -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut AllocationSize: LARGE_INTEGER = core::mem::zeroed();
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();

    let mut FileNameBuffer: [u16; 24] = [
        0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x52, 0x6F, 0x6F, 0x74, 0x5C,
        0x62, 0x6F, 0x6F, 0x74, 0x73, 0x74, 0x61, 0x74, 0x2E, 0x64, 0x61, 0x74,
    ];
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    FileName.Length = (FileNameBuffer.len() * 2) as u16;
    FileName.MaximumLength = (FileNameBuffer.len() * 2) as u16;
    FileName.Buffer = FileNameBuffer.as_mut_ptr();

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;

    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut InitialBsd: RTL_BSD_DATA = core::mem::zeroed();

    AllocationSize.QuadPart = 0x800;

    Status = ZwCreateFile(
        &mut FileHandle,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_SYSTEM,
        0,
        FILE_CREATE,
        FILE_SYNCHRONOUS_IO_NONALERT,
        core::ptr::null_mut(),
        0,
    );

    if NT_SUCCESS(Status) {
        RtlZeroMemory(
            &mut InitialBsd as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<RTL_BSD_DATA>() as usize,
        );
        InitialBsd.Version = core::mem::size_of::<RTL_BSD_DATA>() as u32;
        InitialBsd.ProductType = NtProductWinNt;
        InitialBsd.AabEnabled = 1;
        InitialBsd.AabTimeout = 30;
        InitialBsd.LastBootSucceeded = 1u8;

        ByteOffset.QuadPart = 0;
        Status = ZwWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            &mut InitialBsd as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<RTL_BSD_DATA>() as u32,
            &mut ByteOffset,
            core::ptr::null_mut(),
        );
    }

    ZwClose(FileHandle);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetSetBootStatusData(
    FileHandle: HANDLE,
    Read: BOOLEAN,
    DataClass: RTL_BSD_ITEM_TYPE,
    Buffer: PVOID,
    BufferSize: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let Status: NTSTATUS;

    if DataClass >= RtlBsdItemMax {
        return STATUS_INVALID_PARAMETER;
    }

    if BufferSize > BsdItemTable[DataClass as usize].Size as ULONG {
        return STATUS_BUFFER_TOO_SMALL;
    }

    ByteOffset.__bindgen_anon_1.HighPart = 0;
    ByteOffset.__bindgen_anon_1.LowPart = BsdItemTable[DataClass as usize].Offset as u32;

    if Read != 0 {
        Status = ZwReadFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            Buffer,
            BufferSize,
            &mut ByteOffset,
            core::ptr::null_mut(),
        );
    } else {
        Status = ZwWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            Buffer,
            BufferSize,
            &mut ByteOffset,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(Status) {
        if !ReturnLength.is_null() {
            core::ptr::write_volatile(ReturnLength, BsdItemTable[DataClass as usize].Size as ULONG);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlLockBootStatusData(
    FileHandle: PHANDLE,
) -> NTSTATUS {
    let FileNameBuffer: [u16; 25] = [
        0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F,
        0x006F, 0x0074, 0x005C, 0x0062, 0x006F, 0x006F, 0x0074, 0x0073, 0x0074,
        0x0061, 0x0074, 0x002E, 0x0064, 0x0061, 0x0074, 0x0000,
    ];
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    FileName.Length = (24 * 2) as u16;
    FileName.MaximumLength = (25 * 2) as u16;
    FileName.Buffer = FileNameBuffer.as_ptr() as *mut u16;

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FileName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut LocalFileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    core::ptr::write_volatile(FileHandle, core::ptr::null_mut());

    let Status = ZwOpenFile(
        &mut LocalFileHandle,
        FILE_ALL_ACCESS,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        0,
        0x00000020,
    );

    if NT_SUCCESS(Status) {
        core::ptr::write_volatile(FileHandle, LocalFileHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnlockBootStatusData(FileHandle: HANDLE) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    ZwFlushBuffersFile(FileHandle, &mut IoStatusBlock);
    ZwClose(FileHandle)
}
