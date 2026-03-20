use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;

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

extern "system" {
    static mut PiEnumerationFinished: KEVENT;
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
}

extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID;
}

#[repr(C)]
struct KUSER_SHARED_DATA {
    _padding: [u8; 0x030],
    NtSystemRoot: [u16; 260],
}

#[cfg(target_arch = "aarch64")]
extern "system" {
    static mut Arm64UartKernelBase: PVOID;
}

#[cfg(target_arch = "aarch64")]
unsafe fn arm64_uart_trace(s: &[u8]) {
    let uart = Arm64UartKernelBase as *mut ULONG;
    if uart.is_null() {
        return;
    }
    for &c in s {
        core::ptr::write_volatile(uart, c as ULONG);
    }
}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn arm64_uart_trace(_s: &[u8]) {}

const RAMDISK_BOOT_INITIALIZATION_FAILED: ULONG = 0x7B;
const RD_NO_XIPROM_DESCRIPTOR: ULONG = 1;
const RD_NO_RAMDISK_DRIVER: ULONG = 2;
const RD_FSCTL_FAILED: ULONG = 3;
const RD_GUID_CONVERT_FAILED: ULONG = 4;
const RD_SYMLINK_CREATE_FAILED: ULONG = 5;
const RD_SYSROOT_INIT_FAILED: ULONG = 6;

const FSCTL_CREATE_RAM_DISK: ULONG = 0x00240000; // CTL_CODE(FILE_DEVICE_VIRTUAL_DISK=0x24, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
const LoaderXIPRom: u32 = 25;
const RAMDISK_BOOT_DISK: ULONG = 0;
const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
const GENERIC_ALL: ULONG = 0x10000000;
const SYNCHRONIZE: ULONG = 0x00100000;

const RAMDISK_BOOTDISK_GUID: _GUID = _GUID {
    Data1: 0x000000E0,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};

#[repr(C)]
struct RAMDISK_CREATE_OPTIONS {
    Fixed: ULONG,
}

/// Must match C layout from ntddrdsk.h exactly (sizeof=72 on 64-bit):
/// Version, DiskGuid, DiskType, Options, DiskLength, DiskOffset, union{BasePage+DriveLetter}
/// The C union's largest variant is {ViewCount, ViewLength, FileName[1]} = 24 bytes.
/// We use the {BasePage, DriveLetter} variant (16 bytes) + padding to 24.
#[repr(C)]
struct RAMDISK_CREATE_INPUT {
    Version: ULONG,
    DiskGuid: _GUID,
    DiskType: ULONG,
    Options: RAMDISK_CREATE_OPTIONS,
    DiskLength: _ULARGE_INTEGER,
    DiskOffset: LONG,
    // union: using the BasePage+DriveLetter variant + padding
    BasePage: ULONG_PTR,
    DriveLetter: WCHAR,
    _pad: [u8; 14], // pad union to 24 bytes (8+2+14=24)
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopStartRamdisk(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let mut MemoryDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let CommandLine: *mut i8;
    let mut Offset: *mut i8;
    let mut OffsetValue: *mut i8;
    let mut Length: *mut i8;
    let mut LengthValue: *mut i8;
    let mut DriverHandle: HANDLE = core::ptr::null_mut();
    let mut RamdiskCreate: RAMDISK_CREATE_INPUT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut SymbolicLinkName: _UNICODE_STRING;
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceString: _UNICODE_STRING = core::mem::zeroed();
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SourceString: [WCHAR; 54] = [0; 54];

    arm64_uart_trace(b"[RAMDISK] Scanning for XIPRom descriptor\r\n");
    MemoryDescriptor = core::ptr::null_mut();
    ListHead = core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead) as PLIST_ENTRY;
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

    core::ptr::write_bytes(&mut RamdiskCreate as *mut _ as *mut u8, 0, core::mem::size_of::<RAMDISK_CREATE_INPUT>());
    RamdiskCreate.Version = core::mem::size_of::<RAMDISK_CREATE_INPUT>() as ULONG;
    RamdiskCreate.DiskGuid = RAMDISK_BOOTDISK_GUID;
    RamdiskCreate.DiskType = RAMDISK_BOOT_DISK;
    RamdiskCreate.Options.Fixed = 1;
    RamdiskCreate.DiskLength.QuadPart = ((*MemoryDescriptor).PageCount as ULONGLONG) << PAGE_SHIFT;
    RamdiskCreate.DiskOffset = 0;
    RamdiskCreate.BasePage = (*MemoryDescriptor).BasePage as ULONG_PTR;
    RamdiskCreate.DriveLetter = b'C' as u16;

    CommandLine = (*LoaderBlock).LoadOptions as *mut i8;
    if !CommandLine.is_null() {
        strupr(CommandLine);

        Offset = strstr(CommandLine, b"RDIMAGEOFFSET\0".as_ptr() as *const i8);
        if !Offset.is_null() {
            OffsetValue = strstr(Offset, b"=\0".as_ptr() as *const i8);
            if !OffsetValue.is_null() {
                RamdiskCreate.DiskOffset = atol(OffsetValue.add(1));
            }
        }

        RamdiskCreate.DiskLength.QuadPart -= RamdiskCreate.DiskOffset as ULONGLONG;

        Length = strstr(CommandLine, b"RDIMAGELENGTH\0".as_ptr() as *const i8);
        if !Length.is_null() {
            LengthValue = strstr(Length, b"=\0".as_ptr() as *const i8);
            if !LengthValue.is_null() {
                RamdiskCreate.DiskLength.QuadPart = atoi64(LengthValue.add(1)) as ULONGLONG;
            }
        }
    }

    RtlInitUnicodeString(
        &mut ObjectName,
        [b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'R' as u16, b'a' as u16, b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16, 0].as_ptr(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut ObjectName as *mut _UNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

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
        core::mem::size_of::<RAMDISK_CREATE_INPUT>() as ULONG,
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

    snwprintf(
        SourceString.as_mut_ptr(),
        54,
        [b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'R' as u16, b'a' as u16, b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16, b'%' as u16, b'w' as u16, b'Z' as u16, 0].as_ptr(),
        &GuidString,
    );
    SymbolicLinkName = _UNICODE_STRING {
        Length: 38,
        MaximumLength: 38 + core::mem::size_of::<u16>() as u16,
        Buffer: [b'\\' as u16, b'A' as u16, b'r' as u16, b'c' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b'\\' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16, b'(' as u16, b'0' as u16, b')' as u16, 0].as_ptr() as *mut u16,
    };

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

    {
        let mut AnsiPath: _STRING = core::mem::zeroed();
        let mut Buffer: [i8; 256] = [0; 256];
        let mut NtSystemRoot: _UNICODE_STRING = core::mem::zeroed();
        let DriveLetter = _UNICODE_STRING {
            Length: 12,
            MaximumLength: 14,
            Buffer: [b'\\' as u16, b'?' as u16, b'?' as u16, b'\\' as u16, b'X' as u16, b':' as u16, 0].as_ptr() as *mut u16,
        };

        AnsiPath.Length = sprintf(
            Buffer.as_mut_ptr(),
            b"X:%s\0".as_ptr() as *const i8,
            (*LoaderBlock).NtBootPathName,
        ) as u16;
        AnsiPath.MaximumLength = AnsiPath.Length + 1;
        AnsiPath.Buffer = Buffer.as_mut_ptr();
        RtlInitEmptyUnicodeString(
            &mut NtSystemRoot,
            (*SharedUserData).NtSystemRoot.as_mut_ptr(),
            core::mem::size_of_val(&(*SharedUserData).NtSystemRoot) as u16,
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
        IoCreateSymbolicLink(
            &DriveLetter as *const _UNICODE_STRING as *mut _UNICODE_STRING,
            &mut DeviceString,
        );
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

unsafe fn strupr(s: *mut i8) {
    let mut p = s;
    while *p != 0 {
        if *p >= b'a' as i8 && *p <= b'z' as i8 {
            *p = (*p - 32) as i8;
        }
        p = p.add(1);
    }
}

unsafe fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8 {
    let mut h = haystack;
    let n_first = *needle;
    if n_first == 0 {
        return haystack as *mut i8;
    }
    while *h != 0 {
        if *h == n_first {
            let mut h2 = h;
            let mut n2 = needle;
            loop {
                n2 = n2.add(1);
                if *n2 == 0 {
                    return h as *mut i8;
                }
                h2 = h2.add(1);
                if *h2 != *n2 {
                    break;
                }
            }
        }
        h = h.add(1);
    }
    core::ptr::null_mut()
}

unsafe fn atol(s: *const i8) -> LONG {
    let mut result: LONG = 0;
    let mut p = s;
    let mut negative = false;
    while *p == b' ' as i8 || *p == b'\t' as i8 {
        p = p.add(1);
    }
    if *p == b'-' as i8 {
        negative = true;
        p = p.add(1);
    } else if *p == b'+' as i8 {
        p = p.add(1);
    }
    while *p >= b'0' as i8 && *p <= b'9' as i8 {
        result = result * 10 + (*p - b'0' as i8) as LONG;
        p = p.add(1);
    }
    if negative {
        -result
    } else {
        result
    }
}

unsafe fn atoi64(s: *const i8) -> i64 {
    let mut result: i64 = 0;
    let mut p = s;
    let mut negative = false;
    while *p == b' ' as i8 || *p == b'\t' as i8 {
        p = p.add(1);
    }
    if *p == b'-' as i8 {
        negative = true;
        p = p.add(1);
    } else if *p == b'+' as i8 {
        p = p.add(1);
    }
    while *p >= b'0' as i8 && *p <= b'9' as i8 {
        result = result * 10 + (*p - b'0' as i8) as i64;
        p = p.add(1);
    }
    if negative {
        -result
    } else {
        result
    }
}

unsafe fn sprintf(buf: *mut i8, fmt: *const i8, arg: *const i8) -> usize {
    let mut out = buf;
    let mut f = fmt;
    while *f != 0 {
        if *f == b'%' as i8 {
            f = f.add(1);
            if *f == b's' as i8 {
                let mut a = arg;
                while *a != 0 {
                    *out = *a;
                    out = out.add(1);
                    a = a.add(1);
                }
                f = f.add(1);
            } else {
                *out = b'%' as i8;
                out = out.add(1);
            }
        } else {
            *out = *f;
            out = out.add(1);
            f = f.add(1);
        }
    }
    *out = 0;
    out.offset_from(buf) as usize
}

unsafe fn snwprintf(buf: *mut u16, size: usize, fmt: *const u16, arg: *const _UNICODE_STRING) {
    let mut out = buf;
    let mut f = fmt;
    let mut remaining = size;
    while *f != 0 && remaining > 1 {
        if *f == b'%' as u16 {
            f = f.add(1);
            if *f == b'w' as u16 {
                f = f.add(1);
                if *f == b'Z' as u16 {
                    let len = (*arg).Length as usize / 2;
                    let src = (*arg).Buffer;
                    for i in 0..len {
                        if remaining <= 1 {
                            break;
                        }
                        *out = *src.add(i);
                        out = out.add(1);
                        remaining -= 1;
                    }
                    f = f.add(1);
                }
            } else {
                *out = b'%' as u16;
                out = out.add(1);
                remaining -= 1;
            }
        } else {
            *out = *f;
            out = out.add(1);
            remaining -= 1;
            f = f.add(1);
        }
    }
    *out = 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/ramdisk.c
//  * PURPOSE:         Allows booting from RAM disk
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #include <initguid.h>
// #include <ntddrdsk.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Direct UART trace for ARM64 boot debugging — bypasses PCH limitation */
// #ifdef _M_ARM64
// extern volatile PVOID Arm64UartKernelBase;
// #undef ARM64_UART_TRACE
// #define ARM64_UART_TRACE(s) do { \
//     volatile ULONG *_uart = (volatile ULONG *)Arm64UartKernelBase; \
//     const char *_p = (s); \
//     if (!_uart) break; \
//     while (*_p) *_uart = (ULONG)*_p++; \
// } while(0)
// #endif
//
// /* GLOBALS *******************************************************************/
//
// extern KEVENT PiEnumerationFinished;
//
// /* FUNCTIONS ******************************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopStartRamdisk(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PMEMORY_ALLOCATION_DESCRIPTOR MemoryDescriptor;
//     NTSTATUS Status;
//     PCHAR CommandLine, Offset, OffsetValue, Length, LengthValue;
//     HANDLE DriverHandle;
//     RAMDISK_CREATE_INPUT RamdiskCreate;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING GuidString, SymbolicLinkName, ObjectName, DeviceString;
//     PLIST_ENTRY ListHead, NextEntry;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     WCHAR SourceString[54];
//
//     //
//     // Scan memory descriptors
//     //
//     ARM64_UART_TRACE("[RAMDISK] Scanning for XIPRom descriptor\r\n");
//     MemoryDescriptor = NULL;
//     ListHead = &LoaderBlock->MemoryDescriptorListHead;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         //
//         // Get the descriptor
//         //
//         MemoryDescriptor = CONTAINING_RECORD(NextEntry,
//                                              MEMORY_ALLOCATION_DESCRIPTOR,
//                                              ListEntry);
//
//         //
//         // Needs to be a ROM/RAM descriptor
//         //
//         if (MemoryDescriptor->MemoryType == LoaderXIPRom) break;
//
//         //
//         // Keep trying
//         //
//         NextEntry = NextEntry->Flink;
//     }
//
//     //
//     // Nothing found?
//     //
//     if (NextEntry == ListHead)
//     {
//         //
//         // Bugcheck -- no data
//         //
//         ARM64_UART_TRACE("[RAMDISK] BUG: No XIPRom descriptor found!\r\n");
//         KeBugCheckEx(RAMDISK_BOOT_INITIALIZATION_FAILED,
//                      RD_NO_XIPROM_DESCRIPTOR,
//                      STATUS_INVALID_PARAMETER,
//                      0,
//                      0);
//     }
//
//     //
//     // Setup the input buffer
//     //
//     RtlZeroMemory(&RamdiskCreate, sizeof(RamdiskCreate));
//     RamdiskCreate.Version = sizeof(RamdiskCreate);
//     RamdiskCreate.DiskType = RAMDISK_BOOT_DISK;
//     RamdiskCreate.BasePage = MemoryDescriptor->BasePage;
//     RamdiskCreate.DiskOffset = 0;
//     RamdiskCreate.DiskLength.QuadPart = MemoryDescriptor->PageCount << PAGE_SHIFT;
//     RamdiskCreate.DiskGuid = RAMDISK_BOOTDISK_GUID;
//     RamdiskCreate.DriveLetter = L'C';
//     RamdiskCreate.Options.Fixed = TRUE;
//
//     //
//     // Check for commandline parameters
//     //
//     CommandLine = LoaderBlock->LoadOptions;
//     if (CommandLine)
//     {
//         //
//         // Make everything upper case
//         //
//         _strupr(CommandLine);
//
//         //
//         // Check for offset parameter
//         //
//         Offset = strstr(CommandLine, "RDIMAGEOFFSET");
//         if (Offset)
//         {
//             //
//             // Get to the actual value
//             //
//             OffsetValue = strstr(Offset, "=");
//             if (OffsetValue)
//             {
//                 //
//                 // Set the offset
//                 //
//                 RamdiskCreate.DiskOffset = atol(OffsetValue + 1);
//             }
//         }
//
//         //
//         // Reduce the disk length
//         //
//         RamdiskCreate.DiskLength.QuadPart -= RamdiskCreate.DiskOffset as ULONGLONG;
//
//         //
//         // Check for length parameter
//         //
//         Length = strstr(CommandLine, "RDIMAGELENGTH");
//         if (Length)
//         {
//             //
//             // Get to the actual value
//             //
//             LengthValue = strstr(Length, "=");
//             if (LengthValue)
//             {
//                 //
//                 // Set the offset
//                 //
//                 RamdiskCreate.DiskLength.QuadPart = _atoi64(LengthValue + 1);
//             }
//         }
//     }
//
//     //
//     // Setup object attributes
//     //
//     RtlInitUnicodeString(&ObjectName, L"\\Device\\Ramdisk");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &ObjectName,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     //
//     // Open a handle to the driver
//     //
//     ARM64_UART_TRACE("[RAMDISK] Opening \\Device\\Ramdisk\r\n");
//     Status = ZwOpenFile(&DriverHandle,
//                         GENERIC_ALL | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!(NT_SUCCESS(Status)) || !(NT_SUCCESS(IoStatusBlock.Status)))
//     {
//         //
//         // Bugcheck -- no driver
//         //
//         ARM64_UART_TRACE("[RAMDISK] BUG: Cannot open \\Device\\Ramdisk!\r\n");
//         KeBugCheckEx(RAMDISK_BOOT_INITIALIZATION_FAILED,
//                      RD_NO_RAMDISK_DRIVER,
//                      IoStatusBlock.Status,
//                      0,
//                      0);
//     }
//
//     //
//     // Send create command
//     //
//     ARM64_UART_TRACE("[RAMDISK] Sending FSCTL_CREATE_RAM_DISK\r\n");
//     Status = ZwDeviceIoControlFile(DriverHandle,
//                                    NULL,
//                                    NULL,
//                                    NULL,
//                                    &IoStatusBlock,
//                                    FSCTL_CREATE_RAM_DISK,
//                                    &RamdiskCreate,
//                                    sizeof(RamdiskCreate),
//                                    NULL,
//                                    0);
//     ZwClose(DriverHandle);
//     if (!(NT_SUCCESS(Status)) || !(NT_SUCCESS(IoStatusBlock.Status)))
//     {
//         //
//         // Bugcheck -- driver failed
//         //
//         KeBugCheckEx(RAMDISK_BOOT_INITIALIZATION_FAILED,
//                      RD_FSCTL_FAILED,
//                      IoStatusBlock.Status,
//                      0,
//                      0);
//     }
//
//     //
//     // Convert the GUID
//     //
//     Status = RtlStringFromGUID(&RamdiskCreate.DiskGuid, &GuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // Bugcheck -- GUID convert failed
//         //
//         KeBugCheckEx(RAMDISK_BOOT_INITIALIZATION_FAILED,
//                      RD_GUID_CONVERT_FAILED,
//                      Status,
//                      0,
//                      0);
//     }
//
//     //
//     // Build the symbolic link name and target
//     //
//     _snwprintf(SourceString,
//                sizeof(SourceString)/sizeof(WCHAR),
//                L"\\Device\\Ramdisk%wZ",
//                &GuidString);
//     SymbolicLinkName.Length = 38;
//     SymbolicLinkName.MaximumLength = 38 + sizeof(UNICODE_NULL);
//     SymbolicLinkName.Buffer = L"\\ArcName\\ramdisk(0)";
//
//     //
//     // Create the symbolic link
//     //
//     ARM64_UART_TRACE("[RAMDISK] Creating \\ArcName\\ramdisk(0) symlink\r\n");
//     RtlInitUnicodeString(&DeviceString, SourceString);
//     Status = IoCreateSymbolicLink(&SymbolicLinkName, &DeviceString);
//     RtlFreeUnicodeString(&GuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         //
//         // Bugcheck -- symlink create failed
//         //
//         KeBugCheckEx(RAMDISK_BOOT_INITIALIZATION_FAILED,
//                      RD_SYMLINK_CREATE_FAILED,
//                      Status,
//                      0,
//                      0);
//     }
//
//     //
//     // ReactOS hack (drive letter should not be hardcoded, and maybe set by mountmgr.sys)
//     //
//     {
//         ANSI_STRING AnsiPath;
//         CHAR Buffer[256];
//         UNICODE_STRING NtSystemRoot;
//         UNICODE_STRING DriveLetter = RTL_CONSTANT_STRING(L"\\??\\X:");
//
//         AnsiPath.Length = sprintf(Buffer, "X:%s", LoaderBlock->NtBootPathName);
//         AnsiPath.MaximumLength = AnsiPath.Length + 1;
//         AnsiPath.Buffer = Buffer;
//         RtlInitEmptyUnicodeString(&NtSystemRoot,
//                                   SharedUserData->NtSystemRoot,
//                                   sizeof(SharedUserData->NtSystemRoot));
//         Status = RtlAnsiStringToUnicodeString(&NtSystemRoot, &AnsiPath, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             KeBugCheckEx(RAMDISK_BOOT_INITIALIZATION_FAILED,
//                          RD_SYSROOT_INIT_FAILED,
//                          Status,
//                          0,
//                          0);
//         }
//         IoCreateSymbolicLink(&DriveLetter, &DeviceString);
//     }
//
//     //
//     // Wait for ramdisk relations being initialized
//     //
//     ARM64_UART_TRACE("[RAMDISK] Waiting for PiEnumerationFinished\r\n");
//     KeWaitForSingleObject(&PiEnumerationFinished, Executive, KernelMode, FALSE, NULL);
//     ARM64_UART_TRACE("[RAMDISK] PiEnumerationFinished signaled\r\n");
//
//     //
//     // We made it
//     //
//     ARM64_UART_TRACE("[RAMDISK] IopStartRamdisk SUCCESS\r\n");
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
