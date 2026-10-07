// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_28276425D458: [u16; 2] = [0x002Du16, 0x0000];
pub static RUSTCONV_WSTR_6096F1180D1E: [u16; 3] = [0x004Du16, 0x0042, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_AFA8EDFD6C6D: [u16; 7] = [0x0025u16, 0x006C, 0x0075, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_B00670AF8E90: [u16; 3] = [0x004Bu16, 0x0042, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn ShowFileSystemInfo(VolumeEntry: PVOLENTRY) {
    let mut szBuffer: [u16; 32] = [0; 32];
    let mut pszSizeUnit: PWSTR = RUSTCONV_WSTR_96A296D224F2.as_ptr().cast_mut();
    let mut ulClusterSize: u32;

    ConResPuts(StdOut, (IDS_FILESYSTEMS_CURRENT) as UINT);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    ConResPrintf(StdOut, (IDS_FILESYSTEMS_TYPE) as u32, (*VolumeEntry).pszFilesystem);

    ulClusterSize = (*VolumeEntry).SectorsPerAllocationUnit.wrapping_mul((*VolumeEntry).BytesPerSector);
    if (ulClusterSize as u64) >= SIZE_10MB {
        ulClusterSize = (RoundingDivide((ulClusterSize) as ULONGLONG, SIZE_1MB)) as u32;
        pszSizeUnit = RUSTCONV_WSTR_6096F1180D1E.as_ptr().cast_mut();
    } else if (ulClusterSize as u64) >= SIZE_10KB {
        ulClusterSize = (RoundingDivide((ulClusterSize) as ULONGLONG, SIZE_1KB)) as u32;
        pszSizeUnit = RUSTCONV_WSTR_B00670AF8E90.as_ptr().cast_mut();
    }

    wsprintf(szBuffer.as_mut_ptr(), RUSTCONV_WSTR_AFA8EDFD6C6D.as_ptr(), ulClusterSize, pszSizeUnit);
    ConResPrintf(StdOut, (IDS_FILESYSTEMS_CLUSTERSIZE) as u32, szBuffer.as_mut_ptr());
    ConResPrintf(StdOut, (IDS_FILESYSTEMS_SERIAL_NUMBER) as u32, (*VolumeEntry).SerialNumber);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
}

unsafe extern "C" fn ShowInstalledFileSystems(mut VolumeEntry: PVOLENTRY) {
    let mut szBuffer: [u16; 256] = [0; 256];
    let mut szDefault: [u16; 32] = [0; 32];
    let mut ret: i32 = 0;
    let mut dwIndex: u32 = 0;
    let mut uMajor: u8 = 0;
    let mut uMinor: u8 = 0;
    let mut bLatest: i32 = 0;

    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (IDS_FILESYSTEMS_DEFAULT) as UINT,
        szDefault.as_mut_ptr(),
        (ARRAYSIZE!(szDefault)) as INT,
    );

    ConResPuts(StdOut, (IDS_FILESYSTEMS_FORMATTING) as UINT);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    dwIndex = 0;
    loop {
        ret = (QueryAvailableFileSystemFormat(
            dwIndex,
            szBuffer.as_mut_ptr(),
            &mut uMajor,
            &mut uMinor,
            &mut bLatest as *mut i32 as *mut u8,
        )) as _;
        if ret == FALSE {
            break;
        }

        if wcscmp(szBuffer.as_mut_ptr(), RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0 {
            wcscat(szBuffer.as_mut_ptr(), szDefault.as_mut_ptr());
        }

        ConResPrintf(StdOut, (IDS_FILESYSTEMS_TYPE) as u32, szBuffer.as_mut_ptr());
        wcscpy(szBuffer.as_mut_ptr(), RUSTCONV_WSTR_28276425D458.as_ptr());
        ConResPrintf(StdOut, (IDS_FILESYSTEMS_CLUSTERSIZE) as u32, szBuffer.as_mut_ptr());
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        dwIndex = dwIndex.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn filesystems_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    if CurrentVolume.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_VOLUME) as UINT);
        return EXIT_SUCCESS;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    ShowFileSystemInfo(CurrentVolume);
    ShowInstalledFileSystems(CurrentVolume);

    return EXIT_SUCCESS;
}
