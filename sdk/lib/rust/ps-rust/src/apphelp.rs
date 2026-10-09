use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::mm::PagedPool;

static mut ApphelpCacheEnabled: BOOLEAN = 0u8;
static mut ApphelpCacheLock: ERESOURCE = unsafe { core::mem::zeroed() };
static mut ApphelpShimCache: RTL_AVL_TABLE = unsafe { core::mem::zeroed() };
static mut ApphelpShimCacheAge: LIST_ENTRY = unsafe { core::mem::zeroed() };

extern "system" {
    static mut InitSafeBootMode: ULONG;
    static SeTcbPrivilege: LUID;
}

// L"\Registry\MACHINE\System\CurrentControlSet\Control\Session Manager\AppCompatCache"
static APP_COMPAT_CACHE_KEY_BUF: [u16; 82] = [
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
    b'M' as u16, b'A' as u16, b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16, b'\\' as u16, b'S' as u16, b'y' as u16,
    b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
    b'e' as u16, b't' as u16, b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
    b'\\' as u16, b'S' as u16, b'e' as u16, b's' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b' ' as u16, b'M' as u16,
    b'a' as u16, b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, b'\\' as u16, b'A' as u16, b'p' as u16, b'p' as u16,
    b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'a' as u16, b't' as u16, b'C' as u16, b'a' as u16, b'c' as u16, b'h' as u16,
    b'e' as u16, 0,
];
static mut AppCompatCacheKey: _UNICODE_STRING = _UNICODE_STRING {
    Length: 162,
    MaximumLength: 164,
    Buffer: APP_COMPAT_CACHE_KEY_BUF.as_ptr() as *mut u16,
};

static mut AppCompatKeyAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
    Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
    RootDirectory: core::ptr::null_mut(),
    ObjectName: { core::ptr::addr_of_mut!(AppCompatCacheKey) },
    Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
    SecurityDescriptor: core::ptr::null_mut(),
    SecurityQualityOfService: core::ptr::null_mut(),
};

// L"AppCompatCache" — UTF-16LE
static APPCOMPAT_CACHE_BUF: [u16; 15] = [
    0x41, 0x70, 0x70, 0x43, 0x6F, 0x6D, 0x70, 0x61,  // AppCompa
    0x74, 0x43, 0x61, 0x63, 0x68, 0x65, 0x00,          // tCache\0
];
static mut AppCompatCacheValue: _UNICODE_STRING = _UNICODE_STRING {
    Length: 28,        // 14 chars * 2
    MaximumLength: 30, // 15 u16s * 2
    Buffer: APPCOMPAT_CACHE_BUF.as_ptr() as *mut u16,
};

const MAX_SHIM_ENTRIES: ULONG = 0x200;
const INVALID_HANDLE_VALUE: HANDLE = (-1isize) as HANDLE;

#[repr(C, packed(1))]
struct SHIM_PERSISTENT_CACHE_HEADER_52 {
    Magic: ULONG,
    NumEntries: ULONG,
}

#[repr(C, packed(1))]
#[derive(Copy, Clone)]
struct SHIM_PERSISTENT_CACHE_ENTRY_52 {
    ImageName: _UNICODE_STRING,
    DateTime: LARGE_INTEGER,
    FileSize: LARGE_INTEGER,
}

const CACHE_MAGIC_NT_52: ULONG = 0xbadc0ffe;
const CACHE_HEADER_SIZE_NT_52: ULONG = 0x8;
#[cfg(target_pointer_width = "64")]
const NT52_PERSISTENT_ENTRY_SIZE64: ULONG = 0x20;
#[cfg(not(target_pointer_width = "64"))]
const NT52_PERSISTENT_ENTRY_SIZE32: ULONG = 0x18;

const SHIM_CACHE_MAGIC: ULONG = CACHE_MAGIC_NT_52;
const SHIM_CACHE_HEADER_SIZE: ULONG = CACHE_HEADER_SIZE_NT_52;
#[cfg(target_pointer_width = "64")]
const SHIM_PERSISTENT_CACHE_ENTRY_SIZE: ULONG = NT52_PERSISTENT_ENTRY_SIZE64;
#[cfg(not(target_pointer_width = "64"))]
const SHIM_PERSISTENT_CACHE_ENTRY_SIZE: ULONG = NT52_PERSISTENT_ENTRY_SIZE32;

type SHIM_PERSISTENT_CACHE_HEADER = SHIM_PERSISTENT_CACHE_HEADER_52;
type PSHIM_PERSISTENT_CACHE_HEADER = *mut SHIM_PERSISTENT_CACHE_HEADER_52;
type SHIM_PERSISTENT_CACHE_ENTRY = SHIM_PERSISTENT_CACHE_ENTRY_52;
type PSHIM_PERSISTENT_CACHE_ENTRY = *mut SHIM_PERSISTENT_CACHE_ENTRY_52;

#[repr(C)]
struct SHIM_CACHE_ENTRY {
    List: LIST_ENTRY,
    Persistent: SHIM_PERSISTENT_CACHE_ENTRY,
    CompatFlags: ULONG,
}
type PSHIM_CACHE_ENTRY = *mut SHIM_CACHE_ENTRY;

unsafe fn ApphelpAlloc(ByteSize: ULONG) -> PVOID {
    ExAllocatePoolWithTag(PagedPool, ByteSize as SIZE_T, TAG_SHIM)
}

unsafe fn ApphelpFree(Data: PVOID) {
    ExFreePoolWithTag(Data, TAG_SHIM);
}

unsafe fn ApphelpCacheAcquireLock() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(ApphelpCacheLock), 1u8);
}

unsafe fn ApphelpCacheTryAcquireLock() -> BOOLEAN {
    KeEnterCriticalRegion();
    if ExTryToAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(ApphelpCacheLock)) == 0 {
        KeLeaveCriticalRegion();
        return 0u8;
    }
    1u8
}

unsafe fn ApphelpCacheReleaseLock() {
    ExReleaseResourceLite(core::ptr::addr_of_mut!(ApphelpCacheLock));
    KeLeaveCriticalRegion();
}

unsafe fn ApphelpDuplicateUnicodeString(Destination: PUNICODE_STRING, Source: PCUNICODE_STRING) {
    (*Destination).Length = (*Source).Length;
    if (*Destination).Length != 0 {
        (*Destination).MaximumLength = (*Destination).Length + core::mem::size_of::<u16>() as u16;
        (*Destination).Buffer = ApphelpAlloc((*Destination).MaximumLength as ULONG) as *mut u16;
        core::ptr::copy_nonoverlapping(
            (*Source).Buffer as *const u8,
            (*Destination).Buffer as *mut u8,
            (*Destination).Length as usize,
        );
        *(*Destination)
            .Buffer
            .add(((*Destination).Length / core::mem::size_of::<u16>() as u16) as usize) = 0;
    } else {
        (*Destination).MaximumLength = 0;
        (*Destination).Buffer = core::ptr::null_mut();
    }
}

unsafe fn ApphelpFreeUnicodeString(String: PUNICODE_STRING) {
    if !(*String).Buffer.is_null() {
        ApphelpFree((*String).Buffer as PVOID);
    }
    (*String).Length = 0;
    (*String).MaximumLength = 0;
    (*String).Buffer = core::ptr::null_mut();
}

unsafe fn ApphelpCacheQueryInfo(ImageHandle: HANDLE, Entry: PSHIM_CACHE_ENTRY) -> NTSTATUS {
    let mut io_status: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut file_basic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut file_standard: FILE_STANDARD_INFORMATION = core::mem::zeroed();

    let mut status = ZwQueryInformationFile(
        ImageHandle,
        core::ptr::addr_of_mut!(io_status),
        core::ptr::addr_of_mut!(file_basic) as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        // FileBasicInformation (4); a literal 0 is no class at all and the query failed.
        ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileBasicInformation as _,
    );
    if status < 0 {
        return status;
    }

    status = ZwQueryInformationFile(
        ImageHandle,
        core::ptr::addr_of_mut!(io_status),
        core::ptr::addr_of_mut!(file_standard) as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileStandardInformation as _,
    );
    if status >= 0 {
        (*Entry).Persistent.DateTime = file_basic.LastWriteTime;
        (*Entry).Persistent.FileSize = file_standard.EndOfFile;
    }
    status
}

// NTAPI in the C (ntoskrnl/ps/apphelp.c) and stdcall in bindgen's i386 transcript of
// PRTL_AVL_COMPARE_ROUTINE, like the Allocate/Free routines handed to the same
// RtlInitializeGenericTableAvl call below. This was `extern "C"`: on i386 the AVL code
// calls it expecting the callee to pop 12 bytes of arguments and ours popped none,
// leaking that much stack per comparison — once per node visited on every cache lookup.
// The transmute at the install site erased the type, so nothing could report it.
unsafe extern "system" fn ApphelpShimCacheCompareRoutine(
    _Table: *mut _RTL_AVL_TABLE,
    FirstStruct: PVOID,
    SecondStruct: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    let first_entry = FirstStruct as PSHIM_CACHE_ENTRY;
    let second_entry = SecondStruct as PSHIM_CACHE_ENTRY;

    let result = RtlCompareUnicodeString(
        core::ptr::addr_of_mut!((*first_entry).Persistent.ImageName),
        core::ptr::addr_of_mut!((*second_entry).Persistent.ImageName),
        1u8,
    );
    if result < 0 {
        0
    } else if result == 0 {
        1
    } else {
        2
    }
}

unsafe extern "system" fn ApphelpShimCacheAllocateRoutine(
    _Table: *mut _RTL_AVL_TABLE,
    ByteSize: CLONG,
) -> PVOID {
    ApphelpAlloc(ByteSize as ULONG)
}

unsafe extern "system" fn ApphelpShimCacheFreeRoutine(_Table: *mut _RTL_AVL_TABLE, Buffer: PVOID) {
    ApphelpFree(Buffer);
}

unsafe fn ApphelpCacheParse(Data: *mut u8, DataLength: ULONG) -> NTSTATUS {
    if DataLength < CACHE_HEADER_SIZE_NT_52 {
        return STATUS_INVALID_PARAMETER;
    }

    let header = Data as PSHIM_PERSISTENT_CACHE_HEADER;
    if (*header).Magic != SHIM_CACHE_MAGIC {
        return STATUS_INVALID_PARAMETER;
    }

    let num_entries = (*header).NumEntries;
    for cur in 0..num_entries {
        let persistent = (Data as usize
            + SHIM_CACHE_HEADER_SIZE as usize
            + (cur * SHIM_PERSISTENT_CACHE_ENTRY_SIZE) as usize)
            as PSHIM_PERSISTENT_CACHE_ENTRY;

        let mut string: _UNICODE_STRING = core::mem::zeroed();
        string.Length = (*persistent).ImageName.Length;
        string.MaximumLength = (*persistent).ImageName.MaximumLength;
        string.Buffer = ((*persistent).ImageName.Buffer as usize + Data as usize) as *mut u16;

        let mut entry: SHIM_CACHE_ENTRY = core::mem::zeroed();
        entry.Persistent = *persistent;
        ApphelpDuplicateUnicodeString(
            core::ptr::addr_of_mut!(entry.Persistent.ImageName),
            core::ptr::addr_of_mut!(string) as PCUNICODE_STRING,
        );

        let result = RtlInsertElementGenericTableAvl(
            core::ptr::addr_of_mut!(ApphelpShimCache),
            core::ptr::addr_of_mut!(entry) as PVOID,
            core::mem::size_of::<SHIM_CACHE_ENTRY>() as ULONG,
            core::ptr::null_mut(),
        ) as PSHIM_CACHE_ENTRY;
        if result.is_null() {
            ApphelpFreeUnicodeString(core::ptr::addr_of_mut!(entry.Persistent.ImageName));
            return STATUS_INVALID_PARAMETER;
        }
        InsertTailList(
            core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*result).List) as PLIST_ENTRY,
        );
    }
    STATUS_SUCCESS
}

unsafe fn ApphelpCacheRead() -> BOOLEAN {
    let mut key_handle: HANDLE = core::ptr::null_mut();
    let mut status = ZwOpenKey(
        core::ptr::addr_of_mut!(key_handle),
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(AppCompatKeyAttributes),
    );
    if status < 0 {
        return 0u8;
    }

    let mut key_value_object: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
    let mut key_value_information = core::ptr::addr_of_mut!(key_value_object);
    let mut result_size: ULONG = 0;

    status = ZwQueryValueKey(
        key_handle,
        core::ptr::addr_of_mut!(AppCompatCacheValue),
        2,
        key_value_information as PVOID,
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(result_size),
    );
    if status == STATUS_BUFFER_OVERFLOW {
        let key_info_size = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG
            + (*key_value_information).DataLength;
        key_value_information = ApphelpAlloc(key_info_size) as PKEY_VALUE_PARTIAL_INFORMATION;
        if !key_value_information.is_null() {
            status = ZwQueryValueKey(
                key_handle,
                core::ptr::addr_of_mut!(AppCompatCacheValue),
                2,
                key_value_information as PVOID,
                key_info_size,
                core::ptr::addr_of_mut!(result_size),
            );
        }
    }

    if status >= 0 && (*key_value_information).Type == REG_BINARY {
        status = ApphelpCacheParse(
            (*key_value_information).Data.as_mut_ptr(),
            (*key_value_information).DataLength,
        );
    }

    if key_value_information != core::ptr::addr_of_mut!(key_value_object)
        && !key_value_information.is_null()
    {
        ApphelpFree(key_value_information as PVOID);
    }

    ZwClose(key_handle);
    if status >= 0 {
        1u8
    } else {
        0u8
    }
}

unsafe fn ApphelpCacheWrite() -> BOOLEAN {
    let mut length = SHIM_CACHE_HEADER_SIZE;
    let mut num_entries: ULONG = 0;

    ApphelpCacheAcquireLock();
    let mut list_entry = ApphelpShimCacheAge.Flink;
    while list_entry != core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY {
        let entry = (list_entry as usize - core::mem::offset_of!(SHIM_CACHE_ENTRY, List))
            as PSHIM_CACHE_ENTRY;
        length += SHIM_PERSISTENT_CACHE_ENTRY_SIZE;
        length += (*entry).Persistent.ImageName.MaximumLength as ULONG;
        num_entries += 1;
        list_entry = (*list_entry).Flink;
    }
    length = (length + 7) & !7;

    let buffer = ApphelpAlloc(length) as *mut u8;
    let mut buffer_name_pos = buffer.add(length as usize);
    let header = buffer as PSHIM_PERSISTENT_CACHE_HEADER;
    let mut write_entry =
        buffer.add(SHIM_CACHE_HEADER_SIZE as usize) as PSHIM_PERSISTENT_CACHE_ENTRY;

    (*header).Magic = SHIM_CACHE_MAGIC;
    (*header).NumEntries = num_entries;

    list_entry = ApphelpShimCacheAge.Flink;
    while list_entry != core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY {
        let entry = (list_entry as usize - core::mem::offset_of!(SHIM_CACHE_ENTRY, List))
            as PSHIM_CACHE_ENTRY;
        let image_name_len = (*entry).Persistent.ImageName.MaximumLength;
        *write_entry = (*entry).Persistent;
        buffer_name_pos = buffer_name_pos.sub(image_name_len as usize);
        core::ptr::copy_nonoverlapping(
            (*entry).Persistent.ImageName.Buffer as *const u8,
            buffer_name_pos,
            image_name_len as usize,
        );
        (*write_entry).ImageName.Buffer =
            (buffer_name_pos as usize - buffer as usize) as *mut u16;

        write_entry = write_entry.add(1);
        list_entry = (*list_entry).Flink;
    }
    ApphelpCacheReleaseLock();

    let mut key_handle: HANDLE = core::ptr::null_mut();
    let mut status = ZwOpenKey(
        core::ptr::addr_of_mut!(key_handle),
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(AppCompatKeyAttributes),
    );
    if status >= 0 {
        status = ZwSetValueKey(
            key_handle,
            core::ptr::addr_of_mut!(AppCompatCacheValue),
            0,
            REG_BINARY,
            buffer as PVOID,
            length,
        );
        ZwClose(key_handle);
    }

    ApphelpFree(buffer as PVOID);
    if status >= 0 {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn ApphelpCacheInitialize() -> NTSTATUS {
    if InitSafeBootMode != 0 {
        ApphelpCacheEnabled = 0u8;
    } else {
        ExInitializeResourceLite(core::ptr::addr_of_mut!(ApphelpCacheLock));
        RtlInitializeGenericTableAvl(
            core::ptr::addr_of_mut!(ApphelpShimCache),
            core::mem::transmute(ApphelpShimCacheCompareRoutine as usize),
            core::mem::transmute(ApphelpShimCacheAllocateRoutine as usize),
            core::mem::transmute(ApphelpShimCacheFreeRoutine as usize),
            core::ptr::null_mut(),
        );
        InitializeListHead(core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY);
        ApphelpCacheEnabled = ApphelpCacheRead();
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ApphelpCacheShutdown() {
    if ApphelpCacheEnabled != 0 {
        ApphelpCacheWrite();
    }
}

unsafe fn ApphelpValidateData(
    ServiceData: PAPPHELP_CACHE_SERVICE_LOOKUP,
    ImageName: PUNICODE_STRING,
    ImageHandle: PHANDLE,
) -> NTSTATUS {
    let mut status = STATUS_INVALID_PARAMETER;
    let previous_mode = ExGetPreviousMode();

    if !ServiceData.is_null() {
        let mut local_image_name: _UNICODE_STRING = core::mem::zeroed();
        if let Err(s) = probe_user_buffer(
            ServiceData as *const core::ffi::c_void,
            core::mem::size_of::<APPHELP_CACHE_SERVICE_LOOKUP>(),
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return s;
        }
        // One fault-tolerant capture of the caller's lookup record (the C reads it
        // field by field inside its __try, whose handler returns the code).
        let service = match read_back(ServiceData as *const APPHELP_CACHE_SERVICE_LOOKUP) {
            Ok(v) => v,
            Err(s) => return s,
        };
        local_image_name = service.ImageName;
        // ImageHandle is the caller's KERNEL local. It was stored with
        // probe_write_user, whose user-range probe rejects a kernel stack address,
        // so every user-mode lookup returned STATUS_ACCESS_VIOLATION.
        *ImageHandle = service.ImageHandle;
        if local_image_name.Length != 0 && !local_image_name.Buffer.is_null() {
            if let Err(s) = probe_user_buffer(
                local_image_name.Buffer as *const core::ffi::c_void,
                (local_image_name.Length as usize) * core::mem::size_of::<u16>(),
                1,
                previous_mode,
            ) {
                return s;
            }
            // ApphelpDuplicateUnicodeString copies raw; the source is the caller's.
            let length = local_image_name.Length;
            let buffer = ApphelpAlloc(length as ULONG + core::mem::size_of::<u16>() as ULONG) as *mut u16;
            if buffer.is_null() {
                return STATUS_NO_MEMORY; // the C's duplicate writes through NULL here
            }
            if let Err(s) = safe_copy_raw(buffer.cast(), local_image_name.Buffer as *const _, length as usize) {
                ApphelpFree(buffer as PVOID);
                return s;
            }
            *buffer.add((length / core::mem::size_of::<u16>() as u16) as usize) = 0;
            (*ImageName).Length = length;
            (*ImageName).MaximumLength = length + core::mem::size_of::<u16>() as u16;
            (*ImageName).Buffer = buffer;
            status = STATUS_SUCCESS;
        }
    }
    status
}

unsafe fn ApphelpCacheRemoveEntryNolock(Entry: PSHIM_CACHE_ENTRY) -> NTSTATUS {
    if !Entry.is_null() {
        let buffer = (*Entry).Persistent.ImageName.Buffer;
        RemoveEntryList(core::ptr::addr_of_mut!((*Entry).List) as PLIST_ENTRY);
        if RtlDeleteElementGenericTableAvl(
            core::ptr::addr_of_mut!(ApphelpShimCache),
            Entry as PVOID,
        ) != 0
        {
            ApphelpFree(buffer as PVOID);
        }
        return STATUS_SUCCESS;
    }
    STATUS_NOT_FOUND
}

unsafe fn ApphelpCacheLookupEntry(ImageName: PUNICODE_STRING, ImageHandle: HANDLE) -> NTSTATUS {
    let mut status = STATUS_NOT_FOUND;

    if ApphelpCacheTryAcquireLock() == 0 {
        return status;
    }

    let mut lookup: SHIM_CACHE_ENTRY = core::mem::zeroed();
    lookup.Persistent.ImageName = *ImageName;
    let entry = RtlLookupElementGenericTableAvl(
        core::ptr::addr_of_mut!(ApphelpShimCache),
        core::ptr::addr_of_mut!(lookup) as PVOID,
    ) as PSHIM_CACHE_ENTRY;
    if entry.is_null() {
        ApphelpCacheReleaseLock();
        return status;
    }

    if ImageHandle == INVALID_HANDLE_VALUE {
        status = STATUS_SUCCESS;
    } else {
        status = ApphelpCacheQueryInfo(ImageHandle, core::ptr::addr_of_mut!(lookup));
        if status >= 0
            && lookup.Persistent.DateTime.QuadPart == (*entry).Persistent.DateTime.QuadPart
            && lookup.Persistent.FileSize.QuadPart == (*entry).Persistent.FileSize.QuadPart
        {
            status = STATUS_SUCCESS;
            RemoveEntryList(core::ptr::addr_of_mut!((*entry).List) as PLIST_ENTRY);
            InsertHeadList(
                core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*entry).List) as PLIST_ENTRY,
            );
        } else {
            status = STATUS_NOT_FOUND;
            ApphelpCacheRemoveEntryNolock(entry);
        }
    }

    ApphelpCacheReleaseLock();
    status
}

unsafe fn ApphelpCacheRemoveEntry(ImageName: PUNICODE_STRING) -> NTSTATUS {
    ApphelpCacheAcquireLock();
    let entry = RtlLookupElementGenericTableAvl(
        core::ptr::addr_of_mut!(ApphelpShimCache),
        ImageName as PVOID,
    ) as PSHIM_CACHE_ENTRY;
    let status = ApphelpCacheRemoveEntryNolock(entry);
    ApphelpCacheReleaseLock();
    status
}

unsafe fn ApphelpCacheAccessCheck() -> NTSTATUS {
    if ExGetPreviousMode() != 0 {
        if SeSinglePrivilegeCheck(SeTcbPrivilege, 1i8) == 0 {
            return STATUS_ACCESS_DENIED;
        }
    }
    STATUS_SUCCESS
}

unsafe fn ApphelpCacheUpdateEntry(ImageName: PUNICODE_STRING, ImageHandle: HANDLE) -> NTSTATUS {
    let mut status = STATUS_SUCCESS;
    let mut entry: SHIM_CACHE_ENTRY = core::mem::zeroed();
    let mut node_or_parent: PVOID = core::ptr::null_mut();
    let mut search_result: _TABLE_SEARCH_RESULT = 0;

    ApphelpCacheAcquireLock();

    if ImageHandle != INVALID_HANDLE_VALUE {
        status = ApphelpCacheQueryInfo(ImageHandle, core::ptr::addr_of_mut!(entry));
        if status < 0 {
            ApphelpCacheReleaseLock();
            return status;
        }
    }

    entry.Persistent.ImageName = *ImageName;
    let mut lookup = RtlLookupElementGenericTableFullAvl(
        core::ptr::addr_of_mut!(ApphelpShimCache),
        core::ptr::addr_of_mut!(entry) as PVOID,
        core::ptr::addr_of_mut!(node_or_parent),
        core::ptr::addr_of_mut!(search_result),
    ) as PSHIM_CACHE_ENTRY;
    if !lookup.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*lookup).List) as PLIST_ENTRY);
        (*lookup).Persistent.DateTime = entry.Persistent.DateTime;
        (*lookup).Persistent.FileSize = entry.Persistent.FileSize;
    } else {
        ApphelpDuplicateUnicodeString(
            core::ptr::addr_of_mut!(entry.Persistent.ImageName),
            ImageName as PCUNICODE_STRING,
        );
        lookup = RtlInsertElementGenericTableFullAvl(
            core::ptr::addr_of_mut!(ApphelpShimCache),
            core::ptr::addr_of_mut!(entry) as PVOID,
            core::mem::size_of::<SHIM_CACHE_ENTRY>() as ULONG,
            core::ptr::null_mut(),
            node_or_parent,
            search_result,
        ) as PSHIM_CACHE_ENTRY;
        if lookup.is_null() {
            ApphelpFreeUnicodeString(core::ptr::addr_of_mut!(entry.Persistent.ImageName));
            status = STATUS_NO_MEMORY;
        }
    }
    if !lookup.is_null() {
        InsertHeadList(
            core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*lookup).List) as PLIST_ENTRY,
        );
        if RtlNumberGenericTableElementsAvl(core::ptr::addr_of_mut!(ApphelpShimCache))
            > MAX_SHIM_ENTRIES
        {
            let remove = (ApphelpShimCacheAge.Blink as usize
                - core::mem::offset_of!(SHIM_CACHE_ENTRY, List))
                as PSHIM_CACHE_ENTRY;
            status = ApphelpCacheRemoveEntryNolock(remove);
        }
    }

    ApphelpCacheReleaseLock();
    status
}

unsafe fn ApphelpCacheFlush() -> NTSTATUS {
    ApphelpCacheAcquireLock();
    loop {
        let p =
            RtlEnumerateGenericTableAvl(core::ptr::addr_of_mut!(ApphelpShimCache), 1u8) as PSHIM_CACHE_ENTRY;
        if p.is_null() {
            break;
        }
        ApphelpCacheRemoveEntryNolock(p);
    }
    ApphelpCacheReleaseLock();
    STATUS_SUCCESS
}

unsafe fn ApphelpCacheDump() -> NTSTATUS {
    ApphelpCacheAcquireLock();
    let mut list_entry = ApphelpShimCacheAge.Flink;
    while list_entry != core::ptr::addr_of_mut!(ApphelpShimCacheAge) as PLIST_ENTRY {
        let entry = (list_entry as usize - core::mem::offset_of!(SHIM_CACHE_ENTRY, List))
            as PSHIM_CACHE_ENTRY;
        list_entry = (*list_entry).Flink;
    }
    ApphelpCacheReleaseLock();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtApphelpCacheControl(
    Service: i32,
    ServiceData: PAPPHELP_CACHE_SERVICE_LOOKUP,
) -> NTSTATUS {
    let mut status = STATUS_INVALID_PARAMETER;
    let mut image_name: _UNICODE_STRING = core::mem::zeroed();
    let mut handle: HANDLE = INVALID_HANDLE_VALUE;

    if ApphelpCacheEnabled == 0 {
        return status;
    }
    match Service {
        0 => {
            status = ApphelpValidateData(
                ServiceData,
                core::ptr::addr_of_mut!(image_name),
                core::ptr::addr_of_mut!(handle),
            );
            if status >= 0 {
                status = ApphelpCacheLookupEntry(core::ptr::addr_of_mut!(image_name), handle);
            }
        }
        1 => {
            status = ApphelpValidateData(
                ServiceData,
                core::ptr::addr_of_mut!(image_name),
                core::ptr::addr_of_mut!(handle),
            );
            if status >= 0 {
                status = ApphelpCacheRemoveEntry(core::ptr::addr_of_mut!(image_name));
            }
        }
        2 => {
            status = ApphelpCacheAccessCheck();
            if status >= 0 {
                status = ApphelpValidateData(
                    ServiceData,
                    core::ptr::addr_of_mut!(image_name),
                    core::ptr::addr_of_mut!(handle),
                );
                if status >= 0 {
                    status = ApphelpCacheUpdateEntry(core::ptr::addr_of_mut!(image_name), handle);
                }
            }
        }
        3 => {
            status = ApphelpCacheFlush();
        }
        4 => {
            status = ApphelpCacheDump();
        }
        5 => {
            ApphelpCacheFlush();
            status = if ApphelpCacheRead() != 0 {
                STATUS_SUCCESS
            } else {
                STATUS_NOT_FOUND
            };
        }
        6 => {
            status = if ApphelpCacheWrite() != 0 {
                STATUS_SUCCESS
            } else {
                STATUS_NOT_FOUND
            };
        }
        _ => {}
    }
    if !image_name.Buffer.is_null() {
        ApphelpFreeUnicodeString(core::ptr::addr_of_mut!(image_name));
    }
    status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ps/apphelp.c
//  * PURPOSE:         SHIM engine caching.
//  *                  This caching speeds up checks for the apphelp compatibility layer.
//  * PROGRAMMERS:     Mark Jansen
//  */
//
// /*
// Useful references:
// https://github.com/mandiant/ShimCacheParser/blob/master/ShimCacheParser.py
// https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-7/dd837644(v=ws.10)
// https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database
// https://www.alex-ionescu.com/secrets-of-the-application-compatilibity-database-sdb-part-4/
// https://web.archive.org/web/20170101173150/http://recxltd.blogspot.nl/2012/04/windows-appcompat-research-notes-part-1.html
// http://journeyintoir.blogspot.com/2013/12/revealing-recentfilecachebcf-file.html
// https://web.archive.org/web/20150926070918/https://dl.mandiant.com/EE/library/Whitepaper_ShimCacheParser.pdf
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// static BOOLEAN ApphelpCacheEnabled = FALSE;
// static ERESOURCE ApphelpCacheLock;
// static RTL_AVL_TABLE ApphelpShimCache;
// static LIST_ENTRY ApphelpShimCacheAge;
//
// extern ULONG InitSafeBootMode;
//
// static UNICODE_STRING AppCompatCacheKey = RTL_CONSTANT_STRING(L"\\Registry\\MACHINE\\System\\CurrentControlSet\\Control\\Session Manager\\AppCompatCache");
// static OBJECT_ATTRIBUTES AppCompatKeyAttributes = RTL_CONSTANT_OBJECT_ATTRIBUTES(&AppCompatCacheKey, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE);
// static UNICODE_STRING AppCompatCacheValue = RTL_CONSTANT_STRING(L"AppCompatCache");
//
// #define EMPTY_SHIM_ENTRY    { { 0 }, { { 0 } }, 0 }
// #define MAX_SHIM_ENTRIES    0x200
//
// #ifndef INVALID_HANDLE_VALUE
// #define INVALID_HANDLE_VALUE (HANDLE)(-1)
// #endif
//
// #include <pshpack1.h>
//
// typedef struct SHIM_PERSISTENT_CACHE_HEADER_52
// {
//     ULONG Magic;
//     ULONG NumEntries;
// } SHIM_PERSISTENT_CACHE_HEADER_52, *PSHIM_PERSISTENT_CACHE_HEADER_52;
//
// /* The data that is present in the registry (Win2k3 version) */
// typedef struct SHIM_PERSISTENT_CACHE_ENTRY_52
// {
//     UNICODE_STRING ImageName;
//     LARGE_INTEGER DateTime;
//     LARGE_INTEGER FileSize;
// } SHIM_PERSISTENT_CACHE_ENTRY_52, *PSHIM_PERSISTENT_CACHE_ENTRY_52;
//
// #include <poppack.h>
//
// #define CACHE_MAGIC_NT_52 0xbadc0ffe
// #define CACHE_HEADER_SIZE_NT_52 0x8
// #define NT52_PERSISTENT_ENTRY_SIZE32 0x18
// #define NT52_PERSISTENT_ENTRY_SIZE64 0x20
//
// //#define CACHE_MAGIC_NT_61 0xbadc0fee
// //#define CACHE_HEADER_SIZE_NT_61 0x80
// //#define NT61_PERSISTENT_ENTRY_SIZE32 0x20
// //#define NT61_PERSISTENT_ENTRY_SIZE64 0x30
//
// #define SHIM_CACHE_MAGIC                    CACHE_MAGIC_NT_52
// #define SHIM_CACHE_HEADER_SIZE              CACHE_HEADER_SIZE_NT_52
// #ifdef _WIN64
// #define SHIM_PERSISTENT_CACHE_ENTRY_SIZE    NT52_PERSISTENT_ENTRY_SIZE64
// #else
// #define SHIM_PERSISTENT_CACHE_ENTRY_SIZE    NT52_PERSISTENT_ENTRY_SIZE32
// #endif
// #define SHIM_PERSISTENT_CACHE_HEADER        SHIM_PERSISTENT_CACHE_HEADER_52
// #define PSHIM_PERSISTENT_CACHE_HEADER      PSHIM_PERSISTENT_CACHE_HEADER_52
// #define SHIM_PERSISTENT_CACHE_ENTRY         SHIM_PERSISTENT_CACHE_ENTRY_52
// #define PSHIM_PERSISTENT_CACHE_ENTRY       PSHIM_PERSISTENT_CACHE_ENTRY_52
//
// C_ASSERT(sizeof(SHIM_PERSISTENT_CACHE_ENTRY) == SHIM_PERSISTENT_CACHE_ENTRY_SIZE);
// C_ASSERT(sizeof(SHIM_PERSISTENT_CACHE_HEADER) == SHIM_CACHE_HEADER_SIZE);
//
// /* The struct we keep in memory */
// typedef struct SHIM_CACHE_ENTRY
// {
//     LIST_ENTRY List;
//     SHIM_PERSISTENT_CACHE_ENTRY Persistent;
//     ULONG CompatFlags;
// } SHIM_CACHE_ENTRY, *PSHIM_CACHE_ENTRY;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PVOID
// ApphelpAlloc(
//     _In_ ULONG ByteSize)
// {
//     return ExAllocatePoolWithTag(PagedPool, ByteSize, TAG_SHIM);
// }
//
// VOID
// ApphelpFree(
//     _In_ PVOID Data)
// {
//     ExFreePoolWithTag(Data, TAG_SHIM);
// }
//
// VOID
// ApphelpCacheAcquireLock(VOID)
// {
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&ApphelpCacheLock, TRUE);
// }
//
// BOOLEAN
// ApphelpCacheTryAcquireLock(VOID)
// {
//     KeEnterCriticalRegion();
//     if (!ExTryToAcquireResourceExclusiveLite(&ApphelpCacheLock))
//     {
//         KeLeaveCriticalRegion();
//         return FALSE;
//     }
//     return TRUE;
// }
//
// VOID
// ApphelpCacheReleaseLock(VOID)
// {
//     ExReleaseResourceLite(&ApphelpCacheLock);
//     KeLeaveCriticalRegion();
// }
//
// VOID
// ApphelpDuplicateUnicodeString(
//     _Out_ PUNICODE_STRING Destination,
//     _In_ PCUNICODE_STRING Source)
// {
//     Destination->Length = Source->Length;
//     if (Destination->Length)
//     {
//         Destination->MaximumLength = Destination->Length + sizeof(WCHAR);
//         Destination->Buffer = ApphelpAlloc(Destination->MaximumLength);
//         RtlCopyMemory(Destination->Buffer, Source->Buffer, Destination->Length);
//         Destination->Buffer[Destination->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     else
//     {
//         Destination->MaximumLength = 0;
//         Destination->Buffer = NULL;
//     }
// }
//
// VOID
// ApphelpFreeUnicodeString(
//     _Inout_ PUNICODE_STRING String)
// {
//     if (String->Buffer)
//     {
//         ApphelpFree(String->Buffer);
//     }
//     String->Length = 0;
//     String->MaximumLength = 0;
//     String->Buffer = NULL;
// }
//
// /* Query file info from a handle, storing it in Entry */
// NTSTATUS
// ApphelpCacheQueryInfo(
//     _In_ HANDLE ImageHandle,
//     _Out_ PSHIM_CACHE_ENTRY Entry)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileBasic;
//     FILE_STANDARD_INFORMATION FileStandard;
//     NTSTATUS Status;
//
//     Status = ZwQueryInformationFile(ImageHandle,
//                                     &IoStatusBlock,
//                                     &FileBasic,
//                                     sizeof(FileBasic),
//                                     FileBasicInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     Status = ZwQueryInformationFile(ImageHandle,
//                                     &IoStatusBlock,
//                                     &FileStandard,
//                                     sizeof(FileStandard),
//                                     FileStandardInformation);
//     if (NT_SUCCESS(Status))
//     {
//         Entry->Persistent.DateTime = FileBasic.LastWriteTime;
//         Entry->Persistent.FileSize = FileStandard.EndOfFile;
//     }
//     return Status;
// }
//
// RTL_GENERIC_COMPARE_RESULTS
// NTAPI
// ApphelpShimCacheCompareRoutine(
//     _In_ struct _RTL_AVL_TABLE *Table,
//     _In_ PVOID FirstStruct,
//     _In_ PVOID SecondStruct)
// {
//     PSHIM_CACHE_ENTRY FirstEntry = FirstStruct;
//     PSHIM_CACHE_ENTRY SecondEntry = SecondStruct;
//     LONG Result;
//
//     Result = RtlCompareUnicodeString(&FirstEntry->Persistent.ImageName,
//                                      &SecondEntry->Persistent.ImageName,
//                                      TRUE);
//     if (Result < 0)
//     {
//         return GenericLessThan;
//     }
//     else if (Result == 0)
//     {
//         return GenericEqual;
//     }
//     return GenericGreaterThan;
// }
//
// PVOID
// NTAPI
// ApphelpShimCacheAllocateRoutine(
//     _In_ struct _RTL_AVL_TABLE *Table,
//     _In_ CLONG ByteSize)
// {
//     return ApphelpAlloc(ByteSize);
// }
//
// VOID
// NTAPI
// ApphelpShimCacheFreeRoutine(
//     _In_ struct _RTL_AVL_TABLE *Table,
//     _In_ PVOID Buffer)
// {
//     ApphelpFree(Buffer);
// }
//
// NTSTATUS
// ApphelpCacheParse(
//     _In_reads_(DataLength) PUCHAR Data,
//     _In_ ULONG DataLength)
// {
//     PSHIM_PERSISTENT_CACHE_HEADER Header = (PSHIM_PERSISTENT_CACHE_HEADER)Data;
//     ULONG Cur;
//     ULONG NumEntries;
//     UNICODE_STRING String;
//     SHIM_CACHE_ENTRY Entry = EMPTY_SHIM_ENTRY;
//     PSHIM_CACHE_ENTRY Result;
//     PSHIM_PERSISTENT_CACHE_ENTRY Persistent;
//
//     if (DataLength < CACHE_HEADER_SIZE_NT_52)
//     {
//         DPRINT1("SHIMS: ApphelpCacheParse not enough data for a minimal header (0x%x)\n", DataLength);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (Header->Magic != SHIM_CACHE_MAGIC)
//     {
//         DPRINT1("SHIMS: ApphelpCacheParse found invalid magic (0x%x)\n", Header->Magic);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     NumEntries = Header->NumEntries;
//     DPRINT("SHIMS: ApphelpCacheParse walking %d entries\n", NumEntries);
//     for (Cur = 0; Cur < NumEntries; ++Cur)
//     {
//         Persistent = (PSHIM_PERSISTENT_CACHE_ENTRY)(Data + SHIM_CACHE_HEADER_SIZE +
//                                                     (Cur * SHIM_PERSISTENT_CACHE_ENTRY_SIZE));
//         /* The entry in the Persistent storage is not really a UNICODE_STRING,
//             so we have to convert the offset into a real pointer before using it. */
//         String.Length = Persistent->ImageName.Length;
//         String.MaximumLength = Persistent->ImageName.MaximumLength;
//         String.Buffer = (PWCHAR)((ULONG_PTR)Persistent->ImageName.Buffer + Data);
//
//         /* Now we copy all data to a local buffer, that can be safely duplicated by RtlInsert */
//         Entry.Persistent = *Persistent;
//         ApphelpDuplicateUnicodeString(&Entry.Persistent.ImageName, &String);
//         Result = RtlInsertElementGenericTableAvl(&ApphelpShimCache,
//                                                  &Entry,
//                                                  sizeof(Entry),
//                                                  NULL);
//         if (!Result)
//         {
//             DPRINT1("SHIMS: ApphelpCacheParse insert failed\n");
//             ApphelpFreeUnicodeString(&Entry.Persistent.ImageName);
//             return STATUS_INVALID_PARAMETER;
//         }
//         InsertTailList(&ApphelpShimCacheAge, &Result->List);
//     }
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// ApphelpCacheRead(VOID)
// {
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     KEY_VALUE_PARTIAL_INFORMATION KeyValueObject;
//     PKEY_VALUE_PARTIAL_INFORMATION KeyValueInformation = &KeyValueObject;
//     ULONG KeyInfoSize, ResultSize;
//
//     Status = ZwOpenKey(&KeyHandle, KEY_QUERY_VALUE, &AppCompatKeyAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SHIMS: ApphelpCacheRead could not even open Session Manager\\AppCompatCache (0x%x)\n", Status);
//         return FALSE;
//     }
//
//     Status = ZwQueryValueKey(KeyHandle,
//                              &AppCompatCacheValue,
//                              KeyValuePartialInformation,
//                              KeyValueInformation,
//                              sizeof(KeyValueObject),
//                              &ResultSize);
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         KeyInfoSize = sizeof(KEY_VALUE_PARTIAL_INFORMATION) + KeyValueInformation->DataLength;
//         KeyValueInformation = ApphelpAlloc(KeyInfoSize);
//         if (KeyValueInformation != NULL)
//         {
//             Status = ZwQueryValueKey(KeyHandle,
//                                      &AppCompatCacheValue,
//                                      KeyValuePartialInformation,
//                                      KeyValueInformation,
//                                      KeyInfoSize,
//                                      &ResultSize);
//         }
//     }
//
//     if (NT_SUCCESS(Status) && KeyValueInformation->Type == REG_BINARY)
//     {
//         Status = ApphelpCacheParse(KeyValueInformation->Data,
//                                    KeyValueInformation->DataLength);
//     }
//     else
//     {
//         DPRINT1("SHIMS: ApphelpCacheRead not loaded from registry (0x%x)\n", Status);
//     }
//
//     if (KeyValueInformation != &KeyValueObject && KeyValueInformation != NULL)
//     {
//         ApphelpFree(KeyValueInformation);
//     }
//
//     ZwClose(KeyHandle);
//     return NT_SUCCESS(Status);
// }
//
// BOOLEAN
// ApphelpCacheWrite(VOID)
// {
//     ULONG Length = SHIM_CACHE_HEADER_SIZE;
//     ULONG NumEntries = 0;
//     PLIST_ENTRY ListEntry;
//     PUCHAR Buffer, BufferNamePos;
//     PSHIM_PERSISTENT_CACHE_HEADER Header;
//     PSHIM_PERSISTENT_CACHE_ENTRY WriteEntry;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     /* First we have to calculate the required size. */
//     ApphelpCacheAcquireLock();
//     ListEntry = ApphelpShimCacheAge.Flink;
//     while (ListEntry != &ApphelpShimCacheAge)
//     {
//         PSHIM_CACHE_ENTRY Entry = CONTAINING_RECORD(ListEntry, SHIM_CACHE_ENTRY, List);
//         Length += SHIM_PERSISTENT_CACHE_ENTRY_SIZE;
//         Length += Entry->Persistent.ImageName.MaximumLength;
//         ++NumEntries;
//         ListEntry = ListEntry->Flink;
//     }
//     DPRINT("SHIMS: ApphelpCacheWrite, %d Entries, total size: %d\n", NumEntries, Length);
//     Length = ROUND_UP(Length, sizeof(ULONGLONG));
//     DPRINT("SHIMS: ApphelpCacheWrite, Rounded to: %d\n", Length);
//
//     /* Now we allocate and prepare some helpers */
//     Buffer = ApphelpAlloc(Length);
//     BufferNamePos = Buffer + Length;
//     Header = (PSHIM_PERSISTENT_CACHE_HEADER)Buffer;
//     WriteEntry = (PSHIM_PERSISTENT_CACHE_ENTRY)(Buffer + SHIM_CACHE_HEADER_SIZE);
//
//     Header->Magic = SHIM_CACHE_MAGIC;
//     Header->NumEntries = NumEntries;
//
//     ListEntry = ApphelpShimCacheAge.Flink;
//     while (ListEntry != &ApphelpShimCacheAge)
//     {
//         PSHIM_CACHE_ENTRY Entry = CONTAINING_RECORD(ListEntry, SHIM_CACHE_ENTRY, List);
//         USHORT ImageNameLen = Entry->Persistent.ImageName.MaximumLength;
//         /* Copy the Persistent structure over */
//         *WriteEntry = Entry->Persistent;
//         BufferNamePos -= ImageNameLen;
//         /* Copy the image name over */
//         RtlCopyMemory(BufferNamePos, Entry->Persistent.ImageName.Buffer, ImageNameLen);
//         /* Fix the Persistent structure, so that Buffer is once again an offset */
//         WriteEntry->ImageName.Buffer = (PWCH)(BufferNamePos - Buffer);
//
//         ++WriteEntry;
//         ListEntry = ListEntry->Flink;
//     }
//     ApphelpCacheReleaseLock();
//
//     Status = ZwOpenKey(&KeyHandle, KEY_SET_VALUE, &AppCompatKeyAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         Status = ZwSetValueKey(KeyHandle,
//                                &AppCompatCacheValue,
//                                0,
//                                REG_BINARY,
//                                Buffer,
//                                Length);
//         ZwClose(KeyHandle);
//     }
//     else
//     {
//         DPRINT1("SHIMS: ApphelpCacheWrite could not even open Session Manager\\AppCompatCache (0x%x)\n", Status);
//     }
//
//     ApphelpFree(Buffer);
//     return NT_SUCCESS(Status);
// }
//
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// ApphelpCacheInitialize(VOID)
// {
//     DPRINT("SHIMS: ApphelpCacheInitialize\n");
//     /* If we are booting in safemode we do not want to use the apphelp cache */
//     if (InitSafeBootMode)
//     {
//         DPRINT1("SHIMS: Safe mode detected, disabling cache.\n");
//         ApphelpCacheEnabled = FALSE;
//     }
//     else
//     {
//         ExInitializeResourceLite(&ApphelpCacheLock);
//         RtlInitializeGenericTableAvl(&ApphelpShimCache,
//                                      ApphelpShimCacheCompareRoutine,
//                                      ApphelpShimCacheAllocateRoutine,
//                                      ApphelpShimCacheFreeRoutine,
//                                      NULL);
//         InitializeListHead(&ApphelpShimCacheAge);
//         ApphelpCacheEnabled = ApphelpCacheRead();
//     }
//     DPRINT("SHIMS: ApphelpCacheInitialize: %d\n", ApphelpCacheEnabled);
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// ApphelpCacheShutdown(VOID)
// {
//     if (ApphelpCacheEnabled)
//     {
//         ApphelpCacheWrite();
//     }
// }
//
// NTSTATUS
// ApphelpValidateData(
//     _In_opt_ PAPPHELP_CACHE_SERVICE_LOOKUP ServiceData,
//     _Out_ PUNICODE_STRING ImageName,
//     _Out_ PHANDLE ImageHandle)
// {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//
//     if (ServiceData)
//     {
//         UNICODE_STRING LocalImageName;
//         _SEH2_TRY
//         {
//             ProbeForRead(ServiceData,
//                          sizeof(APPHELP_CACHE_SERVICE_LOOKUP),
//                          sizeof(ULONG));
//             LocalImageName = ServiceData->ImageName;
//             *ImageHandle = ServiceData->ImageHandle;
//             if (LocalImageName.Length && LocalImageName.Buffer)
//             {
//                 ProbeForRead(LocalImageName.Buffer,
//                              LocalImageName.Length * sizeof(WCHAR),
//                              1);
//                 ApphelpDuplicateUnicodeString(ImageName, &LocalImageName);
//                 Status = STATUS_SUCCESS;
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SHIMS: ApphelpValidateData: invalid data passed\n");
//     }
//     return Status;
// }
//
// NTSTATUS
// ApphelpCacheRemoveEntryNolock(
//     _In_ PSHIM_CACHE_ENTRY Entry)
// {
//     if (Entry)
//     {
//         PWSTR Buffer = Entry->Persistent.ImageName.Buffer;
//         RemoveEntryList(&Entry->List);
//         if (RtlDeleteElementGenericTableAvl(&ApphelpShimCache, Entry))
//         {
//             ApphelpFree(Buffer);
//         }
//         return STATUS_SUCCESS;
//     }
//     return STATUS_NOT_FOUND;
// }
//
// NTSTATUS
// ApphelpCacheLookupEntry(
//     _In_ PUNICODE_STRING ImageName,
//     _In_ HANDLE ImageHandle)
// {
//     NTSTATUS Status = STATUS_NOT_FOUND;
//     SHIM_CACHE_ENTRY Lookup = EMPTY_SHIM_ENTRY;
//     PSHIM_CACHE_ENTRY Entry;
//
//     if (!ApphelpCacheTryAcquireLock())
//     {
//         return Status;
//     }
//
//     Lookup.Persistent.ImageName = *ImageName;
//     Entry = RtlLookupElementGenericTableAvl(&ApphelpShimCache, &Lookup);
//     if (Entry == NULL)
//     {
//         DPRINT("SHIMS: ApphelpCacheLookupEntry: could not find %wZ\n", ImageName);
//         goto Cleanup;
//     }
//
//     DPRINT("SHIMS: ApphelpCacheLookupEntry: found %wZ\n", ImageName);
//     if (ImageHandle == INVALID_HANDLE_VALUE)
//     {
//         DPRINT("SHIMS: ApphelpCacheLookupEntry: ok\n");
//         /* just return if we know it, do not query file info */
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         Status = ApphelpCacheQueryInfo(ImageHandle, &Lookup);
//         if (NT_SUCCESS(Status) &&
//             Lookup.Persistent.DateTime.QuadPart == Entry->Persistent.DateTime.QuadPart &&
//             Lookup.Persistent.FileSize.QuadPart == Entry->Persistent.FileSize.QuadPart)
//         {
//             DPRINT("SHIMS: ApphelpCacheLookupEntry: found & validated\n");
//             Status = STATUS_SUCCESS;
//             /* move it to the front to keep it alive */
//             RemoveEntryList(&Entry->List);
//             InsertHeadList(&ApphelpShimCacheAge, &Entry->List);
//         }
//         else
//         {
//             DPRINT1("SHIMS: ApphelpCacheLookupEntry: file info mismatch (%lx)\n", Status);
//             Status = STATUS_NOT_FOUND;
//             /* Could not read file info, or it did not match, drop it from the cache */
//             ApphelpCacheRemoveEntryNolock(Entry);
//         }
//     }
//
// Cleanup:
//     ApphelpCacheReleaseLock();
//     return Status;
// }
//
// NTSTATUS
// ApphelpCacheRemoveEntry(
//     _In_ PUNICODE_STRING ImageName)
// {
//     PSHIM_CACHE_ENTRY Entry;
//     NTSTATUS Status;
//
//     ApphelpCacheAcquireLock();
//     Entry = RtlLookupElementGenericTableAvl(&ApphelpShimCache, ImageName);
//     Status = ApphelpCacheRemoveEntryNolock(Entry);
//     ApphelpCacheReleaseLock();
//     return Status;
// }
//
// /* Validate that we are either called from r0, or from a service-like context */
// NTSTATUS
// ApphelpCacheAccessCheck(VOID)
// {
//     if (ExGetPreviousMode() != KernelMode)
//     {
//         if (!SeSinglePrivilegeCheck(SeTcbPrivilege, UserMode))
//         {
//             DPRINT1("SHIMS: ApphelpCacheAccessCheck failed\n");
//             return STATUS_ACCESS_DENIED;
//         }
//     }
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// ApphelpCacheUpdateEntry(
//     _In_ PUNICODE_STRING ImageName,
//     _In_ HANDLE ImageHandle)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     SHIM_CACHE_ENTRY Entry = EMPTY_SHIM_ENTRY;
//     PSHIM_CACHE_ENTRY Lookup;
//     PVOID NodeOrParent;
//     TABLE_SEARCH_RESULT SearchResult;
//
//     ApphelpCacheAcquireLock();
//
//     /* If we got a file handle, query it for info */
//     if (ImageHandle != INVALID_HANDLE_VALUE)
//     {
//         Status = ApphelpCacheQueryInfo(ImageHandle, &Entry);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//     }
//
//     /* Use ImageName for the lookup, don't actually duplicate it */
//     Entry.Persistent.ImageName = *ImageName;
//     Lookup = RtlLookupElementGenericTableFullAvl(&ApphelpShimCache,
//                                                  &Entry,
//                                                  &NodeOrParent, &SearchResult);
//     if (Lookup)
//     {
//         DPRINT("SHIMS: ApphelpCacheUpdateEntry: Entry already exists, reusing it\n");
//         /* Unlink the found item, so we can put it back at the front,
//             and copy the earlier obtained file info*/
//         RemoveEntryList(&Lookup->List);
//         Lookup->Persistent.DateTime = Entry.Persistent.DateTime;
//         Lookup->Persistent.FileSize = Entry.Persistent.FileSize;
//     }
//     else
//     {
//         DPRINT("SHIMS: ApphelpCacheUpdateEntry: Inserting new Entry\n");
//         /* Insert a new entry, with its own copy of the ImageName */
//         ApphelpDuplicateUnicodeString(&Entry.Persistent.ImageName, ImageName);
//         Lookup = RtlInsertElementGenericTableFullAvl(&ApphelpShimCache,
//                                                      &Entry,
//                                                      sizeof(Entry),
//                                                      0,
//                                                      NodeOrParent,
//                                                      SearchResult);
//         if (!Lookup)
//         {
//             ApphelpFreeUnicodeString(&Entry.Persistent.ImageName);
//             Status = STATUS_NO_MEMORY;
//         }
//     }
//     if (Lookup)
//     {
//         /* Either we re-used an existing item, or we inserted a new one, keep it alive */
//         InsertHeadList(&ApphelpShimCacheAge, &Lookup->List);
//         if (RtlNumberGenericTableElementsAvl(&ApphelpShimCache) > MAX_SHIM_ENTRIES)
//         {
//             PSHIM_CACHE_ENTRY Remove;
//             DPRINT1("SHIMS: ApphelpCacheUpdateEntry: Cache growing too big, dropping oldest item\n");
//             Remove = CONTAINING_RECORD(ApphelpShimCacheAge.Blink, SHIM_CACHE_ENTRY, List);
//             Status = ApphelpCacheRemoveEntryNolock(Remove);
//         }
//     }
//
// Cleanup:
//     ApphelpCacheReleaseLock();
//     return Status;
// }
//
// NTSTATUS
// ApphelpCacheFlush(VOID)
// {
//     PVOID p;
//
//     DPRINT1("SHIMS: ApphelpCacheFlush\n");
//     ApphelpCacheAcquireLock();
//     while ((p = RtlEnumerateGenericTableAvl(&ApphelpShimCache, TRUE)))
//     {
//         ApphelpCacheRemoveEntryNolock((PSHIM_CACHE_ENTRY)p);
//     }
//     ApphelpCacheReleaseLock();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// ApphelpCacheDump(VOID)
// {
//     PLIST_ENTRY ListEntry;
//     PSHIM_CACHE_ENTRY Entry;
//
//     DPRINT1("SHIMS: NtApphelpCacheControl( Dumping entries, newest to oldest )\n");
//     ApphelpCacheAcquireLock();
//     ListEntry = ApphelpShimCacheAge.Flink;
//     while (ListEntry != &ApphelpShimCacheAge)
//     {
//         Entry = CONTAINING_RECORD(ListEntry, SHIM_CACHE_ENTRY, List);
//         DPRINT1("Entry: %wZ\n", &Entry->Persistent.ImageName);
//         DPRINT1("DateTime: 0x%I64x\n", Entry->Persistent.DateTime.QuadPart);
//         DPRINT1("FileSize: 0x%I64x\n", Entry->Persistent.FileSize.QuadPart);
//         DPRINT1("Flags: 0x%x\n", Entry->CompatFlags);
//         ListEntry = ListEntry->Flink;
//     }
//     ApphelpCacheReleaseLock();
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// NtApphelpCacheControl(
//     _In_ APPHELPCACHESERVICECLASS Service,
//     _In_opt_ PAPPHELP_CACHE_SERVICE_LOOKUP ServiceData)
// {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     UNICODE_STRING ImageName = { 0 };
//     HANDLE Handle = INVALID_HANDLE_VALUE;
//
//     if (!ApphelpCacheEnabled)
//     {
//         DPRINT1("NtApphelpCacheControl: ApphelpCacheEnabled == 0\n");
//         return Status;
//     }
//     switch (Service)
//     {
//         case ApphelpCacheServiceLookup:
//             DPRINT("SHIMS: NtApphelpCacheControl( ApphelpCacheServiceLookup )\n");
//             Status = ApphelpValidateData(ServiceData, &ImageName, &Handle);
//             if (NT_SUCCESS(Status))
//                 Status = ApphelpCacheLookupEntry(&ImageName, Handle);
//             break;
//         case ApphelpCacheServiceRemove:
//             DPRINT("SHIMS: NtApphelpCacheControl( ApphelpCacheServiceRemove )\n");
//             Status = ApphelpValidateData(ServiceData, &ImageName, &Handle);
//             if (NT_SUCCESS(Status))
//                 Status = ApphelpCacheRemoveEntry(&ImageName);
//             break;
//         case ApphelpCacheServiceUpdate:
//             DPRINT("SHIMS: NtApphelpCacheControl( ApphelpCacheServiceUpdate )\n");
//             Status = ApphelpCacheAccessCheck();
//             if (NT_SUCCESS(Status))
//             {
//                 Status = ApphelpValidateData(ServiceData, &ImageName, &Handle);
//                 if (NT_SUCCESS(Status))
//                     Status = ApphelpCacheUpdateEntry(&ImageName, Handle);
//             }
//             break;
//         case ApphelpCacheServiceFlush:
//             /* FIXME: Check for admin or system here. */
//             Status = ApphelpCacheFlush();
//             break;
//         case ApphelpCacheServiceDump:
//             Status = ApphelpCacheDump();
//             break;
//         case ApphelpDBGReadRegistry:
//             DPRINT1("SHIMS: NtApphelpCacheControl( ApphelpDBGReadRegistry ): flushing cache.\n");
//             ApphelpCacheFlush();
//             DPRINT1("SHIMS: NtApphelpCacheControl( ApphelpDBGReadRegistry ): reading cache.\n");
//             Status = ApphelpCacheRead() ? STATUS_SUCCESS : STATUS_NOT_FOUND;
//             break;
//         case ApphelpDBGWriteRegistry:
//             DPRINT1("SHIMS: NtApphelpCacheControl( ApphelpDBGWriteRegistry ): writing cache.\n");
//             Status = ApphelpCacheWrite() ? STATUS_SUCCESS : STATUS_NOT_FOUND;
//             break;
//         default:
//             DPRINT1("SHIMS: NtApphelpCacheControl( Invalid service requested )\n");
//             break;
//     }
//     if (ImageName.Buffer)
//     {
//         ApphelpFreeUnicodeString(&ImageName);
//     }
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<SHIM_PERSISTENT_CACHE_HEADER_52>() == 8);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_HEADER_52, Magic) == 0);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_HEADER_52, NumEntries) == 4);
    assert!(core::mem::size_of::<SHIM_PERSISTENT_CACHE_ENTRY_52>() == 32);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_ENTRY_52, ImageName) == 0);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_ENTRY_52, DateTime) == 16);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_ENTRY_52, FileSize) == 24);
    assert!(core::mem::size_of::<SHIM_CACHE_ENTRY>() == 56);
    assert!(core::mem::offset_of!(SHIM_CACHE_ENTRY, List) == 0);
    assert!(core::mem::offset_of!(SHIM_CACHE_ENTRY, Persistent) == 16);
    assert!(core::mem::offset_of!(SHIM_CACHE_ENTRY, CompatFlags) == 48);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<SHIM_PERSISTENT_CACHE_HEADER_52>() == 8);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_HEADER_52, Magic) == 0);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_HEADER_52, NumEntries) == 4);
    assert!(core::mem::size_of::<SHIM_PERSISTENT_CACHE_ENTRY_52>() == 24);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_ENTRY_52, ImageName) == 0);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_ENTRY_52, DateTime) == 8);
    assert!(core::mem::offset_of!(SHIM_PERSISTENT_CACHE_ENTRY_52, FileSize) == 16);
    assert!(core::mem::size_of::<SHIM_CACHE_ENTRY>() == 36);
    assert!(core::mem::offset_of!(SHIM_CACHE_ENTRY, List) == 0);
    assert!(core::mem::offset_of!(SHIM_CACHE_ENTRY, Persistent) == 8);
    assert!(core::mem::offset_of!(SHIM_CACHE_ENTRY, CompatFlags) == 32);
};
// END OFFSET CHECKS
