use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn inactive_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Status: i32 = 0;

    DPRINT!(b"Inactive()\n".as_ptr() as *const i8);

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_DISK);
        return EXIT_SUCCESS;
    }

    if CurrentPartition.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_PARTITION);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        if (*CurrentPartition).__bindgen_anon_1.Mbr.BootIndicator == 0 {
            ConResPuts(StdOut, IDS_INACTIVE_ALREADY);
            return EXIT_SUCCESS;
        }

        (*CurrentPartition).__bindgen_anon_1.Mbr.BootIndicator = (FALSE) as u8;
        (*CurrentDisk).Dirty = (TRUE) as u8;
        UpdateMbrDiskLayout(CurrentDisk);
        Status = WriteMbrPartitions(CurrentDisk);
        if NT_SUCCESS!(Status) {
            ConResPuts(StdOut, IDS_INACTIVE_SUCCESS);
        } else {
            ConResPuts(StdOut, IDS_INACTIVE_FAIL);
        }
    } else {
        ConResPuts(StdOut, IDS_INACTIVE_NO_MBR);
    }

    return EXIT_SUCCESS;
}
