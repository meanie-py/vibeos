use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::KI_USER_SHARED_DATA;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _STRING,
    _IO_STATUS_BLOCK,
    _GUID,
    _ULARGE_INTEGER,
    _UNICODE_STRING,
    _OBJECT_ATTRIBUTES,
    HANDLE,
    IoCreateSymbolicLink,
    KEVENT,
    KeWaitForSingleObject,
    MEMORY_ALLOCATION_DESCRIPTOR,
    PLOADER_PARAMETER_BLOCK,
    PMEMORY_ALLOCATION_DESCRIPTOR,
    RtlAnsiStringToUnicodeString,
    RtlFreeUnicodeString,
    RtlInitEmptyUnicodeString,
    RtlInitUnicodeString,
    RtlStringFromGUID,
    ZwClose,
    ZwDeviceIoControlFile,
    ZwOpenFile,
};

#[cfg(target_arch = "aarch64")]
extern "system" {
    static mut Arm64UartKernelBase: PVOID;
    fn Arm64UartPutChar(c: u8);
}

#[cfg(target_arch = "aarch64")]
unsafe fn arm64_uart_trace(s: &[u8]) {
    let uart = Arm64UartKernelBase as *mut ULONG;
    if uart.is_null() {
        return;
    }
    for &byte in s {
        Arm64UartPutChar(byte as u8);
    }
}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn arm64_uart_trace(_s: &[u8]) {}

extern "C" {
    static mut PiEnumerationFinished: KEVENT;
    // SharedUserData is defined as a constant below, not an extern
    // CONTAINING_RECORD inlined below
    // InitializeObjectAttributes inlined below
    fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
    fn _strupr(s: *mut i8) -> *mut i8;
    fn atol(s: *const i8) -> i32;
    fn _atoi64(s: *const i8) -> i64;
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn _snwprintf(buf: *mut u16, count: usize, fmt: *const u16, ...) -> i32;
}

#[repr(C)]
struct SharedUserDataStruct {
    _padding: [u8; 0x030],
    NtSystemRoot: [u16; 260],
}

unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).ObjectName = n;
    (*p).Attributes = a;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

// CONTAINING_RECORD: compute struct base from member pointer
#[inline(always)]
unsafe fn CONTAINING_RECORD(address: PVOID, _type_size: usize, field_offset: usize) -> PVOID {
    (address as usize - field_offset) as PVOID
}

#[inline(always)]
unsafe fn SharedUserData() -> *mut SharedUserDataStruct {
    KI_USER_SHARED_DATA as usize as *mut SharedUserDataStruct
}

const RAMDISK_BOOT_INITIALIZATION_FAILED: ULONG = 0xF8;
const RD_NO_XIPROM_DESCRIPTOR: ULONG = 1;
const RD_NO_RAMDISK_DRIVER: ULONG = 2;
const RD_FSCTL_FAILED: ULONG = 3;
const RD_GUID_CONVERT_FAILED: ULONG = 4;
const RD_SYMLINK_CREATE_FAILED: ULONG = 5;
const RD_SYSROOT_INIT_FAILED: ULONG = 6;

const LoaderXIPRom: u32 = 25;

const RAMDISK_BOOT_DISK: ULONG = 0x03;

// ntddrdsk.h's value; this literal was once written from memory as {9E475CAB-..}.
static RAMDISK_BOOTDISK_GUID: _GUID = _GUID {
    Data1: 0xD9B257FC,
    Data2: 0x684E,
    Data3: 0x4DCB,
    Data4: [0xAB, 0x79, 0x03, 0xCF, 0xA2, 0xF6, 0xB7, 0x50],
};

// In ntddrdsk.h this is a 6-field bitfield: Readonly:1 | Fixed:1 | NoDriveLetter:1
// | NoDosDevice:1 | Hidden:1 | ExportAsCd:1. Storing it as a single u32 here is
// fine — but the bit positions matter when writing into it. Bit 0 is Readonly,
// bit 1 is Fixed; mixing them up creates a read-only ramdisk that fails every
// fastfat write with STATUS_MEDIA_WRITE_PROTECTED.
#[repr(C)]
struct RAMDISK_CREATE_OPTIONS {
    Bits: ULONG,
}

const RAMDISK_OPTION_READONLY: ULONG = 1 << 0;
const RAMDISK_OPTION_FIXED: ULONG = 1 << 1;
const RAMDISK_OPTION_NO_DRIVE_LETTER: ULONG = 1 << 2;
const RAMDISK_OPTION_NO_DOS_DEVICE: ULONG = 1 << 3;
const RAMDISK_OPTION_HIDDEN: ULONG = 1 << 4;
const RAMDISK_OPTION_EXPORT_AS_CD: ULONG = 1 << 5;

/// Must match C layout from ntddrdsk.h exactly: ramdisk.sys refuses the request unless
/// `Version == sizeof(RAMDISK_CREATE_INPUT)`. The union used to be hand-padded to its
/// 64-bit size (24 bytes), so on i386 this struct was 64 bytes against C's 56 and the
/// boot ramdisk could never be created (BUGCHECK 0xF8, RD_FSCTL_FAILED) -- unseen until
/// x86 first booted a ramdisk (conformance's writable root, 2026-10-10). Declare the
/// union as C does and let the compiler size it per target.
#[repr(C)]
struct _RAMDISK_CREATE_INPUT {
    Version: ULONG,
    DiskGuid: _GUID,
    DiskType: ULONG,
    Options: RAMDISK_CREATE_OPTIONS,
    DiskLength: _ULARGE_INTEGER,
    DiskOffset: LONG,
    u: _RAMDISK_CREATE_INPUT_U,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct _RAMDISK_CREATE_INPUT_VIEW {
    ViewCount: ULONG,
    ViewLength: SIZE_T,
    FileName: [WCHAR; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct _RAMDISK_CREATE_INPUT_BASE {
    BasePage: ULONG_PTR,
    DriveLetter: WCHAR,
}

#[repr(C)]
union _RAMDISK_CREATE_INPUT_U {
    view: _RAMDISK_CREATE_INPUT_VIEW,
    base: _RAMDISK_CREATE_INPUT_BASE,
}

// C's sizeof(RAMDISK_CREATE_INPUT) per target, from ntddrdsk.h.
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<_RAMDISK_CREATE_INPUT>() == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<_RAMDISK_CREATE_INPUT>() == 72);

const FSCTL_CREATE_RAM_DISK: ULONG = 0x240000; // CTL_CODE(FILE_DEVICE_VIRTUAL_DISK, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)

const GENERIC_ALL: u32 = 0x10000000;
const SYNCHRONIZE: u32 = 0x00100000;
const FILE_SHARE_READ: u32 = 0x00000001;
const FILE_SHARE_WRITE: u32 = 0x00000002;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00000020;

const Executive: i32 = 0;
const KernelMode: i8 = 0;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopStartRamdisk(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut MemoryDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let CommandLine: *mut i8;
    let mut Offset: *mut i8;
    let OffsetValue: *mut i8;
    let mut Length: *mut i8;
    let LengthValue: *mut i8;
    let mut DriverHandle: HANDLE = core::ptr::null_mut();
    let mut RamdiskCreate: _RAMDISK_CREATE_INPUT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut SymbolicLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceString: _UNICODE_STRING = core::mem::zeroed();
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SourceString: [u16; 54] = [0; 54];

    arm64_uart_trace(b"[RAMDISK] Scanning for XIPRom descriptor\r\n");
    MemoryDescriptor = core::ptr::null_mut();
    ListHead = core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        MemoryDescriptor = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<MEMORY_ALLOCATION_DESCRIPTOR>(),
            core::mem::offset_of!(MEMORY_ALLOCATION_DESCRIPTOR, ListEntry),
        ) as PMEMORY_ALLOCATION_DESCRIPTOR;

        if (*MemoryDescriptor).MemoryType == LoaderXIPRom {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == ListHead {
        arm64_uart_trace(b"[RAMDISK] BUG: No XIPRom descriptor found!\r\n");
        KeBugCheckEx(
            RAMDISK_BOOT_INITIALIZATION_FAILED,
            RD_NO_XIPROM_DESCRIPTOR as ULONG_PTR,
            STATUS_INVALID_PARAMETER as ULONG_PTR,
            0,
            0,
        );
    }

    RtlZeroMemory(
        &mut RamdiskCreate as *mut _ as PVOID,
        core::mem::size_of::<_RAMDISK_CREATE_INPUT>(),
    );
    RamdiskCreate.Version = core::mem::size_of::<_RAMDISK_CREATE_INPUT>() as ULONG;
    RamdiskCreate.DiskType = RAMDISK_BOOT_DISK;
    RamdiskCreate.u.base.BasePage = (*MemoryDescriptor).BasePage as ULONG_PTR;
    RamdiskCreate.DiskOffset = 0;
    RamdiskCreate.DiskLength.QuadPart =
        ((*MemoryDescriptor).PageCount << PAGE_SHIFT) as u64;
    RamdiskCreate.DiskGuid = RAMDISK_BOOTDISK_GUID;
    RamdiskCreate.u.base.DriveLetter = b'C' as u16;
    RamdiskCreate.Options.Bits = RAMDISK_OPTION_FIXED;

    CommandLine = (*LoaderBlock).LoadOptions as *mut i8;
    if !CommandLine.is_null() {
        _strupr(CommandLine);

        Offset = strstr(CommandLine, b"RDIMAGEOFFSET\0".as_ptr() as *const i8);
        if !Offset.is_null() {
            OffsetValue = strstr(Offset, b"=\0".as_ptr() as *const i8);
            if !OffsetValue.is_null() {
                RamdiskCreate.DiskOffset = atol(OffsetValue.add(1));
            }
        }

        RamdiskCreate.DiskLength.QuadPart -= RamdiskCreate.DiskOffset as u64;

        Length = strstr(CommandLine, b"RDIMAGELENGTH\0".as_ptr() as *const i8);
        if !Length.is_null() {
            LengthValue = strstr(Length, b"=\0".as_ptr() as *const i8);
            if !LengthValue.is_null() {
                RamdiskCreate.DiskLength.QuadPart = _atoi64(LengthValue.add(1)) as u64;
            }
        }
    }

    static DEVICE_RAMDISK: [u16; 17] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16,
        b'c' as u16, b'e' as u16, b'\\' as u16, b'R' as u16, b'a' as u16,
        b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16,
        0, 0,
    ];
    RtlInitUnicodeString(&mut ObjectName, DEVICE_RAMDISK.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut ObjectName,
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    arm64_uart_trace(b"[RAMDISK] Opening \\Device\\Ramdisk\r\n");
    Status = ZwOpenFile(
        &mut DriverHandle,
        GENERIC_ALL | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if Status < 0 || IoStatusBlock.__bindgen_anon_1.Status < 0 {
        arm64_uart_trace(b"[RAMDISK] BUG: Cannot open \\Device\\Ramdisk!\r\n");
        KeBugCheckEx(
            RAMDISK_BOOT_INITIALIZATION_FAILED,
            RD_NO_RAMDISK_DRIVER as ULONG_PTR,
            IoStatusBlock.__bindgen_anon_1.Status as ULONG_PTR,
            0,
            0,
        );
    }

    arm64_uart_trace(b"[RAMDISK] Sending FSCTL_CREATE_RAM_DISK\r\n");
    Status = ZwDeviceIoControlFile(
        DriverHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        FSCTL_CREATE_RAM_DISK,
        &mut RamdiskCreate as *mut _ as PVOID,
        core::mem::size_of::<_RAMDISK_CREATE_INPUT>() as ULONG,
        core::ptr::null_mut(),
        0,
    );
    ZwClose(DriverHandle);
    if Status < 0 || IoStatusBlock.__bindgen_anon_1.Status < 0 {
        KeBugCheckEx(
            RAMDISK_BOOT_INITIALIZATION_FAILED,
            RD_FSCTL_FAILED as ULONG_PTR,
            IoStatusBlock.__bindgen_anon_1.Status as ULONG_PTR,
            0,
            0,
        );
    }

    Status = RtlStringFromGUID(
        &RamdiskCreate.DiskGuid,
        &mut GuidString,
    );
    if Status < 0 {
        KeBugCheckEx(
            RAMDISK_BOOT_INITIALIZATION_FAILED,
            RD_GUID_CONVERT_FAILED as ULONG_PTR,
            Status as ULONG_PTR,
            0,
            0,
        );
    }

    static FMT_STRING: [u16; 21] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16,
        b'c' as u16, b'e' as u16, b'\\' as u16, b'R' as u16, b'a' as u16,
        b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16,
        b'%' as u16, b'w' as u16, b'Z' as u16, 0, 0, 0,
    ];
    _snwprintf(
        SourceString.as_mut_ptr(),
        54,
        FMT_STRING.as_ptr(),
        &GuidString,
    );
    SymbolicLinkName.Length = 38;
    SymbolicLinkName.MaximumLength = 38 + core::mem::size_of::<u16>() as u16;
    static ARCNAME_RAMDISK: [u16; 20] = [
        b'\\' as u16, b'A' as u16, b'r' as u16, b'c' as u16, b'N' as u16,
        b'a' as u16, b'm' as u16, b'e' as u16, b'\\' as u16, b'r' as u16,
        b'a' as u16, b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16,
        b'k' as u16, b'(' as u16, b'0' as u16, b')' as u16, 0,
    ];
    SymbolicLinkName.Buffer = ARCNAME_RAMDISK.as_ptr() as *mut u16;

    arm64_uart_trace(b"[RAMDISK] Creating \\ArcName\\ramdisk(0) symlink\r\n");
    RtlInitUnicodeString(&mut DeviceString, SourceString.as_ptr());
    Status = IoCreateSymbolicLink(&mut SymbolicLinkName, &mut DeviceString);
    RtlFreeUnicodeString(&mut GuidString);
    if Status < 0 {
        KeBugCheckEx(
            RAMDISK_BOOT_INITIALIZATION_FAILED,
            RD_SYMLINK_CREATE_FAILED as ULONG_PTR,
            Status as ULONG_PTR,
            0,
            0,
        );
    }

    // ReactOS hack: set up drive letter and NtSystemRoot
    {
        let mut AnsiPath: _STRING = core::mem::zeroed();
        let mut Buffer: [i8; 256] = [0; 256];
        let mut NtSystemRoot: _UNICODE_STRING = core::mem::zeroed();

        // L"\\??\\X:" — 6 chars + null = 7 WCHARs
        // RTL_CONSTANT_STRING: Length = 6*2=12, MaximumLength = 7*2=14
        static mut DRIVE_LETTER_BUF: [u16; 7] = [
            b'\\' as u16, b'?' as u16, b'?' as u16, b'\\' as u16,
            b'X' as u16, b':' as u16, 0,
        ];
        let mut DriveLetter: _UNICODE_STRING = _UNICODE_STRING {
            Length: 12,
            MaximumLength: 14,
            Buffer: (*(&raw mut DRIVE_LETTER_BUF)).as_mut_ptr(),
        };

        // sprintf(Buffer, "X:%s", LoaderBlock->NtBootPathName)
        AnsiPath.Length = sprintf(
            Buffer.as_mut_ptr(),
            b"X:%s\0".as_ptr() as *const i8,
            (*LoaderBlock).NtBootPathName,
        ) as u16;
        AnsiPath.MaximumLength = AnsiPath.Length + 1;
        AnsiPath.Buffer = Buffer.as_mut_ptr();

        // Write the system root path to SharedUserData->NtSystemRoot
        let usd = SharedUserData();
        RtlInitEmptyUnicodeString(
            &mut NtSystemRoot,
            (*usd).NtSystemRoot.as_mut_ptr(),
            core::mem::size_of_val(&(*usd).NtSystemRoot) as u16,
        );
        Status = RtlAnsiStringToUnicodeString(&mut NtSystemRoot, &mut AnsiPath, 0u8);
        if Status < 0 {
            KeBugCheckEx(
                RAMDISK_BOOT_INITIALIZATION_FAILED,
                RD_SYSROOT_INIT_FAILED as ULONG_PTR,
                Status as ULONG_PTR,
                0,
                0,
            );
        }
        IoCreateSymbolicLink(&mut DriveLetter, &mut DeviceString);
    }

    arm64_uart_trace(b"[RAMDISK] Waiting for PiEnumerationFinished\r\n");
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(PiEnumerationFinished) as PVOID,
        0u32,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
    arm64_uart_trace(b"[RAMDISK] PiEnumerationFinished signaled\r\n");

    arm64_uart_trace(b"[RAMDISK] IopStartRamdisk SUCCESS\r\n");
    STATUS_SUCCESS
}