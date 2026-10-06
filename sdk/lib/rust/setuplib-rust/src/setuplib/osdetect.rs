use crate::setuplib::genlist::AppendGenericListEntry;
use crate::setuplib::arcname::ArcPathToNtPath;
use crate::setuplib::bldrsup::CloseBootStore;
use crate::setuplib::genlist::CreateGenericList;
use crate::setuplib::bldrsup::EnumerateBootStoreEntries;
use crate::setuplib::bldrsup::FindBootStore;
use crate::setuplib::genlist::GetFirstListEntry;
use crate::setuplib::genlist::GetListEntryData;
use crate::setuplib::genlist::GetNextListEntry;
use crate::setuplib::ntverrsrc::NtGetVersionResource;
use crate::setuplib::filesup::NtPathToDiskPartComponents;
use crate::setuplib::ntverrsrc::NtVerQueryValue;
use crate::setuplib::filesup::OpenAndMapFile;
use crate::setuplib::bldrsup::OpenBootStoreByHandle;
use crate::setuplib::partlist::SelectPartition;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0D63BBDFC666: [u16; 36] = [
    0x0053u16, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0058, 0x0025,
    0x0030, 0x0034, 0x0058, 0x005C, 0x0043, 0x006F, 0x006D, 0x0070, 0x0061, 0x006E,
    0x0079, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_2F991909CCF2: [u16; 17] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0063,
    0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_3CBEB9720866: [u16; 18] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0064,
    0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0073, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_4C248048A213: [u16; 10] = [0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_5F7D2D6C0B3C: [u16; 19] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x006E,
    0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_6AC3419A3E2E: [u16; 25] = [
    0x005Cu16, 0x0056, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_7F3E91746FB1: [u16; 22] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x006E,
    0x0074, 0x006F, 0x0073, 0x006B, 0x0072, 0x006E, 0x006C, 0x002E, 0x0065, 0x0078,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_BCF3913447D5: [u16; 4] = [0x0025u16, 0x0073, 0x005C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const NTOS_OPTIONS_SIGNATURE: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"NTOS_5\0\0\0") };

pub const VENDOR_REACTOS: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"R\0e\0a\0c\0t\0O\0S\0\0\0")
};

pub const VENDOR_MICROSOFT: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"M\0i\0c\0r\0o\0s\0o\0f\0t\0\0\0",
    )
};

static mut KnownVendors: [PCWSTR; 2] = [VENDOR_REACTOS.as_ptr(), VENDOR_MICROSOFT.as_ptr()];

#[repr(C)]
pub struct _ENUM_INSTALLS_DATA {
    pub List: PGENERIC_LIST,
    pub PartList: PPARTLIST,
}
pub type ENUM_INSTALLS_DATA = _ENUM_INSTALLS_DATA;
pub type PENUM_INSTALLS_DATA = *mut _ENUM_INSTALLS_DATA;
unsafe extern "system" fn EnumerateInstallations(
    mut Type: BOOT_STORE_TYPE,
    mut BootEntry: PBOOT_STORE_ENTRY,
    mut Parameter: PVOID,
) -> NTSTATUS {
    let mut Data: PENUM_INSTALLS_DATA = Parameter as PENUM_INSTALLS_DATA;
    let mut Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;
    let mut NtOsInstall: PNTOS_INSTALLATION = ::core::ptr::null_mut::<_NTOS_INSTALLATION>();
    let mut DiskNumber: ULONG = 0;
    let mut PartitionNumber: ULONG = 0;
    let mut PathComponent: PCWSTR = ::core::ptr::null::<WCHAR>();
    let mut SystemRootPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut SystemRoot: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Machine: USHORT = 0;
    let mut VendorName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut VendorNameBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if ((*BootEntry).OsOptionsLength as usize) < ::core::mem::size_of::<NTOS_OPTIONS>() as usize
        || RtlCompareMemory(
            &raw mut (*BootEntry).OsOptions as *const ::core::ffi::c_void,
            NTOS_OPTIONS_SIGNATURE.as_ptr() as *const ::core::ffi::c_void,
            RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T,
        ) as usize
            != RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as usize
    {
        DPRINT!(
            b"    An installation '%S' of unsupported type %lu\n\0" as *const u8 as PCSTR,
            (*BootEntry).FriendlyName,
            (*BootEntry).OsOptionsLength,
        );
        return STATUS_SUCCESS;
    }
    if (*Options).OsLoadPath.is_null() || *(*Options).OsLoadPath == 0 {
        DPRINT1!(
            b"    A Win2k3 install '%S' without an ARC path?!\n\0" as *const u8 as PCSTR,
            (*BootEntry).FriendlyName,
        );
        return STATUS_SUCCESS;
    }
    DPRINT!(
        b"    Found a candidate Win2k3 install '%S' with ARC path '%S'\n\0" as *const u8 as PCSTR,
        (*BootEntry).FriendlyName,
        (*Options).OsLoadPath,
    );
    NtOsInstall = FindExistingNTOSInstall(
        (*Data).List,
        (*Options).OsLoadPath,
        ::core::ptr::null_mut::<_UNICODE_STRING>(),
    );
    if !NtOsInstall.is_null() {
        DPRINT!(
            b"    An NTOS installation with name \"%S\" from vendor \"%S\" already exists in SystemRoot '%wZ'\n\0"
                as *const u8 as PCSTR,
            &raw mut (*NtOsInstall).InstallationName as *mut WCHAR,
            &raw mut (*NtOsInstall).VendorName as *mut WCHAR,
            &raw mut (*NtOsInstall).SystemArcPath,
        );
        return STATUS_SUCCESS;
    }
    RtlInitEmptyUnicodeString(
        &raw mut SystemRootPath,
        &raw mut SystemRoot as PWCHAR,
        ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as USHORT,
    );
    if ArcPathToNtPath(
        &raw mut SystemRootPath,
        (*Options).OsLoadPath,
        (*Data).PartList,
    ) == 0
    {
        DPRINT1!(
            b"ArcPathToNtPath(%S) failed, skip the installation.\n\0" as *const u8 as PCSTR,
            (*Options).OsLoadPath,
        );
        return STATUS_SUCCESS;
    }
    DPRINT!(
        b"ArcPathToNtPath() succeeded: '%S' --> '%wZ'\n\0" as *const u8 as PCSTR,
        (*Options).OsLoadPath,
        &raw mut SystemRootPath,
    );
    NtOsInstall = FindExistingNTOSInstall(
        (*Data).List,
        ::core::ptr::null::<WCHAR>(),
        &raw mut SystemRootPath,
    );
    if !NtOsInstall.is_null() {
        DPRINT1!(
            b"    An NTOS installation with name \"%S\" from vendor \"%S\" already exists in SystemRoot '%wZ'\n\0"
                as *const u8 as PCSTR,
            &raw mut (*NtOsInstall).InstallationName as *mut WCHAR,
            &raw mut (*NtOsInstall).VendorName as *mut WCHAR,
            &raw mut (*NtOsInstall).SystemNtPath,
        );
        return STATUS_SUCCESS;
    }
    DPRINT!(
        b"EnumerateInstallations: SystemRootPath: '%wZ'\n\0" as *const u8 as PCSTR,
        &raw mut SystemRootPath,
    );
    RtlInitEmptyUnicodeString(
        &raw mut VendorName,
        &raw mut VendorNameBuffer as PWCHAR,
        ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as USHORT,
    );
    if IsValidNTOSInstallation(
        &raw mut SystemRootPath,
        &raw mut Machine,
        &raw mut VendorName,
    ) == 0
    {
        return STATUS_SUCCESS;
    }
    DPRINT!(
        b"Found a valid NTOS installation in SystemRoot ARC path '%S', NT path '%wZ'\n\0"
            as *const u8 as PCSTR,
        (*Options).OsLoadPath,
        &raw mut SystemRootPath,
    );
    if NtPathToDiskPartComponents(
        SystemRootPath.Buffer as PCWSTR,
        &raw mut DiskNumber,
        &raw mut PartitionNumber,
        &raw mut PathComponent,
    ) != 0
    {
        DPRINT!(
            b"SystemRootPath = '%wZ' points to disk #%d, partition #%d, path '%S'\n\0"
                as *const u8 as PCSTR,
            &raw mut SystemRootPath,
            DiskNumber,
            PartitionNumber,
            PathComponent,
        );
    } else {
        DPRINT1!(
            b"NtPathToDiskPartComponents(%wZ) failed\n\0" as *const u8 as PCSTR,
            &raw mut SystemRootPath,
        );
    }
    NtOsInstall = AddNTOSInstallation(
        (*Data).List,
        (*BootEntry).FriendlyName,
        Machine,
        VendorName.Buffer as PCWSTR,
        (*Options).OsLoadPath,
        &raw mut SystemRootPath,
        PathComponent,
        DiskNumber,
        PartitionNumber,
    );
    if !NtOsInstall.is_null() {
        let mut PartEntry: PPARTENTRY =
            SelectPartition((*Data).PartList, DiskNumber, PartitionNumber);
        if PartEntry.is_null() {
            DPRINT1!(
                b"SelectPartition(disk #%d, partition #%d) failed\n\0" as *const u8 as PCSTR,
                DiskNumber,
                PartitionNumber,
            );
        }
        (*NtOsInstall).Volume = if !PartEntry.is_null() {
            (*PartEntry).Volume
        } else {
            ::core::ptr::null_mut::<_VOLENTRY>()
        };
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn FindSubStrI(
    str: PCWSTR,
    strSearch: PCWSTR,
) -> PCWSTR {
    let mut cp: PCWSTR = str;
    let mut s1: PCWSTR;
    let mut s2: PCWSTR;

    if *strSearch == 0 {
        return str;
    }

    while *cp != 0 {
        s1 = cp;
        s2 = strSearch;

        while *s1 != 0
            && *s2 != 0
            && towupper((*s1) as _) == towupper((*s2) as _)
        {
            s1 = s1.offset(1);
            s2 = s2.offset(1);
        }

        if *s2 == 0 {
            return cp;
        }

        cp = cp.offset(1);
    }

    core::ptr::null_mut()
}

unsafe extern "C" fn CheckForValidPEAndVendor(
    RootDirectory: *mut core::ffi::c_void,
    PathNameToFile: *const u16,
    Machine: *mut u16,
    VendorName: *mut _UNICODE_STRING,
) -> u8 {
    let mut Success: u8 = (FALSE) as u8;
    let mut Status: i32 = 0;
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut SectionHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ViewBase: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut NtHeader: PIMAGE_NT_HEADERS = core::ptr::null_mut();
    let mut VersionBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pvData: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut BufLen: u32 = 0;

    if ((*VendorName).MaximumLength as usize) < core::mem::size_of_val(&UNICODE_NULL) {
        return (FALSE) as u8;
    }

    *(*VendorName).Buffer = UNICODE_NULL;
    (*VendorName).Length = 0;

    Status = OpenAndMapFile(
        RootDirectory,
        PathNameToFile,
        &raw mut FileHandle,
        core::ptr::null_mut::<u32>(),
        &raw mut SectionHandle,
        &raw mut ViewBase,
        (FALSE) as BOOLEAN,
    );
    if (((NT_SUCCESS!(Status))) == 0) {
        DPRINT1!(
            b"Failed to open and map file '%S', Status 0x%08lx\n\0".as_ptr() as *const i8,
            PathNameToFile,
            Status,
        );
        return (FALSE) as u8;
    }

    'unmapclose: {
        NtHeader = RtlImageNtHeader(ViewBase);
        if NtHeader.is_null() {
            DPRINT1!(
                b"File '%S' does not seem to be a valid NT PE file, bail out\n\0".as_ptr() as *const i8,
                PathNameToFile,
            );
            Status = STATUS_INVALID_IMAGE_FORMAT;
            break 'unmapclose;
        }

        *Machine = (*NtHeader).FileHeader.Machine;

        Status = NtGetVersionResource(
            ((ViewBase as usize) | 1 as usize) as *mut core::ffi::c_void,
            &raw mut VersionBuffer,
            core::ptr::null_mut::<u32>(),
        );
        if (((NT_SUCCESS!(Status))) == 0) {
            DPRINT1!(
                b"Failed to get version resource for file '%S', Status 0x%08lx\n\0".as_ptr() as *const i8,
                PathNameToFile,
                Status,
            );
            break 'unmapclose;
        }

        Status = NtVerQueryValue(
            VersionBuffer,
            RUSTCONV_WSTR_6AC3419A3E2E.as_ptr(),
            &raw mut pvData,
            &raw mut BufLen,
        );
        if NT_SUCCESS!(Status) != 0 {
            let mut wCodePage: u16 = 0;
            let mut wLangID: u16 = 0;
            let mut FileInfo = [0u16; MAX_PATH as usize];

            wCodePage = LOWORD!(*(pvData as *mut u32));
            wLangID = HIWORD!(*(pvData as *mut u32));

            RtlStringCchPrintfW!(
                FileInfo.as_mut_ptr(),
                ARRAYSIZE!(FileInfo),
                RUSTCONV_WSTR_0D63BBDFC666.as_ptr(),
                wCodePage as i32,
                wLangID as i32,
            );

            Status = NtVerQueryValue(
                VersionBuffer,
                FileInfo.as_ptr(),
                &raw mut pvData,
                &raw mut BufLen,
            );

            if NT_SUCCESS!(Status) != 0 && pvData.is_null() {
                Status = STATUS_NOT_FOUND;
            }

            if NT_SUCCESS!(Status) != 0 {
                DPRINT!(
                    b"Found version vendor: \"%S\" for file '%S'\n\0".as_ptr() as *const i8,
                    pvData,
                    PathNameToFile,
                );

                RtlStringCbCopyNW(
                    (*VendorName).Buffer,
                    (*VendorName).MaximumLength as usize,
                    pvData as *const u16,
                    (BufLen as usize) * core::mem::size_of::<u16>(),
                );
                (*VendorName).Length = ((wcslen((*VendorName).Buffer as *const u16) as u16) as usize)
                    .wrapping_mul(core::mem::size_of::<u16>()) as u16;

                Success = (TRUE) as u8;
            }
        }

        if (((NT_SUCCESS!(Status))) == 0) {
            DPRINT!(
                b"No version vendor found for file '%S'\n\0".as_ptr() as *const i8,
                PathNameToFile,
            );
        }
    }

    UnMapAndCloseFile!(FileHandle, SectionHandle, ViewBase);

    Success
}

unsafe extern "C" fn IsValidNTOSInstallationByHandle(
    mut SystemRootDirectory: HANDLE,
    mut Machine: PUSHORT,
    mut VendorName: PUNICODE_STRING,
) -> BOOLEAN {
    let mut Success: BOOLEAN = (FALSE) as u8;
    let mut PathName: PCWSTR;
    let mut i: USHORT;
    let mut LocalMachine: USHORT = 0;
    let mut LocalVendorName: _UNICODE_STRING = core::mem::zeroed();
    let mut VendorNameBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if (*VendorName).MaximumLength < core::mem::size_of_val(&UNICODE_NULL) as u16 {
        VendorName = core::ptr::null_mut();
    } else {
        *(*VendorName).Buffer = UNICODE_NULL;
        (*VendorName).Length = 0;
    }

    PathName = RUSTCONV_WSTR_4C248048A213.as_ptr();
    if ((DoesPathExist(SystemRootDirectory, PathName, TRUE as BOOLEAN)) == 0) {
        return (FALSE) as u8;
    }

    PathName = RUSTCONV_WSTR_3CBEB9720866.as_ptr();
    if ((DoesPathExist(SystemRootDirectory, PathName, TRUE as BOOLEAN)) == 0) {
        return (FALSE) as u8;
    }

    PathName = RUSTCONV_WSTR_2F991909CCF2.as_ptr();
    if ((DoesPathExist(SystemRootDirectory, PathName, TRUE as BOOLEAN)) == 0) {
        return (FALSE) as u8;
    }

    RtlInitEmptyUnicodeString(&mut LocalVendorName, VendorNameBuffer.as_mut_ptr(), core::mem::size_of_val(&VendorNameBuffer) as u16);

    PathName = RUSTCONV_WSTR_7F3E91746FB1.as_ptr();
    Success = CheckForValidPEAndVendor(SystemRootDirectory, PathName, &mut LocalMachine, &mut LocalVendorName);
    if ((Success) == 0) {
        DPRINT1!("Kernel executable '%S' is either not a PE file, or does not have any vendor?\n", PathName);
    }

    if ((Success) != 0) {
        for i in 0..ARRAYSIZE!(KnownVendors) {
            Success = ((!(FindSubStrI(LocalVendorName.Buffer, KnownVendors[i as usize])).is_null())) as u8;
            if ((Success) != 0) {
                DPRINT!("IsValidNTOSInstallation: We've got an NTOS installation from %S !\n", KnownVendors[i as usize]);
                break;
            }
        }

        if !Machine.is_null() {
            *Machine = LocalMachine;
            Machine = core::ptr::null_mut();
        }

        if !VendorName.is_null() {
            RtlCopyUnicodeString(VendorName, &LocalVendorName);
            VendorName = core::ptr::null_mut();
        }
    }

    PathName = RUSTCONV_WSTR_5F7D2D6C0B3C.as_ptr();
    Success = CheckForValidPEAndVendor(SystemRootDirectory, PathName, &mut LocalMachine, &mut LocalVendorName);
    if ((Success) == 0) {
        DPRINT1!("User-mode DLL '%S' is either not a PE file, or does not have any vendor?\n", PathName);
    }

    if ((Success) != 0) {
        for i in 0..ARRAYSIZE!(KnownVendors) {
            if (!(FindSubStrI(LocalVendorName.Buffer, KnownVendors[i as usize])).is_null()) {
                DPRINT!("IsValidNTOSInstallation: The user-mode DLL '%S' is from %S\n", PathName, KnownVendors[i as usize]);
                break;
            }
        }

        if !Machine.is_null() {
            *Machine = LocalMachine;
            Machine = core::ptr::null_mut();
        }

        if !VendorName.is_null() {
            RtlCopyUnicodeString(VendorName, &LocalVendorName);
            VendorName = core::ptr::null_mut();
        }
    }

    return Success;
}

unsafe extern "C" fn IsValidNTOSInstallation(
    mut SystemRootPath: PUNICODE_STRING,
    mut Machine: PUSHORT,
    mut VendorName: PUNICODE_STRING,
) -> i32 {
    let mut Status: i32 = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut SystemRootDirectory: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Success: i32 = 0;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = SystemRootPath;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut SystemRootDirectory,
        FILE_LIST_DIRECTORY | FILE_TRAVERSE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"Failed to open SystemRoot '%wZ', Status 0x%08lx\n\0".as_ptr() as *const i8,
            SystemRootPath,
            Status,
        );
        return (FALSE) as i32;
    }

    Success = (IsValidNTOSInstallationByHandle(SystemRootDirectory, Machine, VendorName)) as _;

    NtClose(SystemRootDirectory);
    return Success;
}

unsafe extern "C" fn FindExistingNTOSInstall(
    mut List: PGENERIC_LIST,
    mut SystemRootArcPath: PCWSTR,
    mut SystemRootNtPath: PUNICODE_STRING,
) -> PNTOS_INSTALLATION {
    let mut Entry: PGENERIC_LIST_ENTRY = core::ptr::null_mut();
    let mut NtOsInstall: PNTOS_INSTALLATION = core::ptr::null_mut();
    let mut SystemArcPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    if SystemRootArcPath.is_null() && SystemRootNtPath.is_null() {
        return core::ptr::null_mut();
    }
    RtlInitUnicodeString(&mut SystemArcPath, SystemRootArcPath);
    Entry = GetFirstListEntry(List);
    while !Entry.is_null() {
        NtOsInstall = GetListEntryData(Entry) as PNTOS_INSTALLATION;
        if (!SystemRootArcPath.is_null()
            && RtlEqualUnicodeString(
                &mut (*NtOsInstall).SystemArcPath,
                &mut SystemArcPath,
                (TRUE) as BOOLEAN,
            ) != 0)
            || (!SystemRootNtPath.is_null()
                && RtlEqualUnicodeString(
                    &mut (*NtOsInstall).SystemNtPath,
                    SystemRootNtPath,
                    (TRUE) as BOOLEAN,
                ) != 0)
        {
            return NtOsInstall;
        }
        Entry = GetNextListEntry(Entry);
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn AddNTOSInstallation(
    mut List: PGENERIC_LIST,
    mut InstallationName: PCWSTR,
    mut Machine: USHORT,
    mut VendorName: PCWSTR,
    mut SystemRootArcPath: PCWSTR,
    mut SystemRootNtPath: PUNICODE_STRING,
    mut PathComponent: PCWSTR,
    mut DiskNumber: ULONG,
    mut PartitionNumber: ULONG,
) -> PNTOS_INSTALLATION {
    let mut NtOsInstall: PNTOS_INSTALLATION = core::ptr::null_mut();
    let mut ArcPathLength: usize = 0;
    let mut NtPathLength: usize = 0;

    NtOsInstall = FindExistingNTOSInstall(List, SystemRootArcPath, SystemRootNtPath);
    if !NtOsInstall.is_null() {
        DPRINT1!(
            b"An NTOS installation with name \"%S\" from vendor \"%S\" already exists on disk #%d, partition #%d, in SystemRoot '%wZ'\n\0"
                .as_ptr() as *const i8,
            (*NtOsInstall).InstallationName,
            (*NtOsInstall).VendorName,
            (*NtOsInstall).DiskNumber,
            (*NtOsInstall).PartitionNumber,
            &raw mut (*NtOsInstall).SystemNtPath,
        );
        return NtOsInstall;
    }

    ArcPathLength = (wcslen(SystemRootArcPath) + 1) * core::mem::size_of::<u16>();
    NtPathLength = (*SystemRootNtPath).Length as usize + core::mem::size_of_val(&UNICODE_NULL);

    NtOsInstall = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<NTOS_INSTALLATION>() + ArcPathLength + NtPathLength) as SIZE_T,
    ) as PNTOS_INSTALLATION;
    if NtOsInstall.is_null() {
        return core::ptr::null_mut();
    }

    (*NtOsInstall).DiskNumber = DiskNumber;
    (*NtOsInstall).PartitionNumber = PartitionNumber;
    (*NtOsInstall).Machine = Machine;

    RtlInitEmptyUnicodeString(
        &raw mut (*NtOsInstall).SystemArcPath,
        (NtOsInstall as *mut u8).add(1) as *mut u16,
        ArcPathLength as u16,
    );
    RtlCopyMemory!(
        (*NtOsInstall).SystemArcPath.Buffer as *mut core::ffi::c_void,
        SystemRootArcPath as *const core::ffi::c_void,
        ArcPathLength,
    );
    (*NtOsInstall).SystemArcPath.Length = (ArcPathLength - core::mem::size_of_val(&UNICODE_NULL)) as u16;

    RtlInitEmptyUnicodeString(
        &raw mut (*NtOsInstall).SystemNtPath,
        ((NtOsInstall as *mut u8).add(1) as usize + ArcPathLength) as *mut u16,
        NtPathLength as u16,
    );
    RtlCopyUnicodeString(&raw mut (*NtOsInstall).SystemNtPath, SystemRootNtPath);
    (*NtOsInstall).PathComponent = (*NtOsInstall).SystemNtPath.Buffer.offset(
        PathComponent.offset_from((*SystemRootNtPath).Buffer) as isize,
    );

    RtlStringCchCopyW(
        (*NtOsInstall).InstallationName.as_mut_ptr(),
        (ARRAYSIZE!((*NtOsInstall).InstallationName)) as _,
        InstallationName,
    );

    RtlStringCchCopyW(
        (*NtOsInstall).VendorName.as_mut_ptr(),
        (ARRAYSIZE!((*NtOsInstall).VendorName)) as _,
        VendorName,
    );

    AppendGenericListEntry(List, (NtOsInstall).cast(), (FALSE) as BOOLEAN);

    return NtOsInstall;
}

unsafe extern "C" fn FindNTOSInstallations(
    mut List: PGENERIC_LIST,
    mut PartList: PPARTLIST,
    mut Volume: PVOLENTRY,
) {
    let mut Status: NTSTATUS = 0;
    let mut VolumeRootDirHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut VolumeRootPath: _UNICODE_STRING = core::mem::zeroed();
    let mut Type: BOOT_STORE_TYPE = FreeLdr;
    let mut BootStoreHandle: PVOID = core::ptr::null_mut();
    let mut Data: ENUM_INSTALLS_DATA = core::mem::zeroed();
    let mut Version: ULONG = 0;
    let mut PathBuffer: [WCHAR; RTL_NUMBER_OF_FIELD!(VOLINFO, DeviceName) + 1] = [0; RTL_NUMBER_OF_FIELD!(VOLINFO, DeviceName) + 1];

    RtlStringCchPrintfW!(
        PathBuffer.as_mut_ptr(),
        _countof!(PathBuffer),
        RUSTCONV_WSTR_BCF3913447D5.as_ptr(),
        (*Volume).Info.DeviceName,
    );
    RtlInitUnicodeString(&mut VolumeRootPath, PathBuffer.as_mut_ptr());

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut VolumeRootPath,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenFile(
        &mut VolumeRootDirHandle,
        FILE_LIST_DIRECTORY | FILE_TRAVERSE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return;
    }

    Data.List = List;
    Data.PartList = PartList;

    Type = FreeLdr;
    while (Type as u32) < (BldrTypeMax as u32) {
        Status = FindBootStore(VolumeRootDirHandle, Type, &mut Version);
        if ((NT_SUCCESS!(Status)) == 0) {
            Type += 1;
            continue;
        }

        Status = OpenBootStoreByHandle(
            &mut BootStoreHandle,
            VolumeRootDirHandle,
            Type,
            BS_OpenExisting,
            BS_ReadAccess,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            Type += 1;
            continue;
        }
        EnumerateBootStoreEntries(BootStoreHandle, Some(EnumerateInstallations), &raw mut Data as PVOID);
        CloseBootStore(BootStoreHandle);
        Type += 1;
    }

    NtClose(VolumeRootDirHandle);
}

#[no_mangle]
pub unsafe extern "system" fn CreateNTOSInstallationsList(
    PartList: *mut PARTLIST,
) -> *mut GENERIC_LIST {
    let mut List: *mut GENERIC_LIST;
    let mut Entry: *mut LIST_ENTRY;
    let mut Volume: *mut VOLENTRY;
    let mut CheckVolume: BOOLEAN;

    List = CreateGenericList();
    if List.is_null() {
        return core::ptr::null_mut();
    }

    Entry = (*PartList).VolumesList.Flink;
    while Entry != core::ptr::addr_of_mut!((*PartList).VolumesList) {
        Volume = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);

        if !(*Volume).PartEntry.is_null() {
            CheckVolume = ((*Volume).New == 0 && (*Volume).FormatState == Formatted) as BOOLEAN;

            if CheckVolume != 0 {
                FindNTOSInstallations(List, PartList, Volume);
            }
        }

        Entry = (*Entry).Flink;
    }

    // C: `DumpNTOSInstalls(List)` under `#ifndef NDEBUG`, and osdetect.c defines NDEBUG.

    return List;
}
