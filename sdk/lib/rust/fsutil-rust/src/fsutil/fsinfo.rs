use crate::fsutil::common::FindHandler;
use crate::fsutil::common::OpenVolume;
use crate::fsutil::common::PrintDefaultUsage;
use crate::fsutil::common::PrintErrorMessage;
use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn fprintf(stream: *mut FILE, format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    static mut stdout: *mut FILE;
    static mut _iob: [u8; 0];
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_10C0A8FC9B02: [u16; 37] = [
    0x0050u16, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0073, 0x0020, 0x0069,
    0x006E, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0073, 0x0020, 0x0061, 0x0062, 0x006F, 0x0075, 0x0074, 0x0020, 0x0061, 0x0020,
    0x0076, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_110D82EB78F2: [u16; 9] = [0x006Eu16, 0x0074, 0x0066, 0x0073, 0x0069, 0x006E, 0x0066, 0x006F, 0x0000];
pub static RUSTCONV_C2R_WSTR_3C78D9218DED: [u16; 11] = [
    0x0073u16, 0x0074, 0x0061, 0x0074, 0x0069, 0x0073, 0x0074, 0x0069, 0x0063, 0x0073,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_47E3C498A25A: [u16; 27] = [
    0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0073, 0x0020, 0x0076,
    0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0020, 0x0073, 0x0074, 0x0061, 0x0074,
    0x0069, 0x0073, 0x0074, 0x0069, 0x0063, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_664EFDD4DC11: [u16; 22] = [
    0x0045u16, 0x006E, 0x0075, 0x006D, 0x0065, 0x0072, 0x0061, 0x0074, 0x0065, 0x0073,
    0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0064, 0x0072, 0x0069, 0x0076, 0x0065,
    0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_A95C574DEB2A: [u16; 11] = [
    0x0076u16, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0069, 0x006E, 0x0066, 0x006F,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_B27CA55FCEA4: [u16; 42] = [
    0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0073, 0x0020, 0x0069,
    0x006E, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0073, 0x0020, 0x0061, 0x0062, 0x006F, 0x0075, 0x0074, 0x0020, 0x0061, 0x0020,
    0x004E, 0x0054, 0x0046, 0x0053, 0x0020, 0x0076, 0x006F, 0x006C, 0x0075, 0x006D,
    0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_B7971DDE86F2: [u16; 7] = [0x0064u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_F2BBA717713E: [u16; 10] = [0x0064u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0074, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_F487A5D29425: [u16; 29] = [
    0x0050u16, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0073, 0x0020, 0x0074,
    0x0068, 0x0065, 0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x006F, 0x0066,
    0x0020, 0x0061, 0x0020, 0x0064, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000,
];

static mut HandlersList: [HandlerItem; 5] = unsafe {
    [
        HandlerItem {
            Handler: Some(
                DrivesMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_B7971DDE86F2.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_664EFDD4DC11.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                DriveTypeMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_F2BBA717713E.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_F487A5D29425.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                VolumeInfoMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_A95C574DEB2A.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_10C0A8FC9B02.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                NtfsInfoMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_110D82EB78F2.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_B27CA55FCEA4.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                StatisticsMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_3C78D9218DED.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_47E3C498A25A.as_ptr() as *const TCHAR,
        },
    ]
};

// Forward declarations `static HandlerProc DrivesMain;` etc. are C prototypes only.
// The functions themselves are translated separately, so they are referenced by name below.

// DUMP_VALUE, GET_NEXT and SUM_VALUE are function-like macros and are inlined by the functions that use them.

unsafe extern "C" fn DrivesMain(argc: ::core::ffi::c_int, argv: *const *const TCHAR) -> ::core::ffi::c_int {
    let mut i: UINT;
    let mut Drives: DWORD;

    /* Get the drives bitmap */
    Drives = GetLogicalDrives();
    if Drives == 0 {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    /* And output any found drive */
    _ftprintf(stdout, _T("Drives:"));
    i = 0;
    while i < 26 {
        if Drives & (1 << i) != 0 {
            _ftprintf(stdout, _T(" %c:\\"), ('A' as UINT) + i);
        }
        i = i + 1;
    }
    _ftprintf(stdout, _T("\n"));

    return 0;
}

unsafe extern "C" fn DriveTypeMain(argc: ::core::ffi::c_int, argv: *const *const TCHAR) -> ::core::ffi::c_int {
    let mut Type: UINT;

    /* We need a volume (letter) */
    if argc < 2 {
        _ftprintf(stderr, _T("Usage: fsutil fsinfo drivetype <volume>\n"));
        _ftprintf(stderr, _T("\tFor example: fsutil fsinfo drivetype c:\n"));
        return 1;
    }

    /* Get its drive type and make it readable */
    Type = GetDriveType(*argv.offset(1 as isize));
    match Type {
        DRIVE_UNKNOWN => {
            _ftprintf(stdout, _T("%s - unknown drive type\n"), *argv.offset(1 as isize));
        }

        DRIVE_NO_ROOT_DIR => {
            _ftprintf(stdout, _T("%s - not a root directory\n"), *argv.offset(1 as isize));
        }

        DRIVE_REMOVABLE => {
            _ftprintf(stdout, _T("%s - removable drive\n"), *argv.offset(1 as isize));
        }

        DRIVE_FIXED => {
            _ftprintf(stdout, _T("%s - fixed drive\n"), *argv.offset(1 as isize));
        }

        DRIVE_REMOTE => {
            _ftprintf(stdout, _T("%s - remote or network drive\n"), *argv.offset(1 as isize));
        }

        DRIVE_CDROM => {
            _ftprintf(stdout, _T("%s - CD-ROM drive\n"), *argv.offset(1 as isize));
        }

        DRIVE_RAMDISK => {
            _ftprintf(stdout, _T("%s - RAM disk drive\n"), *argv.offset(1 as isize));
        }

        _ => {}
    }

    return 0;
}

unsafe extern "C" fn VolumeInfoMain(mut argc: ::core::ffi::c_int, mut argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Serial: DWORD = 0;
    let mut MaxComponentLen: DWORD = 0;
    let mut Flags: DWORD = 0;
    let mut VolumeName: [TCHAR; (MAX_PATH + 1) as usize] = [0; (MAX_PATH + 1) as usize];
    let mut FileSystem: [TCHAR; (MAX_PATH + 1) as usize] = [0; (MAX_PATH + 1) as usize];

    /* We need a volume (path) */
    if argc < 2 {
        _ftprintf(stderr, _T("Usage: fsutil fsinfo volumeinfo <volume_path>\n"));
        _ftprintf(stderr, _T("\tFor example: fsutil fsinfo volumeinfo c:\\\n"));
        return 1;
    }

    /* Gather information */
    if GetVolumeInformation(
        *argv.offset(1 as isize) as LPCTSTR,
        core::ptr::addr_of_mut!(VolumeName) as LPTSTR,
        (MAX_PATH + 1) as DWORD,
        core::ptr::addr_of_mut!(Serial),
        core::ptr::addr_of_mut!(MaxComponentLen),
        core::ptr::addr_of_mut!(Flags),
        core::ptr::addr_of_mut!(FileSystem) as LPTSTR,
        (MAX_PATH + 1) as DWORD,
    ) == 0
    {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    /* Display general information */
    _ftprintf(stdout, _T("Volume name: %s\n"), core::ptr::addr_of_mut!(VolumeName) as *mut TCHAR);
    _ftprintf(stdout, _T("Volume serial number: 0x%x\n"), Serial);
    _ftprintf(stdout, _T("Maximum component length: %u\n"), MaxComponentLen);
    _ftprintf(stdout, _T("File system name: %s\n"), core::ptr::addr_of_mut!(FileSystem) as *mut TCHAR);

    /* Display specific flags */
    if (Flags & FILE_CASE_SENSITIVE_SEARCH) != 0 {
        _ftprintf(stdout, _T("Supports case-sensitive file names\n"));
    }
    if (Flags & FILE_CASE_PRESERVED_NAMES) != 0 {
        _ftprintf(stdout, _T("Supports preserved case of file names\n"));
    }
    if (Flags & FILE_UNICODE_ON_DISK) != 0 {
        _ftprintf(stdout, _T("Supports unicode file names\n"));
    }
    if (Flags & FILE_PERSISTENT_ACLS) != 0 {
        _ftprintf(stdout, _T("Preserves and applies ACLs\n"));
    }
    if (Flags & FILE_FILE_COMPRESSION) != 0 {
        _ftprintf(stdout, _T("Supports compression per file\n"));
    }
    if (Flags & FILE_VOLUME_QUOTAS) != 0 {
        _ftprintf(stdout, _T("Supports disk quotas\n"));
    }
    if (Flags & FILE_SUPPORTS_SPARSE_FILES) != 0 {
        _ftprintf(stdout, _T("Supports sparse files\n"));
    }
    if (Flags & FILE_SUPPORTS_REPARSE_POINTS) != 0 {
        _ftprintf(stdout, _T("Supports reparse points\n"));
    }
    if (Flags & FILE_VOLUME_IS_COMPRESSED) != 0 {
        _ftprintf(stdout, _T("Is a compressed volume\n"));
    }
    if (Flags & FILE_SUPPORTS_OBJECT_IDS) != 0 {
        _ftprintf(stdout, _T("Supports object identifiers\n"));
    }
    if (Flags & FILE_SUPPORTS_ENCRYPTION) != 0 {
        _ftprintf(stdout, _T("Supports the Encrypted File System (EFS)\n"));
    }
    if (Flags & FILE_NAMED_STREAMS) != 0 {
        _ftprintf(stdout, _T("Supports named streams\n"));
    }
    if (Flags & FILE_READ_ONLY_VOLUME) != 0 {
        _ftprintf(stdout, _T("Is a read-only volume\n"));
    }
    if (Flags & FILE_SEQUENTIAL_WRITE_ONCE) != 0 {
        _ftprintf(stdout, _T("Supports a single sequential write\n"));
    }
    if (Flags & FILE_SUPPORTS_TRANSACTIONS) != 0 {
        _ftprintf(stdout, _T("Supports transactions\n"));
    }
    if (Flags & FILE_SUPPORTS_HARD_LINKS) != 0 {
        _ftprintf(stdout, _T("Supports hard links\n"));
    }
    if (Flags & FILE_SUPPORTS_EXTENDED_ATTRIBUTES) != 0 {
        _ftprintf(stdout, _T("Supports extended attributes\n"));
    }
    if (Flags & FILE_SUPPORTS_OPEN_BY_FILE_ID) != 0 {
        _ftprintf(stdout, _T("Supports opening files per file identifier\n"));
    }
    if (Flags & FILE_SUPPORTS_USN_JOURNAL) != 0 {
        _ftprintf(stdout, _T("Supports Update Sequence Number (USN) journals\n"));
    }
    if (Flags & FILE_DAX_VOLUME) != 0 {
        _ftprintf(stdout, _T("Is a direct access volume\n"));
    }

    return 0;
}

unsafe extern "C" fn NtfsInfoMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Volume: HANDLE = core::ptr::null_mut();
    let mut BytesRead: DWORD = 0;
    let mut Data: C2RustUnnamed_11 = core::mem::zeroed();
    let _ = (NTFS_VOLUME_DATA_BUFFER, NTFS_EXTENDED_VOLUME_DATA);

    /* We need a volume (letter or GUID) */
    if argc < 2 {
        _ftprintf(stderr, _T("Usage: fsutil fsinfo ntfsinfo <volume>\n"));
        _ftprintf(stderr, _T("\tFor example: fsutil fsinfo ntfsinfo c:\n"));
        return 1;
    }

    /* Get a handle for the volume */
    Volume = OpenVolume(*argv.add(1), FALSE, TRUE);
    if Volume == INVALID_HANDLE_VALUE {
        return 1;
    }

    /* And query the NTFS data */
    if DeviceIoControl(
        Volume,
        FSCTL_GET_NTFS_VOLUME_DATA,
        core::ptr::null_mut(),
        0,
        &mut Data as *mut _ as LPVOID,
        core::mem::size_of::<C2RustUnnamed_11>() as DWORD,
        &mut BytesRead,
        core::ptr::null_mut(),
    ) == FALSE
    {
        PrintErrorMessage(GetLastError());
        CloseHandle(Volume);
        return 1;
    }

    /* We no longer need the volume */
    CloseHandle(Volume);

    /* Dump data */
    _ftprintf(stdout, _T("NTFS volume serial number:\t\t0x%0.16I64x\n"), Data.VolumeSerialNumber.QuadPart);
    /* Only print version if extended structure was returned */
    if BytesRead as usize > core::mem::size_of::<NTFS_VOLUME_DATA_BUFFER>() {
        _ftprintf(stdout, _T("Version:\t\t\t\t%u.%u\n"), Data.MajorVersion, Data.MinorVersion);
    }
    _ftprintf(stdout, _T("Number of sectors:\t\t\t0x%0.16I64x\n"), Data.NumberSectors.QuadPart);
    _ftprintf(stdout, _T("Total number of clusters:\t\t0x%0.16I64x\n"), Data.TotalClusters.QuadPart);
    _ftprintf(stdout, _T("Free clusters:\t\t\t\t0x%0.16I64x\n"), Data.FreeClusters.QuadPart);
    _ftprintf(stdout, _T("Total number of reserved clusters:\t0x%0.16I64x\n"), Data.TotalReserved.QuadPart);
    _ftprintf(stdout, _T("Bytes per sector:\t\t\t%d\n"), Data.BytesPerSector);
    _ftprintf(stdout, _T("Bytes per cluster:\t\t\t%d\n"), Data.BytesPerCluster);
    _ftprintf(stdout, _T("Bytes per file record segment:\t\t%d\n"), Data.BytesPerFileRecordSegment);
    _ftprintf(stdout, _T("Clusters per file record segment:\t%d\n"), Data.ClustersPerFileRecordSegment);
    _ftprintf(stdout, _T("MFT valid data length:\t\t\t0x%0.16I64x\n"), Data.MftValidDataLength.QuadPart);
    _ftprintf(stdout, _T("MFT start LCN:\t\t\t\t0x%0.16I64x\n"), Data.MftStartLcn.QuadPart);
    _ftprintf(stdout, _T("MFT2 start LCN:\t\t\t\t0x%0.16I64x\n"), Data.Mft2StartLcn.QuadPart);
    _ftprintf(stdout, _T("MFT zone start:\t\t\t\t0x%0.16I64x\n"), Data.MftZoneStart.QuadPart);
    _ftprintf(stdout, _T("MFT zone end:\t\t\t\t0x%0.16I64x\n"), Data.MftZoneEnd.QuadPart);

    return 0;
}

unsafe extern "C" fn DumpBase(Base: PFILESYSTEM_STATISTICS, Name: *mut TCHAR) {
    /* Print FS name */
    _ftprintf(stdout, _T("File system type: %s\n\n"), Name);

    /* And then, dump any base stat */
    DUMP_VALUE(Base, UserFileReads);
    DUMP_VALUE(Base, UserFileReadBytes);
    DUMP_VALUE(Base, UserDiskReads);
    DUMP_VALUE(Base, UserFileWrites);
    DUMP_VALUE(Base, UserFileWriteBytes);
    DUMP_VALUE(Base, UserDiskWrites);
    DUMP_VALUE(Base, MetaDataReads);
    DUMP_VALUE(Base, MetaDataReadBytes);
    DUMP_VALUE(Base, MetaDataDiskReads);
    DUMP_VALUE(Base, MetaDataWrites);
    DUMP_VALUE(Base, MetaDataWriteBytes);
    DUMP_VALUE(Base, MetaDataDiskWrites);

    _ftprintf(stdout, _T("\n"));
}

unsafe extern "C" fn DumpExFat(Statistics: PVOID, Specific: PVOID) {
    let mut ExFat: PEXFAT_STATISTICS;
    let mut Base: PFILESYSTEM_STATISTICS;

    Base = Statistics as PFILESYSTEM_STATISTICS;
    ExFat = Specific as PEXFAT_STATISTICS;

    /* First, display the generic stats */
    let ExFatName: [u16; 6] = ['E' as u16, 'X' as u16, 'F' as u16, 'A' as u16, 'T' as u16, 0];
    DumpBase(Base, ExFatName.as_ptr() as *mut TCHAR);

    /* Then, display the EXFAT specific ones */
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"CreateHits\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).CreateHits);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"SuccessfulCreates\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).SuccessfulCreates);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"FailedCreates\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).FailedCreates);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"NonCachedReads\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).NonCachedReads);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"NonCachedReadBytes\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).NonCachedReadBytes);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"NonCachedWrites\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).NonCachedWrites);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"NonCachedWriteBytes\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).NonCachedWriteBytes);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"NonCachedDiskReads\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).NonCachedDiskReads);
    fprintf(stdout, b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char, b"NonCachedDiskWrites\0".as_ptr() as *const ::core::ffi::c_char, (*ExFat).NonCachedDiskWrites);
}

unsafe extern "C" fn DumpFat(mut Statistics: PVOID, mut Specific: PVOID) {
    let mut Fat: PFAT_STATISTICS = ::core::ptr::null_mut::<_FAT_STATISTICS>();
    let mut Base: PFILESYSTEM_STATISTICS = ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
    Base = Statistics as PFILESYSTEM_STATISTICS;
    Fat = Specific as PFAT_STATISTICS;

    /* First, display the generic stats */
    let mut FatName: [u16; 4] = ['F' as u16, 'A' as u16, 'T' as u16, 0];
    DumpBase(Base, FatName.as_mut_ptr() as *mut TCHAR);

    /* Then, display the FAT specific ones */
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"CreateHits\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).CreateHits,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"SuccessfulCreates\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).SuccessfulCreates,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"FailedCreates\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).FailedCreates,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"NonCachedReads\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).NonCachedReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"NonCachedReadBytes\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).NonCachedReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"NonCachedWrites\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).NonCachedWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"NonCachedWriteBytes\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).NonCachedWriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"NonCachedDiskReads\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).NonCachedDiskReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"NonCachedDiskWrites\0".as_ptr() as *const ::core::ffi::c_char,
        (*Fat).NonCachedDiskWrites,
    );
}

unsafe extern "C" fn DumpNtfs(mut Statistics: PVOID, mut Specific: PVOID) {
    let mut Ntfs: PNTFS_STATISTICS = core::ptr::null_mut::<_NTFS_STATISTICS>();
    let mut Base: PFILESYSTEM_STATISTICS = core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
    Base = Statistics as PFILESYSTEM_STATISTICS;
    Ntfs = Specific as PNTFS_STATISTICS;
    let mut NtfsName: [u16; 5] = ['N' as u16, 'T' as u16, 'F' as u16, 'S' as u16, 0];
    DumpBase(Base, NtfsName.as_mut_ptr() as *mut TCHAR);
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftReads\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftReadBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftWrites\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftWriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftWriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"Mft2Writes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).Mft2Writes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"Mft2WriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).Mft2WriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"RootIndexReads\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).RootIndexReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"RootIndexReadBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).RootIndexReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"RootIndexWrites\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).RootIndexWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"RootIndexWriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).RootIndexWriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"BitmapReads\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).BitmapReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"BitmapReadBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).BitmapReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"BitmapWrites\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).BitmapWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"BitmapWriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).BitmapWriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftBitmapReads\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftBitmapReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftBitmapReadBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftBitmapReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftBitmapWrites\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftBitmapWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"MftBitmapWriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).MftBitmapWriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"UserIndexReads\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).UserIndexReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"UserIndexReadBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).UserIndexReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"UserIndexWrites\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).UserIndexWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"UserIndexWriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).UserIndexWriteBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"LogFileReads\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).LogFileReads,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"LogFileReadBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).LogFileReadBytes,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"LogFileWrites\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).LogFileWrites,
    );
    fprintf(
        (&raw mut _iob as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"LogFileWriteBytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*Ntfs).LogFileWriteBytes,
    );
}

unsafe extern "C" fn ValidateSizes(
    mut Length: ULONG,
    mut ProcCount: DWORD,
    mut BytesRead: DWORD,
    mut SpecificSize: DWORD,
) -> ::core::ffi::c_int {
    /* Check whether we could read our base length for every processor */
    if (Length as DWORD).wrapping_mul(ProcCount) > BytesRead {
        _ftprintf(
            stderr,
            _T("Only performed a partial read: %lu (expected: %lu)\n"),
            BytesRead,
            (Length as DWORD).wrapping_mul(ProcCount),
        );
        return 1;
    }

    /* Check whether this covers a specific entry and a generic entry for every processor */
    if (::core::mem::size_of::<FILESYSTEM_STATISTICS>() as usize)
        .wrapping_add(SpecificSize as usize)
        .wrapping_mul(ProcCount as usize)
        > BytesRead as usize
    {
        _ftprintf(
            stderr,
            _T("Only performed a partial read: %lu (expected: %Iu)\n"),
            BytesRead,
            (::core::mem::size_of::<FILESYSTEM_STATISTICS>() as usize)
                .wrapping_add(SpecificSize as usize),
        );
        return 1;
    }

    return 0;
}

unsafe extern "C" fn SumBase(Base: PFILESYSTEM_STATISTICS, NextBase: PFILESYSTEM_STATISTICS) {
    /* Sum any entry in the generic structures */
    SUM_VALUE(Base, NextBase, UserFileReads);
    SUM_VALUE(Base, NextBase, UserFileReadBytes);
    SUM_VALUE(Base, NextBase, UserDiskReads);
    SUM_VALUE(Base, NextBase, UserFileWrites);
    SUM_VALUE(Base, NextBase, UserFileWriteBytes);
    SUM_VALUE(Base, NextBase, UserDiskWrites);
    SUM_VALUE(Base, NextBase, MetaDataReads);
    SUM_VALUE(Base, NextBase, MetaDataReadBytes);
    SUM_VALUE(Base, NextBase, MetaDataDiskReads);
    SUM_VALUE(Base, NextBase, MetaDataWrites);
    SUM_VALUE(Base, NextBase, MetaDataWriteBytes);
    SUM_VALUE(Base, NextBase, MetaDataDiskWrites);
}

unsafe extern "C" fn SumExFat(
    mut Statistics: PVOID,
    mut Specific: PVOID,
    mut Length: ULONG,
    mut ProcCount: DWORD,
    mut BytesRead: DWORD,
) -> ::core::ffi::c_int {
    let mut i: DWORD = 0;
    let mut ExFat: PEXFAT_STATISTICS = ::core::ptr::null_mut::<_EXFAT_STATISTICS>();
    let mut Base: PFILESYSTEM_STATISTICS = ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
    if ValidateSizes(
        Length,
        ProcCount,
        BytesRead,
        ::core::mem::size_of::<EXFAT_STATISTICS>() as DWORD,
    ) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    Base = Statistics as PFILESYSTEM_STATISTICS;
    ExFat = Specific as PEXFAT_STATISTICS;
    i = 1 as DWORD;
    while i < ProcCount {
        let mut NextExFat: PEXFAT_STATISTICS = ::core::ptr::null_mut::<_EXFAT_STATISTICS>();
        let mut NextBase: PFILESYSTEM_STATISTICS =
            ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
        NextBase = (Base as ULONG_PTR).wrapping_add((Length as DWORD).wrapping_mul(i) as ULONG_PTR)
            as PFILESYSTEM_STATISTICS;
        NextExFat = (ExFat as ULONG_PTR)
            .wrapping_add((Length as DWORD).wrapping_mul(i) as ULONG_PTR)
            as PEXFAT_STATISTICS;
        SumBase(Base, NextBase);
        (*ExFat).CreateHits = (*ExFat).CreateHits.wrapping_add((*NextExFat).CreateHits);
        (*ExFat).SuccessfulCreates = (*ExFat)
            .SuccessfulCreates
            .wrapping_add((*NextExFat).SuccessfulCreates);
        (*ExFat).FailedCreates = (*ExFat)
            .FailedCreates
            .wrapping_add((*NextExFat).FailedCreates);
        (*ExFat).NonCachedReads = (*ExFat)
            .NonCachedReads
            .wrapping_add((*NextExFat).NonCachedReads);
        (*ExFat).NonCachedReadBytes = (*ExFat)
            .NonCachedReadBytes
            .wrapping_add((*NextExFat).NonCachedReadBytes);
        (*ExFat).NonCachedWrites = (*ExFat)
            .NonCachedWrites
            .wrapping_add((*NextExFat).NonCachedWrites);
        (*ExFat).NonCachedWriteBytes = (*ExFat)
            .NonCachedWriteBytes
            .wrapping_add((*NextExFat).NonCachedWriteBytes);
        (*ExFat).NonCachedDiskReads = (*ExFat)
            .NonCachedDiskReads
            .wrapping_add((*NextExFat).NonCachedDiskReads);
        (*ExFat).NonCachedDiskWrites = (*ExFat)
            .NonCachedDiskWrites
            .wrapping_add((*NextExFat).NonCachedDiskWrites);
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn SumFat(
    mut Statistics: PVOID,
    mut Specific: PVOID,
    mut Length: ULONG,
    mut ProcCount: DWORD,
    mut BytesRead: DWORD,
) -> ::core::ffi::c_int {
    let mut i: DWORD = 0;
    let mut Fat: PFAT_STATISTICS = ::core::ptr::null_mut::<_FAT_STATISTICS>();
    let mut Base: PFILESYSTEM_STATISTICS = ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
    /* First, validate we won't read beyond allocation */
    if ValidateSizes(
        Length,
        ProcCount,
        BytesRead,
        core::mem::size_of::<FAT_STATISTICS>() as DWORD,
    ) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    Base = Statistics as PFILESYSTEM_STATISTICS;
    Fat = Specific as PFAT_STATISTICS;
    i = 1 as DWORD;
    while i < ProcCount {
        let mut NextFat: PFAT_STATISTICS = ::core::ptr::null_mut::<_FAT_STATISTICS>();
        let mut NextBase: PFILESYSTEM_STATISTICS =
            ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
        /* GET_NEXT(Base, Length, i, COMMENT_TOKEN_ECHO____) */
        NextBase = (Base as ULONG_PTR).wrapping_add((Length as DWORD).wrapping_mul(i) as ULONG_PTR)
            as PFILESYSTEM_STATISTICS;
        /* GET_NEXT(Fat, Length, i, COMMENT_TOKEN_ECHO____) */
        NextFat = (Fat as ULONG_PTR).wrapping_add((Length as DWORD).wrapping_mul(i) as ULONG_PTR)
            as PFAT_STATISTICS;
        /* Generic first */
        SumBase(Base, NextBase);
        /* Specific then */
        /* SUM_VALUE(Fat, NextFat, CreateHits) */
        (*Fat).CreateHits = (*Fat).CreateHits.wrapping_add((*NextFat).CreateHits);
        /* SUM_VALUE(Fat, NextFat, SuccessfulCreates) */
        (*Fat).SuccessfulCreates = (*Fat)
            .SuccessfulCreates
            .wrapping_add((*NextFat).SuccessfulCreates);
        /* SUM_VALUE(Fat, NextFat, FailedCreates) */
        (*Fat).FailedCreates = (*Fat).FailedCreates.wrapping_add((*NextFat).FailedCreates);
        /* SUM_VALUE(Fat, NextFat, NonCachedReads) */
        (*Fat).NonCachedReads = (*Fat)
            .NonCachedReads
            .wrapping_add((*NextFat).NonCachedReads);
        /* SUM_VALUE(Fat, NextFat, NonCachedReadBytes) */
        (*Fat).NonCachedReadBytes = (*Fat)
            .NonCachedReadBytes
            .wrapping_add((*NextFat).NonCachedReadBytes);
        /* SUM_VALUE(Fat, NextFat, NonCachedWrites) */
        (*Fat).NonCachedWrites = (*Fat)
            .NonCachedWrites
            .wrapping_add((*NextFat).NonCachedWrites);
        /* SUM_VALUE(Fat, NextFat, NonCachedWriteBytes) */
        (*Fat).NonCachedWriteBytes = (*Fat)
            .NonCachedWriteBytes
            .wrapping_add((*NextFat).NonCachedWriteBytes);
        /* SUM_VALUE(Fat, NextFat, NonCachedDiskReads) */
        (*Fat).NonCachedDiskReads = (*Fat)
            .NonCachedDiskReads
            .wrapping_add((*NextFat).NonCachedDiskReads);
        /* SUM_VALUE(Fat, NextFat, NonCachedDiskWrites) */
        (*Fat).NonCachedDiskWrites = (*Fat)
            .NonCachedDiskWrites
            .wrapping_add((*NextFat).NonCachedDiskWrites);
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn SumNtfs(
    mut Statistics: PVOID,
    mut Specific: PVOID,
    mut Length: ULONG,
    mut ProcCount: DWORD,
    mut BytesRead: DWORD,
) -> ::core::ffi::c_int {
    let mut i: DWORD = 0;
    let mut Ntfs: PNTFS_STATISTICS = ::core::ptr::null_mut::<_NTFS_STATISTICS>();
    let mut Base: PFILESYSTEM_STATISTICS = ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
    if ValidateSizes(
        Length,
        ProcCount,
        BytesRead,
        ::core::mem::size_of::<NTFS_STATISTICS>() as DWORD,
    ) != 0
    {
        return 1;
    }
    Base = Statistics as PFILESYSTEM_STATISTICS;
    Ntfs = Specific as PNTFS_STATISTICS;
    i = 1;
    while i < ProcCount {
        let mut NextNtfs: PNTFS_STATISTICS = ::core::ptr::null_mut::<_NTFS_STATISTICS>();
        let mut NextBase: PFILESYSTEM_STATISTICS =
            ::core::ptr::null_mut::<_FILESYSTEM_STATISTICS>();
        NextBase = (Base as ULONG_PTR).wrapping_add((Length as DWORD).wrapping_mul(i) as ULONG_PTR)
            as PFILESYSTEM_STATISTICS;
        NextNtfs = (Ntfs as ULONG_PTR).wrapping_add((Length as DWORD).wrapping_mul(i) as ULONG_PTR)
            as PNTFS_STATISTICS;
        SumBase(Base, NextBase);
        (*Ntfs).MftReads = (*Ntfs).MftReads.wrapping_add((*NextNtfs).MftReads);
        (*Ntfs).MftReadBytes = (*Ntfs).MftReadBytes.wrapping_add((*NextNtfs).MftReadBytes);
        (*Ntfs).MftWrites = (*Ntfs).MftWrites.wrapping_add((*NextNtfs).MftWrites);
        (*Ntfs).MftWriteBytes = (*Ntfs)
            .MftWriteBytes
            .wrapping_add((*NextNtfs).MftWriteBytes);
        (*Ntfs).Mft2Writes = (*Ntfs).Mft2Writes.wrapping_add((*NextNtfs).Mft2Writes);
        (*Ntfs).Mft2WriteBytes = (*Ntfs)
            .Mft2WriteBytes
            .wrapping_add((*NextNtfs).Mft2WriteBytes);
        (*Ntfs).RootIndexReads = (*Ntfs)
            .RootIndexReads
            .wrapping_add((*NextNtfs).RootIndexReads);
        (*Ntfs).RootIndexReadBytes = (*Ntfs)
            .RootIndexReadBytes
            .wrapping_add((*NextNtfs).RootIndexReadBytes);
        (*Ntfs).RootIndexWrites = (*Ntfs)
            .RootIndexWrites
            .wrapping_add((*NextNtfs).RootIndexWrites);
        (*Ntfs).RootIndexWriteBytes = (*Ntfs)
            .RootIndexWriteBytes
            .wrapping_add((*NextNtfs).RootIndexWriteBytes);
        (*Ntfs).BitmapReads = (*Ntfs).BitmapReads.wrapping_add((*NextNtfs).BitmapReads);
        (*Ntfs).BitmapReadBytes = (*Ntfs)
            .BitmapReadBytes
            .wrapping_add((*NextNtfs).BitmapReadBytes);
        (*Ntfs).BitmapWrites = (*Ntfs).BitmapWrites.wrapping_add((*NextNtfs).BitmapWrites);
        (*Ntfs).BitmapWriteBytes = (*Ntfs)
            .BitmapWriteBytes
            .wrapping_add((*NextNtfs).BitmapWriteBytes);
        (*Ntfs).MftBitmapReads = (*Ntfs)
            .MftBitmapReads
            .wrapping_add((*NextNtfs).MftBitmapReads);
        (*Ntfs).MftBitmapReadBytes = (*Ntfs)
            .MftBitmapReadBytes
            .wrapping_add((*NextNtfs).MftBitmapReadBytes);
        (*Ntfs).MftBitmapWrites = (*Ntfs)
            .MftBitmapWrites
            .wrapping_add((*NextNtfs).MftBitmapWrites);
        (*Ntfs).MftBitmapWriteBytes = (*Ntfs)
            .MftBitmapWriteBytes
            .wrapping_add((*NextNtfs).MftBitmapWriteBytes);
        (*Ntfs).UserIndexReads = (*Ntfs)
            .UserIndexReads
            .wrapping_add((*NextNtfs).UserIndexReads);
        (*Ntfs).UserIndexReadBytes = (*Ntfs)
            .UserIndexReadBytes
            .wrapping_add((*NextNtfs).UserIndexReadBytes);
        (*Ntfs).UserIndexWrites = (*Ntfs)
            .UserIndexWrites
            .wrapping_add((*NextNtfs).UserIndexWrites);
        (*Ntfs).UserIndexWriteBytes = (*Ntfs)
            .UserIndexWriteBytes
            .wrapping_add((*NextNtfs).UserIndexWriteBytes);
        (*Ntfs).LogFileReads = (*Ntfs).LogFileReads.wrapping_add((*NextNtfs).LogFileReads);
        (*Ntfs).LogFileReadBytes = (*Ntfs)
            .LogFileReadBytes
            .wrapping_add((*NextNtfs).LogFileReadBytes);
        (*Ntfs).LogFileWrites = (*Ntfs)
            .LogFileWrites
            .wrapping_add((*NextNtfs).LogFileWrites);
        (*Ntfs).LogFileWriteBytes = (*Ntfs)
            .LogFileWriteBytes
            .wrapping_add((*NextNtfs).LogFileWriteBytes);
        i = i.wrapping_add(1);
    }
    return 0;
}

unsafe extern "C" fn StatisticsMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Volume: HANDLE = core::ptr::null_mut();
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
    let mut Buffer: FILESYSTEM_STATISTICS = core::mem::zeroed();
    let mut Statistics: PFILESYSTEM_STATISTICS = core::ptr::null_mut();
    let mut BytesRead: DWORD = 0;
    let mut Length: DWORD = 0;
    let mut ProcCount: DWORD = 0;
    /* +1 because 0 isn't assigned to a filesystem */
    let mut DumpFct: [Option<unsafe extern "C" fn(PVOID, PVOID)>; FILESYSTEM_STATISTICS_TYPE_EXFAT + 1] = [None, Some(DumpNtfs), Some(DumpFat), Some(DumpExFat)];
    let mut SumFct: [Option<unsafe extern "C" fn(PVOID, PVOID, ULONG, DWORD, DWORD) -> ::core::ffi::c_int>; FILESYSTEM_STATISTICS_TYPE_EXFAT + 1] = [None, Some(SumNtfs), Some(SumFat), Some(SumExFat)];

    /* We need a volume (letter or GUID) */
    if argc < 2 {
        _ftprintf(stderr, _T("Usage: fsutil fsinfo statistics <volume>\n"));
        _ftprintf(stderr, _T("\tFor example: fsutil fsinfo statistics c:\n"));
        return 1;
    }

    /* Get a handle for the volume */
    Volume = OpenVolume(*argv.offset(1 as isize), FALSE, FALSE);
    if Volume == INVALID_HANDLE_VALUE {
        return 1;
    }

    /* And query the statistics status - this call is expected to fail */
    Statistics = core::ptr::addr_of_mut!(Buffer);
    Length = core::mem::size_of::<FILESYSTEM_STATISTICS>() as DWORD;
    /* Assume we have a single proc for now */
    ProcCount = 1;
    if DeviceIoControl(Volume, FSCTL_FILESYSTEM_GET_STATISTICS, core::ptr::null_mut(), 0, Statistics as LPVOID,
                        Length, core::ptr::addr_of_mut!(BytesRead), core::ptr::null_mut()) == FALSE
    {
        let mut Error: DWORD;

        /* Check we failed because we provided a too small buffer */
        Error = GetLastError();
        if Error == ERROR_MORE_DATA {
            /* Get proc count */
            GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));
            ProcCount = SystemInfo.dwNumberOfProcessors;
            /* Get the maximum size to allocate: it's the total size (generic + specific) for every proc */
            Length = ((*Statistics).SizeOfCompleteStructure as DWORD).wrapping_mul(ProcCount);

            Statistics = LocalAlloc(LPTR, Length as SIZE_T) as PFILESYSTEM_STATISTICS;
            if Statistics.is_null() {
                _ftprintf(stderr, _T("Failed to allocate memory!\n"));
                CloseHandle(Volume);
                return 1;
            }

            /* Reissue the FSCTL, it's supposed to succeed now! */
            if DeviceIoControl(Volume, FSCTL_FILESYSTEM_GET_STATISTICS, core::ptr::null_mut(), 0, Statistics as LPVOID,
                                Length, core::ptr::addr_of_mut!(BytesRead), core::ptr::null_mut()) == FALSE
            {
                PrintErrorMessage(GetLastError());
                LocalFree(Statistics as HLOCAL);
                CloseHandle(Volume);
                return 1;
            }
        } else {
            PrintErrorMessage(Error);
            CloseHandle(Volume);
            return 1;
        }
    }

    /* No need to deal with the volume any longer */
    CloseHandle(Volume);

    /* We only support FAT, EXFAT and NTFS for now */
    if (*Statistics).FileSystemType as ::core::ffi::c_int > FILESYSTEM_STATISTICS_TYPE_EXFAT
        || ((*Statistics).FileSystemType as ::core::ffi::c_int) < FILESYSTEM_STATISTICS_TYPE_NTFS
    {
        _ftprintf(stderr, _T("Unrecognized file system type: %d\n"), (*Statistics).FileSystemType as ::core::ffi::c_int);
        if Statistics != core::ptr::addr_of_mut!(Buffer) {
            LocalFree(Statistics as HLOCAL);
        }

        return 1;
    }

    /* Sum all the statistics (both generic and specific) from all the processors in the first entry */
    if SumFct[(*Statistics).FileSystemType as usize].expect("non-null function pointer")(
        Statistics as PVOID,
        (Statistics as usize).wrapping_add(core::mem::size_of::<FILESYSTEM_STATISTICS>() as usize) as PVOID,
        (*Statistics).SizeOfCompleteStructure,
        ProcCount,
        BytesRead,
    ) != 0
    {
        if Statistics != core::ptr::addr_of_mut!(Buffer) {
            LocalFree(Statistics as HLOCAL);
        }

        return 1;
    }

    /* And finally, display the statistics (both generic and specific) */
    DumpFct[(*Statistics).FileSystemType as usize].expect("non-null function pointer")(
        Statistics as PVOID,
        (Statistics as usize).wrapping_add(core::mem::size_of::<FILESYSTEM_STATISTICS>() as usize) as PVOID,
    );

    /* If we allocated memory, release it */
    if Statistics != core::ptr::addr_of_mut!(Buffer) {
        LocalFree(Statistics as HLOCAL);
    }

    return 0;
}

unsafe extern "C" fn PrintUsage(Command: *const TCHAR) {
    PrintDefaultUsage(
        _T(" FSINFO "),
        Command,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0]))
    );
}
#[no_mangle]
pub unsafe extern "C" fn FsInfoMain(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *const TCHAR,
) -> ::core::ffi::c_int {
    return FindHandler(
        argc,
        argv,
        core::ptr::addr_of!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(
                &*(core::ptr::addr_of!(HandlersList) as *const HandlerItem)
                    .add(0),
            )) as ::core::ffi::c_int,
        Some(PrintUsage as unsafe extern "C" fn(*const TCHAR) -> ()),
    );
}
