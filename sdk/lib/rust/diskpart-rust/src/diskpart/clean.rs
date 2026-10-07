// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5295D3F8988C: [u16; 4] = [0x0061u16, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_AFE3EF446700: [u16; 30] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x0064, 0x005C, 0x0050,
    0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0030, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn clean_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PartEntry: *mut PARTENTRY = core::ptr::null_mut();
    let mut VolumeEntry: *mut VOLENTRY = core::ptr::null_mut();
    let mut bAll: BOOL = FALSE;
    let mut SectorsBuffer: *mut u8 = core::ptr::null_mut();
    let mut LayoutBufferSize: u32 = 0;
    let mut Size: u32 = 0;
    let mut i: INT = 0;
    let mut Buffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Count: LARGE_INTEGER = core::mem::zeroed();
    let mut MaxCount: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    'done: {
        DPRINT!(b"Clean()\n\0".as_ptr() as *const i8);

        if CurrentDisk.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as UINT);
            return EXIT_SUCCESS;
        }

        if (((*CurrentDisk).BiosFound) as i32 == TRUE) &&
            ((*CurrentDisk).BiosDiskNumber == 0)
        {
            ConResPuts(StdOut, (IDS_CLEAN_SYSTEM) as UINT);
            return EXIT_SUCCESS;
        }

        i = 1;
        while i < argc {
            if _wcsicmp(
                *argv.offset(1 as isize) as *const u16,
                RUSTCONV_WSTR_5295D3F8988C.as_ptr(),
            ) == 0
            {
                bAll = TRUE;
            }
            i += 1;
        }

        while IsListEmpty(&mut (*CurrentDisk).LogicalPartListHead) == false {
            Entry = RemoveHeadList(&mut (*CurrentDisk).LogicalPartListHead);
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if (*PartEntry).__bindgen_anon_1.Mbr.PartitionType != 0 {
                DismountVolume(PartEntry);
                VolumeEntry = GetVolumeFromPartition(PartEntry);
                if !VolumeEntry.is_null() {
                    RemoveVolume(VolumeEntry);
                }
            }

            RtlFreeHeap(
                RtlGetProcessHeap!(),
                0,
                PartEntry as *mut core::ffi::c_void,
            );
        }

        while IsListEmpty(&mut (*CurrentDisk).PrimaryPartListHead) == false {
            Entry = RemoveHeadList(&mut (*CurrentDisk).PrimaryPartListHead);
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType != 0) &&
                (IsContainerPartition!((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) == ((FALSE) != 0))
            {
                DismountVolume(PartEntry);
                VolumeEntry = GetVolumeFromPartition(PartEntry);
                if !VolumeEntry.is_null() {
                    RemoveVolume(VolumeEntry);
                }
            }

            RtlFreeHeap(
                RtlGetProcessHeap!(),
                0,
                PartEntry as *mut core::ffi::c_void,
            );
        }

        (*CurrentDisk).ExtendedPartition = core::ptr::null_mut();
        (*CurrentDisk).Dirty = (FALSE) as u8;
        (*CurrentDisk).NewDisk = (TRUE) as u8;
        (*CurrentDisk).PartitionStyle = PARTITION_STYLE_RAW;

        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            (*CurrentDisk).LayoutBuffer as *mut core::ffi::c_void,
        );

        LayoutBufferSize = (core::mem::size_of::<DRIVE_LAYOUT_INFORMATION_EX>() as u32) +
            (((4 - ANYSIZE_ARRAY) as u32) *
                (core::mem::size_of::<PARTITION_INFORMATION_EX>() as u32));

        (*CurrentDisk).LayoutBuffer = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            (LayoutBufferSize) as SIZE_T,
        ) as *mut DRIVE_LAYOUT_INFORMATION_EX;

        if (*CurrentDisk).LayoutBuffer.is_null() {
            DPRINT1!(b"Failed to allocate the disk layout buffer!\n\0".as_ptr() as *const i8);
            return EXIT_SUCCESS;
        }

        (*(*CurrentDisk).LayoutBuffer).PartitionStyle = PARTITION_STYLE_RAW;

        SectorsBuffer = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            1024 * 1024,
        ) as *mut u8;

        if SectorsBuffer.is_null() {
            DPRINT1!(b"Failed to allocate the sectors buffer!\n\0".as_ptr() as *const i8);
            break 'done;
        }

        StringCchPrintfW!(
            Buffer.as_mut_ptr(),
            ARRAYSIZE!(Buffer),
            RUSTCONV_WSTR_AFE3EF446700.as_ptr(),
            (*CurrentDisk).DiskNumber,
        );

        RtlInitUnicodeString(&mut Name as *mut _UNICODE_STRING, Buffer.as_mut_ptr());

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &mut Name as *mut _UNICODE_STRING;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as u32;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &mut FileHandle as *mut HANDLE,
            (GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE) as u32,
            &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            0 as u32,
            FILE_SYNCHRONOUS_IO_NONALERT as u32,
        );

        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"Failed to open the disk! (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            ConResPuts(StdOut, (IDS_CLEAN_FAIL) as UINT);
            break 'done;
        }

        if bAll != 0 {
            MaxCount.QuadPart = (((*CurrentDisk).SectorCount.QuadPart *
                (((*CurrentDisk).BytesPerSector as i64)) as u64) /
                (((1024 * 1024) as i64)) as u64) as i64;

            Count.QuadPart = 0;
            while Count.QuadPart < MaxCount.QuadPart {
                Offset.QuadPart = Count.QuadPart *
                    ((1024 * 1024) as i64);

                Status = NtWriteFile(
                    FileHandle,
                    core::ptr::null_mut(),
                    None,
                    core::ptr::null_mut(),
                    &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                    SectorsBuffer as *mut core::ffi::c_void,
                    (1024 * 1024) as u32,
                    &mut Offset as *mut LARGE_INTEGER,
                    core::ptr::null_mut(),
                );

                if !NT_SUCCESS!(Status) {
                    DPRINT1!(
                        b"Failed to write MB! (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                    ConResPuts(StdOut, (IDS_CLEAN_FAIL) as UINT);
                    break 'done;
                }

                Count.QuadPart += 1;
            }

            Size = ((((*CurrentDisk).SectorCount.QuadPart *
                ((*CurrentDisk).BytesPerSector as i64) as u64) as u32) %
                ((1024 * 1024) as u32)) as u32;

            if Size != 0 {
                Offset.QuadPart += ((1024 * 1024) as i64);

                Status = NtWriteFile(
                    FileHandle,
                    core::ptr::null_mut(),
                    None,
                    core::ptr::null_mut(),
                    &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                    SectorsBuffer as *mut core::ffi::c_void,
                    Size,
                    &mut Offset as *mut LARGE_INTEGER,
                    core::ptr::null_mut(),
                );

                if !NT_SUCCESS!(Status) {
                    DPRINT1!(
                        b"Failed to write the last part! (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                    ConResPuts(StdOut, (IDS_CLEAN_FAIL) as UINT);
                    break 'done;
                }
            }
        } else {
            Offset.QuadPart = 0;

            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                SectorsBuffer as *mut core::ffi::c_void,
                (1024 * 1024) as u32,
                &mut Offset as *mut LARGE_INTEGER,
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS!(Status) {
                DPRINT1!(
                    b"Failed to write the first MB! (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                ConResPuts(StdOut, (IDS_CLEAN_FAIL) as UINT);
                break 'done;
            }

            Offset.QuadPart = (((*CurrentDisk).SectorCount.QuadPart *
                ((*CurrentDisk).BytesPerSector as i64) as u64) -
                (((1024 * 1024) as i64)) as u64) as i64;

            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                SectorsBuffer as *mut core::ffi::c_void,
                (1024 * 1024) as u32,
                &mut Offset as *mut LARGE_INTEGER,
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS!(Status) {
                DPRINT1!(
                    b"Failed to write the last MB! (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                ConResPuts(StdOut, (IDS_CLEAN_FAIL) as UINT);
                break 'done;
            }
        }

        ConResPuts(StdOut, (IDS_CLEAN_SUCCESS) as UINT);
    }

    if !FileHandle.is_null() {
        NtClose(FileHandle);
    }

    if !SectorsBuffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            SectorsBuffer as *mut core::ffi::c_void,
        );
    }

    return EXIT_SUCCESS;
}
