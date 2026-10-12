use crate::vfatlib::vfatlib::UpdateProgress;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetShiftCount(mut Value: ULONG) -> ULONG {
    let mut i: ULONG = 1;
    while Value > 0 {
        i = i.wrapping_add(1);
        Value = Value.wrapping_div(2);
    }
    return i.wrapping_sub(2);
}
#[no_mangle]
pub unsafe extern "C" fn CalcVolumeSerialNumber() -> ULONG {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut Serial: ULONG = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();

    NtQuerySystemTime(core::ptr::addr_of_mut!(SystemTime));
    RtlTimeToTimeFields(core::ptr::addr_of_mut!(SystemTime), core::ptr::addr_of_mut!(TimeFields));

    Buffer = core::ptr::addr_of_mut!(Serial) as PUCHAR;
    *Buffer.offset(0 as isize) = (((TimeFields.Year & 0xFF) as UCHAR as i32)
        + ((TimeFields.Hour & 0xFF) as UCHAR as i32)) as UCHAR;
    *Buffer.offset(1 as isize) = (((TimeFields.Year >> 8) as UCHAR as i32)
        + ((TimeFields.Minute & 0xFF) as UCHAR as i32)) as UCHAR;
    *Buffer.offset(2 as isize) = (((TimeFields.Month & 0xFF) as UCHAR as i32)
        + ((TimeFields.Second & 0xFF) as UCHAR as i32)) as UCHAR;
    *Buffer.offset(3 as isize) = (((TimeFields.Day & 0xFF) as UCHAR as i32)
        + ((TimeFields.Milliseconds & 0xFF) as UCHAR as i32)) as UCHAR;

    return Serial;
}
#[no_mangle]
pub unsafe extern "C" fn FatWipeSectors(FileHandle: HANDLE, TotalSectors: ULONG, SectorsPerCluster: ULONG, BytesPerSector: ULONG, Context: PFORMAT_CONTEXT) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Buffer: PUCHAR;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Sector: ULONGLONG;
    let mut Length: ULONG;
    let mut Status: NTSTATUS = 0;

    Length = SectorsPerCluster.wrapping_mul(BytesPerSector);

    /* Allocate buffer for the cluster */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        Length as SIZE_T,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Wipe all clusters */
    Sector = 0;
    'done: {
        while Sector.wrapping_add(SectorsPerCluster as ULONGLONG) < TotalSectors as ULONGLONG {
            FileOffset.QuadPart = Sector.wrapping_mul(BytesPerSector as ULONGLONG) as LONGLONG;

            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                Buffer as PVOID,
                Length,
                core::ptr::addr_of_mut!(FileOffset),
                core::ptr::null_mut::<ULONG>(),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
                break 'done;
            }

            UpdateProgress(Context, SectorsPerCluster);

            Sector = Sector.wrapping_add(SectorsPerCluster as ULONGLONG);
        }

        /* Wipe the trailing space behind the last cluster */
        if Sector < TotalSectors as ULONGLONG {
            DPRINT!(
                "Remaining sectors %lu\n",
                (TotalSectors as ULONGLONG).wrapping_sub(Sector)
            );

            FileOffset.QuadPart = Sector.wrapping_mul(BytesPerSector as ULONGLONG) as LONGLONG;
            Length = (TotalSectors as ULONGLONG)
                .wrapping_sub(Sector)
                .wrapping_mul(BytesPerSector as ULONGLONG) as ULONG;

            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                Buffer as PVOID,
                Length,
                core::ptr::addr_of_mut!(FileOffset),
                core::ptr::null_mut::<ULONG>(),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
                break 'done;
            }

            UpdateProgress(
                Context,
                (TotalSectors as ULONGLONG).wrapping_sub(Sector) as ULONG,
            );
        }
    }

    /* Free the buffer */
    RtlFreeHeap(
        RtlGetProcessHeap!(),
        0,
        Buffer as PVOID,
    );
    return Status;
}
