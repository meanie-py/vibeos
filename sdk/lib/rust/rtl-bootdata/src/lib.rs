//! Rust translation of `sdk/lib/rtl/bootdata.c` — Boot Status Data
//! implementation (Alex Ionescu / Eric Kohl).
//!
//! Manages `\SystemRoot\bootstat.dat`: create the file with a sane initial
//! BSD, get/set individual items by `RTL_BSD_ITEM_TYPE`, and lock/unlock
//! (open/flush+close) it.
//!
//! Faithful note on `BsdItemTable[].Size`: the C uses
//! `sizeof(&DummyBsd->field)`, i.e. the size of a *pointer to* the field —
//! which is the machine pointer size (4 on i386, 8 on amd64/arm64), NOT
//! the field's own size. We reproduce that exactly with
//! `size_of::<PVOID>()` so behavior is identical to the C the OS boots
//! with today.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals)]

use ntkrnl_sys::base::*; // NTSTATUS, ULONG, UCHAR, PVOID, PULONG, BOOLEAN, SIZE_T, NT_SUCCESS
use ntkrnl_sys::constants::{
    FILE_ALL_ACCESS, OBJ_CASE_INSENSITIVE, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER,
};
use ntkrnl_sys::ddk::{
    ACCESS_MASK, HANDLE, LARGE_INTEGER, PHANDLE, PRTL_BSD_DATA, RTL_BSD_DATA, ZwClose, ZwCreateFile,
    ZwFlushBuffersFile, ZwOpenFile, ZwReadFile, ZwWriteFile, _IO_STATUS_BLOCK, _OBJECT_ATTRIBUTES,
    _UNICODE_STRING, _NT_PRODUCT_TYPE_NtProductWinNt as NtProductWinNt,
    _RTL_BSD_ITEM_TYPE as RTL_BSD_ITEM_TYPE, _RTL_BSD_ITEM_TYPE_RtlBsdItemMax as RtlBsdItemMax,
};
use ntkrnl_sys::rtl::RtlZeroMemory;

// Fixed winnt.h file-access / disposition / attribute values not surfaced
// by ntkrnl-sys.
const FILE_GENERIC_READ: ULONG = 0x0012_0089;
const FILE_GENERIC_WRITE: ULONG = 0x0012_0116;
const FILE_ATTRIBUTE_SYSTEM: ULONG = 0x0000_0004;
const FILE_CREATE: ULONG = 0x0000_0002;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x0000_0020;

#[repr(C)]
pub struct RTL_BSD_ITEM_TABLE_ENTRY {
    pub Offset: UCHAR,
    pub Size: UCHAR,
}

// External-linkage global in C; only ever used to take field addresses for
// the sizeof above (never dereferenced). Kept as a symbol for fidelity.
#[no_mangle]
pub static mut DummyBsd: PRTL_BSD_DATA = core::ptr::null_mut();

// `sizeof(&DummyBsd->field)` == pointer size for every entry (see note above).
const PTR_SIZE: UCHAR = core::mem::size_of::<PVOID>() as UCHAR;

#[no_mangle]
pub static BsdItemTable: [RTL_BSD_ITEM_TABLE_ENTRY; RtlBsdItemMax as usize] = [
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, Version) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, ProductType) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, AabEnabled) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, AabTimeout) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootSucceeded) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootShutdown) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, SleepInProgress) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, PowerTransition) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, BootAttemptCount) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootCheckpoint) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, LastBootId) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, LastSuccessfulShutdownBootId) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, LastReportedAbnormalShutdownBootId) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, ErrorInfo) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, PowerButtonPressInfo) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, Checksum) as UCHAR, Size: PTR_SIZE },
    // The last three arrived with upstream's RTL_BSD_ITEM_TYPE growth (RtlBsdItemMax
    // 16 -> 19) and were MISSING here, because the stale bindgen island still said 16 so
    // the array length matched and nothing complained. Any RtlGetSetBootStatusData call
    // with a DataClass of RtlBsdPowerTransitionExtension or later indexed off the end of
    // this table. Regenerating the island turned it into a compile error, which is the
    // whole argument for regenerating rather than working around it.
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, PowerTransitionExtension) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, ItemFeatureConfigState) as UCHAR, Size: PTR_SIZE },
    RTL_BSD_ITEM_TABLE_ENTRY { Offset: core::mem::offset_of!(RTL_BSD_DATA, ItemRevocationList) as UCHAR, Size: PTR_SIZE },
];

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn RtlCreateBootStatusDataFile() -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    // (C declares an AllocationSize=0x800 here but passes NULL to ZwCreateFile
    // via DBG_UNREFERENCED_LOCAL_VARIABLE — it has no effect, so it is dropped.)

    // L"\SystemRoot\bootstat.dat" (24 wchars, no NUL — UNICODE_STRING).
    let mut FileNameBuffer: [u16; 24] = [
        0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x52, 0x6F, 0x6F, 0x74, 0x5C, 0x62, 0x6F, 0x6F,
        0x74, 0x73, 0x74, 0x61, 0x74, 0x2E, 0x64, 0x61, 0x74,
    ];
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    FileName.Length = (FileNameBuffer.len() * 2) as u16;
    FileName.MaximumLength = (FileNameBuffer.len() * 2) as u16;
    FileName.Buffer = FileNameBuffer.as_mut_ptr();

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;

    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut InitialBsd: RTL_BSD_DATA = core::mem::zeroed();

    // Create the boot status data file.
    let mut Status = ZwCreateFile(
        &mut FileHandle,
        (FILE_GENERIC_READ | FILE_GENERIC_WRITE) as ACCESS_MASK,
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
        // Setup a sane looking initial BSD.
        RtlZeroMemory(
            &mut InitialBsd as *mut _ as PVOID,
            core::mem::size_of::<RTL_BSD_DATA>() as SIZE_T,
        );
        InitialBsd.Version = core::mem::size_of::<RTL_BSD_DATA>() as u32;
        InitialBsd.ProductType = NtProductWinNt;
        InitialBsd.AabEnabled = 1;
        InitialBsd.AabTimeout = 30;
        InitialBsd.LastBootSucceeded = 1; // TRUE

        // Write it to disk.
        let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();
        ByteOffset.QuadPart = 0;
        Status = ZwWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            &mut InitialBsd as *mut _ as PVOID,
            core::mem::size_of::<RTL_BSD_DATA>() as u32,
            &mut ByteOffset,
            core::ptr::null_mut(),
        );
    }

    // Close the file.
    ZwClose(FileHandle);

    Status
}

/// @implemented
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

    if NT_SUCCESS(Status) && !ReturnLength.is_null() {
        *ReturnLength = BsdItemTable[DataClass as usize].Size as ULONG;
    }

    Status
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn RtlLockBootStatusData(FileHandle: PHANDLE) -> NTSTATUS {
    // L"\SystemRoot\bootstat.dat" (24 wchars + NUL).
    let FileNameBuffer: [u16; 25] = [
        0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x52, 0x6F, 0x6F, 0x74, 0x5C, 0x62, 0x6F, 0x6F,
        0x74, 0x73, 0x74, 0x61, 0x74, 0x2E, 0x64, 0x61, 0x74, 0x00,
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

    let mut LocalFileHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    // Initialize the out file handle.
    *FileHandle = core::ptr::null_mut();

    // Open the boot status data file.
    let Status = ZwOpenFile(
        &mut LocalFileHandle,
        FILE_ALL_ACCESS as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        0,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );

    if NT_SUCCESS(Status) {
        *FileHandle = LocalFileHandle;
    }

    Status
}

/// @implemented
#[no_mangle]
pub unsafe extern "system" fn RtlUnlockBootStatusData(FileHandle: HANDLE) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    // Flush the file and close it.
    ZwFlushBuffersFile(FileHandle, &mut IoStatusBlock);
    ZwClose(FileHandle)
}
