use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn active_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Status: i32 = 0;

    DPRINT!(b"Active()\n".as_ptr() as *const i8);

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as UINT);
        return EXIT_SUCCESS;
    }

    if CurrentPartition.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_PARTITION) as UINT);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        if (*CurrentPartition).__bindgen_anon_1.Mbr.BootIndicator != 0 {
            ConResPuts(StdOut, (IDS_ACTIVE_ALREADY) as UINT);
            return EXIT_SUCCESS;
        }

        (*CurrentPartition).__bindgen_anon_1.Mbr.BootIndicator = (TRUE) as u8;
        (*CurrentDisk).Dirty = (TRUE) as u8;
        UpdateMbrDiskLayout(CurrentDisk);
        Status = WriteMbrPartitions(CurrentDisk);
        if NT_SUCCESS!(Status) {
            ConResPuts(StdOut, (IDS_ACTIVE_SUCCESS) as UINT);
        } else {
            ConResPuts(StdOut, (IDS_ACTIVE_FAIL) as UINT);
        }
    } else {
        ConResPuts(StdOut, (IDS_ACTIVE_NO_MBR) as UINT);
    }

    return EXIT_SUCCESS;
}
