use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _MMPAGING_FILE, _RTL_BITMAP,
    _IO_STATUS_BLOCK,
    _UNICODE_STRING,
    _SECURITY_DESCRIPTOR,
    _KEVENT,
    _MDL,
    _OBJECT_ATTRIBUTES,
    _LARGE_INTEGER,
    _FILE_FS_DEVICE_INFORMATION,
    _ACL,
    PFILE_OBJECT,
    PMMPAGING_FILE,
    PPFN_NUMBER,
    PMDL,
    PUNICODE_STRING,
    PLARGE_INTEGER,
    PACL,
    PWSTR,
    KPROCESSOR_MODE,
    ACCESS_MASK,
    SWAPENTRY,
    KEVENT,
    MDL,
    LARGE_INTEGER,
    FILE_FS_DEVICE_INFORMATION,
    ExGetPreviousMode,
    KeInitializeEvent,
    KeWaitForSingleObject,
    IoPageRead,
    IoSynchronousPageWrite,
    KeBugCheck,
    MmUnmapLockedPages,
    SeSinglePrivilegeCheck,
    RtlCreateSecurityDescriptor,
    RtlSetDaclSecurityDescriptor,
    RtlCreateAcl,
    RtlAddAccessAllowedAce,
    RtlLengthSid,
    IoCreateFile,
    IoGetRelatedDeviceObject,
    ZwClose,
    ZwSetInformationFile,
    ZwSetSecurityObject,
    ObReferenceObjectByHandle,
    ExAllocatePoolZero,
};

extern "C" {
    static SeCreatePagefilePrivilege: ntkrnl_sys::ddk::_LUID;
    static mut SeLocalSystemSid: PVOID;
    static mut SeAliasAdminsSid: PVOID;
}

extern "system" {
    fn IoQueryVolumeInformation(
        FileObject: ntkrnl_sys::ddk::PFILE_OBJECT,
        FsInformationClass: ULONG,
        Length: ULONG,
        FsInformation: PVOID,
        ReturnedLength: *mut ULONG,
    ) -> NTSTATUS;
    fn IoInitializeCrashDump(FileHandle: HANDLE) -> BOOLEAN;
}
extern "C" {
    fn MmInitializeMdl(Mdl: *mut ntkrnl_sys::ddk::_MDL, BaseAddress: PVOID, Length: ULONG);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfDereferenceObject(Object: PVOID) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn ObfDereferenceObject(Object: PVOID) -> LONG;
}

#[inline]
unsafe fn ObDereferenceObject(obj: PVOID) {
    ObfDereferenceObject(obj);
}

type DEVICE_TYPE = ULONG;
type _ACE = u8; // placeholder for ACE structure

const STATUS_FLOPPY_VOLUME: NTSTATUS = 0xC0000164u32 as NTSTATUS;
const MINIMUM_PAGEFILE_SIZE: u64 = 256u64 * PAGE_SIZE as u64;

#[cfg(all(target_arch = "x86", not(feature = "pae")))]
const MAXIMUM_PAGEFILE_SIZE: u64 = (1u64 * 1024 * 1024 - 1) * PAGE_SIZE as u64;

#[cfg(any(
    all(target_arch = "x86", feature = "pae"),
    target_arch = "x86_64",
    target_arch = "aarch64"
))]
const MAXIMUM_PAGEFILE_SIZE: u64 = (4u64 * 1024 * 1024 * 1024 - 2) * PAGE_SIZE as u64;

static mut MmPageFileCreationLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MiFreeSwapPages: PFN_COUNT = 0;
#[no_mangle]
pub static mut MiUsedSwapPages: PFN_COUNT = 0;
static mut MiReservedSwapPages: PFN_COUNT = 0;

#[no_mangle]
pub static mut MmNumberOfPagingFiles: ULONG = 0;
#[no_mangle]
pub static mut MmPagingFile: [PVOID; MAX_PAGING_FILES] = [core::ptr::null_mut(); MAX_PAGING_FILES];
#[no_mangle]
pub static mut MmZeroPageFile: BOOLEAN = 0;

// Registry-controlled values owned by mm/pagefile. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static PAGEFILE_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "ClearPageFileAtShutdown",
            core::ptr::addr_of_mut!(MmZeroPageFile)),
    ] });

const MM_PAGEFILE_COMMIT_RATIO: usize = 1;
const MM_PAGEFILE_COMMIT_GRACE: usize = 256;

#[inline(always)]
unsafe fn FILE_FROM_ENTRY(i: SWAPENTRY) -> ULONG {
    (i & 0x0f) as ULONG
}

#[inline(always)]
unsafe fn OFFSET_FROM_ENTRY(i: SWAPENTRY) -> ULONG_PTR {
    (i >> 11) as ULONG_PTR
}

#[inline(always)]
unsafe fn ENTRY_FROM_FILE_OFFSET(i: ULONG, j: ULONG_PTR) -> SWAPENTRY {
    (i as SWAPENTRY) | ((j as SWAPENTRY) << 11) | 0x400
}

static mut MmSwapSpaceMessage: BOOLEAN = 0;
static mut MmSystemPageFileLocated: BOOLEAN = 0;

#[no_mangle]
pub unsafe extern "system" fn MmBuildMdlFromPages(Mdl: PMDL, Pages: PPFN_NUMBER) {
    let Mdl = opaque_ptr(Mdl);
    let page_count = (((*Mdl).ByteOffset as usize + (*Mdl).ByteCount as usize + PAGE_SIZE - 1) / PAGE_SIZE) as usize;
    let dest = (Mdl as *mut u8).add(core::mem::size_of::<MDL>()) as PPFN_NUMBER;
    core::ptr::copy_nonoverlapping(Pages, dest, page_count);
}

#[no_mangle]
pub unsafe extern "system" fn MmIsFileObjectAPagingFile(FileObject: PFILE_OBJECT) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    for i in 0..MmNumberOfPagingFiles as usize {
        if (*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject == FileObject {
            return 1;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn MmShowOutOfSpaceMessagePagingFile() {
    if MmSwapSpaceMessage == 0 {
        MmSwapSpaceMessage = 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmWriteToSwapPage(SwapEntry: SWAPENTRY, Page: PFN_NUMBER) -> NTSTATUS {
    if SwapEntry == 0 {
        mm_guard_trip!("pagefile.MmWriteToSwapPage null-swap-entry", 8, "Page" => Page);
        #[cfg(target_arch = "aarch64")]
        {
            return STATUS_UNSUCCESSFUL;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(MEMORY_MANAGEMENT);
            return STATUS_UNSUCCESSFUL;
        }
    }

    let i = FILE_FROM_ENTRY(SwapEntry) as usize;
    let offset = OFFSET_FROM_ENTRY(SwapEntry) - 1;

    if MmPagingFile[i].is_null() || (*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject.is_null() ||
        (*(*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject).DeviceObject.is_null()
    {
        mm_guard_trip!(
            "pagefile.MmWriteToSwapPage no-paging-file", 8,
            "SwapEntry" => SwapEntry, "index" => i, "Page" => Page,
        );
        #[cfg(target_arch = "aarch64")]
        {
            return STATUS_UNSUCCESSFUL;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
    }

    let mut mdl_base = [0u8; core::mem::size_of::<MDL>() + core::mem::size_of::<PFN_NUMBER>()];
    let mdl = mdl_base.as_mut_ptr() as PMDL;

    MmInitializeMdl(mdl, core::ptr::null_mut(), PAGE_SIZE as ULONG);
    MmBuildMdlFromPages(mdl, &Page as *const PFN_NUMBER as PPFN_NUMBER);
    (*mdl).MdlFlags |= MDL_PAGES_LOCKED as i16;

    let mut file_offset: LARGE_INTEGER = core::mem::zeroed();
    file_offset.QuadPart = (offset * PAGE_SIZE as ULONG_PTR) as i64;

    let mut event: KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut event as *mut KEVENT, 0, 0);

    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status = IoSynchronousPageWrite(
        (*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject,
        mdl,
        &mut file_offset as *mut LARGE_INTEGER,
        &mut event as *mut KEVENT,
        &mut iosb as *mut _IO_STATUS_BLOCK,
    );

    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut event as *mut KEVENT as PVOID,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
        status = iosb.__bindgen_anon_1.Status;
    }

    if ((*mdl).MdlFlags & 0x1) != 0 {
        MmUnmapLockedPages((*mdl).MappedSystemVa, mdl);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn MmReadFromSwapPage(SwapEntry: SWAPENTRY, Page: PFN_NUMBER) -> NTSTATUS {
    MiReadPageFile(Page, FILE_FROM_ENTRY(SwapEntry), OFFSET_FROM_ENTRY(SwapEntry))
}

#[no_mangle]
pub unsafe extern "system" fn MiReadPageFile(
    Page: PFN_NUMBER,
    PageFileIndex: ULONG,
    PageFileOffset: ULONG_PTR,
) -> NTSTATUS {
    if PageFileOffset == 0 {
        mm_guard_trip!(
            "pagefile.MiReadPageFile null-offset", 8,
            "Page" => Page, "PageFileIndex" => PageFileIndex,
        );
        #[cfg(target_arch = "aarch64")]
        {
            return STATUS_UNSUCCESSFUL;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(MEMORY_MANAGEMENT);
            return STATUS_UNSUCCESSFUL;
        }
    }

    let page_file_offset = PageFileOffset - 1;

    let paging_file = MmPagingFile[PageFileIndex as usize] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE;

    if paging_file.is_null() || (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject.is_null() || (*(*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject).DeviceObject.is_null() {
        mm_guard_trip!(
            "pagefile.MiReadPageFile no-paging-file", 8,
            "PageFileIndex" => PageFileIndex, "offset" => PageFileOffset, "Page" => Page,
        );
        #[cfg(target_arch = "aarch64")]
        {
            return STATUS_UNSUCCESSFUL;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
    }

    let mut mdl_base = [0u8; core::mem::size_of::<MDL>() + core::mem::size_of::<PFN_NUMBER>()];
    let mdl = mdl_base.as_mut_ptr() as PMDL;

    MmInitializeMdl(mdl, core::ptr::null_mut(), PAGE_SIZE as ULONG);
    MmBuildMdlFromPages(mdl, &Page as *const PFN_NUMBER as PPFN_NUMBER);
    (*mdl).MdlFlags |= (MDL_PAGES_LOCKED | 0x4000) as i16;

    let mut file_offset: LARGE_INTEGER = core::mem::zeroed();
    file_offset.QuadPart = (page_file_offset * PAGE_SIZE as ULONG_PTR) as i64;

    let mut event: KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut event as *mut KEVENT, 0, 0);

    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status = IoPageRead(
        (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject,
        mdl,
        &mut file_offset as *mut LARGE_INTEGER,
        &mut event as *mut KEVENT,
        &mut iosb as *mut _IO_STATUS_BLOCK,
    );

    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut event as *mut KEVENT as PVOID,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
        status = iosb.__bindgen_anon_1.Status;
    }

    if ((*mdl).MdlFlags & 0x1) != 0 {
        MmUnmapLockedPages((*mdl).MappedSystemVa, mdl);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn MmInitPagingFile() {
    KeInitializeGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);

    MiFreeSwapPages = 0;
    MiUsedSwapPages = 0;
    MiReservedSwapPages = 0;

    for i in 0..MAX_PAGING_FILES {
        MmPagingFile[i] = core::ptr::null_mut();
    }
    MmNumberOfPagingFiles = 0;
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeSwapPage(Entry: SWAPENTRY) {
    let i = FILE_FROM_ENTRY(Entry) as usize;
    let off = OFFSET_FROM_ENTRY(Entry) - 1;

    KeAcquireGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);

    let paging_file = MmPagingFile[i] as PMMPAGING_FILE;
    if paging_file.is_null() {
        // Freeing a swap page into a paging file that isn't there — the swap
        // entry outlived its file, so the entry is stale.
        mm_guard_trip!("pagefile.MmFreeSwapPage no-paging-file", 8, "index" => i);
        #[cfg(target_arch = "aarch64")]
        {
            KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
            return;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
    }

    RtlClearBit((*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap as *mut RTL_BITMAP, (off >> 5) as ULONG);

    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FreeSpace += 1;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).CurrentUsage -= 1;

    MiFreeSwapPages += 1;
    MiUsedSwapPages -= 1;
    UpdateTotalCommittedPages(-1);

    KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocSwapPage() -> SWAPENTRY {
    KeAcquireGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);

    if MiFreeSwapPages == 0 {
        KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
        return 0;
    }

    for i in 0..MAX_PAGING_FILES {
        if !MmPagingFile[i].is_null() && (*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FreeSpace >= 1 {
            let off = RtlFindClearBitsAndSet((*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap as *mut RTL_BITMAP, 1, 0);
            if off == 0xFFFFFFFF {
                KeBugCheck(MEMORY_MANAGEMENT);
                KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
                return STATUS_UNSUCCESSFUL as SWAPENTRY;
            }
            MiUsedSwapPages += 1;
            MiFreeSwapPages -= 1;
            UpdateTotalCommittedPages(1);

            KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);

            let entry = ENTRY_FROM_FILE_OFFSET(i as ULONG, (off + 1) as ULONG_PTR);
            return entry;
        }
    }

    KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
    // No paging file had a free page. x86 calls that fatal; arm64 returns 0 and
    // lets the caller cope. Arm it so a swap-exhaustion round is distinguishable
    // from a corruption round rather than both reading as "the sweep stalled".
    mm_guard_trip!("pagefile.MmAllocSwapPage swap-exhausted", 4);
    #[cfg(target_arch = "aarch64")]
    {
        return 0;
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        KeBugCheck(MEMORY_MANAGEMENT);
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtCreatePagingFile(
    FileName: PUNICODE_STRING,
    MinimumSize: PLARGE_INTEGER,
    MaximumSize: PLARGE_INTEGER,
    Reserved: ULONG,
) -> NTSTATUS {
    let FileName = opaque_ptr(FileName);
    let MinimumSize = opaque_ptr(MinimumSize);
    let MaximumSize = opaque_ptr(MaximumSize);
    if MmNumberOfPagingFiles >= MAX_PAGING_FILES as ULONG {
        return STATUS_TOO_MANY_PAGING_FILES;
    }

    let previous_mode = ExGetPreviousMode();

    let (safe_minimum_size, safe_maximum_size, mut page_file_name) = if previous_mode != 0 {
        if SeSinglePrivilegeCheck(SeCreatePagefilePrivilege, previous_mode) != 1 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }

        // C: ProbeForReadLargeInteger x2 + ProbeForReadUnicodeString inside one
        // _SEH2_TRY whose handler returns the exception code. Those typed macros
        // check range only (no alignment), hence alignment 1.
        let safe_min = match ntkrnl_sys::probe::probe_read_generic(MinimumSize as *const LARGE_INTEGER, 1, previous_mode) {
            Ok(v) => v,
            Err(status) => return status,
        };
        let safe_max = match ntkrnl_sys::probe::probe_read_generic(MaximumSize as *const LARGE_INTEGER, 1, previous_mode) {
            Ok(v) => v,
            Err(status) => return status,
        };
        let page_fn = match ntkrnl_sys::probe::probe_read_generic(FileName as *const _UNICODE_STRING, 1, previous_mode) {
            Ok(v) => v,
            Err(status) => return status,
        };

        (safe_min, safe_max, page_fn)
    } else {
        (*MinimumSize, *MaximumSize, *FileName)
    };

    if (safe_minimum_size.QuadPart as u64) < MINIMUM_PAGEFILE_SIZE ||
        (safe_minimum_size.QuadPart as u64) > MAXIMUM_PAGEFILE_SIZE
    {
        return STATUS_INVALID_PARAMETER_2;
    }

    if safe_maximum_size.QuadPart < safe_minimum_size.QuadPart ||
        (safe_maximum_size.QuadPart as u64) > MAXIMUM_PAGEFILE_SIZE
    {
        return STATUS_INVALID_PARAMETER_3;
    }

    if page_file_name.Length == 0 || page_file_name.Length > 520 {
        return STATUS_OBJECT_NAME_INVALID;
    }

    page_file_name.MaximumLength = page_file_name.Length;
    let buffer = ExAllocatePoolWithTag(PagedPool, page_file_name.Length as SIZE_T, TAG_MM);
    if buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if previous_mode != 0 {
        // C: probe + copy in _SEH2_TRY; the handler frees the buffer and returns
        // the exception code.
        if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
            buffer,
            page_file_name.Buffer as *const core::ffi::c_void,
            page_file_name.Length as usize,
            core::mem::size_of::<u16>(),
            previous_mode,
        ) {
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }
    } else {
        core::ptr::copy_nonoverlapping(
            page_file_name.Buffer as *const u8,
            buffer as *mut u8,
            page_file_name.Length as usize,
        );
    }

    page_file_name.Buffer = buffer as PWSTR;

    let mut security_descriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut status = RtlCreateSecurityDescriptor(
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID,
        1,
    );
    if status < 0 {
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    let count = core::mem::size_of::<_ACL>() +
        (core::mem::size_of::<_ACE>() + RtlLengthSid(SeLocalSystemSid) as usize) +
        (core::mem::size_of::<_ACE>() + RtlLengthSid(SeAliasAdminsSid) as usize);
    let dacl = ExAllocatePoolWithTag(PagedPool, count, TAG_DACL);
    if dacl.is_null() {
        ExFreePoolWithTag(buffer, TAG_MM);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = RtlCreateAcl(dacl as *mut _ACL, count as ULONG, 2);
    if status < 0 {
        ExFreePoolWithTag(dacl, TAG_DACL);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    status = RtlAddAccessAllowedAce(dacl as *mut _ACL, 2, 0x001F01FF, SeAliasAdminsSid);
    if status < 0 {
        ExFreePoolWithTag(dacl, TAG_DACL);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    status = RtlAddAccessAllowedAce(dacl as *mut _ACL, 2, 0x001F01FF, SeLocalSystemSid);
    if status < 0 {
        ExFreePoolWithTag(dacl, TAG_DACL);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    status = RtlSetDaclSecurityDescriptor(
        &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID,
        1,
        dacl as PACL,
        0,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl, TAG_DACL);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName = &mut page_file_name as *mut _UNICODE_STRING;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    object_attributes.SecurityDescriptor = &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID;
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut allocation_size: LARGE_INTEGER = core::mem::zeroed();
    allocation_size.QuadPart = safe_minimum_size.QuadPart + PAGE_SIZE as i64;

    let mut file_handle: HANDLE = core::ptr::null_mut();
    let mut io_status: _IO_STATUS_BLOCK = core::mem::zeroed();

    status = IoCreateFile(
        &mut file_handle as *mut HANDLE,
        0x00100000 | 0x00040000 | 0x0001 | 0x0002,
        &mut object_attributes as *mut _OBJECT_ATTRIBUTES,
        &mut io_status as *mut _IO_STATUS_BLOCK,
        &mut allocation_size as *mut LARGE_INTEGER,
        0x00000004 | 0x00000002,
        0x00000002,
        5,
        0x00001000 | 0x00000200 | 0x00000008,
        core::ptr::null_mut(),
        0,
        0,
        core::ptr::null_mut(),
        0x00800000 | 0x00000100,
    );

    if status < 0 {
        status = IoCreateFile(
            &mut file_handle as *mut HANDLE,
            0x00100000 | 0x0002,
            &mut object_attributes as *mut _OBJECT_ATTRIBUTES,
            &mut io_status as *mut _IO_STATUS_BLOCK,
            &mut allocation_size as *mut LARGE_INTEGER,
            0x00000004 | 0x00000002,
            0x00000002 | 0x00000001,
            1,
            0x00000200 | 0x00000008,
            core::ptr::null_mut(),
            0,
            0,
            core::ptr::null_mut(),
            0x00800000 | 0x00000100,
        );

        if status < 0 {
            ExFreePoolWithTag(dacl, TAG_DACL);
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }

        let mut file_object: PFILE_OBJECT = core::ptr::null_mut();
        status = ObReferenceObjectByHandle(
            file_handle,
            0x0001 | 0x0002,
            IoFileObjectType as *mut ntkrnl_sys::ddk::_OBJECT_TYPE,
            0,
            &mut file_object as *mut PFILE_OBJECT as *mut PVOID,
            core::ptr::null_mut(),
        );

        if status < 0 {
            ZwClose(file_handle);
            ExFreePoolWithTag(dacl, TAG_DACL);
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }

        let mut paging_file: PMMPAGING_FILE = core::ptr::null_mut();

        KeAcquireGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);

        for i in 0..MmNumberOfPagingFiles as usize {
            if (*(MmPagingFile[i] as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject == file_object {
                paging_file = MmPagingFile[i] as PMMPAGING_FILE;
                break;
            }
        }

        if paging_file.is_null() {
            KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
            ObDereferenceObject(file_object as PVOID);
            ZwClose(file_handle);
            status = STATUS_NOT_FOUND;
            ExFreePoolWithTag(dacl, TAG_DACL);
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }

        if (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).MinimumSize > ((safe_minimum_size.QuadPart as ntkrnl_sys::ddk::PFN_NUMBER) >> 12) {
            KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
            ObDereferenceObject(file_object as PVOID);
            ZwClose(file_handle);
            status = STATUS_INVALID_PARAMETER_2;
            ExFreePoolWithTag(dacl, TAG_DACL);
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }

        if ((safe_maximum_size.QuadPart as ntkrnl_sys::ddk::PFN_NUMBER) >> 12) < (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).MaximumSize {
            KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
            ObDereferenceObject(file_object as PVOID);
            ZwClose(file_handle);
            status = STATUS_INVALID_PARAMETER_3;
            ExFreePoolWithTag(dacl, TAG_DACL);
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }

        KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
        ObDereferenceObject(file_object as PVOID);
        ZwClose(file_handle);
        status = STATUS_NOT_IMPLEMENTED;
        ExFreePoolWithTag(dacl, TAG_DACL);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    ExFreePoolWithTag(dacl, TAG_DACL);

    if io_status.__bindgen_anon_1.Status >= 0 {
        status = ZwSetSecurityObject(file_handle, 0x00000004, &mut security_descriptor as *mut _SECURITY_DESCRIPTOR as PVOID);
        if status < 0 {
            ZwClose(file_handle);
            ExFreePoolWithTag(buffer, TAG_MM);
            return status;
        }
    }

    status = ZwSetInformationFile(
        file_handle,
        &mut io_status as *mut _IO_STATUS_BLOCK,
        &safe_minimum_size as *const LARGE_INTEGER as PVOID,
        core::mem::size_of::<LARGE_INTEGER>() as ULONG,
        20,
    );

    if status < 0 || io_status.__bindgen_anon_1.Status < 0 {
        ZwClose(file_handle);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    let mut file_object: PFILE_OBJECT = core::ptr::null_mut();
    status = ObReferenceObjectByHandle(
        file_handle,
        0x0001 | 0x0002,
        IoFileObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
        0,
        &mut file_object as *mut PFILE_OBJECT as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status < 0 {
        ZwClose(file_handle);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    let device_type = (*IoGetRelatedDeviceObject(file_object)).DeviceType;
    if device_type != 0x00000007 &&
        device_type != 0x00000014 &&
        device_type != 0x00000016 &&
        device_type != 0x00000013
    {
        ObDereferenceObject(file_object as PVOID);
        ZwClose(file_handle);
        ExFreePoolWithTag(buffer, TAG_MM);
        return status;
    }

    let mut fs_device_info: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
    let mut count: ULONG = 0;
    IoQueryVolumeInformation(
        file_object,
        4,
        core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
        &mut fs_device_info as *mut FILE_FS_DEVICE_INFORMATION as PVOID,
        &mut count as *mut ULONG,
    );

    if (fs_device_info.Characteristics & 0x00000001) != 0 {
        ObDereferenceObject(file_object as PVOID);
        ZwClose(file_handle);
        ExFreePoolWithTag(buffer, TAG_MM);
        return STATUS_FLOPPY_VOLUME;
    }

    let paging_file = ExAllocatePoolZero(NonPagedPool, core::mem::size_of::<ntkrnl_sys::ddk::_MMPAGING_FILE>() as ntkrnl_sys::ddk::SIZE_T, TAG_MM) as *mut ntkrnl_sys::ddk::_MMPAGING_FILE;
    if paging_file.is_null() {
        ObDereferenceObject(file_object as PVOID);
        ZwClose(file_handle);
        ExFreePoolWithTag(buffer, TAG_MM);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileHandle = file_handle;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FileObject = file_object;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Size = (safe_minimum_size.QuadPart as ntkrnl_sys::ddk::PFN_NUMBER) >> 12;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).MinimumSize = (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Size;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).MaximumSize = (safe_maximum_size.QuadPart as ntkrnl_sys::ddk::PFN_NUMBER) >> 12;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FreeSpace = (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Size - 1;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).CurrentUsage = 0;
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).PageFileName = page_file_name;

    let alloc_map_size = core::mem::size_of::<_RTL_BITMAP>() + (((*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).MaximumSize + 31) / 32) as usize * core::mem::size_of::<ULONG>();
    (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap = ExAllocatePoolWithTag(NonPagedPool, alloc_map_size, TAG_MM) as *mut ntkrnl_sys::ddk::_RTL_BITMAP;
    if (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap.is_null() {
        ExFreePoolWithTag(paging_file as PVOID, TAG_MM);
        ObDereferenceObject(file_object as PVOID);
        ZwClose(file_handle);
        ExFreePoolWithTag(buffer, TAG_MM);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlInitializeBitMap(
        (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap as *mut RTL_BITMAP,
        ((*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap as *mut u8).add(core::mem::size_of::<_RTL_BITMAP>()) as *mut ULONG,
        (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).MaximumSize as ULONG,
    );
    RtlClearAllBits((*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).Bitmap as *mut RTL_BITMAP);

    KeAcquireGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);
    MmPagingFile[MmNumberOfPagingFiles as usize] = paging_file as PVOID;
    MmNumberOfPagingFiles += 1;
    MiFreeSwapPages = MiFreeSwapPages + (*(paging_file as *mut ntkrnl_sys::ddk::_MMPAGING_FILE)).FreeSpace as PFN_COUNT;
    KeReleaseGuardedMutex(&mut MmPageFileCreationLock as *mut KGUARDED_MUTEX);

    MmSwapSpaceMessage = 0;

    if MmSystemPageFileLocated == 0 && ((*(*file_object).DeviceObject).Flags & 0x00000010) != 0 {
        MmSystemPageFileLocated = IoInitializeCrashDump(file_handle) as BOOLEAN;
    }

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Paging file functions
//  * COPYRIGHT:   Copyright 1998-2003 David Welch <welch@mcmail.com>
//  *              Copyright 2010-2018 Pierre Schweitzer <pierre@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* Minimum pagefile size is 256 pages (1 MB) */
// #define MINIMUM_PAGEFILE_SIZE       (256ULL * PAGE_SIZE)
//
// /* Maximum pagefile sizes for different architectures */
// #if defined(_M_IX86) && !defined(_X86PAE_)
// /* Around 4 GB */
//     #define MAXIMUM_PAGEFILE_SIZE   ((1ULL * 1024 * 1024 - 1) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(4ULL * GIGABYTE - 1)
// /* PAE uses the same size as x64 */
// #elif (defined(_M_IX86) && defined(_X86PAE_)) || defined (_M_AMD64) || defined(_M_ARM64)
// /* Around 16 TB */
//     #if (NTDDI_VERSION >= NTDDI_WIN10)
//     #define MAXIMUM_PAGEFILE_SIZE   ((4ULL * 1024 * 1024 * 1024 - 2) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(16ULL * TERABYTE - PAGE_SIZE - 1)
//     #else
//     #define MAXIMUM_PAGEFILE_SIZE   ((4ULL * 1024 * 1024 * 1024 - 1) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(16ULL * TERABYTE - 1)
//     #endif
// #elif defined (_M_IA64)
// /* Around 32 TB */
//     #define MAXIMUM_PAGEFILE_SIZE   ((8ULL * 1024 * 1024 * 1024 - 1) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(32ULL * TERABYTE - 1)
// #elif defined(_M_ARM)
// /* Around 2 GB */
//     #if (NTDDI_VERSION >= NTDDI_WIN10)
//     #define MAXIMUM_PAGEFILE_SIZE   ((512ULL * 1024 - 2) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(2ULL * GIGABYTE - PAGE_SIZE - 1)
//     #elif (NTDDI_VERSION >= NTDDI_WINBLUE) // NTDDI_WIN81
//     #define MAXIMUM_PAGEFILE_SIZE   ((512ULL * 1024 - 1) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(2ULL * GIGABYTE - 1)
//     #else
// /* Around 4 GB */
//     #define MAXIMUM_PAGEFILE_SIZE   ((1ULL * 1024 * 1024 - 1) * PAGE_SIZE)
//                                  // PAGE_ROUND_DOWN(4ULL * GIGABYTE - 1)
//     #endif
// #else
// #error Unknown architecture
// #endif
//
// /* List of paging files, both used and free */
// PMMPAGING_FILE MmPagingFile[MAX_PAGING_FILES];
//
// /* Lock for examining the list of paging files */
// KGUARDED_MUTEX MmPageFileCreationLock;
//
// /* Number of paging files */
// ULONG MmNumberOfPagingFiles;
//
// /* Number of pages that are available for swapping */
// PFN_COUNT MiFreeSwapPages;
//
// /* Number of pages that have been allocated for swapping */
// PFN_COUNT MiUsedSwapPages;
//
// BOOLEAN MmZeroPageFile;
//
// /*
//  * Number of pages that have been reserved for swapping but not yet allocated
//  */
// static PFN_COUNT MiReservedSwapPages;
//
// /*
//  * Ratio between reserved and available swap pages, e.g. setting this to five
//  * forces one swap page to be available for every five swap pages that are
//  * reserved. Setting this to zero turns off commit checking altogether.
//  */
// #define MM_PAGEFILE_COMMIT_RATIO      (1)
//
// /*
//  * Number of pages that can be used for potentially swapable memory without
//  * pagefile space being reserved. The intention is that this allows smss
//  * to start up and create page files while ordinarily having a commit
//  * ratio of one.
//  */
// #define MM_PAGEFILE_COMMIT_GRACE      (256)
//
// /*
//  * Translate between a swap entry and a file and offset pair.
//  */
// #define FILE_FROM_ENTRY(i) ((i) & 0x0f)
// #define OFFSET_FROM_ENTRY(i) ((i) >> 11)
// #define ENTRY_FROM_FILE_OFFSET(i, j) ((i) | ((j) << 11) | 0x400)
//
// /* Make sure there can be only 16 paging files */
// C_ASSERT(FILE_FROM_ENTRY(0xffffffff) < MAX_PAGING_FILES);
//
// static BOOLEAN MmSwapSpaceMessage = FALSE;
//
// static BOOLEAN MmSystemPageFileLocated = FALSE;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// MmBuildMdlFromPages(PMDL Mdl, PPFN_NUMBER Pages)
// {
//     memcpy(Mdl + 1, Pages, sizeof(PFN_NUMBER) * (PAGE_ROUND_UP(Mdl->ByteOffset+Mdl->ByteCount)/PAGE_SIZE));
// }
//
//
// BOOLEAN
// NTAPI
// MmIsFileObjectAPagingFile(PFILE_OBJECT FileObject)
// {
//     ULONG i;
//
//     /* Loop through all the paging files */
//     for (i = 0; i < MmNumberOfPagingFiles; i++)
//     {
//         /* Check if this is one of them */
//         if (MmPagingFile[i]->FileObject == FileObject) return TRUE;
//     }
//
//     /* Nothing found */
//     return FALSE;
// }
//
// VOID
// NTAPI
// MmShowOutOfSpaceMessagePagingFile(VOID)
// {
//     if (!MmSwapSpaceMessage)
//     {
//         DPRINT1("MM: Out of swap space.\n");
//         MmSwapSpaceMessage = TRUE;
//     }
// }
//
// NTSTATUS
// NTAPI
// MmWriteToSwapPage(SWAPENTRY SwapEntry, PFN_NUMBER Page)
// {
//     ULONG i;
//     ULONG_PTR offset;
//     LARGE_INTEGER file_offset;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//     KEVENT Event;
//     UCHAR MdlBase[sizeof(MDL) + sizeof(PFN_NUMBER)];
//     PMDL Mdl = (PMDL)MdlBase;
//
//     DPRINT("MmWriteToSwapPage\n");
//
//     if (SwapEntry == 0)
//     {
// #ifdef _M_ARM64
//         DPRINT1("MmWriteToSwapPage: SwapEntry is 0\n");
//         return(STATUS_UNSUCCESSFUL);
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
//         return(STATUS_UNSUCCESSFUL);
// #endif
//     }
//
//     i = FILE_FROM_ENTRY(SwapEntry);
//     offset = OFFSET_FROM_ENTRY(SwapEntry) - 1;
//
//     if (MmPagingFile[i] == NULL || MmPagingFile[i]->FileObject == NULL ||
//             MmPagingFile[i]->FileObject->DeviceObject == NULL)
//     {
//         DPRINT1("Bad paging file 0x%.8X\n", SwapEntry);
// #ifdef _M_ARM64
//         return(STATUS_UNSUCCESSFUL);
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
// #endif
//     }
//
//     MmInitializeMdl(Mdl, NULL, PAGE_SIZE);
//     MmBuildMdlFromPages(Mdl, &Page);
//     Mdl->MdlFlags |= MDL_PAGES_LOCKED;
//
//     file_offset.QuadPart = offset * PAGE_SIZE;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Status = IoSynchronousPageWrite(MmPagingFile[i]->FileObject,
//                                     Mdl,
//                                     &file_offset,
//                                     &Event,
//                                     &Iosb);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = Iosb.Status;
//     }
//
//     if (Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA)
//     {
//         MmUnmapLockedPages (Mdl->MappedSystemVa, Mdl);
//     }
//     return(Status);
// }
//
//
// NTSTATUS
// NTAPI
// MmReadFromSwapPage(SWAPENTRY SwapEntry, PFN_NUMBER Page)
// {
//     return MiReadPageFile(Page, FILE_FROM_ENTRY(SwapEntry), OFFSET_FROM_ENTRY(SwapEntry));
// }
//
// NTSTATUS
// NTAPI
// MiReadPageFile(
//     _In_ PFN_NUMBER Page,
//     _In_ ULONG PageFileIndex,
//     _In_ ULONG_PTR PageFileOffset)
// {
//     LARGE_INTEGER file_offset;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//     KEVENT Event;
//     UCHAR MdlBase[sizeof(MDL) + sizeof(PFN_NUMBER)];
//     PMDL Mdl = (PMDL)MdlBase;
//     PMMPAGING_FILE PagingFile;
//
//     DPRINT("MiReadSwapFile\n");
//
//     if (PageFileOffset == 0)
//     {
// #ifdef _M_ARM64
//         DPRINT1("MiReadPageFile: PageFileOffset is 0\n");
//         return(STATUS_UNSUCCESSFUL);
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
//         return(STATUS_UNSUCCESSFUL);
// #endif
//     }
//
//     /* Normalize offset. */
//     PageFileOffset--;
//
//     ASSERT(PageFileIndex < MAX_PAGING_FILES);
//
//     PagingFile = MmPagingFile[PageFileIndex];
//
//     if (PagingFile == NULL || PagingFile->FileObject == NULL || PagingFile->FileObject->DeviceObject == NULL)
//     {
//         DPRINT1("Bad paging file %u\n", PageFileIndex);
// #ifdef _M_ARM64
//         return(STATUS_UNSUCCESSFUL);
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
// #endif
//     }
//
//     MmInitializeMdl(Mdl, NULL, PAGE_SIZE);
//     MmBuildMdlFromPages(Mdl, &Page);
//     Mdl->MdlFlags |= MDL_PAGES_LOCKED | MDL_IO_PAGE_READ;
//
//     file_offset.QuadPart = PageFileOffset * PAGE_SIZE;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Status = IoPageRead(PagingFile->FileObject,
//                         Mdl,
//                         &file_offset,
//                         &Event,
//                         &Iosb);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = Iosb.Status;
//     }
//     if (Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA)
//     {
//         MmUnmapLockedPages (Mdl->MappedSystemVa, Mdl);
//     }
//     return(Status);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MmInitPagingFile(VOID)
// {
//     ULONG i;
//
//     KeInitializeGuardedMutex(&MmPageFileCreationLock);
//
//     MiFreeSwapPages = 0;
//     MiUsedSwapPages = 0;
//     MiReservedSwapPages = 0;
//
//     for (i = 0; i < MAX_PAGING_FILES; i++)
//     {
//         MmPagingFile[i] = NULL;
//     }
//     MmNumberOfPagingFiles = 0;
// }
//
// VOID
// NTAPI
// MmFreeSwapPage(SWAPENTRY Entry)
// {
//     ULONG i;
//     ULONG_PTR off;
//     PMMPAGING_FILE PagingFile;
//
//     i = FILE_FROM_ENTRY(Entry);
//     off = OFFSET_FROM_ENTRY(Entry) - 1;
//
//     KeAcquireGuardedMutex(&MmPageFileCreationLock);
//
//     PagingFile = MmPagingFile[i];
//     if (PagingFile == NULL)
//     {
// #ifdef _M_ARM64
//         /* ARM64: No pagefile on ramdisk — log and return gracefully */
//         DPRINT1("MmFreeSwapPage: PagingFile[%lu] is NULL for Entry 0x%lx — no pagefile?\n", i, Entry);
//         KeReleaseGuardedMutex(&MmPageFileCreationLock);
//         return;
// #else
//         KeBugCheck(MEMORY_MANAGEMENT);
// #endif
//     }
//
//     RtlClearBit(PagingFile->Bitmap, off >> 5);
//
//     PagingFile->FreeSpace++;
//     PagingFile->CurrentUsage--;
//
//     MiFreeSwapPages++;
//     MiUsedSwapPages--;
//     UpdateTotalCommittedPages(-1);
//
//     KeReleaseGuardedMutex(&MmPageFileCreationLock);
// }
//
// SWAPENTRY
// NTAPI
// MmAllocSwapPage(VOID)
// {
//     ULONG i;
//     ULONG off;
//     SWAPENTRY entry;
//
//     KeAcquireGuardedMutex(&MmPageFileCreationLock);
//
//     if (MiFreeSwapPages == 0)
//     {
//         KeReleaseGuardedMutex(&MmPageFileCreationLock);
//         return(0);
//     }
//
//     for (i = 0; i < MAX_PAGING_FILES; i++)
//     {
//         if (MmPagingFile[i] != NULL &&
//                 MmPagingFile[i]->FreeSpace >= 1)
//         {
//             off = RtlFindClearBitsAndSet(MmPagingFile[i]->Bitmap, 1, 0);
//             if (off == 0xFFFFFFFF)
//             {
//                 KeBugCheck(MEMORY_MANAGEMENT);
//                 KeReleaseGuardedMutex(&MmPageFileCreationLock);
//                 return(STATUS_UNSUCCESSFUL);
//             }
//             MiUsedSwapPages++;
//             MiFreeSwapPages--;
//             UpdateTotalCommittedPages(1);
//
//             KeReleaseGuardedMutex(&MmPageFileCreationLock);
//
//             entry = ENTRY_FROM_FILE_OFFSET(i, off + 1);
//             return(entry);
//         }
//     }
//
//     KeReleaseGuardedMutex(&MmPageFileCreationLock);
// #ifdef _M_ARM64
//     /* ARM64: No pagefile on ramdisk — return 0 (allocation failed) */
//     DPRINT1("MmAllocSwapPage: No free swap pages and MiFreeSwapPages > 0 — inconsistent state\n");
//     return(0);
// #else
//     KeBugCheck(MEMORY_MANAGEMENT);
//     return(0);
// #endif
// }
//
// NTSTATUS
// NTAPI
// NtCreatePagingFile(
//     _In_ PUNICODE_STRING FileName,
//     _In_ PLARGE_INTEGER MinimumSize,
//     _In_ PLARGE_INTEGER MaximumSize,
//     _In_ ULONG Reserved)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE FileHandle;
//     IO_STATUS_BLOCK IoStatus;
//     PFILE_OBJECT FileObject;
//     PMMPAGING_FILE PagingFile;
//     SIZE_T AllocMapSize;
//     ULONG Count;
//     KPROCESSOR_MODE PreviousMode;
//     UNICODE_STRING PageFileName;
//     LARGE_INTEGER SafeMinimumSize, SafeMaximumSize, AllocationSize;
//     FILE_FS_DEVICE_INFORMATION FsDeviceInfo;
//     SECURITY_DESCRIPTOR SecurityDescriptor;
//     PACL Dacl;
//     PWSTR Buffer;
//     DEVICE_TYPE DeviceType;
//
//     PAGED_CODE();
//
//     DPRINT("NtCreatePagingFile(FileName: '%wZ', MinimumSize: %I64d, MaximumSize: %I64d)\n",
//            FileName, MinimumSize->QuadPart, MaximumSize->QuadPart);
//
//     if (MmNumberOfPagingFiles >= MAX_PAGING_FILES)
//     {
//         return STATUS_TOO_MANY_PAGING_FILES;
//     }
//
//     PreviousMode = ExGetPreviousMode();
//
//     if (PreviousMode != KernelMode)
//     {
//         if (SeSinglePrivilegeCheck(SeCreatePagefilePrivilege, PreviousMode) != TRUE)
//         {
//             return STATUS_PRIVILEGE_NOT_HELD;
//         }
//
//         _SEH2_TRY
//         {
//             SafeMinimumSize = ProbeForReadLargeInteger(MinimumSize);
//             SafeMaximumSize = ProbeForReadLargeInteger(MaximumSize);
//             PageFileName = ProbeForReadUnicodeString(FileName);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         SafeMinimumSize = *MinimumSize;
//         SafeMaximumSize = *MaximumSize;
//         PageFileName = *FileName;
//     }
//
//     /*
//      * Pagefiles cannot be larger than the platform-specific memory addressable
//      * limits, and of course the minimum should be smaller than the maximum.
//      */
//     if (SafeMinimumSize.QuadPart < MINIMUM_PAGEFILE_SIZE ||
//         SafeMinimumSize.QuadPart > MAXIMUM_PAGEFILE_SIZE)
//     {
//         return STATUS_INVALID_PARAMETER_2;
//     }
//     if (SafeMaximumSize.QuadPart < SafeMinimumSize.QuadPart ||
//         SafeMaximumSize.QuadPart > MAXIMUM_PAGEFILE_SIZE)
//     {
//         return STATUS_INVALID_PARAMETER_3;
//     }
//
//     /* Validate the name length */
//     if ((PageFileName.Length == 0) ||
//         (PageFileName.Length > MAXIMUM_FILENAME_LENGTH))
//     {
//         return STATUS_OBJECT_NAME_INVALID;
//     }
//
//     /* Allocate a buffer to keep the name copy. Note that it is kept only
//      * for information purposes, so it gets allocated in the paged pool,
//      * even if it will be stored in the PagingFile structure, that is
//      * allocated from non-paged pool (see below). */
//     PageFileName.MaximumLength = PageFileName.Length;
//     Buffer = ExAllocatePoolWithTag(PagedPool, PageFileName.Length, TAG_MM);
//     if (Buffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy the name */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(PageFileName.Buffer, PageFileName.Length, sizeof(WCHAR));
//             RtlCopyMemory(Buffer, PageFileName.Buffer, PageFileName.Length);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             ExFreePoolWithTag(Buffer, TAG_MM);
//
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         RtlCopyMemory(Buffer, PageFileName.Buffer, PageFileName.Length);
//     }
//
//     /* Replace caller's buffer with ours */
//     PageFileName.Buffer = Buffer;
//
//     /* Create the security descriptor for the page file */
//     Status = RtlCreateSecurityDescriptor(&SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return Status;
//     }
//
//     /* Create the DACL: we will only allow two SIDs */
//     Count = sizeof(ACL) + (sizeof(ACE) + RtlLengthSid(SeLocalSystemSid)) +
//                           (sizeof(ACE) + RtlLengthSid(SeAliasAdminsSid));
//     Dacl = ExAllocatePoolWithTag(PagedPool, Count, TAG_DACL);
//     if (Dacl == NULL)
//     {
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize the DACL */
//     Status = RtlCreateAcl(Dacl, Count, ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto EarlyQuit;
//
//     /* Grant full access to admins */
//     Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, FILE_ALL_ACCESS, SeAliasAdminsSid);
//     if (!NT_SUCCESS(Status))
//         goto EarlyQuit;
//
//     /* Grant full access to SYSTEM */
//     Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, FILE_ALL_ACCESS, SeLocalSystemSid);
//     if (!NT_SUCCESS(Status))
//         goto EarlyQuit;
//
//     /* Attach the DACL to the security descriptor */
//     Status = RtlSetDaclSecurityDescriptor(&SecurityDescriptor, TRUE, Dacl, FALSE);
//     if (!NT_SUCCESS(Status))
//         goto EarlyQuit;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &PageFileName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                &SecurityDescriptor);
//
//     /* Make sure we can at least store a complete page:
//      * If we have 2048 BytesPerAllocationUnit (FAT16 < 128MB) there is
//      * a problem if the paging file is fragmented. Suppose the first cluster
//      * of the paging file is cluster 3042 but cluster 3043 is NOT part of the
//      * paging file but of another file. We can't write a complete page (4096
//      * bytes) to the physical location of cluster 3042 then. */
//     AllocationSize.QuadPart = SafeMinimumSize.QuadPart + PAGE_SIZE;
//
//     /* First, attempt to replace the page file, if existing */
//     Status = IoCreateFile(&FileHandle,
//                           SYNCHRONIZE | WRITE_DAC | FILE_READ_DATA | FILE_WRITE_DATA,
//                           &ObjectAttributes,
//                           &IoStatus,
//                           &AllocationSize,
//                           FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
//                           FILE_SHARE_WRITE,
//                           FILE_SUPERSEDE,
//                           FILE_DELETE_ON_CLOSE | FILE_NO_COMPRESSION | FILE_NO_INTERMEDIATE_BUFFERING,
//                           NULL,
//                           0,
//                           CreateFileTypeNone,
//                           NULL,
//                           IO_OPEN_PAGING_FILE | IO_NO_PARAMETER_CHECKING);
//     /* If we failed, relax a bit constraints, someone may be already holding the
//      * the file, so share write, don't attempt to replace and don't delete on close
//      * (basically, don't do anything conflicting).
//      * This can happen if the caller attempts to extend a page file.
//      */
//     if (!NT_SUCCESS(Status))
//     {
//         ULONG i;
//
//         Status = IoCreateFile(&FileHandle,
//                               SYNCHRONIZE | FILE_WRITE_DATA,
//                               &ObjectAttributes,
//                               &IoStatus,
//                               &AllocationSize,
//                               FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
//                               FILE_SHARE_WRITE | FILE_SHARE_READ,
//                               FILE_OPEN,
//                               FILE_NO_COMPRESSION | FILE_NO_INTERMEDIATE_BUFFERING,
//                               NULL,
//                               0,
//                               CreateFileTypeNone,
//                               NULL,
//                               IO_OPEN_PAGING_FILE | IO_NO_PARAMETER_CHECKING);
//         if (!NT_SUCCESS(Status))
//             goto EarlyQuit;
//
//         /* We opened it! Check we are that "someone" ;-)
//          * First, get the opened file object.
//          */
//         Status = ObReferenceObjectByHandle(FileHandle,
//                                            FILE_READ_DATA | FILE_WRITE_DATA,
//                                            IoFileObjectType as *mut ntkrnl_sys::ddk::_OBJECT_TYPE,
//                                            KernelMode,
//                                            (PVOID*)&FileObject,
//                                            NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             ZwClose(FileHandle);
//             goto EarlyQuit;
//         }
//
//         /* Find if it matches a previous page file */
//         PagingFile = NULL;
//
//         KeAcquireGuardedMutex(&MmPageFileCreationLock);
//
//         for (i = 0; i < MmNumberOfPagingFiles; ++i)
//         {
//             if (MmPagingFile[i]->FileObject->SectionObjectPointer == FileObject->SectionObjectPointer)
//             {
//                 /* Same object pointer: this is the matching page file */
//                 PagingFile = MmPagingFile[i];
//                 break;
//             }
//         }
//
//         /* If we didn't find the page file, fail */
//         if (PagingFile == NULL)
//         {
//             KeReleaseGuardedMutex(&MmPageFileCreationLock);
//             ObDereferenceObject(FileObject);
//             ZwClose(FileHandle);
//             Status = STATUS_NOT_FOUND;
//             goto EarlyQuit;
//         }
//
//         /* Don't allow page file shrinking */
//         if (PagingFile->MinimumSize > (SafeMinimumSize.QuadPart >> PAGE_SHIFT))
//         {
//             KeReleaseGuardedMutex(&MmPageFileCreationLock);
//             ObDereferenceObject(FileObject);
//             ZwClose(FileHandle);
//             Status = STATUS_INVALID_PARAMETER_2;
//             goto EarlyQuit;
//         }
//
//         if ((SafeMaximumSize.QuadPart >> PAGE_SHIFT) < PagingFile->MaximumSize)
//         {
//             KeReleaseGuardedMutex(&MmPageFileCreationLock);
//             ObDereferenceObject(FileObject);
//             ZwClose(FileHandle);
//             Status = STATUS_INVALID_PARAMETER_3;
//             goto EarlyQuit;
//         }
//
//         /* FIXME: implement parameters checking and page file extension */
//         UNIMPLEMENTED;
//
//         KeReleaseGuardedMutex(&MmPageFileCreationLock);
//         ObDereferenceObject(FileObject);
//         ZwClose(FileHandle);
//         Status = STATUS_NOT_IMPLEMENTED;
//         goto EarlyQuit;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
// EarlyQuit:
//         DPRINT1("Failed creating page file: %lx\n", Status);
//         ExFreePoolWithTag(Dacl, TAG_DACL);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return Status;
//     }
//
//     /* Set the security descriptor */
//     if (NT_SUCCESS(IoStatus.Status))
//     {
//         Status = ZwSetSecurityObject(FileHandle, DACL_SECURITY_INFORMATION, &SecurityDescriptor);
//         if (!NT_SUCCESS(Status))
//         {
//             ZwClose(FileHandle);
//             ExFreePoolWithTag(Dacl, TAG_DACL);
//             ExFreePoolWithTag(Buffer, TAG_MM);
//             return Status;
//         }
//     }
//
//     /* DACL is no longer needed, free it */
//     ExFreePoolWithTag(Dacl, TAG_DACL);
//
//     /* FIXME: To enable once page file management is moved to ARM3 */
// #if 0
//     /* Check we won't overflow commit limit with the page file */
//     if (MmTotalCommitLimitMaximum + (SafeMaximumSize.QuadPart >> PAGE_SHIFT) <= MmTotalCommitLimitMaximum)
//     {
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return STATUS_INVALID_PARAMETER_3;
//     }
// #endif
//
//     /* Set its end of file to minimal size */
//     Status = ZwSetInformationFile(FileHandle,
//                                   &IoStatus,
//                                   &SafeMinimumSize,
//                                   sizeof(LARGE_INTEGER),
//                                   FileEndOfFileInformation);
//     if (!NT_SUCCESS(Status) || !NT_SUCCESS(IoStatus.Status))
//     {
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return Status;
//     }
//
//     Status = ObReferenceObjectByHandle(FileHandle,
//                                        FILE_READ_DATA | FILE_WRITE_DATA,
//                                        IoFileObjectType as *mut ntkrnl_sys::ddk::_OBJECT_TYPE,
//                                        KernelMode,
//                                        (PVOID*)&FileObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return Status;
//     }
//
//     /* Only allow page file on a few device types */
//     DeviceType = IoGetRelatedDeviceObject(FileObject)->DeviceType;
//     if (DeviceType != FILE_DEVICE_DISK_FILE_SYSTEM &&
//         DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM &&
//         DeviceType != FILE_DEVICE_DFS_VOLUME &&
//         DeviceType != FILE_DEVICE_DFS_FILE_SYSTEM)
//     {
//         ObDereferenceObject(FileObject);
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return Status;
//     }
//
//     /* Deny page file creation on a floppy disk */
//     FsDeviceInfo.Characteristics = 0;
//     IoQueryVolumeInformation(FileObject, FileFsDeviceInformation,
//                              sizeof(FsDeviceInfo), &FsDeviceInfo, &Count);
//     if (BooleanFlagOn(FsDeviceInfo.Characteristics, FILE_FLOPPY_DISKETTE))
//     {
//         ObDereferenceObject(FileObject);
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return STATUS_FLOPPY_VOLUME;
//     }
//
//     /*
//      * Missing validation steps TODO:
//      * (see https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/mm/modwrite/create.htm )
//      *
//      * - Verify that no file system driver or any filter driver has done file
//      *   I/O while opening the file.
//      *   Verify that nothing of the paging file is yet in memory. Specifically,
//      *   the file object must either have no SectionObjectPointer or the latter
//      *   must have neither a DataSectionObject nor an ImageSectionObject.
//      *   Otherwise, we should fail, returning STATUS_INCOMPATIBLE_FILE_MAP.
//      *
//      * - Inform all the applicable drivers to prepare for the possibility of
//      *   paging I/O. Much of the point to paging I/O is to resolve page faults.
//      *   Especially important is that drivers that handle paging I/O do not
//      *   cause more page faults. All the code and data that each driver might
//      *   ever use for access to the paging file must be locked into physical
//      *   memory. This can’t be left until paging I/O actually occurs.
//      *   It must be done in advance.
//      */
//
//     PagingFile = ExAllocatePoolZero(NonPagedPool, sizeof(*PagingFile), TAG_MM);
//     if (PagingFile == NULL)
//     {
//         ObDereferenceObject(FileObject);
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     PagingFile->FileHandle = FileHandle;
//     PagingFile->FileObject = FileObject;
//     PagingFile->Size = (SafeMinimumSize.QuadPart >> PAGE_SHIFT);
//     PagingFile->MinimumSize = PagingFile->Size;
//     PagingFile->MaximumSize = (SafeMaximumSize.QuadPart >> PAGE_SHIFT);
//     /* First page is never used: it's the header
//      * TODO: write it
//      */
//     PagingFile->FreeSpace = PagingFile->Size - 1;
//     PagingFile->CurrentUsage = 0;
//     PagingFile->PageFileName = PageFileName;
//     ASSERT(PagingFile->Size == PagingFile->FreeSpace + PagingFile->CurrentUsage + 1);
//
//     AllocMapSize = sizeof(RTL_BITMAP) + (((PagingFile->MaximumSize + 31) / 32) * sizeof(ULONG));
//     PagingFile->Bitmap = ExAllocatePoolWithTag(NonPagedPool,
//                                                AllocMapSize,
//                                                TAG_MM);
//     if (PagingFile->Bitmap == NULL)
//     {
//         ExFreePoolWithTag(PagingFile, TAG_MM);
//         ObDereferenceObject(FileObject);
//         ZwClose(FileHandle);
//         ExFreePoolWithTag(Buffer, TAG_MM);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlInitializeBitMap(PagingFile->Bitmap,
//                         (PULONG)(PagingFile->Bitmap + 1),
//                         (ULONG)(PagingFile->MaximumSize));
//     RtlClearAllBits(PagingFile->Bitmap);
//
//     /* Insert the new paging file information into the list */
//     KeAcquireGuardedMutex(&MmPageFileCreationLock);
//     /* Ensure the corresponding slot is empty yet */
//     ASSERT(MmPagingFile[MmNumberOfPagingFiles] == NULL);
//     MmPagingFile[MmNumberOfPagingFiles] = PagingFile;
//     MmNumberOfPagingFiles++;
//     MiFreeSwapPages = MiFreeSwapPages + PagingFile->FreeSpace;
//     KeReleaseGuardedMutex(&MmPageFileCreationLock);
//
//     MmSwapSpaceMessage = FALSE;
//
//     if (!MmSystemPageFileLocated && BooleanFlagOn(FileObject->DeviceObject->Flags, DO_SYSTEM_BOOT_PARTITION))
//     {
//         MmSystemPageFileLocated = IoInitializeCrashDump(FileHandle);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
