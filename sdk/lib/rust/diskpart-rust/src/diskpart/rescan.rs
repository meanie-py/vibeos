use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn rescan_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    ConResPuts(StdOut, (IDS_RESCAN_START) as UINT);
    DestroyVolumeList();
    DestroyPartitionList();
    CreatePartitionList();
    CreateVolumeList();
    ConResPuts(StdOut, (IDS_RESCAN_END) as UINT);

    return EXIT_SUCCESS;
}
