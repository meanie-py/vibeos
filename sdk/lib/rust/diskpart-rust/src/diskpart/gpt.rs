// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_55E7B24010E3: [u16; 12] = [
    0x0061u16, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073,
    0x003D, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn gpt_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut ullAttributes: u64 = 0;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut i: i32;
    let mut Status: i32;

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if CurrentPartition.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_PARTITION) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle != PARTITION_STYLE_GPT {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_INVALID_STYLE) as u32);
        return EXIT_SUCCESS;
    }

    i = 1;
    while i < argc {
        if HasPrefix(
            *argv.offset(i as isize),
            RUSTCONV_WSTR_55E7B24010E3.as_ptr().cast_mut(),
            &mut pszSuffix,
        ) != 0
        {
            DPRINT!(b"Attributes: %s\n\0".as_ptr() as *const i8, pszSuffix);

            ullAttributes = _wcstoui64(
                pszSuffix,
                core::ptr::null_mut(),
                0,
            );
            if ullAttributes == 0 && *_errno() == ERANGE as c_int {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    DPRINT!(b"Attributes: 0x%I64x\n\0".as_ptr() as *const i8, ullAttributes);
    (*CurrentPartition).__bindgen_anon_1.Gpt.Attributes = ullAttributes;
    (*CurrentDisk).Dirty = (TRUE) as u8;

    UpdateGptDiskLayout(CurrentDisk, FALSE);
    Status = WriteGptPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_GPT_FAIL) as u32);
        return EXIT_SUCCESS;
    }

    ConResPuts(StdOut, (IDS_GPT_SUCCESS) as u32);

    return EXIT_SUCCESS;
}
