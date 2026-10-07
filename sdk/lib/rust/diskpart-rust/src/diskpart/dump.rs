// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5D7CADABD34F: [u16; 7] = [0x0020u16, 0x0025, 0x0030, 0x0034, 0x0078, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_836DAD312A4A: [u16; 6] = [0x0020u16, 0x0025, 0x0030, 0x0032, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_AFE3EF446700: [u16; 30] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x0064, 0x005C, 0x0050,
    0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_F503AE040218: [u16; 6] = [0x0020u16, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_FF9CD7F1925A: [u16; 31] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x0064, 0x005C, 0x0050,
    0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0025, 0x0064,
    0x0000,
];

use core::ffi::c_void;

// drop_pseh_macro_externs: dropped 1 bogus extern decl(s): RtlGetProcessHeap — PSEH _SEH2_* (and RtlGetProcessHeap) are macros, not symbols. Under USE_DUMMY_PSEH the SEH path is dead; translate the body directly. RtlGetProcessHeap → GetProcessHeap().
extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
}

unsafe extern "C" fn HexDump(mut addr: *mut UCHAR, mut len: i32) {
    let mut Buffer: [WCHAR; 17] = [0; 17];
    let mut pc: *mut UCHAR;
    let mut i: i32;

    Buffer[16] = 0x0000u16;

    pc = addr;
    i = 0;
    while i < len {
        if i % 16 == 0 {
            ConPrintf(StdOut, RUSTCONV_WSTR_5D7CADABD34F.as_ptr(), i);
        }

        ConPrintf(StdOut, RUSTCONV_WSTR_836DAD312A4A.as_ptr(), *pc.offset(i as isize) as i32);

        if *pc.offset(i as isize) < 0x20 || *pc.offset(i as isize) > 0x7e {
            Buffer[(i % 16) as usize] = 0x002Eu16;
        } else {
            Buffer[(i % 16) as usize] = *pc.offset(i as isize) as u16;
        }

        if i % 16 == 16 - 1 {
            ConPrintf(StdOut, RUSTCONV_WSTR_F503AE040218.as_ptr(), Buffer.as_mut_ptr());
        }

        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DumpDisk(argc: INT, argv: *mut LPWSTR) -> EXIT_CODE {
    unsafe {
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut Status: i32 = 0;
        let mut Buffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut Name: _UNICODE_STRING = core::mem::zeroed();
        let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut pSectorBuffer: *mut u8 = core::ptr::null_mut();
        let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
        let mut Sector: i64 = 0;
        let mut endptr: *mut u16 = core::ptr::null_mut();

        if CurrentDisk.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as UINT);
            return EXIT_SUCCESS;
        }

        Sector = _wcstoi64(
            *argv.offset(2) as *const u16,
            &raw mut endptr,
            0,
        );

        if (Sector == 0 && endptr == *argv.offset(2))
            || Sector < 0
        {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as UINT);
            return EXIT_SUCCESS;
        }

        pSectorBuffer = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            ((*CurrentDisk).BytesPerSector) as SIZE_T,
        ) as *mut u8;

        'done: {
            if pSectorBuffer.is_null() {
                if DbgPrint(
                    b"(%s:%d) \n\0".as_ptr() as *const i8,
                    b"base/system/diskpart/dump.c\0".as_ptr() as *const i8,
                    88,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"base/system/diskpart/dump.c\0".as_ptr() as *const i8,
                        88,
                    );
                }
                break 'done;
            }

            _swprintf(
                Buffer.as_mut_ptr(),
                RUSTCONV_WSTR_AFE3EF446700.as_ptr(),
                (*CurrentDisk).DiskNumber,
            );

            RtlInitUnicodeString(
                &raw mut Name,
                Buffer.as_ptr(),
            );

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>();
            ObjectAttributes.Attributes = 0;
            ObjectAttributes.ObjectName = &raw mut Name;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

            Status = NtOpenFile(
                &raw mut FileHandle,
                FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                &raw mut ObjectAttributes,
                &raw mut Iosb,
                FILE_SHARE_READ,
                FILE_SYNCHRONOUS_IO_NONALERT,
            );

            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) \n\0".as_ptr() as *const i8,
                    b"base/system/diskpart/dump.c\0".as_ptr() as *const i8,
                    113,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"base/system/diskpart/dump.c\0".as_ptr() as *const i8,
                        113,
                    );
                }
                break 'done;
            }

            FileOffset.QuadPart = Sector * (*CurrentDisk).BytesPerSector as i64;
            Status = NtReadFile(
                FileHandle,
                core::ptr::null_mut::<core::ffi::c_void>(),
                None,
                core::ptr::null_mut::<core::ffi::c_void>(),
                &raw mut Iosb,
                pSectorBuffer as *mut core::ffi::c_void,
                (*CurrentDisk).BytesPerSector,
                &raw mut FileOffset,
                core::ptr::null_mut::<u32>(),
            );

            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) NtReadFile failed, status=%x\n\0".as_ptr() as *const i8,
                    b"base/system/diskpart/dump.c\0".as_ptr() as *const i8,
                    129,
                    Status,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"base/system/diskpart/dump.c\0".as_ptr() as *const i8,
                        129,
                    );
                }
                break 'done;
            }

            HexDump(pSectorBuffer, (*CurrentDisk).BytesPerSector as i32);
        }

        if !FileHandle.is_null() {
            NtClose(FileHandle);
        }

        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            pSectorBuffer as *mut core::ffi::c_void,
        );

        return EXIT_SUCCESS;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DumpPartition(argc: INT, argv: *mut LPWSTR) -> EXIT_CODE {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: i32 = 0;
    let mut Buffer = [0u16; (MAX_PATH) as _];
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pSectorBuffer: *mut u8 = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Sector: i64 = 0;
    let mut endptr: *mut u16 = core::ptr::null_mut();

    'done: {
        if CurrentDisk.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as UINT);
            return EXIT_SUCCESS;
        }

        if CurrentPartition.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_PARTITION) as UINT);
            return EXIT_SUCCESS;
        }

        Sector = _wcstoi64(
            *argv.offset(2 as isize) as *const u16,
            &raw mut endptr,
            0,
        );

        if ((Sector == 0)
            && (endptr == *argv.offset(2 as isize)))
            || (Sector < 0)
        {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as UINT);
            return EXIT_SUCCESS;
        }

        pSectorBuffer = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            ((*CurrentDisk).BytesPerSector) as SIZE_T,
        ) as *mut u8;

        if pSectorBuffer.is_null() {
            DPRINT1!(b"\n\0".as_ptr() as *const i8);
            break 'done;
        }

        _swprintf(
            Buffer.as_mut_ptr(),
            RUSTCONV_WSTR_FF9CD7F1925A.as_ptr(),
            (*CurrentDisk).DiskNumber,
            (*CurrentPartition).PartitionNumber,
        );

        RtlInitUnicodeString(
            &raw mut Name,
            Buffer.as_mut_ptr() as *const u16,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0 as u32;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(Name);
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &raw mut FileHandle,
            FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            &raw mut ObjectAttributes,
            &raw mut Iosb,
            FILE_SHARE_READ,
            FILE_SYNCHRONOUS_IO_NONALERT,
        );

        if !NT_SUCCESS!(Status) {
            DPRINT1!(b"\n\0".as_ptr() as *const i8);
            break 'done;
        }

        FileOffset.QuadPart = Sector * (*CurrentDisk).BytesPerSector as i64;

        Status = NtReadFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &raw mut Iosb,
            pSectorBuffer as *mut core::ffi::c_void,
            (*CurrentDisk).BytesPerSector,
            &raw mut FileOffset,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"NtReadFile failed, status=%x\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        HexDump(pSectorBuffer, (*CurrentDisk).BytesPerSector as i32);
    }

    if !FileHandle.is_null() {
        NtClose(FileHandle);
    }

    RtlFreeHeap(
        RtlGetProcessHeap!(),
        0,
        pSectorBuffer as *mut core::ffi::c_void,
    );

    return EXIT_SUCCESS;
}
