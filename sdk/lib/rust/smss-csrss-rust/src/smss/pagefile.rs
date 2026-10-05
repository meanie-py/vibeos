use ntdll_sys::nt::*;

extern "C" {
    fn wcscpy(dest: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
}

// RtlGetProcessHeap is a PEB inline (TEB->Peb->ProcessHeap), not an export.
#[inline]
unsafe fn RtlGetProcessHeap() -> *mut core::ffi::c_void {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

// ---- file-scope preamble (translated) ----
pub const STANDARD_PAGING_FILE_NAME: &[u16] = &[
    0x005C, 0x003F, 0x003F, 0x005C, 0x003F, 0x003A, 0x005C, 0x0070, 0x0061, 0x0067, 0x0065, 0x0066,
    0x0069, 0x006C, 0x0065, 0x002E, 0x0073, 0x0079, 0x0073, 0x0000,
];

pub const STANDARD_DRIVE_LETTER_OFFSET: ULONG = 4;

pub const MEGABYTE: ULONGLONG = 1024 * 1024;

pub const GIGABYTE: ULONGLONG = 1024 * MEGABYTE;

pub const TERABYTE: ULONGLONG = 1024 * GIGABYTE;

pub const MAXIMUM_PAGEFILE_SIZE32: ULONGLONG = 4095 * MEGABYTE;

pub const MAXIMUM_PAGEFILE_SIZE64: ULONGLONG = 16 * TERABYTE;

pub const MINIMUM_TO_KEEP_FREE: ULONGLONG = 256 * MEGABYTE;

pub const FUZZ_FACTOR: ULONGLONG = 16 * MEGABYTE;

pub const MAX_PAGING_FILES: ULONG = 16;

// Arch-resolved maximum pagefile size (rustconv flattens the `#ifdef _M_IX86`
// chain; restore per-arch). i386 uses the 32-bit cap (PAE bumps to the 64-bit
// cap); amd64/arm64 use the 64-bit cap.
#[cfg(target_arch = "x86")]
pub const MAXIMUM_PAGEFILE_SIZE: ULONGLONG = MAXIMUM_PAGEFILE_SIZE32;
#[cfg(target_arch = "x86")]
pub const MAXIMUM_PAGEFILE_SIZE_PAE: ULONGLONG = MAXIMUM_PAGEFILE_SIZE64;
#[cfg(not(target_arch = "x86"))]
pub const MAXIMUM_PAGEFILE_SIZE: ULONGLONG = MAXIMUM_PAGEFILE_SIZE64;

// KUSER_SHARED_DATA at its fixed user-mode VA (mirrors smutil.rs).
#[allow(non_upper_case_globals)]
const SharedUserData: *mut _KUSER_SHARED_DATA = 0x7FFE0000 as *mut _KUSER_SHARED_DATA;

pub const SMP_PAGEFILE_CREATED: ULONG = 0x01;

pub const SMP_PAGEFILE_DEFAULT: ULONG = 0x02;

pub const SMP_PAGEFILE_SYSTEM_MANAGED: ULONG = 0x04;

pub const SMP_PAGEFILE_WAS_TOO_BIG: ULONG = 0x08;

pub const SMP_PAGEFILE_ON_ANY_DRIVE: ULONG = 0x10;

pub const SMP_PAGEFILE_EMERGENCY: ULONG = 0x20;

pub const SMP_PAGEFILE_DUMP_PROCESSED: ULONG = 0x40;

#[repr(C)]
pub struct _SMP_PAGEFILE_DESCRIPTOR {
    pub Entry: LIST_ENTRY,
    pub Name: _UNICODE_STRING,
    pub Token: _UNICODE_STRING,
    pub MinSize: LARGE_INTEGER,
    pub MaxSize: LARGE_INTEGER,
    pub ActualMinSize: LARGE_INTEGER,
    pub ActualMaxSize: LARGE_INTEGER,
    pub Flags: ULONG,
}
pub type PSMP_PAGEFILE_DESCRIPTOR = *mut _SMP_PAGEFILE_DESCRIPTOR;

pub const SMP_VOLUME_INSERTED: USHORT = 0x01;

pub const SMP_VOLUME_PAGEFILE_CREATED: USHORT = 0x04;

pub const SMP_VOLUME_IS_BOOT: USHORT = 0x08;

#[repr(C)]
pub struct _SMP_VOLUME_DESCRIPTOR {
    pub Entry: LIST_ENTRY,
    pub Flags: USHORT,
    pub PageFileCount: USHORT,
    pub DriveLetter: WCHAR,
    pub FreeSpace: LARGE_INTEGER,
    pub DeviceInfo: FILE_FS_DEVICE_INFORMATION,
}
pub type PSMP_VOLUME_DESCRIPTOR = *mut _SMP_VOLUME_DESCRIPTOR;

pub type SMP_PAGEFILE_DESCRIPTOR = _SMP_PAGEFILE_DESCRIPTOR;
pub type SMP_VOLUME_DESCRIPTOR = _SMP_VOLUME_DESCRIPTOR;

// NTSTATUS codes not present in the ntdll-sys bindgen (values from ntstatus.h).
const STATUS_TOO_MANY_PAGING_FILES: NTSTATUS = 0xC0000097u32 as NTSTATUS;
const STATUS_DISK_FULL: NTSTATUS = 0xC000007Fu32 as NTSTATUS;
const STATUS_UNEXPECTED_IO_ERROR: NTSTATUS = 0xC00000E9u32 as NTSTATUS;

#[inline]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// PC-98 detection global — always FALSE on our targets (C `extern BOOLEAN IsNEC_98`).
#[allow(non_upper_case_globals)]
const IsNEC_98: BOOLEAN = 0;

// ReactOS `widestr!()` builds a wide-char string literal. Inputs here are
// ASCII; convert at const-eval time to a NUL-terminated [u16; N].
macro_rules! widestr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const N: usize = BYTES.len() + 1;
        const ARR: [u16; N] = {
            let mut arr = [0u16; N];
            let mut i = 0;
            while i < BYTES.len() {
                arr[i] = BYTES[i] as u16;
                i += 1;
            }
            arr
        };
        ARR
    }};
}

pub static mut SmpPagingFileDescriptorList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut SmpVolumeDescriptorList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut SmpRegistrySpecifierPresent: BOOLEAN = 0;
pub static mut SmpNumberOfPagingFiles: ULONG = 0;

unsafe extern "system" fn SmpValidatePagingFileSizes(
    Descriptor: PSMP_PAGEFILE_DESCRIPTOR,
) -> NTSTATUS {
    let Status: NTSTATUS = STATUS_SUCCESS;
    let mut WasTooBig: BOOLEAN = 0u8;
    let mut MinSize: ULONGLONG;
    let mut MaxSize: ULONGLONG;

    #[cfg(target_arch = "x86")]
    let MaxPageFileSize: ULONGLONG =
        if (*SharedUserData).ProcessorFeatures[PF_PAE_ENABLED as usize] != 0 {
            MAXIMUM_PAGEFILE_SIZE_PAE
        } else {
            MAXIMUM_PAGEFILE_SIZE
        };
    #[cfg(not(target_arch = "x86"))]
    let MaxPageFileSize: ULONGLONG = MAXIMUM_PAGEFILE_SIZE;

    MinSize = (*Descriptor).MinSize.QuadPart as ULONGLONG;
    MaxSize = (*Descriptor).MaxSize.QuadPart as ULONGLONG;

    if MinSize > MaxSize {
        MaxSize = MinSize;
    }

    if MinSize > MaxPageFileSize {
        WasTooBig = 1u8;
        MinSize = MaxPageFileSize;
    }
    if MaxSize > MaxPageFileSize {
        WasTooBig = 1u8;
        MaxSize = MaxPageFileSize;
    }

    if WasTooBig != 0 {
        (*Descriptor).Flags |= SMP_PAGEFILE_WAS_TOO_BIG;
    }

    (*Descriptor).MinSize.QuadPart = MinSize as i64;
    (*Descriptor).MaxSize.QuadPart = MaxSize as i64;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpPagingFileInitialize() {
    InitializeListHead(core::ptr::addr_of_mut!(SmpPagingFileDescriptorList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpVolumeDescriptorList));
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreatePagingFileDescriptor(
    PageFileToken: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut MinSize: ULONG = 0;
    let mut MaxSize: ULONG = 0;
    let mut SystemManaged: BOOLEAN = 0u8;
    let mut ZeroSize: BOOLEAN = 1u8;
    let Descriptor: PSMP_PAGEFILE_DESCRIPTOR;
    let mut ListDescriptor: PSMP_PAGEFILE_DESCRIPTOR;
    let mut i: ULONG;
    let mut c: WCHAR;
    let mut NextEntry: PLIST_ENTRY;
    let mut PageFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut Arguments: _UNICODE_STRING = core::mem::zeroed();
    let mut SecondArgument: _UNICODE_STRING = core::mem::zeroed();

    /* Make sure we don't have too many */
    if SmpNumberOfPagingFiles >= MAX_PAGING_FILES {
        return STATUS_TOO_MANY_PAGING_FILES;
    }

    /* Parse the specified and get the name and arguments out of it */
    Status = SmpParseCommandLine(
        PageFileToken,
        core::ptr::null_mut(),
        &mut PageFileName,
        core::ptr::null_mut(),
        &mut Arguments,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Set the variable to let everyone know we have a pagefile token */
    SmpRegistrySpecifierPresent = 1u8;

    /* Parse the arguments, if any */
    if !Arguments.Buffer.is_null() {
        i = 0;
        while i < (Arguments.Length as ULONG) / (core::mem::size_of::<WCHAR>() as ULONG) {
            c = *Arguments.Buffer.add(i as usize);
            if c != b' ' as WCHAR && c != b'\t' as WCHAR && c != b'0' as WCHAR {
                ZeroSize = 0u8;
                break;
            }
            i += 1;
        }
    }

    /* Was a pagefile not specified, or was it specified with no size? */
    if Arguments.Buffer.is_null() || ZeroSize != 0 {
        SystemManaged = 1u8;
    } else {
        Status = RtlUnicodeStringToInteger(&Arguments, 0, &mut MinSize);
        if !NT_SUCCESS(Status) {
            RtlFreeUnicodeString(&mut PageFileName);
            RtlFreeUnicodeString(&mut Arguments);
            return Status;
        }

        i = 0;
        while i < (Arguments.Length as ULONG) / (core::mem::size_of::<WCHAR>() as ULONG) {
            if *Arguments.Buffer.add(i as usize) == b' ' as WCHAR {
                SecondArgument.Buffer = Arguments.Buffer.add(i as usize);
                SecondArgument.Length = (Arguments.Length as ULONG
                    - i * core::mem::size_of::<WCHAR>() as ULONG)
                    as USHORT;
                SecondArgument.MaximumLength = (Arguments.MaximumLength as ULONG
                    - i * core::mem::size_of::<WCHAR>() as ULONG)
                    as USHORT;
                Status = RtlUnicodeStringToInteger(&SecondArgument, 0, &mut MaxSize);
                if !NT_SUCCESS(Status) {
                    RtlFreeUnicodeString(&mut PageFileName);
                    RtlFreeUnicodeString(&mut Arguments);
                    return Status;
                }
                break;
            }
            i += 1;
        }
    }

    /* We are done parsing arguments */
    RtlFreeUnicodeString(&mut Arguments);

    /* Now we can allocate our descriptor */
    Descriptor = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SMP_PAGEFILE_DESCRIPTOR>() as _,
    ) as PSMP_PAGEFILE_DESCRIPTOR;
    if Descriptor.is_null() {
        RtlFreeUnicodeString(&mut PageFileName);
        return STATUS_NO_MEMORY;
    }

    /* Capture all our data into the descriptor */
    (*Descriptor).Token = *PageFileToken;
    (*Descriptor).Name = PageFileName;
    (*Descriptor).MinSize.QuadPart = (MinSize as i64) * (MEGABYTE as i64);
    (*Descriptor).MaxSize.QuadPart = (MaxSize as i64) * (MEGABYTE as i64);
    if SystemManaged != 0 {
        (*Descriptor).Flags |= SMP_PAGEFILE_SYSTEM_MANAGED;
    }
    *(*Descriptor)
        .Name
        .Buffer
        .add(STANDARD_DRIVE_LETTER_OFFSET as usize) = RtlUpcaseUnicodeChar(
        *(*Descriptor)
            .Name
            .Buffer
            .add(STANDARD_DRIVE_LETTER_OFFSET as usize),
    );
    if *(*Descriptor)
        .Name
        .Buffer
        .add(STANDARD_DRIVE_LETTER_OFFSET as usize)
        == b'?' as WCHAR
    {
        (*Descriptor).Flags |= SMP_PAGEFILE_ON_ANY_DRIVE;
    }

    /* Now loop the existing descriptors */
    NextEntry = SmpPagingFileDescriptorList.Flink;
    loop {
        if NextEntry == core::ptr::addr_of_mut!(SmpPagingFileDescriptorList) {
            InsertTailList(
                core::ptr::addr_of_mut!(SmpPagingFileDescriptorList),
                core::ptr::addr_of_mut!((*Descriptor).Entry),
            );
            SmpNumberOfPagingFiles += 1;
            return STATUS_SUCCESS;
        }

        ListDescriptor = (NextEntry as *mut u8)
            .sub(core::mem::offset_of!(SMP_PAGEFILE_DESCRIPTOR, Entry))
            as PSMP_PAGEFILE_DESCRIPTOR;
        NextEntry = (*NextEntry).Flink;

        if ((*ListDescriptor).Flags & SMP_PAGEFILE_ON_ANY_DRIVE) != 0
            && ((*Descriptor).Flags & SMP_PAGEFILE_ON_ANY_DRIVE) != 0
        {
            break;
        }
    }

    /* We found a duplicate, so skip this descriptor/pagefile and fail */
    RtlFreeUnicodeString(&mut PageFileName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, Descriptor as *mut _);
    STATUS_INVALID_PARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn SmpGetPagingFileSize(
    FileName: PUNICODE_STRING,
    Size: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut StandardInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();

    (*Size).QuadPart = 0;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        FILE_READ_ATTRIBUTES | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtQueryInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut StandardInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
        FileStandardInformation,
    );
    if !NT_SUCCESS(Status) {
        NtClose(FileHandle);
        return Status;
    }

    NtClose(FileHandle);
    (*Size).QuadPart = StandardInfo.AllocationSize.QuadPart;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpDeletePagingFile(FileName: PUNICODE_STRING) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Disposition: FILE_DISPOSITION_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = FileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        DELETE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_NON_DIRECTORY_FILE,
    );
    if NT_SUCCESS(Status) {
        Disposition.DeleteFileA = 1u8;
        Status = NtSetInformationFile(
            FileHandle,
            &mut IoStatusBlock,
            &mut Disposition as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as u32,
            FileDispositionInformation,
        );

        NtClose(FileHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpGetVolumeFreeSpace(Volume: PSMP_VOLUME_DESCRIPTOR) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut FreeSpace: LARGE_INTEGER = core::mem::zeroed();
    let mut FinalFreeSpace: LARGE_INTEGER = core::mem::zeroed();
    let mut SizeInfo: FILE_FS_SIZE_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut VolumeName: _UNICODE_STRING = core::mem::zeroed();
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();
    let mut PathString: [WCHAR; 32] = [0; 32];

    /* Build the standard path */
    let src: [WCHAR; 8] = [0x5C, 0x3F, 0x3F, 0x5C, 0x41, 0x3A, 0x5C, 0x00];
    wcscpy(PathString.as_mut_ptr(), src.as_ptr());
    RtlInitUnicodeString(&mut VolumeName, PathString.as_ptr());
    *VolumeName.Buffer.add(STANDARD_DRIVE_LETTER_OFFSET as usize) = (*Volume).DriveLetter;

    /* Open the volume */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut VolumeName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut VolumeHandle,
        FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Now get size information on the volume */
    Status = NtQueryVolumeInformationFile(
        VolumeHandle,
        &mut IoStatusBlock,
        &mut SizeInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as u32,
        FileFsSizeInformation,
    );
    if !NT_SUCCESS(Status) {
        NtClose(VolumeHandle);
        return Status;
    }
    NtClose(VolumeHandle);

    /* Compute how much free space we have */
    FreeSpace.QuadPart =
        SizeInfo.AvailableAllocationUnits.QuadPart * SizeInfo.SectorsPerAllocationUnit as i64;
    FinalFreeSpace.QuadPart = FreeSpace.QuadPart * SizeInfo.BytesPerSector as i64;

    /* Check if there is less than 32 MB free so we don't starve the disk */
    if FinalFreeSpace.QuadPart <= MINIMUM_TO_KEEP_FREE as i64 {
        /* In this case, act as if there is no free space */
        (*Volume).FreeSpace.QuadPart = 0;
    } else {
        /* Trim off 32 MB to give the disk a bit of breathing room */
        (*Volume).FreeSpace.QuadPart = FinalFreeSpace.QuadPart - MINIMUM_TO_KEEP_FREE as i64;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpSearchVolumeDescriptor(
    DriveLetter: WCHAR,
) -> PSMP_VOLUME_DESCRIPTOR {
    let UpLetter: WCHAR;
    let mut Volume: PSMP_VOLUME_DESCRIPTOR = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;

    UpLetter = RtlUpcaseUnicodeChar(DriveLetter);

    NextEntry = SmpVolumeDescriptorList.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpVolumeDescriptorList) {
        Volume = (NextEntry as usize - core::mem::offset_of!(_SMP_VOLUME_DESCRIPTOR, Entry))
            as PSMP_VOLUME_DESCRIPTOR;

        if (*Volume).DriveLetter == UpLetter {
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == core::ptr::addr_of_mut!(SmpVolumeDescriptorList) {
        Volume = core::ptr::null_mut();
    }
    Volume
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreatePagingFile(
    Name: PUNICODE_STRING,
    MinSize: PLARGE_INTEGER,
    MaxSize: PLARGE_INTEGER,
    Priority: ULONG,
) -> NTSTATUS {
    let Status = NtCreatePagingFile(Name, MinSize, MaxSize, Priority);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreatePagingFileOnFixedDrive(
    Descriptor: PSMP_PAGEFILE_DESCRIPTOR,
    FuzzFactor: PLARGE_INTEGER,
    MinimumSize: PLARGE_INTEGER,
) -> NTSTATUS {
    let Volume: PSMP_VOLUME_DESCRIPTOR;
    let mut ShouldDelete: BOOLEAN;
    let mut Status: NTSTATUS;
    let mut PageFileSize: LARGE_INTEGER = core::mem::zeroed();

    /* Try to find the volume descriptor for this drive letter */
    ShouldDelete = 0u8;
    Volume = SmpSearchVolumeDescriptor(
        *(*Descriptor)
            .Name
            .Buffer
            .offset(STANDARD_DRIVE_LETTER_OFFSET as isize),
    );
    if Volume.is_null() {
        /* Couldn't find it, fail */
        return STATUS_INVALID_PARAMETER;
    }

    /* Check if this is the boot volume */
    if ((*Volume).Flags & SMP_VOLUME_IS_BOOT) != 0 {
        /* Check if we haven't yet processed a crash dump on this volume */
        if ((*Descriptor).Flags & SMP_PAGEFILE_DUMP_PROCESSED) == 0 {
            /* Try to find a crash dump and extract it */
            SmpCheckForCrashDump(core::ptr::addr_of_mut!((*Descriptor).Name));

            /* Update how much free space we have now that we extracted a dump */
            let _Status = SmpGetVolumeFreeSpace(Volume); // nolint: let-shadow (Status overwritten before any read, as in the C)

            /* Don't process crashdump on this volume anymore */
            (*Descriptor).Flags |= SMP_PAGEFILE_DUMP_PROCESSED;
        }
    }

    /* Update the size after dump extraction */
    (*Descriptor).ActualMinSize = (*Descriptor).MinSize;
    (*Descriptor).ActualMaxSize = (*Descriptor).MaxSize;

    /* Check how big we can make the pagefile */
    Status = SmpGetPagingFileSize(
        core::ptr::addr_of_mut!((*Descriptor).Name),
        core::ptr::addr_of_mut!(PageFileSize),
    );
    if NT_SUCCESS(Status) && PageFileSize.QuadPart > 0 {
        ShouldDelete = 1u8;
    }

    /* Now update our size and make sure none of these are too big */
    PageFileSize.QuadPart += (*Volume).FreeSpace.QuadPart;
    if (*Descriptor).ActualMinSize.QuadPart > PageFileSize.QuadPart {
        (*Descriptor).ActualMinSize = PageFileSize;
    }
    if (*Descriptor).ActualMaxSize.QuadPart > PageFileSize.QuadPart {
        (*Descriptor).ActualMaxSize = PageFileSize;
    }

    /* Keep going until we've created a pagefile of the right size */
    while (*Descriptor).ActualMinSize.QuadPart >= (*MinimumSize).QuadPart {
        /* Call NT to do it */
        Status = SmpCreatePagingFile(
            core::ptr::addr_of_mut!((*Descriptor).Name),
            core::ptr::addr_of_mut!((*Descriptor).ActualMinSize),
            core::ptr::addr_of_mut!((*Descriptor).ActualMaxSize),
            0,
        );
        if NT_SUCCESS(Status) {
            /* We're done, update flags and increase the count */
            (*Descriptor).Flags |= SMP_PAGEFILE_CREATED;
            (*Volume).Flags |= SMP_VOLUME_PAGEFILE_CREATED as u16;
            (*Volume).PageFileCount += 1;
            break;
        }

        /* We failed, try a slightly smaller pagefile */
        (*Descriptor).ActualMinSize.QuadPart -= (*FuzzFactor).QuadPart;
    }

    /* Check if we weren't able to create it */
    if (*Descriptor).ActualMinSize.QuadPart < (*MinimumSize).QuadPart {
        /* Delete the current page file and fail */
        if ShouldDelete != 0 {
            SmpDeletePagingFile(core::ptr::addr_of_mut!((*Descriptor).Name));

            (*Volume).FreeSpace.QuadPart = PageFileSize.QuadPart;
        }
        Status = STATUS_DISK_FULL;
    }

    /* Return the status */
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreatePagingFileOnAnyDrive(
    Descriptor: PSMP_PAGEFILE_DESCRIPTOR,
    FuzzFactor: PLARGE_INTEGER,
    MinimumSize: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut Volume: PSMP_VOLUME_DESCRIPTOR;
    let mut Status: NTSTATUS = STATUS_DISK_FULL;
    let mut NextEntry: PLIST_ENTRY;

    /* Loop the volume list */
    NextEntry = SmpVolumeDescriptorList.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpVolumeDescriptorList) {
        /* Get the volume */
        Volume = (NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_SMP_VOLUME_DESCRIPTOR, Entry) as isize))
            as PSMP_VOLUME_DESCRIPTOR;

        /* Write the drive letter to try creating it on this volume */
        *(*Descriptor)
            .Name
            .Buffer
            .offset(STANDARD_DRIVE_LETTER_OFFSET as isize) = (*Volume).DriveLetter;
        Status = SmpCreatePagingFileOnFixedDrive(Descriptor, FuzzFactor, MinimumSize);
        if NT_SUCCESS(Status) {
            break;
        }

        /* It didn't work, make it an any pagefile again and keep going */
        *(*Descriptor)
            .Name
            .Buffer
            .offset(STANDARD_DRIVE_LETTER_OFFSET as isize) = b'?' as u16;
        NextEntry = (*NextEntry).Flink;
    }

    /* Return disk full or success */
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpMakeDefaultPagingFileDescriptor(
    Descriptor: PSMP_PAGEFILE_DESCRIPTOR,
) {
    (*Descriptor).Flags |= SMP_PAGEFILE_DEFAULT;
    (*Descriptor).MinSize.QuadPart = (128 * MEGABYTE) as i64;
    (*Descriptor).MaxSize.QuadPart = (128 * MEGABYTE) as i64;
}

#[no_mangle]
pub unsafe extern "system" fn SmpMakeSystemManagedPagingFileDescriptor(
    Descriptor: PSMP_PAGEFILE_DESCRIPTOR,
) {
    let mut BasicInfo: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();

    let Status = NtQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!(BasicInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        SmpMakeDefaultPagingFileDescriptor(Descriptor);
        return;
    }

    let Ram: u64 = BasicInfo.NumberOfPhysicalPages as u64 * BasicInfo.PageSize as u64;
    let MaximumSize: u64 = 3 * Ram;

    let MinimumSize: u64 = if Ram >= (1024 * MEGABYTE) as u64 {
        Ram
    } else {
        MaximumSize / 2
    };

    (*Descriptor).MinSize.QuadPart = MinimumSize as i64;
    (*Descriptor).MaxSize.QuadPart = MaximumSize as i64;
    (*Descriptor).Flags |= SMP_PAGEFILE_SYSTEM_MANAGED;
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreateSystemManagedPagingFile(
    Descriptor: PSMP_PAGEFILE_DESCRIPTOR,
    DecreaseSize: BOOLEAN,
) -> NTSTATUS {
    let mut FuzzFactor: LARGE_INTEGER = core::mem::zeroed();
    let mut Size: LARGE_INTEGER = core::mem::zeroed();

    /* Keep decreasing the pagefile by this amount if we run out of space */
    FuzzFactor.QuadPart = FUZZ_FACTOR as i64;

    /* Create the descriptor for it (mainly the right sizes) and validate */
    SmpMakeSystemManagedPagingFileDescriptor(Descriptor);
    SmpValidatePagingFileSizes(Descriptor);

    /* Use either the minimum size in the descriptor, or 16 MB in minimal mode */
    Size.QuadPart = if DecreaseSize != 0 {
        (16 * MEGABYTE) as i64
    } else {
        (*Descriptor).MinSize.QuadPart
    };

    /* Check if this should be a fixed pagefile or an any pagefile */
    if *(*Descriptor)
        .Name
        .Buffer
        .offset(STANDARD_DRIVE_LETTER_OFFSET as isize)
        == b'?' as u16
    {
        /* Find a disk for it */
        return SmpCreatePagingFileOnAnyDrive(
            Descriptor,
            core::ptr::addr_of_mut!(FuzzFactor),
            core::ptr::addr_of_mut!(Size),
        );
    }

    /* Use the disk that was given */
    SmpCreatePagingFileOnFixedDrive(
        Descriptor,
        core::ptr::addr_of_mut!(FuzzFactor),
        core::ptr::addr_of_mut!(Size),
    )
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreateEmergencyPagingFile() -> NTSTATUS {
    let mut Buffer: [WCHAR; 32] = core::mem::zeroed();

    /* Allocate a descriptor */
    let Descriptor = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SMP_PAGEFILE_DESCRIPTOR>() as _,
    ) as PSMP_PAGEFILE_DESCRIPTOR;
    if Descriptor.is_null() {
        return STATUS_NO_MEMORY;
    }

    /* Initialize it */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*Descriptor).Token),
        core::ptr::null_mut(),
    );

    /* Copy the default pagefile name */
    wcscpy(Buffer.as_mut_ptr(), STANDARD_PAGING_FILE_NAME.as_ptr());

    /* Fill the rest of the descriptor out */
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Descriptor).Name), Buffer.as_ptr());
    *(*Descriptor)
        .Name
        .Buffer
        .add(STANDARD_DRIVE_LETTER_OFFSET as usize) = b'?' as WCHAR;
    (*Descriptor).Flags |=
        SMP_PAGEFILE_SYSTEM_MANAGED | SMP_PAGEFILE_EMERGENCY | SMP_PAGEFILE_ON_ANY_DRIVE;

    /* Insert it into the descriptor list */
    InsertHeadList(
        core::ptr::addr_of_mut!(SmpPagingFileDescriptorList),
        core::ptr::addr_of_mut!((*Descriptor).Entry),
    );
    SmpNumberOfPagingFiles += 1;

    /* Go ahead and create it now, with the minimal size possible */
    SmpCreateSystemManagedPagingFile(Descriptor, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreateVolumeDescriptors() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut VolumePath: _UNICODE_STRING = core::mem::zeroed();
    let mut BootVolumeFound: BOOLEAN = 0u8;
    let StartChar: WCHAR;
    let mut Drive: WCHAR;
    let mut DriveDiff: WCHAR;
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_DEVICEMAP_INFORMATION = core::mem::zeroed();
    let mut DeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
    let mut SizeInfo: FILE_FS_SIZE_INFORMATION = core::mem::zeroed();
    let mut Volume: PSMP_VOLUME_DESCRIPTOR;
    let mut FreeSpace: LARGE_INTEGER = core::mem::zeroed();
    let mut FinalFreeSpace: LARGE_INTEGER = core::mem::zeroed();
    let mut Buffer: [WCHAR; 32] = [0; 32];

    /* Query the device map so we can get the drive letters */
    Status = NtQueryInformationProcess(
        -1isize as HANDLE,
        ProcessDeviceMap,
        core::ptr::addr_of_mut!(ProcessInformation.__bindgen_anon_1.Query) as *mut _,
        core::mem::size_of_val(&ProcessInformation.__bindgen_anon_1.Query) as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Build the volume string, starting with A: (we'll edit this in place) */
    wcscpy(Buffer.as_mut_ptr(), widestr!("\\??\\A:\\").as_ptr());
    RtlInitUnicodeString(&mut VolumePath, Buffer.as_ptr());

    /* Start with the C drive, except on NEC PC-98 */
    StartChar = if IsNEC_98 != 0 { 0x41u16 } else { 0x43u16 };
    Drive = StartChar;
    DriveDiff = StartChar - 0x41u16;
    while Drive <= 0x5Au16 {
        loop {
            /* Skip the disk if it's not in the drive map */
            if ((1u32 << DriveDiff) & ProcessInformation.__bindgen_anon_1.Query.DriveMap) == 0 {
                break;
            }

            /* Write the drive letter and try to open the volume */
            *VolumePath.Buffer.add(STANDARD_DRIVE_LETTER_OFFSET as usize) = Drive;

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.ObjectName = &mut VolumePath;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtOpenFile(
                &mut VolumeHandle,
                FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE,
            );
            if !NT_SUCCESS(Status) {
                break;
            }

            /* Now query device information on the volume */
            Status = NtQueryVolumeInformationFile(
                VolumeHandle,
                &mut IoStatusBlock,
                core::ptr::addr_of_mut!(DeviceInfo) as *mut _,
                core::mem::size_of_val(&DeviceInfo) as u32,
                FileFsDeviceInformation,
            );
            if !NT_SUCCESS(Status) {
                NtClose(VolumeHandle);
                break;
            }

            /* Check if this is a fixed disk */
            if (DeviceInfo.Characteristics
                & (FILE_FLOPPY_DISKETTE
                    | FILE_READ_ONLY_DEVICE
                    | FILE_REMOTE_DEVICE
                    | FILE_REMOVABLE_MEDIA))
                != 0
            {
                NtClose(VolumeHandle);
                break;
            }

            /* We found a fixed volume, allocate a descriptor for it */
            Volume = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<SMP_VOLUME_DESCRIPTOR>() as _,
            ) as PSMP_VOLUME_DESCRIPTOR;
            if Volume.is_null() {
                NtClose(VolumeHandle);
                break;
            }

            /* Save the drive letter and device information */
            (*Volume).DriveLetter = Drive;
            (*Volume).DeviceInfo = DeviceInfo;

            /* Check if this is the boot volume */
            if RtlUpcaseUnicodeChar(Drive)
                == RtlUpcaseUnicodeChar((*SharedUserData).NtSystemRoot[0])
            {
                (*Volume).Flags |= SMP_VOLUME_IS_BOOT as u16;
                BootVolumeFound = 1u8;
            }

            /* Now get size information on the volume */
            Status = NtQueryVolumeInformationFile(
                VolumeHandle,
                &mut IoStatusBlock,
                core::ptr::addr_of_mut!(SizeInfo) as *mut _,
                core::mem::size_of_val(&SizeInfo) as u32,
                FileFsSizeInformation,
            );
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(RtlGetProcessHeap(), 0, Volume as *mut _);
                NtClose(VolumeHandle);
                break;
            }

            /* Done querying volume information, close the handle */
            NtClose(VolumeHandle);

            /* Compute how much free space we have */
            FreeSpace.QuadPart = SizeInfo.AvailableAllocationUnits.QuadPart
                * SizeInfo.SectorsPerAllocationUnit as i64;
            FinalFreeSpace.QuadPart = FreeSpace.QuadPart * SizeInfo.BytesPerSector as i64;

            /* Check if there is less than 32 MB free so we don't starve the disk */
            if FinalFreeSpace.QuadPart <= MINIMUM_TO_KEEP_FREE as i64 {
                (*Volume).FreeSpace.QuadPart = 0;
            } else {
                (*Volume).FreeSpace.QuadPart =
                    FinalFreeSpace.QuadPart - MINIMUM_TO_KEEP_FREE as i64;
            }

            /* All done, add this volume to our descriptor list */
            InsertTailList(
                &raw mut SmpVolumeDescriptorList,
                core::ptr::addr_of_mut!((*Volume).Entry),
            );
            (*Volume).Flags |= SMP_VOLUME_INSERTED as u16;
            break;
        }

        Drive += 1;
        DriveDiff += 1;
    }

    let _ = BootVolumeFound;

    if !IsListEmpty(&raw mut SmpVolumeDescriptorList) {
        return STATUS_SUCCESS;
    }

    STATUS_UNEXPECTED_IO_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreatePagingFiles() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Descriptor: PSMP_PAGEFILE_DESCRIPTOR;
    let mut Size: LARGE_INTEGER = core::mem::zeroed();
    let mut FuzzFactor: LARGE_INTEGER = core::mem::zeroed();
    let mut Created: BOOLEAN = 0u8;
    let mut NextEntry: PLIST_ENTRY;

    /* Check if no paging files were requested */
    if SmpNumberOfPagingFiles == 0 && SmpRegistrySpecifierPresent == 0 {
        /* The list should be empty -- nothing to do */
        return STATUS_SUCCESS;
    }

    /* Initialize the volume descriptors so we can know what's available */
    Status = SmpCreateVolumeDescriptors();
    if !NT_SUCCESS(Status) {
        /* We can't make decisions without this, so fail */
        return Status;
    }

    /* If we fail creating pagefiles, try to reduce by this much each time */
    FuzzFactor.QuadPart = FUZZ_FACTOR as i64;

    /* Loop the descriptor list */
    NextEntry = SmpPagingFileDescriptorList.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpPagingFileDescriptorList) {
        /* Check what kind of descriptor this is */
        Descriptor = (NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_SMP_PAGEFILE_DESCRIPTOR, Entry) as isize))
            as PSMP_PAGEFILE_DESCRIPTOR;
        if (*Descriptor).Flags & SMP_PAGEFILE_SYSTEM_MANAGED != 0 {
            /* This is a system-managed descriptor. Create the correct file */
            Status = SmpCreateSystemManagedPagingFile(Descriptor, 0u8);
            if !NT_SUCCESS(Status) {
                /* We failed -- try again, with size minimization this time */
                Status = SmpCreateSystemManagedPagingFile(Descriptor, 1u8);
            }
        } else {
            /* This is a manually entered descriptor. Validate its size first */
            SmpValidatePagingFileSizes(Descriptor);

            /* Check if this is an ANY pagefile or a FIXED pagefile */
            if *(*Descriptor)
                .Name
                .Buffer
                .offset(STANDARD_DRIVE_LETTER_OFFSET as isize)
                == b'?' as WCHAR
            {
                /* It's an any pagefile, try to create it wherever possible */
                Size = (*Descriptor).MinSize;
                Status = SmpCreatePagingFileOnAnyDrive(
                    Descriptor,
                    core::ptr::addr_of_mut!(FuzzFactor),
                    core::ptr::addr_of_mut!(Size),
                );
                if !NT_SUCCESS(Status) {
                    /* We failed to create it. Try again with a smaller size */
                    Size.QuadPart = (16 * MEGABYTE) as i64;
                    Status = SmpCreatePagingFileOnAnyDrive(
                        Descriptor,
                        core::ptr::addr_of_mut!(FuzzFactor),
                        core::ptr::addr_of_mut!(Size),
                    );
                }
            } else {
                /* It's a fixed pagefile: override the minimum and use ours */
                Size.QuadPart = (16 * MEGABYTE) as i64;
                Status = SmpCreatePagingFileOnFixedDrive(
                    Descriptor,
                    core::ptr::addr_of_mut!(FuzzFactor),
                    core::ptr::addr_of_mut!(Size),
                );
            }
        }

        /* Go to the next descriptor */
        if NT_SUCCESS(Status) {
            Created = 1u8;
        }
        NextEntry = (*NextEntry).Flink;
    }

    /* Check if none of the code in our loops above was able to create it */
    if Created == 0 {
        /* Build an emergency pagefile ourselves */
        Status = SmpCreateEmergencyPagingFile();
    }

    /* All done */
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/pagefile.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// //
// // Constants
// //
// #define STANDARD_PAGING_FILE_NAME       L"\\??\\?:\\pagefile.sys"
// #define STANDARD_DRIVE_LETTER_OFFSET    4
// #define MAX_PAGING_FILES                16  // See also ntoskrnl/include/internal/mm.h
// #define MEGABYTE                        (1024 * 1024)
//
// /* Minimum pagefile size is 256 pages (1 MB) */
// // #define MINIMUM_PAGEFILE_SIZE           (256ULL * PAGE_SIZE)
//
// /* Maximum pagefile sizes for different architectures */
// #define GIGABYTE                        (1024ULL * MEGABYTE)
// #define TERABYTE                        (1024ULL * GIGABYTE)
//
// // NOTE: No changes for NTDDI_WIN10
// #if (NTDDI_VERSION >= NTDDI_WINBLUE) // NTDDI_WIN81
// #define MAXIMUM_PAGEFILE_SIZE32         ((1ULL * 1024 * 1024 - 1) * PAGE_SIZE)
//                                      // PAGE_ROUND_DOWN(4ULL * GIGABYTE - 1)
// #else
// /* 4095 MB */
// #define MAXIMUM_PAGEFILE_SIZE32         (4095ULL * MEGABYTE)
// #endif
//
// // NOTE: No changes for NTDDI_WIN10
// #if (NTDDI_VERSION >= NTDDI_WINBLUE) // NTDDI_WIN81
// #define MAXIMUM_PAGEFILE_SIZE64         ((4ULL * 1024 * 1024 * 1024 - 1) * PAGE_SIZE)
//                                      // PAGE_ROUND_DOWN(16ULL * TERABYTE - 1)
// #else
// /* 16 TB */
// #define MAXIMUM_PAGEFILE_SIZE64         (16ULL * TERABYTE)
// #endif
//
// #if defined(_M_IX86)
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE32
//     /* PAE uses the same size as x64 */
//     #define MAXIMUM_PAGEFILE_SIZE_PAE   MAXIMUM_PAGEFILE_SIZE64
// #elif defined (_M_AMD64) || defined(_M_ARM64)
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE64
// #elif defined (_M_IA64)
// /* 32 TB */
//     #define MAXIMUM_PAGEFILE_SIZE       (32ULL * TERABYTE)
// #elif defined(_M_ARM)
// /* Around 2 GB */
//     // NOTE: No changes for NTDDI_WIN10
//     #if (NTDDI_VERSION >= NTDDI_WINBLUE) // NTDDI_WIN81
//     #define MAXIMUM_PAGEFILE_SIZE       ((512ULL * 1024 - 1) * PAGE_SIZE)
//                                      // PAGE_ROUND_DOWN(2ULL * GIGABYTE - 1)
//     #else
// /* 4095 MB */
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE32
//     #endif
// #else
// /* On unknown architectures, default to either one of the 32 or 64 bit sizes */
// #pragma message("Unknown architecture")
//     #ifdef _WIN64
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE64
//     #else
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE32
//     #endif
// #endif
//
// /* This should be 32 MB, but we need more than that for 2nd stage setup */
// #define MINIMUM_TO_KEEP_FREE            (256 * MEGABYTE)
// #define FUZZ_FACTOR                     (16 * MEGABYTE)
//
// //
// // Structure and flags describing each pagefile
// //
// #define SMP_PAGEFILE_CREATED            0x01
// #define SMP_PAGEFILE_DEFAULT            0x02
// #define SMP_PAGEFILE_SYSTEM_MANAGED     0x04
// #define SMP_PAGEFILE_WAS_TOO_BIG        0x08
// #define SMP_PAGEFILE_ON_ANY_DRIVE       0x10
// #define SMP_PAGEFILE_EMERGENCY          0x20
// #define SMP_PAGEFILE_DUMP_PROCESSED     0x40
// typedef struct _SMP_PAGEFILE_DESCRIPTOR
// {
//     LIST_ENTRY Entry;
//     UNICODE_STRING Name;
//     UNICODE_STRING Token;
//     LARGE_INTEGER MinSize;
//     LARGE_INTEGER MaxSize;
//     LARGE_INTEGER ActualMinSize;
//     LARGE_INTEGER ActualMaxSize;
//     ULONG Flags;
// } SMP_PAGEFILE_DESCRIPTOR, *PSMP_PAGEFILE_DESCRIPTOR;
//
// //
// // Structure and flags describing each volume
// //
// #define SMP_VOLUME_INSERTED             0x01
// #define SMP_VOLUME_PAGEFILE_CREATED     0x04
// #define SMP_VOLUME_IS_BOOT              0x08
// typedef struct _SMP_VOLUME_DESCRIPTOR
// {
//     LIST_ENTRY Entry;
//     USHORT Flags;
//     USHORT PageFileCount;
//     WCHAR DriveLetter;
//     LARGE_INTEGER FreeSpace;
//     FILE_FS_DEVICE_INFORMATION DeviceInfo;
// } SMP_VOLUME_DESCRIPTOR, *PSMP_VOLUME_DESCRIPTOR;
//
// LIST_ENTRY SmpPagingFileDescriptorList, SmpVolumeDescriptorList;
// BOOLEAN SmpRegistrySpecifierPresent;
// ULONG SmpNumberOfPagingFiles;
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// NTAPI
// SmpPagingFileInitialize(VOID)
// {
//     /* Initialize the two lists */
//     InitializeListHead(&SmpPagingFileDescriptorList);
//     InitializeListHead(&SmpVolumeDescriptorList);
// }
//
// NTSTATUS
// NTAPI
// SmpCreatePagingFileDescriptor(IN PUNICODE_STRING PageFileToken)
// {
//     NTSTATUS Status;
//     ULONG MinSize = 0, MaxSize = 0;
//     BOOLEAN SystemManaged = FALSE, ZeroSize = TRUE;
//     PSMP_PAGEFILE_DESCRIPTOR Descriptor, ListDescriptor;
//     ULONG i;
//     WCHAR c;
//     PLIST_ENTRY NextEntry;
//     UNICODE_STRING PageFileName, Arguments, SecondArgument;
//
//     /* Make sure we don't have too many */
//     if (SmpNumberOfPagingFiles >= MAX_PAGING_FILES)
//     {
//         DPRINT1("SMSS:PFILE: Too many paging files specified - %lu\n",
//                 SmpNumberOfPagingFiles);
//         return STATUS_TOO_MANY_PAGING_FILES;
//     }
//
//     /* Parse the specified and get the name and arguments out of it */
//     DPRINT("SMSS:PFILE: Paging file specifier `%wZ'\n", PageFileToken);
//     Status = SmpParseCommandLine(PageFileToken,
//                                  NULL,
//                                  &PageFileName,
//                                  NULL,
//                                  &Arguments);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("SMSS:PFILE: SmpParseCommandLine(%wZ) failed - Status == %lx\n",
//                 PageFileToken, Status);
//         return Status;
//     }
//
//     /* Set the variable to let everyone know we have a pagefile token */
//     SmpRegistrySpecifierPresent = TRUE;
//
//     /* Parse the arguments, if any */
//     if (Arguments.Buffer)
//     {
//         /* Parse the pagefile size */
//         for (i = 0; i < Arguments.Length / sizeof(WCHAR); i++)
//         {
//             /* Check if it's zero */
//             c = Arguments.Buffer[i];
//             if ((c != L' ') && (c != L'\t') && (c != L'0'))
//             {
//                 /* It isn't, break out */
//                 ZeroSize = FALSE;
//                 break;
//             }
//         }
//     }
//
//     /* Was a pagefile not specified, or was it specified with no size? */
//     if (!(Arguments.Buffer) || (ZeroSize))
//     {
//         /* In this case, the system will manage its size */
//         SystemManaged = TRUE;
//     }
//     else
//     {
//         /* We do have a size, so convert the arguments into a number */
//         Status = RtlUnicodeStringToInteger(&Arguments, 0, &MinSize);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             RtlFreeUnicodeString(&PageFileName);
//             RtlFreeUnicodeString(&Arguments);
//             return Status;
//         }
//
//         /* Now advance to the next argument */
//         for (i = 0; i < Arguments.Length / sizeof(WCHAR); i++)
//         {
//             /* Found a space -- second argument must start here */
//             if (Arguments.Buffer[i] == L' ')
//             {
//                 /* Use the rest of the arguments as a maximum size */
//                 SecondArgument.Buffer = &Arguments.Buffer[i];
//                 SecondArgument.Length = (USHORT)(Arguments.Length -
//                                         i * sizeof(WCHAR));
//                 SecondArgument.MaximumLength = (USHORT)(Arguments.MaximumLength -
//                                                i * sizeof(WCHAR));
//                 Status = RtlUnicodeStringToInteger(&SecondArgument, 0, &MaxSize);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Fail */
//                     RtlFreeUnicodeString(&PageFileName);
//                     RtlFreeUnicodeString(&Arguments);
//                     return Status;
//                 }
//
//                 break;
//             }
//         }
//     }
//
//     /* We are done parsing arguments */
//     RtlFreeUnicodeString(&Arguments);
//
//     /* Now we can allocate our descriptor */
//     Descriptor = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  sizeof(SMP_PAGEFILE_DESCRIPTOR));
//     if (!Descriptor)
//     {
//         /* Fail if we couldn't */
//         RtlFreeUnicodeString(&PageFileName);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Capture all our data into the descriptor */
//     Descriptor->Token = *PageFileToken;
//     Descriptor->Name = PageFileName;
//     Descriptor->MinSize.QuadPart = MinSize * MEGABYTE;
//     Descriptor->MaxSize.QuadPart = MaxSize * MEGABYTE;
//     if (SystemManaged)
//         Descriptor->Flags |= SMP_PAGEFILE_SYSTEM_MANAGED;
//     Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] =
//     RtlUpcaseUnicodeChar(Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET]);
//     if (Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] == '?')
//     {
//         Descriptor->Flags |= SMP_PAGEFILE_ON_ANY_DRIVE;
//     }
//
//     /* Now loop the existing descriptors */
//     NextEntry = SmpPagingFileDescriptorList.Flink;
//     do
//     {
//         /* Are there none, or have we looped back to the beginning? */
//         if (NextEntry == &SmpPagingFileDescriptorList)
//         {
//             /* This means no duplicates exist, so insert our descriptor! */
//             InsertTailList(&SmpPagingFileDescriptorList, &Descriptor->Entry);
//             SmpNumberOfPagingFiles++;
//             DPRINT("SMSS:PFILE: Created descriptor for `%wZ' (`%wZ')\n",
//                     PageFileToken, &Descriptor->Name);
//             return STATUS_SUCCESS;
//         }
//
//         /* Keep going until we find a duplicate, unless we are in "any" mode */
//         ListDescriptor = CONTAINING_RECORD(NextEntry, SMP_PAGEFILE_DESCRIPTOR, Entry);
//         NextEntry = NextEntry->Flink;
//     } while (!(ListDescriptor->Flags & SMP_PAGEFILE_ON_ANY_DRIVE) ||
//              !(Descriptor->Flags & SMP_PAGEFILE_ON_ANY_DRIVE));
//
//     /* We found a duplicate, so skip this descriptor/pagefile and fail */
//     DPRINT1("SMSS:PFILE: Skipping duplicate specifier `%wZ'\n", PageFileToken);
//     RtlFreeUnicodeString(&PageFileName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Descriptor);
//     return STATUS_INVALID_PARAMETER;
// }
//
// NTSTATUS
// NTAPI
// SmpGetPagingFileSize(IN PUNICODE_STRING FileName,
//                      OUT PLARGE_INTEGER Size)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE FileHandle;
//     FILE_STANDARD_INFORMATION StandardInfo;
//
//     DPRINT("SMSS:PFILE: Trying to get size for `%wZ'\n", FileName);
//     Size->QuadPart = 0;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenFile(&FileHandle,
//                         FILE_READ_ATTRIBUTES | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     Status = NtQueryInformationFile(FileHandle,
//                                     &IoStatusBlock,
//                                     &StandardInfo,
//                                     sizeof(StandardInfo),
//                                     FileStandardInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS:PFILE: Failed query for size potential pagefile `%wZ' with status %X\n",
//                 FileName, Status);
//         NtClose(FileHandle);
//         return Status;
//     }
//
//     NtClose(FileHandle);
//     Size->QuadPart = StandardInfo.AllocationSize.QuadPart;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpDeletePagingFile(IN PUNICODE_STRING FileName)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE FileHandle;
//     FILE_DISPOSITION_INFORMATION Disposition;
//
//     /* Open the page file */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenFile(&FileHandle,
//                         DELETE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_NON_DIRECTORY_FILE);
//     if (NT_SUCCESS(Status))
//     {
//         /* Delete it */
//         Disposition.DeleteFile = TRUE;
//         Status = NtSetInformationFile(FileHandle,
//                                       &IoStatusBlock,
//                                       &Disposition,
//                                       sizeof(Disposition),
//                                       FileDispositionInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("SMSS:PFILE: Failed to delete page file `%wZ' (status %X)\n",
//                     FileName, Status);
//         }
//         else
//         {
//             DPRINT("SMSS:PFILE: Deleted stale paging file - %wZ\n", FileName);
//         }
//
//         /* Close the handle */
//         NtClose(FileHandle);
//     }
//     else
//     {
//         DPRINT1("SMSS:PFILE: Failed to open for deletion page file `%wZ' (status %X)\n",
//                 FileName, Status);
//     }
//
//     /* All done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpGetVolumeFreeSpace(IN PSMP_VOLUME_DESCRIPTOR Volume)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER FreeSpace, FinalFreeSpace;
//     FILE_FS_SIZE_INFORMATION SizeInfo;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING VolumeName;
//     HANDLE VolumeHandle;
//     WCHAR PathString[32];
//     ASSERT(Volume->Flags & SMP_VOLUME_IS_BOOT); // ASSERT says "BootVolume == 1"
//
//     /* Build the standard path */
//     wcscpy(PathString, L"\\??\\A:\\");
//     RtlInitUnicodeString(&VolumeName, PathString);
//     VolumeName.Buffer[STANDARD_DRIVE_LETTER_OFFSET] = Volume->DriveLetter;
//     DPRINT("SMSS:PFILE: Querying volume `%wZ' for free space\n", &VolumeName);
//
//     /* Open the volume */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &VolumeName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenFile(&VolumeHandle,
//                         FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS:PFILE: Open volume `%wZ' failed with status %X\n", &VolumeName, Status);
//         return Status;
//     }
//
//     /* Now get size information on the volume */
//     Status = NtQueryVolumeInformationFile(VolumeHandle,
//                                           &IoStatusBlock,
//                                           &SizeInfo,
//                                           sizeof(SizeInfo),
//                                           FileFsSizeInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         DPRINT1("SMSS:PFILE: Query volume `%wZ' (handle %p) for size failed"
//                 " with status %X\n",
//                 &VolumeName,
//                 VolumeHandle,
//                 Status);
//         NtClose(VolumeHandle);
//         return Status;
//     }
//     NtClose(VolumeHandle);
//
//     /* Compute how much free space we have */
//     FreeSpace.QuadPart = SizeInfo.AvailableAllocationUnits.QuadPart *
//                          SizeInfo.SectorsPerAllocationUnit;
//     FinalFreeSpace.QuadPart = FreeSpace.QuadPart * SizeInfo.BytesPerSector;
//
//     /* Check if there is less than 32 MB free so we don't starve the disk */
//     if (FinalFreeSpace.QuadPart <= MINIMUM_TO_KEEP_FREE)
//     {
//         /* In this case, act as if there is no free space */
//         Volume->FreeSpace.QuadPart = 0;
//     }
//     else
//     {
//         /* Trim off 32 MB to give the disk a bit of breathing room */
//         Volume->FreeSpace.QuadPart = FinalFreeSpace.QuadPart -
//                                      MINIMUM_TO_KEEP_FREE;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// PSMP_VOLUME_DESCRIPTOR
// NTAPI
// SmpSearchVolumeDescriptor(IN WCHAR DriveLetter)
// {
//     WCHAR UpLetter;
//     PSMP_VOLUME_DESCRIPTOR Volume = NULL;
//     PLIST_ENTRY NextEntry;
//
//     /* Use upper case to reduce differences */
//     UpLetter = RtlUpcaseUnicodeChar(DriveLetter);
//
//     /* Loop each volume */
//     NextEntry = SmpVolumeDescriptorList.Flink;
//     while (NextEntry != &SmpVolumeDescriptorList)
//     {
//         /* Grab the entry */
//         Volume = CONTAINING_RECORD(NextEntry, SMP_VOLUME_DESCRIPTOR, Entry);
//
//         /* Make sure it's a valid entry with an uppcase drive letter */
//         ASSERT(Volume->Flags & SMP_VOLUME_INSERTED); // Volume->Initialized in ASSERT
//         ASSERT(Volume->DriveLetter >= L'A' && Volume->DriveLetter <= L'Z');
//
//         /* Break if it matches, if not, keep going */
//         if (Volume->DriveLetter == UpLetter) break;
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Return the volume if one was found */
//     if (NextEntry == &SmpVolumeDescriptorList) Volume = NULL;
//     return Volume;
// }
//
// NTSTATUS
// NTAPI
// SmpCreatePagingFile(IN PUNICODE_STRING Name,
//                     IN PLARGE_INTEGER MinSize,
//                     IN PLARGE_INTEGER MaxSize,
//                     IN ULONG Priority)
// {
//     NTSTATUS Status;
//
//     /* Tell the kernel to create the pagefile */
//     Status = NtCreatePagingFile(Name, MinSize, MaxSize, Priority);
//     if (NT_SUCCESS(Status))
//     {
//         DPRINT("SMSS:PFILE: NtCreatePagingFile(%wZ, 0x%I64X, 0x%I64X) succeeded\n",
//                 Name,
//                 MinSize->QuadPart,
//                 MaxSize->QuadPart);
//     }
//     else
//     {
//         DPRINT1("SMSS:PFILE: NtCreatePagingFile(%wZ, 0x%I64X, 0x%I64X) failed with %X\n",
//                 Name,
//                 MinSize->QuadPart,
//                 MaxSize->QuadPart,
//                 Status);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpCreatePagingFileOnFixedDrive(IN PSMP_PAGEFILE_DESCRIPTOR Descriptor,
//                                 IN PLARGE_INTEGER FuzzFactor,
//                                 IN PLARGE_INTEGER MinimumSize)
// {
//     PSMP_VOLUME_DESCRIPTOR Volume;
//     BOOLEAN ShouldDelete;
//     NTSTATUS Status;
//     LARGE_INTEGER PageFileSize;
//     ASSERT(Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] != L'?');
//
//     /* Try to find the volume descriptor for this drive letter */
//     ShouldDelete = FALSE;
//     Volume = SmpSearchVolumeDescriptor(Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET]);
//     if (!Volume)
//     {
//         /* Couldn't find it, fail */
//         DPRINT1("SMSS:PFILE: No volume descriptor for `%wZ'\n",
//                 &Descriptor->Name);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if this is the boot volume */
//     if (Volume->Flags & SMP_VOLUME_IS_BOOT)
//     {
//         /* Check if we haven't yet processed a crash dump on this volume */
//         if (!(Descriptor->Flags & SMP_PAGEFILE_DUMP_PROCESSED))
//         {
//             /* Try to find a crash dump and extract it */
//             DPRINT("SMSS:PFILE: Checking for crash dump in `%wZ' on boot volume\n",
//                     &Descriptor->Name);
//             SmpCheckForCrashDump(&Descriptor->Name);
//
//             /* Update how much free space we have now that we extracted a dump */
//             Status = SmpGetVolumeFreeSpace(Volume);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("SMSS:PFILE: Failed to query free space for boot volume `%wC'\n",
//                         Volume->DriveLetter);
//             }
//             else
//             {
//                 DPRINT("Queried free space for boot volume `%wC: 0x%I64x'\n",
//                         Volume->DriveLetter, Volume->FreeSpace.QuadPart);
//             }
//
//             /* Don't process crashdump on this volume anymore */
//             Descriptor->Flags |= SMP_PAGEFILE_DUMP_PROCESSED;
//         }
//     }
//     else
//     {
//         /* Crashdumps can only be on the boot volume */
//         DPRINT("SMSS:PFILE: Skipping crash dump checking for `%wZ' on non boot"
//                 "volume `%wC'\n",
//                 &Descriptor->Name,
//                 Volume->DriveLetter);
//     }
//
//     /* Update the size after dump extraction */
//     Descriptor->ActualMinSize = Descriptor->MinSize;
//     Descriptor->ActualMaxSize = Descriptor->MaxSize;
//
//     /* Check how big we can make the pagefile */
//     Status = SmpGetPagingFileSize(&Descriptor->Name, &PageFileSize);
//     if (NT_SUCCESS(Status) && PageFileSize.QuadPart > 0) ShouldDelete = TRUE;
//     DPRINT("SMSS:PFILE: Detected size 0x%I64X for future paging file `%wZ'\n",
//             PageFileSize,
//             &Descriptor->Name);
//     DPRINT("SMSS:PFILE: Free space on volume `%wC' is 0x%I64X\n",
//             Volume->DriveLetter,
//             Volume->FreeSpace.QuadPart);
//
//     /* Now update our size and make sure none of these are too big */
//     PageFileSize.QuadPart += Volume->FreeSpace.QuadPart;
//     if (Descriptor->ActualMinSize.QuadPart > PageFileSize.QuadPart)
//     {
//         Descriptor->ActualMinSize = PageFileSize;
//     }
//     if (Descriptor->ActualMaxSize.QuadPart > PageFileSize.QuadPart)
//     {
//         Descriptor->ActualMaxSize = PageFileSize;
//     }
//     DPRINT("SMSS:PFILE: min 0x%I64X, max 0x%I64X, real min 0x%I64X\n",
//             Descriptor->ActualMinSize.QuadPart,
//             Descriptor->ActualMaxSize.QuadPart,
//             MinimumSize->QuadPart);
//
//     /* Keep going until we've created a pagefile of the right size */
//     while (Descriptor->ActualMinSize.QuadPart >= MinimumSize->QuadPart)
//     {
//         /* Call NT to do it */
//         Status = SmpCreatePagingFile(&Descriptor->Name,
//                                      &Descriptor->ActualMinSize,
//                                      &Descriptor->ActualMaxSize,
//                                      0);
//         if (NT_SUCCESS(Status))
//         {
//             /* We're done, update flags and increase the count */
//             Descriptor->Flags |= SMP_PAGEFILE_CREATED;
//             Volume->Flags |= SMP_VOLUME_PAGEFILE_CREATED;
//             Volume->PageFileCount++;
//             break;
//         }
//
//         /* We failed, try a slightly smaller pagefile */
//         Descriptor->ActualMinSize.QuadPart -= FuzzFactor->QuadPart;
//     }
//
//     /* Check if we weren't able to create it */
//     if (Descriptor->ActualMinSize.QuadPart < MinimumSize->QuadPart)
//     {
//         /* Delete the current page file and fail */
//         if (ShouldDelete)
//         {
//             SmpDeletePagingFile(&Descriptor->Name);
//
//             /* FIXFIX: Windows Vista does this, and it seems like we should too, so try to see if this fixes KVM */
//             Volume->FreeSpace.QuadPart = PageFileSize.QuadPart;
//         }
//         DPRINT1("SMSS:PFILE: Failing for min 0x%I64X, max 0x%I64X, real min 0x%I64X\n",
//                 Descriptor->ActualMinSize.QuadPart,
//                 Descriptor->ActualMaxSize.QuadPart,
//                 MinimumSize->QuadPart);
//         Status = STATUS_DISK_FULL;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpCreatePagingFileOnAnyDrive(IN PSMP_PAGEFILE_DESCRIPTOR Descriptor,
//                               IN PLARGE_INTEGER FuzzFactor,
//                               IN PLARGE_INTEGER MinimumSize)
// {
//     PSMP_VOLUME_DESCRIPTOR Volume;
//     NTSTATUS Status = STATUS_DISK_FULL;
//     PLIST_ENTRY NextEntry;
//     ASSERT(Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] == L'?');
//
//     /* Loop the volume list */
//     NextEntry = SmpVolumeDescriptorList.Flink;
//     while (NextEntry != &SmpVolumeDescriptorList)
//     {
//         /* Get the volume */
//         Volume = CONTAINING_RECORD(NextEntry, SMP_VOLUME_DESCRIPTOR, Entry);
//
//         /* Make sure it's inserted and on a valid drive letter */
//         ASSERT(Volume->Flags & SMP_VOLUME_INSERTED); // Volume->Initialized in ASSERT
//         ASSERT(Volume->DriveLetter >= L'A' && Volume->DriveLetter <= L'Z');
//
//         /* Write the drive letter to try creating it on this volume */
//         Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] = Volume->DriveLetter;
//         Status = SmpCreatePagingFileOnFixedDrive(Descriptor,
//                                                  FuzzFactor,
//                                                  MinimumSize);
//         if (NT_SUCCESS(Status)) break;
//
//         /* It didn't work, make it an any pagefile again and keep going */
//         Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] = L'?';
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Return disk full or success */
//     return Status;
// }
//
// VOID
// NTAPI
// SmpMakeDefaultPagingFileDescriptor(IN PSMP_PAGEFILE_DESCRIPTOR Descriptor)
// {
//     /* The default descriptor uses 128 MB as a pagefile size */
//     Descriptor->Flags |= SMP_PAGEFILE_DEFAULT;
//     Descriptor->MinSize.QuadPart = 128 * MEGABYTE;
//     Descriptor->MaxSize.QuadPart = 128 * MEGABYTE;
// }
//
// VOID
// NTAPI
// SmpMakeSystemManagedPagingFileDescriptor(IN PSMP_PAGEFILE_DESCRIPTOR Descriptor)
// {
//     NTSTATUS Status;
//     ULONGLONG MinimumSize, MaximumSize, Ram;
//     SYSTEM_BASIC_INFORMATION BasicInfo;
//
//     /* Query the page size of the system, and the amount of RAM */
//     Status = NtQuerySystemInformation(SystemBasicInformation,
//                                       &BasicInfo,
//                                       sizeof(BasicInfo),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* If we failed, use defaults since we have no idea otherwise */
//         DPRINT1("SMSS:PFILE: NtQuerySystemInformation failed with %x\n", Status);
//         SmpMakeDefaultPagingFileDescriptor(Descriptor);
//         return;
//     }
//
//     /* Check how much RAM we have and set three times this amount as maximum */
//     Ram = BasicInfo.NumberOfPhysicalPages * BasicInfo.PageSize;
//     MaximumSize = 3 * Ram;
//
//     /* If we have more than 1GB, use that as minimum, otherwise, use 1.5X RAM */
//     MinimumSize = (Ram >= 1024 * MEGABYTE) ? Ram : MaximumSize / 2;
//
//     /* Write the new sizes in the descriptor and mark it as system managed */
//     Descriptor->MinSize.QuadPart = MinimumSize;
//     Descriptor->MaxSize.QuadPart = MaximumSize;
//     Descriptor->Flags |= SMP_PAGEFILE_SYSTEM_MANAGED;
// }
//
// NTSTATUS
// NTAPI
// SmpValidatePagingFileSizes(IN PSMP_PAGEFILE_DESCRIPTOR Descriptor)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     BOOLEAN WasTooBig = FALSE;
//     ULONGLONG MinSize, MaxSize;
// #ifdef _M_IX86
//     ULONGLONG MaxPageFileSize =
//         (SharedUserData->ProcessorFeatures[PF_PAE_ENABLED])
//             ? MAXIMUM_PAGEFILE_SIZE_PAE : MAXIMUM_PAGEFILE_SIZE;
// #else
//     static const ULONGLONG MaxPageFileSize = MAXIMUM_PAGEFILE_SIZE;
// #endif
//
//     /* Capture the min and max */
//     MinSize = Descriptor->MinSize.QuadPart;
//     MaxSize = Descriptor->MaxSize.QuadPart;
//
//     DPRINT("SMSS:PFILE: Validating sizes for `%wZ' 0x%I64X 0x%I64X\n",
//             &Descriptor->Name, MinSize, MaxSize);
//
//     /* Don't let minimum be bigger than maximum */
//     if (MinSize > MaxSize)
//         MaxSize = MinSize;
//
//     /* Validate the minimum and maximum and trim them if they are too large */
//     if (MinSize > MaxPageFileSize)
//     {
//         WasTooBig = TRUE;
//         MinSize = MaxPageFileSize;
//     }
//     if (MaxSize > MaxPageFileSize)
//     {
//         WasTooBig = TRUE;
//         MaxSize = MaxPageFileSize;
//     }
//
//     /* If we trimmed, write a flag in the descriptor */
//     if (WasTooBig)
//     {
//         DPRINT("SMSS:PFILE: Trimmed size of `%wZ' to maximum allowed\n",
//                 &Descriptor->Name);
//         Descriptor->Flags |= SMP_PAGEFILE_WAS_TOO_BIG;
//     }
//
//     /* Now write the (possibly trimmed) sizes back */
//     Descriptor->MinSize.QuadPart = MinSize;
//     Descriptor->MaxSize.QuadPart = MaxSize;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpCreateSystemManagedPagingFile(IN PSMP_PAGEFILE_DESCRIPTOR Descriptor,
//                                  IN BOOLEAN DecreaseSize)
// {
//     LARGE_INTEGER FuzzFactor, Size;
//
//     /* Make sure there is at least 1 paging file and that we are system-managed */
//     ASSERT(SmpNumberOfPagingFiles >= 1);
//     ASSERT(!IsListEmpty(&SmpPagingFileDescriptorList));
//     ASSERT(Descriptor->Flags & SMP_PAGEFILE_SYSTEM_MANAGED);
//
//     /* Keep decreasing the pagefile by this amount if we run out of space */
//     FuzzFactor.QuadPart = FUZZ_FACTOR;
//
//     /* Create the descriptor for it (mainly the right sizes) and validate */
//     SmpMakeSystemManagedPagingFileDescriptor(Descriptor);
//     SmpValidatePagingFileSizes(Descriptor);
//
//     /* Use either the minimum size in the descriptor, or 16 MB in minimal mode */
//     Size.QuadPart = DecreaseSize ? 16 * MEGABYTE : Descriptor->MinSize.QuadPart;
//
//     /* Check if this should be a fixed pagefile or an any pagefile */
//     if (Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] == '?')
//     {
//         /* Find a disk for it */
//         return SmpCreatePagingFileOnAnyDrive(Descriptor, &FuzzFactor, &Size);
//     }
//
//     /* Use the disk that was given */
//     return SmpCreatePagingFileOnFixedDrive(Descriptor, &FuzzFactor, &Size);
// }
//
// NTSTATUS
// NTAPI
// SmpCreateEmergencyPagingFile(VOID)
// {
//     PSMP_PAGEFILE_DESCRIPTOR Descriptor;
//     WCHAR Buffer[32];
//
//     /* Allocate a descriptor */
//     Descriptor = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  sizeof(SMP_PAGEFILE_DESCRIPTOR));
//     if (!Descriptor) return STATUS_NO_MEMORY;
//
//     /* Initialize it */
//     RtlInitUnicodeString(&Descriptor->Token, NULL);
//
//     /* Copy the default pagefile name */
//     ASSERT(sizeof(Buffer) >= sizeof(STANDARD_PAGING_FILE_NAME));
//     wcscpy(Buffer, STANDARD_PAGING_FILE_NAME);
//
//     /* Fill the rest of the descriptor out */
//     RtlInitUnicodeString(&Descriptor->Name, Buffer);
//     Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] = '?';
//     Descriptor->Flags |= SMP_PAGEFILE_SYSTEM_MANAGED |
//                          SMP_PAGEFILE_EMERGENCY |
//                          SMP_PAGEFILE_ON_ANY_DRIVE;
//
//     /* Insert it into the descriptor list */
//     InsertHeadList(&SmpPagingFileDescriptorList, &Descriptor->Entry);
//     SmpNumberOfPagingFiles++;
//
//     /* Go ahead and create it now, with the minimal size possible */
//     return SmpCreateSystemManagedPagingFile(Descriptor, TRUE);
// }
//
// NTSTATUS
// NTAPI
// SmpCreateVolumeDescriptors(VOID)
// {
//     NTSTATUS Status;
//     UNICODE_STRING VolumePath;
//     BOOLEAN BootVolumeFound = FALSE;
//     WCHAR StartChar, Drive, DriveDiff;
//     HANDLE VolumeHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PROCESS_DEVICEMAP_INFORMATION ProcessInformation;
//     FILE_FS_DEVICE_INFORMATION DeviceInfo;
//     FILE_FS_SIZE_INFORMATION SizeInfo;
//     PSMP_VOLUME_DESCRIPTOR Volume;
//     LARGE_INTEGER FreeSpace, FinalFreeSpace;
//     WCHAR Buffer[32];
//
//     /* We should be starting with an empty list */
//     ASSERT(IsListEmpty(&SmpVolumeDescriptorList));
//
//     /* Query the device map so we can get the drive letters */
//     Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                        ProcessDeviceMap,
//                                        &ProcessInformation.Query,
//                                        sizeof(ProcessInformation.Query),
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS:PFILE: Query(ProcessDeviceMap) failed with status %X\n",
//                 Status);
//         return Status;
//     }
//
//     /* Build the volume string, starting with A: (we'll edit this in place) */
//     wcscpy(Buffer, L"\\??\\A:\\");
//     RtlInitUnicodeString(&VolumePath, Buffer);
//
//     /* Start with the C drive, except on NEC PC-98 */
//     StartChar = IsNEC_98 ? L'A' : L'C';
//     for (Drive = StartChar, DriveDiff = StartChar - L'A'; Drive <= L'Z'; Drive++, DriveDiff++)
//     {
//         /* Skip the disk if it's not in the drive map */
//         if (!((1 << DriveDiff) & ProcessInformation.Query.DriveMap)) continue;
//
//         /* Write the drive letter and try to open the volume */
//         VolumePath.Buffer[STANDARD_DRIVE_LETTER_OFFSET] = Drive;
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &VolumePath,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         Status = NtOpenFile(&VolumeHandle,
//                             FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Skip the volume if we failed */
//             DPRINT1("SMSS:PFILE: Open volume `%wZ' failed with status %X\n",
//                     &VolumePath, Status);
//             continue;
//         }
//
//         /* Now query device information on the volume */
//         Status = NtQueryVolumeInformationFile(VolumeHandle,
//                                               &IoStatusBlock,
//                                               &DeviceInfo,
//                                               sizeof(DeviceInfo),
//                                               FileFsDeviceInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Move to the next volume if we failed */
//             DPRINT1("SMSS:PFILE: Query volume `%wZ' (handle %p) for device info"
//                     " failed with status %X\n",
//                     &VolumePath,
//                     VolumeHandle,
//                     Status);
//             NtClose(VolumeHandle);
//             continue;
//         }
//
//         /* Check if this is a fixed disk */
//         if (DeviceInfo.Characteristics & (FILE_FLOPPY_DISKETTE |
//                                           FILE_READ_ONLY_DEVICE |
//                                           FILE_REMOTE_DEVICE |
//                                           FILE_REMOVABLE_MEDIA))
//         {
//             /* It isn't, so skip it */
//             DPRINT1("SMSS:PFILE: Volume `%wZ' (%X) cannot store a paging file\n",
//                     &VolumePath,
//                     DeviceInfo.Characteristics);
//             NtClose(VolumeHandle);
//             continue;
//         }
//
//         /* We found a fixed volume, allocate a descriptor for it */
//         Volume = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  sizeof(SMP_VOLUME_DESCRIPTOR));
//         if (!Volume)
//         {
//             /* Failed to allocate memory, try the next disk */
//             DPRINT1("SMSS:PFILE: Failed to allocate a volume descriptor (%u bytes)\n",
//                     sizeof(SMP_VOLUME_DESCRIPTOR));
//             NtClose(VolumeHandle);
//             continue;
//         }
//
//         /* Save the drive letter and device information */
//         Volume->DriveLetter = Drive;
//         Volume->DeviceInfo = DeviceInfo;
//
//         /* Check if this is the boot volume */
//         if (RtlUpcaseUnicodeChar(Drive) ==
//             RtlUpcaseUnicodeChar(SharedUserData->NtSystemRoot[0]))
//         {
//             /* Save it */
//             ASSERT(BootVolumeFound == FALSE);
//             Volume->Flags |= SMP_VOLUME_IS_BOOT;
//             BootVolumeFound = TRUE;
//         }
//
//         /* Now get size information on the volume */
//         Status = NtQueryVolumeInformationFile(VolumeHandle,
//                                               &IoStatusBlock,
//                                               &SizeInfo,
//                                               sizeof(SizeInfo),
//                                               FileFsSizeInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed -- keep going */
//             DPRINT1("SMSS:PFILE: Query volume `%wZ' (handle %p) for size failed"
//                     " with status %X\n",
//                     &VolumePath,
//                     VolumeHandle,
//                     Status);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Volume);
//             NtClose(VolumeHandle);
//             continue;
//         }
//
//         /* Done querying volume information, close the handle */
//         NtClose(VolumeHandle);
//
//         /* Compute how much free space we have */
//         FreeSpace.QuadPart = SizeInfo.AvailableAllocationUnits.QuadPart *
//                              SizeInfo.SectorsPerAllocationUnit;
//         FinalFreeSpace.QuadPart = FreeSpace.QuadPart * SizeInfo.BytesPerSector;
//
//         /* Check if there is less than 32 MB free so we don't starve the disk */
//         if (FinalFreeSpace.QuadPart <= MINIMUM_TO_KEEP_FREE)
//         {
//             /* In this case, act as if there is no free space */
//             Volume->FreeSpace.QuadPart = 0;
//         }
//         else
//         {
//             /* Trim off 32 MB to give the disk a bit of breathing room */
//             Volume->FreeSpace.QuadPart = FinalFreeSpace.QuadPart -
//                                          MINIMUM_TO_KEEP_FREE;
//         }
//
//         /* All done, add this volume to our descriptor list */
//         InsertTailList(&SmpVolumeDescriptorList, &Volume->Entry);
//         Volume->Flags |= SMP_VOLUME_INSERTED;
//         DPRINT("SMSS:PFILE: Created volume descriptor for`%wZ'\n", &VolumePath);
//     }
//
//     /* We must've found at least the boot volume */
//     ASSERT(BootVolumeFound == TRUE);
//     ASSERT(!IsListEmpty(&SmpVolumeDescriptorList));
//     if (!IsListEmpty(&SmpVolumeDescriptorList)) return STATUS_SUCCESS;
//
//     /* Something is really messed up if we found no disks at all */
//     return STATUS_UNEXPECTED_IO_ERROR;
// }
//
// NTSTATUS
// NTAPI
// SmpCreatePagingFiles(VOID)
// {
//     NTSTATUS Status;
//     PSMP_PAGEFILE_DESCRIPTOR Descriptor;
//     LARGE_INTEGER Size, FuzzFactor;
//     BOOLEAN Created = FALSE;
//     PLIST_ENTRY NextEntry;
//
//     /* Check if no paging files were requested */
//     if (!(SmpNumberOfPagingFiles) && !(SmpRegistrySpecifierPresent))
//     {
//         /* The list should be empty -- nothing to do */
//         ASSERT(IsListEmpty(&SmpPagingFileDescriptorList));
//         DPRINT1("SMSS:PFILE: No paging file was requested\n");
//         return STATUS_SUCCESS;
//     }
//
//     /* Initialize the volume descriptors so we can know what's available */
//     Status = SmpCreateVolumeDescriptors();
//     if (!NT_SUCCESS(Status))
//     {
//         /* We can't make decisions without this, so fail */
//         DPRINT1("SMSS:PFILE: Failed to create volume descriptors (status %X)\n",
//                 Status);
//         return Status;
//     }
//
//     /* If we fail creating pagefiles, try to reduce by this much each time */
//     FuzzFactor.QuadPart = FUZZ_FACTOR;
//
//     /* Loop the descriptor list */
//     NextEntry = SmpPagingFileDescriptorList.Flink;
//     while (NextEntry != &SmpPagingFileDescriptorList)
//     {
//         /* Check what kind of descriptor this is */
//         Descriptor = CONTAINING_RECORD(NextEntry, SMP_PAGEFILE_DESCRIPTOR, Entry);
//         if (Descriptor->Flags & SMP_PAGEFILE_SYSTEM_MANAGED)
//         {
//             /* This is a system-managed descriptor. Create the correct file */
//             DPRINT("SMSS:PFILE: Creating a system managed paging file (`%wZ')\n",
//                     &Descriptor->Name);
//             Status = SmpCreateSystemManagedPagingFile(Descriptor, FALSE);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* We failed -- try again, with size minimization this time */
//                 DPRINT("SMSS:PFILE: Trying lower sizes for (`%wZ')\n",
//                         &Descriptor->Name);
//                 Status = SmpCreateSystemManagedPagingFile(Descriptor, TRUE);
//             }
//         }
//         else
//         {
//             /* This is a manually entered descriptor. Validate its size first */
//             SmpValidatePagingFileSizes(Descriptor);
//
//             /* Check if this is an ANY pagefile or a FIXED pagefile */
//             DPRINT("SMSS:PFILE: Creating a normal paging file (`%wZ')\n",
//                     &Descriptor->Name);
//             if (Descriptor->Name.Buffer[STANDARD_DRIVE_LETTER_OFFSET] == L'?')
//             {
//                 /* It's an any pagefile, try to create it wherever possible */
//                 Size = Descriptor->MinSize;
//                 Status = SmpCreatePagingFileOnAnyDrive(Descriptor,
//                                                        &FuzzFactor,
//                                                        &Size);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* We failed to create it. Try again with a smaller size */
//                     DPRINT("SMSS:PFILE: Trying lower sizes for (`%wZ')\n",
//                             &Descriptor->Name);
//                     Size.QuadPart = 16 * MEGABYTE;
//                     Status = SmpCreatePagingFileOnAnyDrive(Descriptor,
//                                                            &FuzzFactor,
//                                                            &Size);
//                 }
//             }
//             else
//             {
//                 /* It's a fixed pagefile: override the minimum and use ours */
//                 Size.QuadPart = 16 * MEGABYTE;
//                 Status = SmpCreatePagingFileOnFixedDrive(Descriptor,
//                                                          &FuzzFactor,
//                                                          &Size);
//             }
//         }
//
//         /* Go to the next descriptor */
//         if (NT_SUCCESS(Status)) Created = TRUE;
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Check if none of the code in our loops above was able to create it */
//     if (!Created)
//     {
//         /* Build an emergency pagefile ourselves */
//         DPRINT1("SMSS:PFILE: Creating emergency paging file.\n");
//         Status = SmpCreateEmergencyPagingFile();
//     }
//
//     /* All done */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
