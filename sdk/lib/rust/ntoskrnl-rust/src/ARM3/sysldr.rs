use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{ACCESS_MASK, ANSI_STRING, ERESOURCE, ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExInitializeResourceLite, ExReleaseResourceLite, FILE_STANDARD_INFORMATION, HANDLE, IMAGE_INFO, IMAGE_LOAD_CONFIG_DIRECTORY, IMAGE_THUNK_DATA, KMUTANT, KPROCESSOR_MODE, KeAcquireSpinLockRaiseToSynch, KeEnterCriticalRegion, KeFlushCurrentTb, KeFlushQueuedDpcs, KeInitializeSpinLock, KeLeaveCriticalRegion, KeQueryPerformanceCounter, KeReleaseMutant, KeStackAttachProcess, KeUnstackDetachProcess, KeWaitForSingleObject, LARGE_INTEGER, LDR_DATA_TABLE_ENTRY, LIST_ENTRY, LOAD_IMPORTS, LOGICAL, LdrRelocateImageWithBias, LdrVerifyMappedImageMatchesChecksum, OBJECT_ATTRIBUTES, OB_OPEN_REASON, ObReferenceObjectByHandle, PEPROCESS, PETHREAD, PHANDLE_TABLE, PHANDLE_TABLE_ENTRY, PIMAGE_EXPORT_DIRECTORY, PIMAGE_IMPORT_DESCRIPTOR, PIMAGE_NT_HEADERS, PIMAGE_SECTION_HEADER, POBJECT_HEADER, POBJECT_TYPE, PSECTION, ZwClose, ZwCreateSection, ZwOpenFile, _EPROCESS, _ETHREAD, _HANDLE_TABLE_ENTRY, _IMAGE_NT_HEADERS, _IO_STATUS_BLOCK, _KAPC_STATE, _LDR_DATA_TABLE_ENTRY, _MM_IMAGE_SECTION_OBJECT, _MMPTE, _OBJECT_HEADER, _OBJECT_TYPE, _UNICODE_STRING};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::helpers::*;

#[no_mangle]
pub static mut PsLoadedModuleList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmLoadedUserImageList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut PsLoadedModuleSpinLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut PsLoadedModuleResource: ERESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut PsNtosImageBase: ULONG_PTR = 0;
#[no_mangle]
pub static mut MmSystemLoadLock: KMUTANT = unsafe { core::mem::zeroed() };

pub static mut MmTotalSystemDriverPages: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmUnloadedDrivers: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmLastUnloadedDrivers: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MmMakeLowMemory: BOOLEAN = 0u8;
#[no_mangle]
pub static mut MmEnforceWriteProtection: BOOLEAN = 1u8;

// Registry-controlled values owned by ARM3/sysldr. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static SYSLDR_CONTROL_VECTOR: ntkrnl_sys::SubVector<2> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "EnforceWriteProtection",
            core::ptr::addr_of_mut!(MmEnforceWriteProtection)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "MakeLowMemory",
            core::ptr::addr_of_mut!(MmMakeLowMemory)),
    ] });

static mut MiKernelResourceStartPte: PMMPTE = core::ptr::null_mut();
static mut MiKernelResourceEndPte: PMMPTE = core::ptr::null_mut();
static mut ExPoolCodeStart: ULONG_PTR = 0;
static mut ExPoolCodeEnd: ULONG_PTR = 0;
static mut MmPoolCodeStart: ULONG_PTR = 0;
static mut MmPoolCodeEnd: ULONG_PTR = 0;
static mut MmPteCodeStart: ULONG_PTR = 0;
static mut MmPteCodeEnd: ULONG_PTR = 0;

#[cfg(target_pointer_width = "64")]
const DEFAULT_SECURITY_COOKIE: u64 = 0x00002B992DDFA232u64;
#[cfg(not(target_pointer_width = "64"))]
const DEFAULT_SECURITY_COOKIE: u32 = 0xBB40E64E;

unsafe fn MiCacheImageSymbols(base_address: PVOID) -> PVOID {
    let mut debug_size: ULONG = 0;
    let mut debug_directory: PVOID = core::ptr::null_mut();

    debug_directory = RtlImageDirectoryEntryToData(
        base_address,
        1,
        IMAGE_DIRECTORY_ENTRY_DEBUG,
        core::ptr::addr_of_mut!(debug_size),
    );

    debug_directory
}

#[no_mangle]
pub unsafe extern "system" fn MiLoadImageSection(
    section_ptr: *mut PSECTION,
    image_base: *mut PVOID,
    file_name: *mut _UNICODE_STRING,
    session_load: BOOLEAN,
    ldr_entry: *mut LDR_DATA_TABLE_ENTRY,
) -> NTSTATUS {
    let _ = file_name;
    let _ = ldr_entry; // ASSERT(LdrEntry == NULL) — non-session load
    let section = *section_ptr;
    let mut status: NTSTATUS;
    let pte_count: PFN_COUNT;
    let mut pointer_pte: PMMPTE;
    let last_pte: PMMPTE;
    let driver_base: PVOID;
    let mut temp_pte: MMPTE;
    let old_irql: KIRQL;
    let mut page_frame_index: PFN_NUMBER;

    if session_load != 0 {
        return STATUS_NOT_IMPLEMENTED;
    }

    pte_count = (ROUND_TO_PAGES((*((*section).Segment as *mut _MM_IMAGE_SECTION_OBJECT)).ImageInformation.ImageFileSize as usize) >> PAGE_SHIFT) as u32;
    pointer_pte = MiReserveSystemPtes(pte_count, SystemPteSpace);
    if pointer_pte.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    last_pte = pointer_pte.add((pte_count) as usize);
    driver_base = MiPteToAddress(pointer_pte);
    *image_base = driver_base;

    old_irql = MiAcquirePfnLock();

    temp_pte = ValidKernelPte;
    while pointer_pte < last_pte {
        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_DRIVER_PAGE);
        page_frame_index = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MiInitializePfn(page_frame_index, pointer_pte, 1);
        temp_pte.set_page_frame_number((page_frame_index) as PFN_NUMBER);
        MI_WRITE_VALID_PTE(pointer_pte, temp_pte);
        pointer_pte = pointer_pte.add(1);
    }

    MiReleasePfnLock(old_irql);

    #[cfg(target_arch = "aarch64")]
    {
        extern "C" {
    fn MiSimpleRead( file_object: *mut core::ffi::c_void, file_offset: *mut LARGE_INTEGER, buffer: PVOID, length: ULONG, paging: BOOLEAN, read_status: *mut _IO_STATUS_BLOCK, ) -> NTSTATUS; // nolint: cc-mismatch (decl is inside a cfg(aarch64) code block)
}

        let image_section_object = (*section).Segment as *mut _MM_IMAGE_SECTION_OBJECT;
        let file_object = (*image_section_object).FileObject;
        let image_size = (pte_count as usize) << PAGE_SHIFT;

        core::ptr::write_bytes(driver_base as *mut u8, 0, image_size);

        for i in 0..(*image_section_object).NrSegments {
            let seg = (*image_section_object).Segments.add(i as usize);
            let dest_va = (*seg).Image.VirtualAddress as usize;
            let src_file_offset = (*seg).Image.FileOffset as usize;
            let mut raw_len = (*seg).RawLength.QuadPart as usize;

            if raw_len == 0 || src_file_offset == 0 {
                if i != 0 || raw_len == 0 {
                    continue;
                }
            }

            if dest_va + raw_len > image_size {
                raw_len = image_size - dest_va;
            }
            if dest_va >= image_size {
                continue;
            }

            let mut bytes_remaining = raw_len;
            let mut dest_offset = dest_va;
            let mut src_offset = src_file_offset;

            while bytes_remaining > 0 {
                let chunk_size = if bytes_remaining > PAGE_SIZE { PAGE_SIZE as ULONG } else { bytes_remaining as ULONG };
                let mut file_offset = LARGE_INTEGER { QuadPart: src_offset as i64 };
                let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();

                status = MiSimpleRead(
                    file_object as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(file_offset),
                    ((driver_base as ULONG_PTR) + dest_offset) as PVOID,
                    chunk_size,
                    1,
                    core::ptr::addr_of_mut!(io_status_block),
                );
                if status < 0 {
                    break;
                }

                bytes_remaining -= chunk_size as usize;
                dest_offset += chunk_size as usize;
                src_offset += chunk_size as usize;
            }
        }

        status = STATUS_SUCCESS;
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        const FLG_ENABLE_KDEBUG_SYMBOL_LOAD: ULONG = 0x00040000;

        let mut apc_state: _KAPC_STATE = core::mem::zeroed();
        let mut base: PVOID = core::ptr::null_mut();
        let mut view_size: usize = 0;
        let mut section_offset = LARGE_INTEGER { QuadPart: 0 };

        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
            core::ptr::addr_of_mut!(apc_state),
        );

        // The image-section probe path can re-enter DbgLoadImageSymbols and
        // recursively trip MiLoadImageSection again. Suppress the global flag
        // around the map and restore it after.
        let load_symbols = (NtGlobalFlag & FLG_ENABLE_KDEBUG_SYMBOL_LOAD) != 0;
        if load_symbols { NtGlobalFlag &= !FLG_ENABLE_KDEBUG_SYMBOL_LOAD; }

        let process = PsGetCurrentProcess() as *mut _EPROCESS;
        status = MmMapViewOfSection(
            section,
            process as PEPROCESS,
            core::ptr::addr_of_mut!(base) as *mut PVOID,
            0,
            0,
            core::ptr::addr_of_mut!(section_offset),
            core::ptr::addr_of_mut!(view_size),
            ViewUnmap,
            0,
            PAGE_EXECUTE,
        );

        if load_symbols { NtGlobalFlag |= FLG_ENABLE_KDEBUG_SYMBOL_LOAD; }

        if status == STATUS_IMAGE_MACHINE_TYPE_MISMATCH {
            status = STATUS_INVALID_IMAGE_FORMAT;
        }

        if status < 0 {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(apc_state));
            return status;
        }

        // amd64-only: clear stale PTEs in the section view range before
        // reading. FreeLDR's identity-mapped page table pages can be reused
        // for user-space PTs with stale Valid PTEs (e.g., PFN=0).
        // MmUnmapViewOfSection also doesn't always clear PTEs from previous
        // section views. The stale PTEs prevent page faults from firing, so
        // the section data never gets demand-paged in. Zero them and fix up
        // UsedPageTableEntries on the parent PDE.
        #[cfg(target_arch = "x86_64")] // nolint: cfg-vs-c (amd64 stale-PTE workaround, Rust-only)
        {
            let start_pte = MiAddressToPte(base);
            let end_pte = MiAddressToPte(((base as ULONG_PTR) + ((pte_count as ULONG_PTR) << PAGE_SHIFT) - 1) as PVOID);
            let mut pte = start_pte;
            let mut cleared: ULONG = 0;
            while pte <= end_pte {
                if (*pte).Long != 0 {
                    (*pte).Long = 0;
                    let va = MiPteToAddress(pte);
                    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack, preserves_flags));
                    cleared += 1;
                }
                pte = pte.add(1);
            }
            if cleared > 0 {
                let pde = MiAddressToPde(base);
                if (*pde).valid() {
                    let pt_page_pfn = MiGetPfnEntry((*pde).page_frame_number());
                    if !pt_page_pfn.is_null() {
                        let old_count = mmpfn_used_page_table_entries(pt_page_pfn);
                        let new_count = if old_count >= cleared { old_count - cleared } else { 0 };
                        mmpfn_set_used_page_table_entries(pt_page_pfn, new_count);
                    }
                }
            }
        }

        core::ptr::copy_nonoverlapping(base as *const u8, driver_base as *mut u8, (pte_count << PAGE_SHIFT) as usize);

        status = MmUnmapViewOfSection(process as PEPROCESS, base);

        KeUnstackDetachProcess(core::ptr::addr_of_mut!(apc_state));
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NameToOrdinal(
    export_name: *const i8,
    image_base: PVOID,
    number_of_names: ULONG,
    name_table: *mut ULONG,
    ordinal_table: *mut u16,
) -> u16 {
    if number_of_names == 0 {
        return 0xFFFF;
    }

    let mut low: i32 = 0;
    let mut high: i32 = (number_of_names as i32) - 1;
    let mut mid: i32;

    while high >= low {
        mid = (low + high) >> 1;
        let name_rva = *name_table.add(mid as usize);
        let name_ptr = ((image_base as ULONG_PTR) + (name_rva as ULONG_PTR)) as *const i8;
        let ret = strcmp(export_name, name_ptr);
        if ret < 0 {
            high = mid - 1;
        } else if ret > 0 {
            low = mid + 1;
        } else {
            return *ordinal_table.add(mid as usize);
        }
    }

    0xFFFF
}

unsafe fn strcmp(s1: *const i8, s2: *const i8) -> i32 {
    let mut i = 0;
    loop {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpFindExportedRoutineByName(
    image_base: PVOID,
    export_name: *const i8,
    function: *mut PVOID,
    is_forwarder: *mut BOOLEAN,
    not_found_status: NTSTATUS,
) -> NTSTATUS {
    let image_base = opaque_ptr(image_base);
    let function = opaque_ptr(function);
    let is_forwarder = opaque_ptr(is_forwarder);
    let mut export_size: ULONG = 0;
    let export_directory = RtlImageDirectoryEntryToData(
        image_base,
        1,
        IMAGE_DIRECTORY_ENTRY_EXPORT,
        core::ptr::addr_of_mut!(export_size),
    ) as PIMAGE_EXPORT_DIRECTORY;

    if export_directory.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let name_table = ((image_base as ULONG_PTR) + ((*export_directory).AddressOfNames as ULONG_PTR)) as *mut ULONG;
    let ordinal_table = ((image_base as ULONG_PTR) + ((*export_directory).AddressOfNameOrdinals as ULONG_PTR)) as *mut u16;

    let ordinal = NameToOrdinal(
        export_name,
        image_base,
        (*export_directory).NumberOfNames,
        name_table,
        ordinal_table,
    );

    if ordinal == 0xFFFF {
        return not_found_status;
    }

    if (ordinal as ULONG) >= (*export_directory).NumberOfFunctions {
        return not_found_status;
    }

    let export_table = ((image_base as ULONG_PTR) + ((*export_directory).AddressOfFunctions as ULONG_PTR)) as *mut ULONG;
    let function_address = (image_base as ULONG_PTR) + (*export_table.add(ordinal as usize) as ULONG_PTR);

    if !is_forwarder.is_null() {
        *is_forwarder = 0u8;
        if (function_address > (export_directory as ULONG_PTR)) &&
           (function_address < (export_directory as ULONG_PTR) + (export_size as ULONG_PTR))
        {
            *is_forwarder = 1u8;
        }
    }

    *function = function_address as PVOID;
    STATUS_SUCCESS
}

unsafe fn RtlFindExportedRoutineByName(image_base: PVOID, export_name: *const i8) -> PVOID {
    let mut is_forwarder: BOOLEAN = 0u8;
    let mut function: PVOID = core::ptr::null_mut();

    let status = RtlpFindExportedRoutineByName(
        image_base,
        export_name,
        core::ptr::addr_of_mut!(function),
        core::ptr::addr_of_mut!(is_forwarder),
        STATUS_ENTRYPOINT_NOT_FOUND,
    );

    if status < 0 {
        return core::ptr::null_mut();
    }

    if is_forwarder != 0 {
        return core::ptr::null_mut();
    }

    function
}

#[no_mangle]
pub unsafe extern "system" fn MmCallDllInitialize(
    ldr_entry: *mut LDR_DATA_TABLE_ENTRY,
    _module_list_head: *mut LIST_ENTRY,
) -> NTSTATUS {
    let ldr_entry = opaque_ptr(ldr_entry);
    let _module_list_head = opaque_ptr(_module_list_head);
    type PMM_DLL_INITIALIZE = Option<unsafe extern "system" fn(*mut _UNICODE_STRING) -> NTSTATUS>;

    let dll_init: PMM_DLL_INITIALIZE = core::mem::transmute(RtlFindExportedRoutineByName(
        (*ldr_entry).DllBase,
        b"DllInitialize\0".as_ptr() as *const i8,
    ));
    if dll_init.is_none() {
        return STATUS_SUCCESS;
    }

    let mut import_name = _UNICODE_STRING {
        Length: (*ldr_entry).BaseDllName.Length,
        MaximumLength: (*ldr_entry).BaseDllName.MaximumLength,
        Buffer: (*ldr_entry).BaseDllName.Buffer,
    };

    static SERVICES_KEY_NAME: [u16; 58] = [
        92,82,101,103,105,115,116,114,121,92,77,97,99,104,105,110,101,92,83,121,115,116,101,109,92,67,117,114,114,101,110,116,67,111,110,116,114,111,108,83,101,116,92,83,101,114,118,105,99,101,115,92,0,0,0,0,0,0
    ];
    let services_key_name = _UNICODE_STRING {
        Length: 106,
        MaximumLength: 108,
        Buffer: SERVICES_KEY_NAME.as_ptr() as *mut u16,
    };

    let reg_path_max_len = services_key_name.Length as usize + import_name.Length as usize + 2;
    let reg_path_buffer = ExAllocatePoolWithTag(NonPagedPool, reg_path_max_len, TAG_LDR_WSTR);
    if reg_path_buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut reg_path = _UNICODE_STRING {
        Length: services_key_name.Length,
        MaximumLength: reg_path_max_len as u16,
        Buffer: reg_path_buffer as *mut u16,
    };

    core::ptr::copy_nonoverlapping(
        services_key_name.Buffer as *const u8,
        reg_path.Buffer as *mut u8,
        services_key_name.Length as usize,
    );

    let extension = wcschr(import_name.Buffer, b'.' as u16);
    if !extension.is_null() {
        import_name.Length = ((extension as usize - import_name.Buffer as usize) / 2) as u16;
    }

    RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(reg_path), core::ptr::addr_of_mut!(import_name));

    let status = (dll_init.unwrap())(core::ptr::addr_of_mut!(reg_path));

    ExFreePoolWithTag(reg_path_buffer, TAG_LDR_WSTR);

    status
}

unsafe fn wcschr(s: *const u16, c: u16) -> *const u16 {
    let mut p = s;
    loop {
        if *p == c {
            return p;
        }
        if *p == 0 {
            return core::ptr::null();
        }
        p = p.add(1);
    }
}

unsafe fn MiCallDllUnloadAndUnloadDll(ldr_entry: *mut LDR_DATA_TABLE_ENTRY) -> BOOLEAN {
    type PMM_DLL_UNLOAD = Option<unsafe extern "system" fn() -> NTSTATUS>;

    let dll_unload: PMM_DLL_UNLOAD = core::mem::transmute(RtlFindExportedRoutineByName(
        (*ldr_entry).DllBase,
        b"DllUnload\0".as_ptr() as *const i8,
    ));
    if dll_unload.is_none() {
        return 0u8;
    }

    let status = (dll_unload.unwrap())();
    if status < 0 {
        return 0u8;
    }

    (*ldr_entry).LoadCount = 1;
    MmUnloadSystemImage(ldr_entry as PVOID);
    1u8
}

unsafe fn MiDereferenceImports(_import_list: *mut LOAD_IMPORTS) -> NTSTATUS {
    // LoadedImports field not in bindgen _LDR_DATA_TABLE_ENTRY, function body elided.
    // The original C code iterates import_list entries and decrements LoadCount /
    // calls MiCallDllUnloadAndUnloadDll for entries reaching zero.
    STATUS_SUCCESS
}

unsafe fn MiClearImports(_ldr_entry: *mut LDR_DATA_TABLE_ENTRY) {
    // LoadedImports field not in bindgen _LDR_DATA_TABLE_ENTRY, entire function elided
}

unsafe fn MiProcessLoaderEntry(ldr_entry: *mut LDR_DATA_TABLE_ENTRY, insert: BOOLEAN) {
    let old_irql: KIRQL;

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(PsLoadedModuleResource), 1);

    old_irql = KeAcquireSpinLockRaiseToSynch(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock) as *mut _);

    if insert != 0 {
        InsertTailList(
            core::ptr::addr_of_mut!(PsLoadedModuleList),
            core::ptr::addr_of_mut!((*ldr_entry).InLoadOrderLinks),
        );
    } else {
        RemoveEntryList(core::ptr::addr_of_mut!((*ldr_entry).InLoadOrderLinks));
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock), old_irql);
    ExReleaseResourceLite(core::ptr::addr_of_mut!(PsLoadedModuleResource));
    KeLeaveCriticalRegion();
}

unsafe fn MiUpdateThunks(
    _loader_block: *mut LOADER_PARAMETER_BLOCK,
    _old_base: PVOID,
    _new_base: PVOID,
    _size: ULONG,
) {
}

extern "system" {
    fn RtlAnsiStringToUnicodeString(
        dest: *mut _UNICODE_STRING,
        src: *mut ANSI_STRING,
        alloc: BOOLEAN,
    ) -> NTSTATUS;
    fn RtlPrefixUnicodeString(
        s1: *const _UNICODE_STRING,
        s2: *const _UNICODE_STRING,
        case_insensitive: BOOLEAN,
    ) -> BOOLEAN;
    fn RtlFreeUnicodeString(s: *mut _UNICODE_STRING);
}

unsafe fn strchr(s: *const i8, c: i8) -> *const i8 {
    let mut p = s;
    loop {
        let v = *p;
        if v == c { return p; }
        if v == 0 { return core::ptr::null(); }
        p = p.add(1);
    }
}

unsafe fn strlen_(s: *const i8) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 { n += 1; }
    n
}

unsafe fn unicode_string_eq_ascii_ci(unicode: *const _UNICODE_STRING, ascii: *const i8) -> bool {
    if unicode.is_null() || ascii.is_null() || (*unicode).Buffer.is_null() {
        return false;
    }

    let ascii_len = strlen_(ascii);
    if (*unicode).Length as usize != ascii_len * core::mem::size_of::<u16>() {
        return false;
    }

    for i in 0..ascii_len {
        let mut a = *ascii.add(i) as u8;
        let mut u = *(*unicode).Buffer.add(i) as u16;
        if a >= b'A' && a <= b'Z' {
            a = a.wrapping_add(b'a' - b'A');
        }
        if u >= b'A' as u16 && u <= b'Z' as u16 {
            u = u.wrapping_add((b'a' - b'A') as u16);
        }
        if u != a as u16 {
            return false;
        }
    }

    true
}

unsafe fn unicode_basename_starts_with_ascii_ci(
    unicode: *const _UNICODE_STRING,
    ascii: *const i8,
    ascii_len: usize,
) -> bool {
    if unicode.is_null() || ascii.is_null() || (*unicode).Buffer.is_null() {
        return false;
    }

    let unicode_len = (*unicode).Length as usize / core::mem::size_of::<u16>();
    if unicode_len < ascii_len {
        return false;
    }

    for i in 0..ascii_len {
        let mut a = *ascii.add(i) as u8;
        let mut u = *(*unicode).Buffer.add(i) as u16;
        if a >= b'A' && a <= b'Z' {
            a = a.wrapping_add(b'a' - b'A');
        }
        if u >= b'A' as u16 && u <= b'Z' as u16 {
            u = u.wrapping_add((b'a' - b'A') as u16);
        }
        if u != a as u16 {
            return false;
        }
    }

    unicode_len == ascii_len || *(*unicode).Buffer.add(ascii_len) == b'.' as u16
}

unsafe fn unicode_string_eq_unicode_ci(
    left: *const _UNICODE_STRING,
    right: *const _UNICODE_STRING,
) -> bool {
    if left.is_null() || right.is_null() || (*left).Buffer.is_null() || (*right).Buffer.is_null() {
        return false;
    }

    if (*left).Length != (*right).Length {
        return false;
    }

    let len = (*left).Length as usize / core::mem::size_of::<u16>();
    for i in 0..len {
        let mut l = *(*left).Buffer.add(i);
        let mut r = *(*right).Buffer.add(i);
        if l >= b'A' as u16 && l <= b'Z' as u16 {
            l = l.wrapping_add((b'a' - b'A') as u16);
        }
        if r >= b'A' as u16 && r <= b'Z' as u16 {
            r = r.wrapping_add((b'a' - b'A') as u16);
        }
        if l != r {
            return false;
        }
    }

    true
}

#[no_mangle]
pub unsafe extern "system" fn MiSnapThunk(
    dll_base: PVOID,
    image_base: PVOID,
    name: *mut IMAGE_THUNK_DATA,
    address: *mut IMAGE_THUNK_DATA,
    export_directory: PIMAGE_EXPORT_DIRECTORY,
    export_size: ULONG,
    snap_forwarder: BOOLEAN,
    missing_api: *mut *mut i8,
) -> NTSTATUS {
    let is_ordinal = ((*name).u1.Ordinal & 0x80000000) != 0;
    let mut ordinal: u16;

    if is_ordinal && snap_forwarder == 0 {
        ordinal = ((((*name).u1.Ordinal & 0xFFFF) as usize) - ((*export_directory).Base as usize)) as u16;
        *missing_api = ordinal as usize as *mut i8;
    } else {
        if snap_forwarder == 0 {
            (*name).u1.AddressOfData = ((*name).u1.AddressOfData as usize + image_base as usize) as _;
        }
        let name_import = (*name).u1.AddressOfData as *mut IMAGE_IMPORT_BY_NAME;

        // Copy the procedure name into the missing-api buffer (mirrors C
        // RtlStringCbCopyA into MAXIMUM_FILENAME_LENGTH-sized buffer).
        if !missing_api.is_null() && !(*missing_api).is_null() {
            let dst = *missing_api;
            let src = (*name_import).Name.as_ptr() as *const i8;
            let len = strlen_(src);
            let copy = core::cmp::min(len, 255);
            core::ptr::copy_nonoverlapping(src, dst, copy);
            *dst.add(copy) = 0;
        }

        let name_table = ((dll_base as ULONG_PTR) + ((*export_directory).AddressOfNames as ULONG_PTR)) as *mut ULONG;
        let ordinal_table = ((dll_base as ULONG_PTR) + ((*export_directory).AddressOfNameOrdinals as ULONG_PTR)) as *mut u16;

        let hint = (*name_import).Hint;
        if (hint as ULONG) < (*export_directory).NumberOfNames {
            let hint_name_rva = *name_table.add(hint as usize);
            let hint_name = ((dll_base as ULONG_PTR) + (hint_name_rva as ULONG_PTR)) as *const i8;
            if strcmp((*name_import).Name.as_ptr() as *const i8, hint_name) == 0 {
                ordinal = *ordinal_table.add(hint as usize);
            } else {
                ordinal = NameToOrdinal(
                    (*name_import).Name.as_ptr() as *const i8,
                    dll_base,
                    (*export_directory).NumberOfNames,
                    name_table,
                    ordinal_table,
                );
                if ordinal == 0xFFFF {
                    return STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
                }
            }
        } else {
            ordinal = NameToOrdinal(
                (*name_import).Name.as_ptr() as *const i8,
                dll_base,
                (*export_directory).NumberOfNames,
                name_table,
                ordinal_table,
            );
            if ordinal == 0xFFFF {
                return STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
            }
        }
    }

    if (ordinal as ULONG) >= (*export_directory).NumberOfFunctions {
        return STATUS_DRIVER_ORDINAL_NOT_FOUND;
    }

    let export_table = ((dll_base as ULONG_PTR) + ((*export_directory).AddressOfFunctions as ULONG_PTR)) as *mut ULONG;
    (*address).u1.Function = ((dll_base as ULONG_PTR) + (*export_table.add(ordinal as usize) as ULONG_PTR)) as _;

    let mut status = STATUS_SUCCESS;

    // Forwarder check: function pointer falls inside the export directory range.
    if ((*address).u1.Function as ULONG_PTR) > (export_directory as ULONG_PTR)
        && ((*address).u1.Function as ULONG_PTR) < (export_directory as ULONG_PTR) + (export_size as ULONG_PTR)
    {
        status = STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;

        // Build the forwarder DLL name as an ANSI_STRING up to (and including) '.'
        let mut dll_buffer = (*address).u1.Function as *mut i8;

        // lld prepends '_' to forwarded export DLL names on i386. Strip it
        // when the rest of the buffer still contains a '.'.
        if *dll_buffer == b'_' as i8 && !strchr(dll_buffer.add(1), b'.' as i8).is_null() {
            dll_buffer = dll_buffer.add(1);
        }

        let dot = strchr(dll_buffer, b'.' as i8);
        if dot.is_null() {
            return status;
        }
        let dll_name_len = dot.offset_from(dll_buffer) as usize;
        let dll_len = dll_name_len + 1; // include the '.'

        let mut dll_ansi = ANSI_STRING {
            Length: dll_len as u16,
            MaximumLength: dll_len as u16,
            Buffer: dll_buffer,
        };
        let mut forwarder_unicode: _UNICODE_STRING = core::mem::zeroed();
        if !NT_SUCCESS(RtlAnsiStringToUnicodeString(
            core::ptr::addr_of_mut!(forwarder_unicode),
            core::ptr::addr_of_mut!(dll_ansi),
            1u8,
        )) {
            return status;
        }

        // Walk PsLoadedModuleList for the target DLL.
        let head = core::ptr::addr_of_mut!(PsLoadedModuleList);
        let mut next = PsLoadedModuleList.Flink;
        while next != head {
            let ldr_entry = (next as *mut u8).sub(
                core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
            ) as *mut LDR_DATA_TABLE_ENTRY;

            if unicode_basename_starts_with_ascii_ci(
                core::ptr::addr_of!((*ldr_entry).BaseDllName),
                dll_buffer,
                dll_name_len,
            ) {
                let mut fwd_export_size: ULONG = 0;
                let fwd_export_dir = RtlImageDirectoryEntryToData(
                    (*ldr_entry).DllBase,
                    1,
                    IMAGE_DIRECTORY_ENTRY_EXPORT,
                    core::ptr::addr_of_mut!(fwd_export_size),
                ) as PIMAGE_EXPORT_DIRECTORY;
                if fwd_export_dir.is_null() { break; }

                let fn_name_ptr = dll_buffer.add(dll_len);
                let fn_name_len = strlen_(fn_name_ptr) + 1;
                let alloc_size = core::mem::size_of::<IMAGE_IMPORT_BY_NAME>() + fn_name_len;
                let fwd_name = ExAllocatePoolWithTag(
                    PagedPool, alloc_size, TAG_LDR_WSTR,
                ) as *mut IMAGE_IMPORT_BY_NAME;
                if fwd_name.is_null() { break; }

                core::ptr::copy_nonoverlapping(
                    fn_name_ptr,
                    core::ptr::addr_of_mut!((*fwd_name).Name) as *mut i8,
                    fn_name_len,
                );
                (*fwd_name).Hint = 0;

                let mut forward_thunk: IMAGE_THUNK_DATA = core::mem::zeroed();
                forward_thunk.u1.AddressOfData = fwd_name as ULONG_PTR as _;

                // Recursive call uses a stack-local "missing forwarder" buffer
                // so we don't clobber the caller's missing_api on success.
                let mut missing_buf: [i8; 256] = [0; 256];
                let mut missing_ptr: *mut i8 = missing_buf.as_mut_ptr();

                status = MiSnapThunk(
                    (*ldr_entry).DllBase,
                    image_base,
                    core::ptr::addr_of_mut!(forward_thunk),
                    core::ptr::addr_of_mut!(forward_thunk),
                    fwd_export_dir,
                    fwd_export_size,
                    1u8,
                    core::ptr::addr_of_mut!(missing_ptr),
                );

                ExFreePoolWithTag(fwd_name as PVOID, TAG_LDR_WSTR);
                (*address).u1 = forward_thunk.u1;
                break;
            }

            next = (*next).Flink;
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(forwarder_unicode));
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn MmUnloadSystemImage(image_handle: PVOID) -> NTSTATUS {
    let image_handle = opaque_ptr(image_handle);
    let ldr_entry = image_handle as *mut LDR_DATA_TABLE_ENTRY;
    let mut had_entry = 0u8;

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(MmSystemLoadLock) as PVOID,
        _KWAIT_REASON_WrVirtualMemory,
        0i8,
        0,
        core::ptr::null_mut(),
    );

    // LoadedImports not in bindgen — elide the BOOT_LOADED early-return check

    (*ldr_entry).LoadCount -= 1;
    if (*ldr_entry).LoadCount != 0 {
        KeReleaseMutant(core::ptr::addr_of_mut!(MmSystemLoadLock), MUTANT_INCREMENT, 0, 0);
        KeLeaveCriticalRegion();
        return STATUS_SUCCESS;
    }

    if ((*ldr_entry).Flags & LDRP_DEBUG_SYMBOLS_LOADED) != 0 {
        let mut temp_name: ANSI_STRING = core::mem::zeroed();
        let status = RtlUnicodeStringToAnsiString(
            core::ptr::addr_of_mut!(temp_name),
            core::ptr::addr_of_mut!((*ldr_entry).BaseDllName),
            1,
        );
        if status >= 0 {
            DbgUnLoadImageSymbols(
                core::ptr::addr_of_mut!(temp_name),
                (*ldr_entry).DllBase,
                PsGetCurrentProcessId() as ULONG_PTR,
            );
            RtlFreeAnsiString(core::ptr::addr_of_mut!(temp_name));
        }
    }

    if !(*ldr_entry).InLoadOrderLinks.Flink.is_null() {
        MiProcessLoaderEntry(ldr_entry, 0u8);
        had_entry = 1u8;
    }

    // LoadedImports not in bindgen, elided
    // MiDereferenceImports((*ldr_entry).LoadedImports);
    MiClearImports(ldr_entry);

    if had_entry != 0 {
        if !(*ldr_entry).FullDllName.Buffer.is_null() {
            ExFreePoolWithTag((*ldr_entry).FullDllName.Buffer as PVOID, TAG_LDR_WSTR);
        }

        if !(*ldr_entry).__bindgen_anon_1.__bindgen_anon_1.SectionPointer.is_null() {
            ObDereferenceObject((*ldr_entry).__bindgen_anon_1.__bindgen_anon_1.SectionPointer);
        }

        ExFreePoolWithTag(ldr_entry as PVOID, TAG_MODULE_OBJECT);
    }

    KeReleaseMutant(core::ptr::addr_of_mut!(MmSystemLoadLock), MUTANT_INCREMENT, 0, 0);
    KeLeaveCriticalRegion();
    STATUS_SUCCESS
}

// MiResolveImageReferences: provided by C shim (mm_sysldr_shims.c)
// Rust stub removed to avoid symbol conflict.

#[no_mangle]
pub unsafe extern "system" fn MiFreeInitializationCode(init_start: PVOID, init_end: PVOID) {
    let init_start = opaque_ptr(init_start);
    let init_end = opaque_ptr(init_end);
    let pointer_pte = MiAddressToPte(init_start);
    let pages_freed = (MiAddressToPte(init_end) as usize - pointer_pte as usize) / core::mem::size_of::<MMPTE>();
    MiDeleteSystemPageableVm(pointer_pte, pages_freed as PFN_NUMBER, 0, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn MiFindInitializationCode(_start_va: *mut PVOID, _end_va: *mut PVOID) {
    let _start_va = opaque_ptr(_start_va);
    let _end_va = opaque_ptr(_end_va);
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeDriverInitialization(_ldr_entry: *mut LDR_DATA_TABLE_ENTRY) {
    let _ldr_entry = opaque_ptr(_ldr_entry);
}

#[no_mangle]
pub unsafe extern "system" fn MiReloadBootLoadedDrivers(_loader_block: *mut LOADER_PARAMETER_BLOCK) {
    let _loader_block = opaque_ptr(_loader_block);
}

unsafe fn MiBuildImportsForBootDrivers() -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe fn MiLocateKernelSections(_ldr_entry: *mut LDR_DATA_TABLE_ENTRY) {
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeLoadedModuleList(loader_block: *mut LOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let loader_block = opaque_ptr(loader_block);
    extern "system" {
        fn RtlImageNtHeader(base: PVOID) -> PVOID;
    }

    /* Setup the loaded module list and locks */
    ExInitializeResourceLite(core::ptr::addr_of_mut!(PsLoadedModuleResource));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!(PsLoadedModuleList));

    /* Get loop variables and the kernel entry */
    let list_head = core::ptr::addr_of_mut!((*loader_block).LoadOrderListHead);
    let mut next_entry = (*list_head).Flink;
    let ldr_entry = (next_entry as *mut u8).sub(
        core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)
    ) as *mut LDR_DATA_TABLE_ENTRY;
    PsNtosImageBase = (*ldr_entry).DllBase as ULONG_PTR;

    // Publish the kernel's RUNTIME base on serial. Offline triage needs it and had no
    // way to get it: a BUGCHECK reports a runtime VA, the symbol index holds FILE VAs,
    // and `qemu/addr2sym.sh --reloc <base - 0x400000>` bridges the two. Nothing printed
    // it (freeldr logs "[FLDR] load ...\ntoskrnl.exe" with no address), so every saved
    // arm64 crash log was a bare hex address unless a live VM was still around for
    // inspect.sh to query over gdb. One line here makes the whole retained crash corpus
    // symbolicable with no VM. Deliberately NOT guessed by trying candidate bases until
    // a symbol looks plausible — a confident wrong base relocates the hunt (§9.124).
    // arm64 is both the arch that needs this most and the one where DbgPrint is not
    // dependable this early — so use the UART trace there, exactly as procsup.rs does.
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" {
            fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
            fn ARM64_UART_TRACE(msg: *const u8);
        }
        ARM64_UART_TRACE_HEX(b"[KRNLBASE] ntoskrnl runtime base=\0".as_ptr(), PsNtosImageBase);
        // ARM64_UART_TRACE_HEX emits prefix + "0x" + digits and NO newline, so without
        // this the line runs into whatever prints next.
        ARM64_UART_TRACE(b"\n\0".as_ptr());
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        extern "C" {
            fn DbgPrint(Format: *const i8, ...) -> u32;
        }
        DbgPrint(
            b"[KRNLBASE] ntoskrnl runtime base=%p\n\0".as_ptr() as *const i8,
            PsNtosImageBase as PVOID,
        );
    }

    /* Locate resource section, pool code, and system pte code */
    MiLocateKernelSections(ldr_entry);

    /* Loop the loader block */
    while next_entry != list_head {
        let ldr_entry = (next_entry as *mut u8).sub(
            core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)
        ) as *mut LDR_DATA_TABLE_ENTRY;

        /* FIXME: ROS HACK. Make sure this is a driver */
        if RtlImageNtHeader((*ldr_entry).DllBase).is_null() {
            next_entry = (*next_entry).Flink;
            continue;
        }

        // DBG: after RtlImageNtHeader check passed

        /* Calculate the size we'll need and allocate a copy */
        let entry_size = core::mem::size_of::<LDR_DATA_TABLE_ENTRY>()
            + (*ldr_entry).BaseDllName.MaximumLength as usize
            + core::mem::size_of::<u16>(); /* UNICODE_NULL */

        // DBG: before NonPagedPool alloc

        let new_entry = ExAllocatePoolWithTag(
            NonPagedPool, entry_size, TAG_MODULE_OBJECT
        ) as *mut LDR_DATA_TABLE_ENTRY;

        // DBG: after NonPagedPool alloc

        if new_entry.is_null() {
            return 0;
        }

        /* Copy the entry over */
        // DBG: before copy

        core::ptr::copy_nonoverlapping(
            ldr_entry as *const u8,
            new_entry as *mut u8,
            core::mem::size_of::<LDR_DATA_TABLE_ENTRY>(),
        );

        // DBG: after copy, before PagedPool alloc

        /* Allocate the full DLL name buffer */
        let full_name_size = (*ldr_entry).FullDllName.MaximumLength as usize
            + core::mem::size_of::<u16>(); /* UNICODE_NULL */

        // DBG: before PagedPool alloc (size computed)

        let full_name_buf = ExAllocatePoolWithTag(
            PagedPool, full_name_size, TAG_LDR_WSTR
        ) as *mut u16;
        if full_name_buf.is_null() {
            ExFreePoolWithTag(new_entry as PVOID, TAG_MODULE_OBJECT);
            return 0;
        }
        (*new_entry).FullDllName.Buffer = full_name_buf;

        // DBG: after PagedPool alloc

        /* Set the base name buffer (immediately after the struct) */
        (*new_entry).BaseDllName.Buffer = new_entry.add(1) as *mut u16;

        /* Copy the full and base name */
        core::ptr::copy_nonoverlapping(
            (*ldr_entry).FullDllName.Buffer as *const u8,
            (*new_entry).FullDllName.Buffer as *mut u8,
            (*ldr_entry).FullDllName.MaximumLength as usize,
        );
        core::ptr::copy_nonoverlapping(
            (*ldr_entry).BaseDllName.Buffer as *const u8,
            (*new_entry).BaseDllName.Buffer as *mut u8,
            (*ldr_entry).BaseDllName.MaximumLength as usize,
        );

        /* Null-terminate the base name */
        let base_name_chars = (*new_entry).BaseDllName.Length as usize / 2;
        *(*new_entry).BaseDllName.Buffer.add(base_name_chars) = 0u16;

        // DBG: after name copies, before InsertTailList

        /* Insert the entry into the list */
        InsertTailList(
            core::ptr::addr_of_mut!(PsLoadedModuleList),
            core::ptr::addr_of_mut!((*new_entry).InLoadOrderLinks),
        );

        next_entry = (*next_entry).Flink;
    }

    /* Build the import lists for the boot drivers */
    MiBuildImportsForBootDrivers();

    1
}

#[no_mangle]
pub unsafe extern "system" fn MmChangeKernelResourceSectionProtection(_protection_mask: ULONG_PTR) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmMakeKernelResourceSectionWritable() {
}

unsafe fn MiUseLargeDriverPage(
    _number_of_ptes: ULONG,
    _image_base_address: *mut PVOID,
    _base_image_name: *mut _UNICODE_STRING,
    _boot_driver: BOOLEAN,
) -> LOGICAL {
    (0u8) as u32
}

unsafe fn MiSetSystemCodeProtection(
    _first_pte: PMMPTE,
    _last_pte: PMMPTE,
    _protection: ULONG,
) {
}

#[no_mangle]
pub unsafe extern "system" fn MiWriteProtectSystemImage(_image_base: PVOID) {
    let _image_base = opaque_ptr(_image_base);
}

unsafe fn MiSetPagingOfDriver(_pointer_pte: PMMPTE, _last_pte: PMMPTE) {
}

unsafe fn MiEnablePagingOfDriver(_ldr_entry: *mut LDR_DATA_TABLE_ENTRY) {
}

#[no_mangle]
pub unsafe extern "system" fn MmCheckSystemImage(_image_handle: HANDLE) -> NTSTATUS {
    let _image_handle = opaque_ptr(_image_handle);
    STATUS_SUCCESS
}

/// The image's /GS cookie, from its load-config directory, or null when it has none in range.
unsafe fn LdrpFetchAddressOfSecurityCookie(base_address: PVOID, size_of_image: ULONG) -> PVOID {
    const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: u16 = 10;
    let mut dir_size: ULONG = 0;
    let config_dir = RtlImageDirectoryEntryToData(
        base_address,
        TRUE as u32,
        IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
        &mut dir_size,
    ) as *const IMAGE_LOAD_CONFIG_DIRECTORY;
    let through_cookie = core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, SecurityCookie)
        + core::mem::size_of::<ULONG_PTR>();
    if config_dir.is_null() || (dir_size as usize) < through_cookie {
        return core::ptr::null_mut();
    }
    let cookie = (*config_dir).SecurityCookie as usize;
    let base = base_address as usize;
    if cookie <= base || cookie >= base + size_of_image as usize - core::mem::size_of::<ULONG_PTR>() {
        return core::ptr::null_mut();
    }
    cookie as PVOID
}

/// Give a freshly loaded image its own /GS cookie before its entry point runs. Drivers built
/// with a current WDK check this in `__security_init_cookie` and `__fastfail` when they still
/// find the compiler's default (Realtek's Windows 10 RTL8153 driver did, at DriverEntry).
unsafe fn LdrpInitSecurityCookie(ldr_entry: *mut LDR_DATA_TABLE_ENTRY) -> PVOID {
    let cookie = LdrpFetchAddressOfSecurityCookie((*ldr_entry).DllBase, (*ldr_entry).SizeOfImage)
        as *mut ULONG_PTR;
    if cookie.is_null() {
        return core::ptr::null_mut();
    }
    if *cookie as u64 == DEFAULT_SECURITY_COOKIE as u64 || *cookie == 0 {
        let counter = KeQueryPerformanceCounter(core::ptr::null_mut());
        // The address is unique; the tick only needs to differ between loads.
        let mut new_cookie = (cookie as ULONG_PTR) ^ (counter.u.LowPart as ULONG_PTR);
        // Some images expect the top 16 bits clear, as in the default cookie.
        #[cfg(target_pointer_width = "64")]
        if new_cookie as u64 > 0x0000_FFFF_FFFF_FFFFu64 {
            new_cookie >>= 16;
        }
        if new_cookie == 0 || new_cookie == *cookie {
            new_cookie = (DEFAULT_SECURITY_COOKIE as ULONG_PTR).wrapping_add(1);
        }
        *cookie = new_cookie;
    }
    cookie as PVOID
}

// MmLoadSystemImage: translated from mm_sysldr_shims.c. Loads a kernel-mode
// PE image from disk: opens the file, creates an image section, maps it via
// MiLoadImageSection, allocates and initializes the LDR_DATA_TABLE_ENTRY,
// resolves imports via MiResolveImageReferences, runs the load-image notify
// chain, and registers debug symbols. Goto-driven cleanup in the C version
// is rewritten as a Result<NTSTATUS, NTSTATUS> closure with a single cleanup
// block at the bottom.
#[no_mangle]
pub unsafe extern "system" fn MmLoadSystemImage(
    file_name: *mut _UNICODE_STRING,
    name_prefix: *mut _UNICODE_STRING,
    loaded_name: *mut _UNICODE_STRING,
    flags: ULONG,
    module_object: *mut PVOID,
    image_base_address: *mut PVOID,
) -> NTSTATUS {
    const FLG_SHOW_LDR_SNAPS: ULONG = 0x00000002;
    const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;
    const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
    const OBJ_KERNEL_HANDLE: ULONG = 0x00000200;
    const FILE_EXECUTE: ULONG = 0x00000020;
    const FILE_SHARE_READ: ULONG = 0x00000001;
    const FILE_SHARE_DELETE: ULONG = 0x00000004;
    const SECTION_MAP_READ: ULONG = 0x0004;
    const SECTION_MAP_EXECUTE: ULONG = 0x0008;
    const SECTION_ALL_ACCESS: ULONG = 0xF001F;
    const STATUS_IMAGE_ALREADY_LOADED: NTSTATUS = 0xC000010Eu32 as i32;
    const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as i32;
    const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as i32;
    const STATUS_NOT_IMPLEMENTED_LOCAL: NTSTATUS = 0xC0000002u32 as i32;
    const STATUS_IMAGE_CHECKSUM_MISMATCH: NTSTATUS = 0xC0000221u32 as i32;
    const STATUS_IMAGE_MP_UP_MISMATCH: NTSTATUS = 0xC0000249u32 as i32;
    const STATUS_INVALID_IMAGE_PROTECT: NTSTATUS = 0xC0000130u32 as i32;
    const KERNEL_MODE: KPROCESSOR_MODE = 0;

    // Session-load detection: defer until session loader is translated.
    // Smoke never exercises this path; STATUS_NO_MEMORY mirrors the C behavior
    // when ProcessInSession is unset.
    if flags != 0 { return STATUS_NO_MEMORY; }

    // Allocate the name-formatting buffer up front; cleanup is unconditional.
    let buffer = ExAllocatePoolWithTag(
        NonPagedPool, 256, TAG_LDR_WSTR,
    ) as *mut i8;
    if buffer.is_null() { return STATUS_INSUFFICIENT_RESOURCES; }

    let mut prefixed_buffer: *mut u16 = core::ptr::null_mut();
    let mut file_handle: HANDLE = core::ptr::null_mut();
    let mut section: PSECTION = core::ptr::null_mut();
    let mut lock_owned = false;
    let mut ldr_entry: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut loaded_imports: PVOID = MM_SYSLDR_NO_IMPORTS;

    // Split FileName into BaseDirectory + BaseName.
    let mut base_name: _UNICODE_STRING = core::mem::zeroed();
    let file_name_buf = (*file_name).Buffer;
    let file_name_len_chars = (*file_name).Length as usize / 2;
    if file_name_len_chars > 0 && *file_name_buf == OBJ_NAME_PATH_SEPARATOR {
        let end = file_name_buf.add(file_name_len_chars);
        let mut p = end;
        while p > file_name_buf && *p.sub(1) != OBJ_NAME_PATH_SEPARATOR { p = p.sub(1); }
        let base_chars = end.offset_from(p) as usize;
        base_name.Length = (base_chars * 2) as u16;
        base_name.Buffer = p;
    } else {
        base_name.Length = (*file_name).Length;
        base_name.Buffer = file_name_buf;
    }
    base_name.MaximumLength = base_name.Length;

    let mut base_directory: _UNICODE_STRING = *file_name;
    base_directory.Length -= base_name.Length;
    base_directory.MaximumLength = base_directory.Length;

    let mut prefix_name: _UNICODE_STRING = *file_name;

    let result: Result<(), NTSTATUS> = (|| -> Result<(), NTSTATUS> {

    // Optional prefix concatenation.
    if !name_prefix.is_null() {
        let total = (base_directory.Length as usize)
            .checked_add((*name_prefix).Length as usize)
            .and_then(|n| n.checked_add(base_name.Length as usize))
            .ok_or(STATUS_INVALID_PARAMETER)?;
        if total > u16::MAX as usize { return Err(STATUS_INVALID_PARAMETER); }

        prefixed_buffer = ExAllocatePoolWithTag(PagedPool, total, TAG_LDR_WSTR) as *mut u16;
        if prefixed_buffer.is_null() { return Err(STATUS_INSUFFICIENT_RESOURCES); }
        prefix_name.Buffer = prefixed_buffer;
        prefix_name.Length = 0;
        prefix_name.MaximumLength = total as u16;

        RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(prefix_name), core::ptr::addr_of_mut!(base_directory));
        RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(prefix_name), name_prefix);
        RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(prefix_name), core::ptr::addr_of_mut!(base_name));

        base_name.Buffer = prefix_name.Buffer.add(base_directory.Length as usize / 2);
        base_name.Length += (*name_prefix).Length;
        base_name.MaximumLength = prefix_name.MaximumLength - base_directory.Length;
    }

    if !loaded_name.is_null() { base_name = *loaded_name; }

    // FLG_SHOW_LDR_SNAPS gate left as a no-op trace point (DPRINT1 isn't wired
    // through here yet; preserve the gate but don't log).
    let _show_snaps = (NtGlobalFlag & FLG_SHOW_LDR_SNAPS) != 0;

    let head = core::ptr::addr_of_mut!(PsLoadedModuleList);

    // LoaderScan loop — restarts after section creation.
    loop {
        // Acquire the load lock.
        ntkrnl_sys::ASSERT!(!lock_owned);
        lock_owned = true;
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(MmSystemLoadLock) as PVOID,
            WrVirtualMemory, KERNEL_MODE, 0, core::ptr::null_mut(),
        );

        // Scan the loaded-module list.
        let mut next_entry = PsLoadedModuleList.Flink;
        let mut found_entry: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
        while next_entry != head {
            let entry = (next_entry as *mut u8).sub(
                core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
            ) as *mut LDR_DATA_TABLE_ENTRY;
            if unicode_string_eq_unicode_ci(
                core::ptr::addr_of!(prefix_name),
                core::ptr::addr_of!((*entry).FullDllName),
            ) {
                found_entry = entry;
                break;
            }
            next_entry = (*next_entry).Flink;
        }

        if !found_entry.is_null() {
            // Image already loaded.
            if !section.is_null() {
                ObDereferenceObject(section as PVOID);
                section = core::ptr::null_mut();
            }
            if flags == 0 {
                *module_object = found_entry as PVOID;
                *image_base_address = (*found_entry).DllBase;
                return Err(STATUS_IMAGE_ALREADY_LOADED);
            } else {
                return Err(STATUS_NOT_IMPLEMENTED_LOCAL);
            }
        }

        if section.is_null() {
            // Release lock, open file, create section, retry the scan.
            KeReleaseMutant(core::ptr::addr_of_mut!(MmSystemLoadLock), MUTANT_INCREMENT, 0, 0);
            KeLeaveCriticalRegion();
            lock_owned = false;
            ldr_entry = core::ptr::null_mut();

            let mut object_attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
            object_attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
            object_attributes.ObjectName = file_name;
            object_attributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;

            let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
            let st = ZwOpenFile(
                core::ptr::addr_of_mut!(file_handle),
                FILE_EXECUTE,
                core::ptr::addr_of_mut!(object_attributes),
                core::ptr::addr_of_mut!(io_status_block),
                FILE_SHARE_READ | FILE_SHARE_DELETE,
                0,
            );
            if !NT_SUCCESS(st) { return Err(st); }

            let st = MmCheckSystemImage(file_handle);
            if st == STATUS_IMAGE_CHECKSUM_MISMATCH
                || st == STATUS_IMAGE_MP_UP_MISMATCH
                || st == STATUS_INVALID_IMAGE_PROTECT
            {
                return Err(st);
            }

            let desired_access = if flags != 0 {
                SECTION_MAP_READ | SECTION_MAP_EXECUTE
            } else {
                SECTION_ALL_ACCESS
            };

            let mut object_attributes_sec: OBJECT_ATTRIBUTES = core::mem::zeroed();
            object_attributes_sec.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
            object_attributes_sec.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;

            let mut section_handle: HANDLE = core::ptr::null_mut();
            let st = ZwCreateSection(
                core::ptr::addr_of_mut!(section_handle),
                desired_access,
                core::ptr::addr_of_mut!(object_attributes_sec),
                core::ptr::null_mut(),
                PAGE_EXECUTE,
                SEC_IMAGE,
                file_handle,
            );
            if !NT_SUCCESS(st) { return Err(st); }

            let st = ObReferenceObjectByHandle(
                section_handle,
                SECTION_MAP_EXECUTE,
                MmSectionObjectType,
                KERNEL_MODE,
                core::ptr::addr_of_mut!(section) as *mut PVOID,
                core::ptr::null_mut(),
            );
            ZwClose(section_handle);
            if !NT_SUCCESS(st) { return Err(st); }

            if flags != 0 { return Err(STATUS_SUCCESS); }
            // Restart scan with section in hand.
            continue;
        }

        // Section is set, found_entry is null — proceed with the actual load.
        ldr_entry = core::ptr::null_mut();
        break;
    }

    // Load the image into a kernel-mode region.
    let mut module_load_base: PVOID = core::ptr::null_mut();
    let st = MiLoadImageSection(
        core::ptr::addr_of_mut!(section),
        core::ptr::addr_of_mut!(module_load_base),
        file_name,
        0u8,
        core::ptr::null_mut(),
    );

    let driver_size = (*((*section).Segment as *mut _MM_IMAGE_SECTION_OBJECT))
        .ImageInformation.ImageFileSize as ULONG;

    if flags == 0 {
        if NT_SUCCESS(st) {
            MiUseLargeDriverPage(
                driver_size / PAGE_SIZE as u32,
                core::ptr::addr_of_mut!(module_load_base) as *mut PVOID,
                core::ptr::addr_of_mut!(base_name),
                1u8,
            );
        }
        ObDereferenceObject(section as PVOID);
        section = core::ptr::null_mut();
    }

    if !NT_SUCCESS(st) { return Err(st); }

    let st = LdrRelocateImageWithBias(
        module_load_base,
        0,
        b"SYSLDR\0".as_ptr() as *mut i8,
        STATUS_SUCCESS as ULONG,
        0xC0000018u32, // STATUS_CONFLICTING_ADDRESSES
        0xC000007Bu32, // STATUS_INVALID_IMAGE_FORMAT
    ) as NTSTATUS;
    if !NT_SUCCESS(st) { return Err(st); }

    let nt_header = RtlImageNtHeader(module_load_base) as *mut _IMAGE_NT_HEADERS;

    // Allocate LDR_DATA_TABLE_ENTRY + space for the BaseDllName payload.
    let entry_size = core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() + base_name.Length as usize + 2;
    ldr_entry = ExAllocatePoolWithTag(NonPagedPool, entry_size, TAG_MODULE_OBJECT) as *mut LDR_DATA_TABLE_ENTRY;
    if ldr_entry.is_null() { return Err(STATUS_INSUFFICIENT_RESOURCES); }

    core::ptr::write_bytes(ldr_entry as *mut u8, 0, core::mem::size_of::<LDR_DATA_TABLE_ENTRY>());
    (*ldr_entry).Flags = LDRP_LOAD_IN_PROGRESS;
    (*ldr_entry).LoadCount = 1;
    // LoadedImports + PatchInformation live in the bindgen union; skip the
    // C field assignments (covered by the zero-init above).

    if (*nt_header).OptionalHeader.MajorOperatingSystemVersion >= 5
        && (*nt_header).OptionalHeader.MajorImageVersion >= 5
    {
        (*ldr_entry).Flags |= LDRP_ENTRY_NATIVE;
    }

    (*ldr_entry).DllBase = module_load_base;
    (*ldr_entry).EntryPoint = (((module_load_base as ULONG_PTR)
        + (*nt_header).OptionalHeader.AddressOfEntryPoint as ULONG_PTR)) as PVOID;
    (*ldr_entry).SizeOfImage = driver_size;
    (*ldr_entry).__bindgen_anon_1.__bindgen_anon_1.CheckSum = (*nt_header).OptionalHeader.CheckSum;
    (*ldr_entry).__bindgen_anon_1.__bindgen_anon_1.SectionPointer = section as PVOID;

    // BaseDllName lives directly after the entry struct.
    (*ldr_entry).BaseDllName.Buffer = (ldr_entry.add(1)) as *mut u16;
    (*ldr_entry).BaseDllName.Length = base_name.Length;
    (*ldr_entry).BaseDllName.MaximumLength = base_name.Length;
    core::ptr::copy_nonoverlapping(
        base_name.Buffer as *const u16,
        (*ldr_entry).BaseDllName.Buffer,
        base_name.Length as usize / 2,
    );
    *(*ldr_entry).BaseDllName.Buffer.add(base_name.Length as usize / 2) = 0u16;

    (*ldr_entry).FullDllName.Buffer = ExAllocatePoolWithTag(
        PagedPool, prefix_name.Length as usize + 2, TAG_LDR_WSTR,
    ) as *mut u16;
    if (*ldr_entry).FullDllName.Buffer.is_null() {
        (*ldr_entry).FullDllName.Length = 0;
        (*ldr_entry).FullDllName.MaximumLength = 0;
    } else {
        (*ldr_entry).FullDllName.Length = prefix_name.Length;
        (*ldr_entry).FullDllName.MaximumLength = prefix_name.Length;
        core::ptr::copy_nonoverlapping(
            prefix_name.Buffer as *const u16,
            (*ldr_entry).FullDllName.Buffer,
            prefix_name.Length as usize / 2,
        );
        *(*ldr_entry).FullDllName.Buffer.add(prefix_name.Length as usize / 2) = 0u16;
    }

    MiProcessLoaderEntry(ldr_entry, 1u8);

    // Resolve imports.
    let mut missing_api_name: *mut i8 = buffer;
    let mut missing_driver_name: *mut u16 = core::ptr::null_mut();
    let st = MiResolveImageReferences(
        module_load_base,
        core::ptr::addr_of_mut!(base_directory),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(missing_api_name),
        core::ptr::addr_of_mut!(missing_driver_name),
        core::ptr::addr_of_mut!(loaded_imports),
    );
    if !NT_SUCCESS(st) {
        let driver_word = missing_driver_name as ULONG_PTR;
        let need_to_free = (driver_word & 1) != 0;
        if need_to_free {
            let untagged = (driver_word & !1) as *mut u16;
            ExFreePoolWithTag(untagged as PVOID, TAG_LDR_WSTR);
        }

        MiProcessLoaderEntry(ldr_entry, 0u8);
        if !(*ldr_entry).FullDllName.Buffer.is_null() {
            ExFreePoolWithTag((*ldr_entry).FullDllName.Buffer as PVOID, TAG_LDR_WSTR);
        }
        ExFreePoolWithTag(ldr_entry as PVOID, TAG_MODULE_OBJECT);
        ldr_entry = core::ptr::null_mut();
        return Err(st);
    }

    (*ldr_entry).Flags |= LDRP_SYSTEM_MAPPED | LDRP_ENTRY_PROCESSED | LDRP_MM_LOADED;
    (*ldr_entry).Flags &= !LDRP_LOAD_IN_PROGRESS;
    // LoadedImports stash skipped (field not in bindgen).

    // The image was copied into system space, relocated and import-snapped
    // through the D-cache, and is about to be executed. ARM instruction fetch
    // does not see dirty D-cache lines, so without this a driver loaded after
    // boot (win32k.sys, from smss) runs whatever the RAM held before. The same
    // bug on the paging path killed smss on a Raspberry Pi 4.
    #[cfg(target_arch = "aarch64")]
    ntkrnl_sys::inline::ke::KeSweepICache(
        (*ldr_entry).DllBase,
        (*ldr_entry).SizeOfImage as ULONG_PTR,
    );

    MiWriteProtectSystemImage((*ldr_entry).DllBase);
    LdrpInitSecurityCookie(ldr_entry);

    // PsImageNotifyEnabled / PspRunLoadImageNotifyRoutines / DbgLoadImageSymbols
    // are deferred — none are exercised in current smoke and they require
    // bindgen access to several globals/structs that the bindgen layer
    // doesn't yet expose cleanly. Once those land, port the corresponding
    // blocks from the C version.

    if !MiCacheImageSymbols((*ldr_entry).DllBase).is_null() {
        // DbgLoadImageSymbols path elided (see above).
        (*ldr_entry).Flags |= LDRP_DEBUG_SYMBOLS_LOADED;
    }

    ntkrnl_sys::ASSERT!(section.is_null());
    MiEnablePagingOfDriver(ldr_entry);

    *module_object = ldr_entry as PVOID;
    *image_base_address = (*ldr_entry).DllBase;

    Ok(())
    })();

    let final_status = match result {
        Ok(()) => STATUS_SUCCESS,
        Err(s) => s,
    };

    if lock_owned {
        KeReleaseMutant(core::ptr::addr_of_mut!(MmSystemLoadLock), MUTANT_INCREMENT, 0, 0);
        KeLeaveCriticalRegion();
    }
    if !file_handle.is_null() { ZwClose(file_handle); }
    if !prefixed_buffer.is_null() {
        ExFreePoolWithTag(prefixed_buffer as PVOID, TAG_LDR_WSTR);
    }
    ExFreePoolWithTag(buffer as PVOID, TAG_LDR_WSTR);
    final_status
}

// MiLookupDataTableEntry: pure-logic linear scan of PsLoadedModuleList.
// Exported as the canonical impl; C version deleted from mm_sysldr_shims.c.
#[no_mangle]
pub unsafe extern "system" fn MiLookupDataTableEntry(address: PVOID) -> *mut LDR_DATA_TABLE_ENTRY {
    let mut next_entry = PsLoadedModuleList.Flink;
    let list_head = core::ptr::addr_of_mut!(PsLoadedModuleList);
    loop {
        let ldr_entry = (next_entry as *mut u8).sub(
            core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)
        ) as *mut LDR_DATA_TABLE_ENTRY;
        let base = (*ldr_entry).DllBase as ULONG_PTR;
        let end = base + (*ldr_entry).SizeOfImage as ULONG_PTR;
        if (address as ULONG_PTR) >= base && (address as ULONG_PTR) < end {
            return ldr_entry;
        }
        next_entry = (*next_entry).Flink;
        if next_entry == list_head {
            break;
        }
    }
    core::ptr::null_mut()
}

// Case-insensitive ASCII compare-N. Returns 0 on match (matches C semantics).
unsafe fn strnicmp_ascii(a: *const i8, b: &[u8], n: usize) -> i32 {
    for i in 0..n {
        let av = (*a.add(i)) as u8;
        let bv = b[i];
        let alc = if av.is_ascii_uppercase() { av | 0x20 } else { av };
        let blc = if bv.is_ascii_uppercase() { bv | 0x20 } else { bv };
        if alc != blc { return (alc as i32) - (blc as i32); }
    }
    0
}

// Translated from C MiResolveImageReferences in mm_sysldr_shims.c.
// Walks the PE import descriptors, lazily loads each referenced DLL via
// MmLoadSystemImage (still C this round; will be Rust after the next round),
// then per-import calls Rust MiSnapThunk to resolve every IAT entry. Builds a
// deduplicated LOAD_IMPORTS array for the caller to track refcounts, with the
// single-entry MM_SYSLDR_SINGLE_ENTRY pointer-LSB encoding to save an
// allocation when only one import remains.
#[no_mangle]
pub unsafe extern "system" fn MiResolveImageReferences(
    image_base: PVOID,
    image_file_directory: *mut _UNICODE_STRING,
    name_prefix: *mut _UNICODE_STRING,
    missing_api: *mut *mut i8,
    missing_driver: *mut *mut u16,
    load_imports: *mut PVOID,
) -> NTSTATUS {
    static DRIVERS_FOLDER_BUF: [u16; 8] = [b'd' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'\\' as u16];
    let mut drivers_folder_name = _UNICODE_STRING {
        Length: 16,
        MaximumLength: 16,
        Buffer: DRIVERS_FOLDER_BUF.as_ptr() as *mut u16,
    };

    let missing_api_buffer = *missing_api;
    let mut import_size: ULONG = 0;
    *load_imports = MM_SYSLDR_NO_IMPORTS;

    let import_descriptor = RtlImageDirectoryEntryToData(
        image_base, 1, IMAGE_DIRECTORY_ENTRY_IMPORT,
        core::ptr::addr_of_mut!(import_size),
    ) as PIMAGE_IMPORT_DESCRIPTOR;
    if import_descriptor.is_null() { return STATUS_SUCCESS; }

    let mut import_count: usize = 0;
    let mut cur = import_descriptor;
    while (*cur).Name != 0 && (*cur).__bindgen_anon_1.OriginalFirstThunk != 0 {
        import_count += 1;
        cur = cur.add(1);
    }

    let loaded_imports: *mut LOAD_IMPORTS = if import_count > 0 {
        let size = import_count * core::mem::size_of::<*mut _LDR_DATA_TABLE_ENTRY>()
                 + core::mem::size_of::<usize>();
        let p = ExAllocatePoolWithTag(PagedPool, size, TAG_LDR_IMPORTS) as *mut LOAD_IMPORTS;
        if !p.is_null() {
            core::ptr::write_bytes(p as *mut u8, 0, size);
            (*p).Count = import_count as _;
        }
        p
    } else {
        core::ptr::null_mut()
    };

    // Inner closure-style helper does the import walk and returns a Result so
    // the failure path is uniform.
    let mut name_string: _UNICODE_STRING = core::mem::zeroed();
    let mut import_count_built: usize = 0;

    let result: Result<(), NTSTATUS> = (|| -> Result<(), NTSTATUS> {
        let head = core::ptr::addr_of_mut!(PsLoadedModuleList);
        let mut gdi_link = false;
        let mut normal_link = false;
        let mut current_descriptor = import_descriptor;

        while (*current_descriptor).Name != 0 && (*current_descriptor).__bindgen_anon_1.OriginalFirstThunk != 0 {
            let import_name = ((image_base as ULONG_PTR) + (*current_descriptor).Name as ULONG_PTR) as *const i8;
            let imp_len = strlen_(import_name);

            let is_win32k = imp_len >= 6 && strnicmp_ascii(import_name, b"win32k", 6) == 0;
            gdi_link = gdi_link || is_win32k;
            normal_link = normal_link || (
                !is_win32k &&
                !(imp_len >= 5 && strnicmp_ascii(import_name, b"dxapi", 5) == 0) &&
                !(imp_len >= 8 && strnicmp_ascii(import_name, b"coverage", 8) == 0) &&
                !(imp_len >= 3 && strnicmp_ascii(import_name, b"irt", 3) == 0)
            );
            if gdi_link && normal_link { return Err(STATUS_PROCEDURE_NOT_FOUND); }

            // User-mode driver imports forbidden.
            if (imp_len >= 5 && strnicmp_ascii(import_name, b"ntdll", 5) == 0)
                || (imp_len >= 6 && strnicmp_ascii(import_name, b"winsrv", 6) == 0)
                || (imp_len >= 8 && strnicmp_ascii(import_name, b"advapi32", 8) == 0)
                || (imp_len >= 8 && strnicmp_ascii(import_name, b"kernel32", 8) == 0)
                || (imp_len >= 6 && strnicmp_ascii(import_name, b"user32", 6) == 0)
                || (imp_len >= 5 && strnicmp_ascii(import_name, b"gdi32", 5) == 0)
            {
                return Err(STATUS_PROCEDURE_NOT_FOUND);
            }

            let reference_needed = !(
                (imp_len >= 8 && strnicmp_ascii(import_name, b"ntoskrnl", 8) == 0)
                || is_win32k
                || (imp_len >= 3 && strnicmp_ascii(import_name, b"hal", 3) == 0)
            );

            let mut temp_string: ANSI_STRING = core::mem::zeroed();
            RtlInitAnsiString(core::ptr::addr_of_mut!(temp_string), import_name);
            let st = RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(name_string),
                core::ptr::addr_of_mut!(temp_string),
                1u8,
            );
            if !NT_SUCCESS(st) { return Err(st); }

            let import_base: PVOID;
            let mut ldr_entry: *mut LDR_DATA_TABLE_ENTRY;

            // CheckDllState loop — re-scan after each lazy-load.
            loop {
                let mut found: PVOID = core::ptr::null_mut();
                let mut next = PsLoadedModuleList.Flink;
                ldr_entry = core::ptr::null_mut();
                while next != head {
                    let entry = (next as *mut u8).sub(
                        core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
                    ) as *mut LDR_DATA_TABLE_ENTRY;
                    if unicode_string_eq_ascii_ci(
                        core::ptr::addr_of!((*entry).BaseDllName),
                        import_name,
                    ) {
                        found = (*entry).DllBase;
                        ldr_entry = entry;
                        if reference_needed && ((*entry).Flags & LDRP_LOAD_IN_PROGRESS) == 0 {
                            (*entry).LoadCount += 1;
                        }
                        break;
                    }
                    next = (*next).Flink;
                }
                if !found.is_null() {
                    import_base = found;
                    break;
                }

                // Lazy-load the missing DLL.
                let mut dll_name = _UNICODE_STRING {
                    Length: 0,
                    MaximumLength: name_string.Length + (*image_file_directory).Length + 2,
                    Buffer: core::ptr::null_mut(),
                };
                dll_name.Buffer = ExAllocatePoolWithTag(
                    NonPagedPool, dll_name.MaximumLength as usize, TAG_LDR_WSTR,
                ) as *mut u16;
                if dll_name.Buffer.is_null() {
                    return Err(STATUS_INSUFFICIENT_RESOURCES);
                }
                RtlCopyUnicodeString(core::ptr::addr_of_mut!(dll_name), image_file_directory);
                RtlAppendUnicodeStringToString(
                    core::ptr::addr_of_mut!(dll_name),
                    core::ptr::addr_of_mut!(name_string),
                );

                let mut dll_entry: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                let mut dll_base: PVOID = core::ptr::null_mut();
                let mut load_status = MmLoadSystemImage(
                    core::ptr::addr_of_mut!(dll_name),
                    name_prefix,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(dll_entry) as *mut PVOID,
                    core::ptr::addr_of_mut!(dll_base),
                );

                // win32k / GDI drivers can also import from drivers\ subdir.
                // C does `(MI_IS_SESSION_ADDRESS(ImageBase) || 1)` — always true.
                if load_status == STATUS_OBJECT_NAME_NOT_FOUND {
                    ExFreePoolWithTag(dll_name.Buffer as PVOID, TAG_LDR_WSTR);
                    dll_name.MaximumLength += drivers_folder_name.Length;
                    dll_name.Buffer = ExAllocatePoolWithTag(
                        NonPagedPool, dll_name.MaximumLength as usize, TAG_LDR_WSTR,
                    ) as *mut u16;
                    if dll_name.Buffer.is_null() {
                        return Err(STATUS_INSUFFICIENT_RESOURCES);
                    }
                    dll_name.Length = 0;
                    RtlCopyUnicodeString(core::ptr::addr_of_mut!(dll_name), image_file_directory);
                    RtlAppendUnicodeStringToString(
                        core::ptr::addr_of_mut!(dll_name),
                        core::ptr::addr_of_mut!(drivers_folder_name),
                    );
                    RtlAppendUnicodeStringToString(
                        core::ptr::addr_of_mut!(dll_name),
                        core::ptr::addr_of_mut!(name_string),
                    );

                    load_status = MmLoadSystemImage(
                        core::ptr::addr_of_mut!(dll_name),
                        name_prefix,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::addr_of_mut!(dll_entry) as *mut PVOID,
                        core::ptr::addr_of_mut!(dll_base),
                    );
                }

                if !NT_SUCCESS(load_status) {
                    *missing_driver = dll_name.Buffer;
                    *(missing_driver as *mut ULONG_PTR) |= 1;
                    *missing_api = core::ptr::null_mut();
                    return Err(load_status);
                }

                ExFreePoolWithTag(dll_name.Buffer as PVOID, TAG_LDR_WSTR);

                let init_status = MmCallDllInitialize(dll_entry, head);
                if !NT_SUCCESS(init_status) {
                    MmUnloadSystemImage(dll_entry as PVOID);
                    return Err(init_status);
                }

                ldr_entry = dll_entry;
                import_base = dll_base;
                break;
            }

            if reference_needed && !loaded_imports.is_null()
                && ((*ldr_entry).Flags & LDRP_LOAD_IN_PROGRESS) == 0
            {
                let slot = (*loaded_imports).Entry.as_mut_ptr().add(import_count_built);
                *slot = ldr_entry;
                import_count_built += 1;
            }

            RtlFreeUnicodeString(core::ptr::addr_of_mut!(name_string));
            name_string.Buffer = core::ptr::null_mut();

            *missing_driver = (*ldr_entry).BaseDllName.Buffer;

            let mut export_size: ULONG = 0;
            let export_directory = RtlImageDirectoryEntryToData(
                import_base, 1, IMAGE_DIRECTORY_ENTRY_EXPORT,
                core::ptr::addr_of_mut!(export_size),
            ) as PIMAGE_EXPORT_DIRECTORY;
            if export_directory.is_null() {
                return Err(STATUS_DRIVER_ENTRYPOINT_NOT_FOUND);
            }

            let oft = (*current_descriptor).__bindgen_anon_1.OriginalFirstThunk;
            if oft != 0 {
                let mut orig_thunk = ((image_base as ULONG_PTR) + oft as ULONG_PTR) as *mut IMAGE_THUNK_DATA;
                let mut first_thunk = ((image_base as ULONG_PTR) + (*current_descriptor).FirstThunk as ULONG_PTR) as *mut IMAGE_THUNK_DATA;
                while (*orig_thunk).u1.AddressOfData != 0 {
                    let snap_status = MiSnapThunk(
                        import_base, image_base,
                        orig_thunk, first_thunk,
                        export_directory, export_size,
                        0u8, missing_api,
                    );
                    orig_thunk = orig_thunk.add(1);
                    first_thunk = first_thunk.add(1);
                    if !NT_SUCCESS(snap_status) { return Err(snap_status); }
                    *missing_api = missing_api_buffer;
                }
            }

            current_descriptor = current_descriptor.add(1);
        }
        Ok(())
    })();

    // Free name_string if still allocated (failure path).
    if !name_string.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(name_string));
    }

    if let Err(status) = result {
        // MiDereferenceImports is a no-op in the C shim; just free the array.
        if !loaded_imports.is_null() {
            ExFreePoolWithTag(loaded_imports as PVOID, TAG_LDR_IMPORTS);
        }
        return status;
    }

    // Compress LoadedImports (drop NULL entries; encode single in pointer LSB).
    if !loaded_imports.is_null() {
        let total = (*loaded_imports).Count as usize;
        let mut single_entry: *mut _LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
        let mut nonzero: usize = 0;
        for i in 0..total {
            let slot = *(*loaded_imports).Entry.as_ptr().add(i);
            if !slot.is_null() {
                single_entry = ((slot as ULONG_PTR) | MM_SYSLDR_SINGLE_ENTRY) as *mut _LDR_DATA_TABLE_ENTRY;
                nonzero += 1;
            }
        }

        if nonzero == 0 {
            ExFreePoolWithTag(loaded_imports as PVOID, TAG_LDR_IMPORTS);
            *load_imports = MM_SYSLDR_NO_IMPORTS;
        } else if nonzero == 1 {
            ExFreePoolWithTag(loaded_imports as PVOID, TAG_LDR_IMPORTS);
            *load_imports = single_entry as PVOID;
        } else if nonzero != total {
            let new_size = nonzero * core::mem::size_of::<*mut _LDR_DATA_TABLE_ENTRY>()
                         + core::mem::size_of::<usize>();
            let new_imports = ExAllocatePoolWithTag(PagedPool, new_size, TAG_LDR_IMPORTS) as *mut LOAD_IMPORTS;
            if !new_imports.is_null() {
                (*new_imports).Count = 0;
                let mut written: usize = 0;
                for i in 0..total {
                    let slot = *(*loaded_imports).Entry.as_ptr().add(i);
                    if !slot.is_null() {
                        let dst = (*new_imports).Entry.as_mut_ptr().add(written);
                        *dst = slot;
                        written += 1;
                    }
                }
                (*new_imports).Count = written as _;
                ExFreePoolWithTag(loaded_imports as PVOID, TAG_LDR_IMPORTS);
                *load_imports = new_imports as PVOID;
            } else {
                *load_imports = loaded_imports as PVOID;
            }
        } else {
            *load_imports = loaded_imports as PVOID;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmPageEntireDriver(address_within_section: PVOID) -> PVOID {
    let address_within_section = opaque_ptr(address_within_section);
    let ldr_entry = MiLookupDataTableEntry(address_within_section);
    if ldr_entry.is_null() {
        return core::ptr::null_mut();
    }

    if MmDisablePagingExecutive != 0 || !(*ldr_entry).__bindgen_anon_1.__bindgen_anon_1.SectionPointer.is_null() {
        return (*ldr_entry).DllBase;
    }

    KeFlushQueuedDpcs();

    let start_pte = MiAddressToPte((*ldr_entry).DllBase as ULONG_PTR as PVOID);
    let end_pte = MiAddressToPte(((*ldr_entry).DllBase as ULONG_PTR + (*ldr_entry).SizeOfImage as ULONG_PTR) as PVOID);

    MiSetPagingOfDriver(start_pte, end_pte);

    (*ldr_entry).DllBase
}

#[no_mangle]
pub unsafe extern "system" fn MmResetDriverPaging(_address_within_section: PVOID) {
    let _address_within_section = opaque_ptr(_address_within_section);
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSystemRoutineAddress(system_routine_name: *mut _UNICODE_STRING) -> PVOID {
    let system_routine_name = opaque_ptr(system_routine_name);
    let mut ansi_routine_name: ANSI_STRING = core::mem::zeroed();
    let status = RtlUnicodeStringToAnsiString(
        core::ptr::addr_of_mut!(ansi_routine_name),
        system_routine_name,
        1,
    );
    if status < 0 {
        return core::ptr::null_mut();
    }

    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(PsLoadedModuleResource), 1);

    let mut proc_address: PVOID = core::ptr::null_mut();
    static KERNEL_NAME: [u16; 13] = [110,116,111,115,107,114,110,108,46,101,120,101,0];
    static HAL_NAME: [u16; 8] = [104,97,108,46,100,108,108,0];

    let kernel_name = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: KERNEL_NAME.as_ptr() as *mut u16,
    };
    let hal_name = _UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: HAL_NAME.as_ptr() as *mut u16,
    };

    let mut modules = 0usize;
    let mut found = 0u8;
    let mut total_modules = 0usize;
    let mut next_entry = (*core::ptr::addr_of_mut!(PsLoadedModuleList)).Flink;
    while next_entry != core::ptr::addr_of_mut!(PsLoadedModuleList) {
        let ldr_entry = next_entry as *mut LDR_DATA_TABLE_ENTRY;
        total_modules += 1;

        if RtlEqualUnicodeString(core::ptr::addr_of!(kernel_name), core::ptr::addr_of_mut!((*ldr_entry).BaseDllName), 1) != 0 {
            found = 1u8;
            modules += 1;
        } else if RtlEqualUnicodeString(core::ptr::addr_of!(hal_name), core::ptr::addr_of_mut!((*ldr_entry).BaseDllName), 1) != 0 {
            found = 1u8;
            modules += 1;
        }

        if found != 0 {
            proc_address = RtlFindExportedRoutineByName((*ldr_entry).DllBase, ansi_routine_name.Buffer);
            // i386 LLD exports stdcall functions with decoration (_Name@N).
            // If undecorated lookup failed, try linear scan stripping decoration.
            #[cfg(target_arch = "x86")]
            if proc_address.is_null() {
                proc_address = find_export_stripping_decoration((*ldr_entry).DllBase, ansi_routine_name.Buffer);
            }
            if !proc_address.is_null() || modules == 2 {
                break;
            }
        }

        next_entry = (*next_entry).Flink;
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(PsLoadedModuleResource));
    KeLeaveCriticalRegion();

    if proc_address.is_null() {
        crate::dbg_print!(b"MmGetSystemRoutineAddress: '%s' NOT FOUND (checked %lu modules, matched %lu)\n\0", ansi_routine_name.Buffer, total_modules as u32, modules as u32);
    }

    RtlFreeAnsiString(core::ptr::addr_of_mut!(ansi_routine_name));
    proc_address
}

/// i386 LLD exports stdcall functions with `_Name@N` decoration in the PE
/// export table. This helper does a linear scan of exports, stripping the
/// leading `_` and trailing `@N` before comparing with the undecorated name.
#[cfg(target_arch = "x86")]
unsafe fn find_export_stripping_decoration(image_base: PVOID, name: *const i8) -> PVOID {
    use core::ffi::CStr;
    let target = CStr::from_ptr(name).to_bytes();
    if target.is_empty() {
        return core::ptr::null_mut();
    }

    let mut export_size: ULONG = 0;
    let export_dir = RtlImageDirectoryEntryToData(
        image_base, 1, IMAGE_DIRECTORY_ENTRY_EXPORT,
        core::ptr::addr_of_mut!(export_size),
    ) as PIMAGE_EXPORT_DIRECTORY;
    if export_dir.is_null() {
        return core::ptr::null_mut();
    }

    let base = image_base as usize;
    let num_names = (*export_dir).NumberOfNames;
    let name_table = (base + (*export_dir).AddressOfNames as usize) as *const ULONG;
    let ordinal_table = (base + (*export_dir).AddressOfNameOrdinals as usize) as *const u16;
    let func_table = (base + (*export_dir).AddressOfFunctions as usize) as *const ULONG;

    for i in 0..num_names {
        let export_name_ptr = (base + *name_table.add(i as usize) as usize) as *const u8;
        let export_name = CStr::from_ptr(export_name_ptr as *const i8).to_bytes();

        // Check for _Name@N pattern: starts with _, ends with @digits
        if export_name.len() > 1 && export_name[0] == b'_' {
            let stripped = &export_name[1..]; // remove leading _
            // Find trailing @N
            if let Some(at_pos) = stripped.iter().rposition(|&b| b == b'@') {
                let base_name = &stripped[..at_pos];
                if base_name == target {
                    let ordinal = *ordinal_table.add(i as usize);
                    if (ordinal as ULONG) < (*export_dir).NumberOfFunctions {
                        let rva = *func_table.add(ordinal as usize);
                        let func_addr = base + rva as usize;
                        // Check it's not a forwarder (inside export directory)
                        let dir_start = export_dir as usize;
                        let dir_end = dir_start + export_size as usize;
                        if func_addr >= dir_start && func_addr < dir_end {
                            continue; // forwarder, skip
                        }
                        return func_addr as PVOID;
                    }
                }
            }
        }
    }
    core::ptr::null_mut()
}
extern "system" {
    fn RtlImageDirectoryEntryToData(base: PVOID, mapped: u32, dir: u16, size: *mut ULONG) -> PVOID;
    fn RtlUnicodeStringToAnsiString(dest: *mut ANSI_STRING, src: *mut _UNICODE_STRING, alloc: u32) -> NTSTATUS;
    fn RtlFreeAnsiString(s: *mut ANSI_STRING);
    // RtlEqualUnicodeString returns BOOLEAN (u8) and takes BOOLEAN CaseInSensitive.
    // Declaring it `u32 -> u32` was an ABI mismatch: on amd64 the impl sets only AL,
    // so the caller read garbage upper EAX bits and saw a false "equal" — making
    // win32k's "ftfd.dll" import match the same-length "usbd.sys" module (BUGCHECK 0x4C).
    fn RtlEqualUnicodeString(s1: *const _UNICODE_STRING, s2: *mut _UNICODE_STRING, case_insensitive: u8) -> u8;
    fn RtlAppendUnicodeStringToString(dest: *mut _UNICODE_STRING, src: *mut _UNICODE_STRING) -> NTSTATUS;
    fn RtlInitAnsiString(dest: *mut ANSI_STRING, src: *const i8);
    fn RtlCopyUnicodeString(dest: *mut _UNICODE_STRING, src: *const _UNICODE_STRING);
    fn DbgUnLoadImageSymbols(name: *mut ANSI_STRING, base: PVOID, pid: ULONG_PTR);
    fn PsGetCurrentProcessId() -> HANDLE;
    fn MmMapViewOfSection( section: PSECTION, process: PEPROCESS, base: *mut PVOID, zero_bits: ULONG_PTR, commit_size: usize, section_offset: *mut LARGE_INTEGER, view_size: *mut usize, inherit: u32, alloc_type: ULONG, protect: ULONG, ) -> NTSTATUS;
    fn MmUnmapViewOfSection(process: PEPROCESS, base: PVOID) -> NTSTATUS;
    // MmLoadSystemImage is now a Rust impl in the same module; the recursive
    // call from MiResolveImageReferences resolves directly without a linker
    // round-trip.
    static MmSectionObjectType: POBJECT_TYPE;
}

const IMAGE_DIRECTORY_ENTRY_DEBUG: u16 = 6;
const IMAGE_DIRECTORY_ENTRY_EXPORT: u16 = 0;
const IMAGE_DIRECTORY_ENTRY_IMPORT: u16 = 1;

type PIMAGE_IMPORT_BY_NAME = *mut IMAGE_IMPORT_BY_NAME;
#[repr(C)]
struct IMAGE_IMPORT_BY_NAME {
    Hint: u16,
    Name: [i8; 1],
}

type LOADER_PARAMETER_BLOCK = ntkrnl_sys::ddk::_LOADER_PARAMETER_BLOCK;
type MM_IMAGE_SECTION_OBJECT = core::ffi::c_void;
const ViewUnmap: u32 = 0x02;
const WrVirtualMemory: u32 = 0x00000012;
const MUTANT_INCREMENT: i32 = 1;
const LDRP_DEBUG_SYMBOLS_LOADED: ULONG = 0x100000;
const LDRP_LOAD_IN_PROGRESS: ULONG = 0x1000;
const LDRP_SYSTEM_MAPPED: ULONG = 0x1000000;
const LDRP_ENTRY_PROCESSED: ULONG = 0x4000;
const LDRP_MM_LOADED: ULONG = 0x40000000;
const LDRP_ENTRY_NATIVE: ULONG = 0x8000000;
const TAG_LDR_WSTR: ULONG = 0x73776C6B; // 'swlk' ntoskrnl/include/internal/tag.h
const TAG_LDR_IMPORTS: ULONG = 0x706d4c4d;
const TAG_MODULE_OBJECT: ULONG = 0x6f4d4d4d;
const SEC_IMAGE: ULONG = 0x1000000;

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/sysldr.c
//  * PURPOSE:         Contains the Kernel Loader (SYSLDR) for loading PE files.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// #ifdef _M_ARM64
// /* Declaration for direct file reading (declared in cache/section/newmm.h) */
// NTSTATUS NTAPI
// MiSimpleRead(PFILE_OBJECT FileObject,
//              PLARGE_INTEGER FileOffset,
//              PVOID Buffer,
//              ULONG Length,
//              BOOLEAN Paging,
//              PIO_STATUS_BLOCK ReadStatus);
// #endif
//
// /* GLOBALS ********************************************************************/
//
// LIST_ENTRY PsLoadedModuleList;
// LIST_ENTRY MmLoadedUserImageList;
// KSPIN_LOCK PsLoadedModuleSpinLock;
// ERESOURCE PsLoadedModuleResource;
// ULONG_PTR PsNtosImageBase;
// KMUTANT MmSystemLoadLock;
//
// PFN_NUMBER MmTotalSystemDriverPages;
//
// PVOID MmUnloadedDrivers;
// PVOID MmLastUnloadedDrivers;
//
// BOOLEAN MmMakeLowMemory;
// BOOLEAN MmEnforceWriteProtection = TRUE;
//
// PMMPTE MiKernelResourceStartPte, MiKernelResourceEndPte;
// ULONG_PTR ExPoolCodeStart, ExPoolCodeEnd, MmPoolCodeStart, MmPoolCodeEnd;
// ULONG_PTR MmPteCodeStart, MmPteCodeEnd;
//
// #ifdef _WIN64
// #define COOKIE_MAX 0x0000FFFFFFFFFFFFll
// #define DEFAULT_SECURITY_COOKIE 0x00002B992DDFA232ll
// #else
// #define DEFAULT_SECURITY_COOKIE 0xBB40E64E
// #endif
//
// /* FUNCTIONS ******************************************************************/
//
// PVOID
// NTAPI
// MiCacheImageSymbols(IN PVOID BaseAddress)
// {
//     ULONG DebugSize;
//     PVOID DebugDirectory = NULL;
//     PAGED_CODE();
//
//     /* Make sure it's safe to access the image */
//     _SEH2_TRY
//     {
//         /* Get the debug directory */
//         DebugDirectory = RtlImageDirectoryEntryToData(BaseAddress,
//                                                       TRUE,
//                                                       IMAGE_DIRECTORY_ENTRY_DEBUG,
//                                                       &DebugSize);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Nothing */
//     }
//     _SEH2_END;
//
//     /* Return the directory */
//     return DebugDirectory;
// }
//
// NTSTATUS
// NTAPI
// MiLoadImageSection(_Inout_ PSECTION *SectionPtr,
//                    _Out_ PVOID *ImageBase,
//                    _In_ PUNICODE_STRING FileName,
//                    _In_ BOOLEAN SessionLoad,
//                    _In_ PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PSECTION Section = *SectionPtr;
//     NTSTATUS Status;
//     PFN_COUNT PteCount;
//     PMMPTE PointerPte, LastPte;
//     PVOID DriverBase;
//     MMPTE TempPte;
//     KIRQL OldIrql;
//     PFN_NUMBER PageFrameIndex;
// #ifndef _M_ARM64
//     PEPROCESS Process;
//     PVOID Base = NULL;
//     SIZE_T ViewSize = 0;
//     KAPC_STATE ApcState;
//     LARGE_INTEGER SectionOffset = {{0, 0}};
//     BOOLEAN LoadSymbols = FALSE;
// #endif
//     PAGED_CODE();
//
//     /* Detect session load */
//     if (SessionLoad)
//     {
//         /* Fail */
//         UNIMPLEMENTED_DBGBREAK("Session loading not yet supported!\n");
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* Not session load, shouldn't have an entry */
//     ASSERT(LdrEntry == NULL);
//
//     /* Reserve system PTEs needed */
//     PteCount = ROUND_TO_PAGES(((PMM_IMAGE_SECTION_OBJECT)Section->Segment)->ImageInformation.ImageFileSize) >> PAGE_SHIFT;
//     PointerPte = MiReserveSystemPtes(PteCount, SystemPteSpace);
//     if (!PointerPte)
//     {
//         DPRINT1("MiReserveSystemPtes failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* New driver base */
//     LastPte = PointerPte + PteCount;
//     DriverBase = MiPteToAddress(PointerPte);
//     *ImageBase = DriverBase;
//
// #ifdef ARM64_UART_TRACE_ENABLE
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         if (_u)
//         {
//             const char *_s; ULONG_PTR _v; int _i;
//             _s = "[SYSMOD] base=0x"; while (*_s) *_u = (ULONG)*_s++;
//             _v = (ULONG_PTR)DriverBase;
//             for (_i = 60; _i >= 0; _i -= 4) {
//                 ULONG _n = (_v >> _i) & 0xF;
//                 *_u = _n < 10 ? '0' + _n : 'A' + _n - 10;
//             }
//             _s = " pages="; while (*_s) *_u = (ULONG)*_s++;
//             _v = (ULONG_PTR)PteCount;
//             for (_i = 12; _i >= 0; _i -= 4) {
//                 ULONG _n = (_v >> _i) & 0xF;
//                 *_u = _n < 10 ? '0' + _n : 'A' + _n - 10;
//             }
//             _s = " name="; while (*_s) *_u = (ULONG)*_s++;
//             if (FileName && FileName->Buffer) {
//                 USHORT _j;
//                 for (_j = 0; _j < FileName->Length / sizeof(WCHAR) && _j < 60; _j++)
//                     *_u = (ULONG)(FileName->Buffer[_j] & 0x7F);
//             }
//             *_u = '\n';
//         }
//     }
// #endif
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Loop the new driver PTEs */
//     TempPte = ValidKernelPte;
//     while (PointerPte < LastPte)
//     {
//         ASSERT(PointerPte->u.Hard.Valid == 0);
//         MI_SET_USAGE(MI_USAGE_DRIVER_PAGE);
// #if MI_TRACE_PFNS
//         MI_SET_PROCESS_USTR(FileName);
// #endif
//         PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
//         MiInitializePfn(PageFrameIndex, PointerPte, TRUE);
//         TempPte.set_page_frame_number((PageFrameIndex) as PFN_NUMBER);
//         MI_WRITE_VALID_PTE(PointerPte, TempPte);
//         PointerPte++;
//     }
//
//     MiReleasePfnLock(OldIrql);
//
// #ifdef _M_ARM64
//     /*
//      * ARM64: Read PE segments directly from the file into kernel memory.
//      * This bypasses SEC_IMAGE user-space mapping which has two bugs on ARM64:
//      * 1) Relocated SEC_IMAGE mappings return corrupted page data
//      * 2) MmUnmapViewOfSection hangs for large SEC_IMAGE mappings
//      */
//     {
//         PMM_IMAGE_SECTION_OBJECT ImageSectionObject =
//             (PMM_IMAGE_SECTION_OBJECT)Section->Segment;
//         PFILE_OBJECT FileObject = ImageSectionObject->FileObject;
//         ULONG i;
//         IO_STATUS_BLOCK IoStatusBlock;
//         LARGE_INTEGER FileOffset;
//         SIZE_T ImageSize = (SIZE_T)PteCount << PAGE_SHIFT;
//
//         /* Zero the entire image area first (handles BSS and gaps) */
//         RtlZeroMemory(DriverBase, ImageSize);
//
//         /* Read each segment from the file */
//         for (i = 0; i < ImageSectionObject->NrSegments; i++)
//         {
//             PMM_SECTION_SEGMENT Seg = &ImageSectionObject->Segments[i];
//             ULONG_PTR DestVA = Seg->Image.VirtualAddress;
//             ULONGLONG SrcFileOffset = Seg->Image.FileOffset;
//             SIZE_T RawLen = (SIZE_T)Seg->RawLength.QuadPart;
//
//             /* Skip segments with no file data */
//             if (RawLen == 0 || SrcFileOffset == 0)
//             {
//                 /* Segment 0 (PE header) may have FileOffset=0, still read it */
//                 if (i != 0 || RawLen == 0) continue;
//             }
//
//             /* Clamp to image bounds */
//             if (DestVA + RawLen > ImageSize)
//                 RawLen = ImageSize - DestVA;
//             if (DestVA >= ImageSize) continue;
//
//             /* Read in page-sized chunks to avoid MDL size limits */
//             {
//                 SIZE_T BytesRemaining = RawLen;
//                 SIZE_T DestOffset = DestVA;
//                 ULONGLONG SrcOffset = SrcFileOffset;
//
//                 while (BytesRemaining > 0)
//                 {
//                     ULONG ChunkSize = (BytesRemaining > PAGE_SIZE) ? PAGE_SIZE : (ULONG)BytesRemaining;
//
//                     FileOffset.QuadPart = SrcOffset;
//                     Status = MiSimpleRead(FileObject,
//                                           &FileOffset,
//                                           (PVOID)((ULONG_PTR)DriverBase + DestOffset),
//                                           ChunkSize,
//                                           TRUE,
//                                           &IoStatusBlock);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         break;
//                     }
//
//                     BytesRemaining -= ChunkSize;
//                     DestOffset += ChunkSize;
//                     SrcOffset += ChunkSize;
//                 }
//             }
//         }
//
//         Status = STATUS_SUCCESS;
//     }
// #else
//     /* Attach to the system process */
//     KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//
//     /* Check if we need to load symbols */
//     if (NtGlobalFlag & FLG_ENABLE_KDEBUG_SYMBOL_LOAD)
//     {
//         LoadSymbols = TRUE;
//         NtGlobalFlag &= ~FLG_ENABLE_KDEBUG_SYMBOL_LOAD;
//     }
//
//     /* Map the driver */
//     Process = PsGetCurrentProcess();
//     Status = MmMapViewOfSection(Section,
//                                 Process,
//                                 &Base,
//                                 0,
//                                 0,
//                                 &SectionOffset,
//                                 &ViewSize,
//                                 ViewUnmap,
//                                 0,
//                                 PAGE_EXECUTE);
//
//     /* Re-enable the flag */
//     if (LoadSymbols) NtGlobalFlag |= FLG_ENABLE_KDEBUG_SYMBOL_LOAD;
//
//     /* Check if we failed with distinguished status code */
//     if (Status == STATUS_IMAGE_MACHINE_TYPE_MISMATCH)
//     {
//         Status = STATUS_INVALID_IMAGE_FORMAT;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MmMapViewOfSection failed with status 0x%x\n", Status);
//         KeUnstackDetachProcess(&ApcState);
//         return Status;
//     }
//
//     /* Copy the image */
//     RtlCopyMemory(DriverBase, Base, PteCount << PAGE_SHIFT);
//
//     /* Now unmap the view */
//     Status = MmUnmapViewOfSection(Process, Base);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Detach and return status */
//     KeUnstackDetachProcess(&ApcState);
// #endif
//
//     return Status;
// }
//
// #ifndef RVA
// #define RVA(m, b) ((PVOID)((ULONG_PTR)(b) + (ULONG_PTR)(m)))
// #endif
//
// USHORT
// NTAPI
// NameToOrdinal(
//     _In_ PCSTR ExportName,
//     _In_ PVOID ImageBase,
//     _In_ ULONG NumberOfNames,
//     _In_ PULONG NameTable,
//     _In_ PUSHORT OrdinalTable)
// {
//     LONG Low, Mid, High, Ret;
//
//     /* Fail if no names */
//     if (!NumberOfNames)
//         return -1;
//
//     /* Do a binary search */
//     Low = Mid = 0;
//     High = NumberOfNames - 1;
//     while (High >= Low)
//     {
//         /* Get new middle value */
//         Mid = (Low + High) >> 1;
//
//         /* Compare name */
//         Ret = strcmp(ExportName, (PCHAR)RVA(ImageBase, NameTable[Mid]));
//         if (Ret < 0)
//         {
//             /* Update high */
//             High = Mid - 1;
//         }
//         else if (Ret > 0)
//         {
//             /* Update low */
//             Low = Mid + 1;
//         }
//         else
//         {
//             /* We got it */
//             break;
//         }
//     }
//
//     /* Check if we couldn't find it */
//     if (High < Low)
//         return -1;
//
//     /* Otherwise, this is the ordinal */
//     return OrdinalTable[Mid];
// }
//
// /**
//  * @brief
//  * ReactOS-only helper routine for RtlFindExportedRoutineByName(),
//  * that provides a finer granularity regarding the nature of the
//  * export, and the failure reasons.
//  *
//  * @param[in]   ImageBase
//  * The base address of the loaded image.
//  *
//  * @param[in]   ExportName
//  * The name of the export, given as an ANSI NULL-terminated string.
//  *
//  * @param[out]  Function
//  * The address of the named exported routine, or NULL if not found.
//  * If the export is a forwarder (see @p IsForwarder below), this
//  * address points to the forwarder name.
//  *
//  * @param[out]  IsForwarder
//  * An optional pointer to a BOOLEAN variable, that is set to TRUE
//  * if the found export is a forwarder, and FALSE otherwise.
//  *
//  * @param[in]   NotFoundStatus
//  * The status code to return in case the export could not be found
//  * (examples: STATUS_ENTRYPOINT_NOT_FOUND, STATUS_PROCEDURE_NOT_FOUND).
//  *
//  * @return
//  * A status code as follows:
//  * - STATUS_SUCCESS if the named exported routine is found;
//  * - The custom @p NotFoundStatus if the export could not be found;
//  * - STATUS_INVALID_PARAMETER if the image is invalid or does not
//  *   contain an Export Directory.
//  *
//  * @note
//  * See RtlFindExportedRoutineByName() for more remarks.
//  * Used by psmgr.c PspLookupSystemDllEntryPoint() as well.
//  **/
// NTSTATUS
// NTAPI
// RtlpFindExportedRoutineByName(
//     _In_ PVOID ImageBase,
//     _In_ PCSTR ExportName,
//     _Out_ PVOID* Function,
//     _Out_opt_ PBOOLEAN IsForwarder,
//     _In_ NTSTATUS NotFoundStatus)
// {
//     PIMAGE_EXPORT_DIRECTORY ExportDirectory;
//     PULONG NameTable;
//     PUSHORT OrdinalTable;
//     ULONG ExportSize;
//     USHORT Ordinal;
//     PULONG ExportTable;
//     ULONG_PTR FunctionAddress;
//
//     PAGED_CODE();
//
//     /* Get the export directory */
//     ExportDirectory = RtlImageDirectoryEntryToData(ImageBase,
//                                                    TRUE,
//                                                    IMAGE_DIRECTORY_ENTRY_EXPORT,
//                                                    &ExportSize);
//     if (!ExportDirectory)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Setup name tables */
//     NameTable = (PULONG)RVA(ImageBase, ExportDirectory->AddressOfNames);
//     OrdinalTable = (PUSHORT)RVA(ImageBase, ExportDirectory->AddressOfNameOrdinals);
//
//     /* Get the ordinal */
//     Ordinal = NameToOrdinal(ExportName,
//                             ImageBase,
//                             ExportDirectory->NumberOfNames,
//                             NameTable,
//                             OrdinalTable);
//
//     /* Check if we couldn't find it */
//     if (Ordinal == -1)
//         return NotFoundStatus;
//
//     /* Validate the ordinal */
//     if (Ordinal >= ExportDirectory->NumberOfFunctions)
//         return NotFoundStatus;
//
//     /* Resolve the function's address */
//     ExportTable = (PULONG)RVA(ImageBase, ExportDirectory->AddressOfFunctions);
//     FunctionAddress = (ULONG_PTR)RVA(ImageBase, ExportTable[Ordinal]);
//
//     /* Check if the function is actually a forwarder */
//     if (IsForwarder)
//     {
//         *IsForwarder = FALSE;
//         if ((FunctionAddress > (ULONG_PTR)ExportDirectory) &&
//             (FunctionAddress < (ULONG_PTR)ExportDirectory + ExportSize))
//         {
//             /* It is, and points to the forwarder name */
//             *IsForwarder = TRUE;
//         }
//     }
//
//     /* We've found it */
//     *Function = (PVOID)FunctionAddress;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Finds the address of a given named exported routine in a loaded image.
//  * Note that this function does not support forwarders.
//  *
//  * @param[in]   ImageBase
//  * The base address of the loaded image.
//  *
//  * @param[in]   ExportName
//  * The name of the export, given as an ANSI NULL-terminated string.
//  *
//  * @return
//  * The address of the named exported routine, or NULL if not found.
//  * If the export is a forwarder, this function returns NULL as well.
//  *
//  * @note
//  * This routine was originally named MiLocateExportName(), with a separate
//  * duplicated MiFindExportedRoutineByName() one (taking a PANSI_STRING)
//  * on Windows <= 2003. Both routines have been then merged and renamed
//  * to MiFindExportedRoutineByName() on Windows 8 (taking a PCSTR instead),
//  * and finally renamed and exported as RtlFindExportedRoutineByName() on
//  * Windows 10.
//  *
//  * @see https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/mm/sysload/mmgetsystemroutineaddress.htm
//  **/
// PVOID
// NTAPI
// RtlFindExportedRoutineByName(
//     _In_ PVOID ImageBase,
//     _In_ PCSTR ExportName)
// {
//     NTSTATUS Status;
//     BOOLEAN IsForwarder = FALSE;
//     PVOID Function;
//
//     PAGED_CODE();
//
//     /* Call the internal API */
//     Status = RtlpFindExportedRoutineByName(ImageBase,
//                                            ExportName,
//                                            &Function,
//                                            &IsForwarder,
//                                            STATUS_ENTRYPOINT_NOT_FOUND);
//     if (!NT_SUCCESS(Status))
//         return NULL;
//
//     /* If the export is actually a forwarder, log the error and fail */
//     if (IsForwarder)
//     {
//         DPRINT1("RtlFindExportedRoutineByName does not support forwarders!\n", FALSE);
//         return NULL;
//     }
//
//     /* We've found the export */
//     return Function;
// }
//
// NTSTATUS
// NTAPI
// MmCallDllInitialize(
//     _In_ PLDR_DATA_TABLE_ENTRY LdrEntry,
//     _In_ PLIST_ENTRY ModuleListHead)
// {
//     UNICODE_STRING ServicesKeyName = RTL_CONSTANT_STRING(
//         L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\");
//     PMM_DLL_INITIALIZE DllInit;
//     UNICODE_STRING RegPath, ImportName;
//     PCWCH Extension;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Try to see if the image exports a DllInitialize routine */
//     DllInit = (PMM_DLL_INITIALIZE)
//         RtlFindExportedRoutineByName(LdrEntry->DllBase, "DllInitialize");
//     if (!DllInit)
//         return STATUS_SUCCESS;
//
//     /* Make a temporary copy of BaseDllName because we will alter its length */
//     ImportName.Length = LdrEntry->BaseDllName.Length;
//     ImportName.MaximumLength = LdrEntry->BaseDllName.MaximumLength;
//     ImportName.Buffer = LdrEntry->BaseDllName.Buffer;
//
//     /* Obtain the path to this dll's service in the registry */
//     RegPath.MaximumLength = ServicesKeyName.Length +
//         ImportName.Length + sizeof(UNICODE_NULL);
//     RegPath.Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                            RegPath.MaximumLength,
//                                            TAG_LDR_WSTR);
//
//     /* Check if this allocation was unsuccessful */
//     if (!RegPath.Buffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Build and append the service name itself */
//     RegPath.Length = ServicesKeyName.Length;
//     RtlCopyMemory(RegPath.Buffer,
//                   ServicesKeyName.Buffer,
//                   ServicesKeyName.Length);
//
//     /* If the filename has an extension, remove it */
//     Extension = wcschr(ImportName.Buffer, L'.');
//     if (Extension)
//         ImportName.Length = (USHORT)(Extension - ImportName.Buffer) * sizeof(WCHAR);
//
//     /* Append the service name (base name without extension) */
//     RtlAppendUnicodeStringToString(&RegPath, &ImportName);
//
//     /* Now call DllInitialize */
//     DPRINT("Calling DllInit(%wZ)\n", &RegPath);
//     Status = DllInit(&RegPath);
//
//     /* Clean up */
//     ExFreePoolWithTag(RegPath.Buffer, TAG_LDR_WSTR);
//
//     // TODO: This is for Driver Verifier support.
//     UNREFERENCED_PARAMETER(ModuleListHead);
//
//     /* Return the DllInitialize status value */
//     return Status;
// }
//
// BOOLEAN
// MiCallDllUnloadAndUnloadDll(
//     _In_ PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     NTSTATUS Status;
//     PMM_DLL_UNLOAD DllUnload;
//
//     PAGED_CODE();
//
//     /* Retrieve the DllUnload routine */
//     DllUnload = (PMM_DLL_UNLOAD)
//         RtlFindExportedRoutineByName(LdrEntry->DllBase, "DllUnload");
//     if (!DllUnload)
//         return FALSE;
//
//     /* Call it and check for success */
//     Status = DllUnload();
//     if (!NT_SUCCESS(Status))
//         return FALSE;
//
//     /* Lie about the load count so we can unload the image */
//     ASSERT(LdrEntry->LoadCount == 0);
//     LdrEntry->LoadCount = 1;
//
//     /* Unload it */
//     MmUnloadSystemImage(LdrEntry);
//     return TRUE;
// }
//
// NTSTATUS
// NTAPI
// MiDereferenceImports(IN PLOAD_IMPORTS ImportList)
// {
//     SIZE_T i;
//     LOAD_IMPORTS SingleEntry;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PVOID CurrentImports;
//     PAGED_CODE();
//
//     /* Check if there's no imports or if we're a boot driver */
//     if ((ImportList == MM_SYSLDR_NO_IMPORTS) ||
//         (ImportList == MM_SYSLDR_BOOT_LOADED) ||
//         (ImportList->Count == 0))
//     {
//         /* Then there's nothing to do */
//         return STATUS_SUCCESS;
//     }
//
//     /* Check for single-entry */
//     if ((ULONG_PTR)ImportList & MM_SYSLDR_SINGLE_ENTRY)
//     {
//         /* Set it up */
//         SingleEntry.Count = 1;
//         SingleEntry.Entry[0] = (PVOID)((ULONG_PTR)ImportList &~ MM_SYSLDR_SINGLE_ENTRY);
//
//         /* Use this as the import list */
//         ImportList = &SingleEntry;
//     }
//
//     /* Loop the import list */
//     for (i = 0; (i < ImportList->Count) && (ImportList->Entry[i]); i++)
//     {
//         /* Get the entry */
//         LdrEntry = ImportList->Entry[i];
//         DPRINT1("%wZ <%wZ>\n", &LdrEntry->FullDllName, &LdrEntry->BaseDllName);
//
//         /* Skip boot loaded images */
//         if (LdrEntry->LoadedImports == MM_SYSLDR_BOOT_LOADED) continue;
//
//         /* Dereference the entry */
//         ASSERT(LdrEntry->LoadCount >= 1);
//         if (!--LdrEntry->LoadCount)
//         {
//             /* Save the import data in case unload fails */
//             CurrentImports = LdrEntry->LoadedImports;
//
//             /* This is the last entry */
//             LdrEntry->LoadedImports = MM_SYSLDR_NO_IMPORTS;
//             if (MiCallDllUnloadAndUnloadDll(LdrEntry))
//             {
//                 /* Unloading worked, parse this DLL's imports too */
//                 MiDereferenceImports(CurrentImports);
//
//                 /* Check if we had valid imports */
//                 if ((CurrentImports != MM_SYSLDR_BOOT_LOADED) &&
//                     (CurrentImports != MM_SYSLDR_NO_IMPORTS) &&
//                     !((ULONG_PTR)CurrentImports & MM_SYSLDR_SINGLE_ENTRY))
//                 {
//                     /* Free them */
//                     ExFreePoolWithTag(CurrentImports, TAG_LDR_IMPORTS);
//                 }
//             }
//             else
//             {
//                 /* Unload failed, restore imports */
//                 LdrEntry->LoadedImports = CurrentImports;
//             }
//         }
//     }
//
//     /* Done */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// MiClearImports(IN PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PAGED_CODE();
//
//     /* Check if there's no imports or we're a boot driver or only one entry */
//     if ((LdrEntry->LoadedImports == MM_SYSLDR_BOOT_LOADED) ||
//         (LdrEntry->LoadedImports == MM_SYSLDR_NO_IMPORTS) ||
//         ((ULONG_PTR)LdrEntry->LoadedImports & MM_SYSLDR_SINGLE_ENTRY))
//     {
//         /* Nothing to do */
//         return;
//     }
//
//     /* Otherwise, free the import list */
//     ExFreePoolWithTag(LdrEntry->LoadedImports, TAG_LDR_IMPORTS);
//     LdrEntry->LoadedImports = MM_SYSLDR_BOOT_LOADED;
// }
//
// VOID
// NTAPI
// MiProcessLoaderEntry(IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                      IN BOOLEAN Insert)
// {
//     KIRQL OldIrql;
//
//     /* Acquire module list lock */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&PsLoadedModuleResource, TRUE);
//
//     /* Acquire the spinlock too as we will insert or remove the entry */
//     OldIrql = KeAcquireSpinLockRaiseToSynch(&PsLoadedModuleSpinLock);
//
//     /* Insert or remove from the list */
//     if (Insert)
//         InsertTailList(&PsLoadedModuleList, &LdrEntry->InLoadOrderLinks);
//     else
//         RemoveEntryList(&LdrEntry->InLoadOrderLinks);
//
//     /* Release locks */
//     KeReleaseSpinLock(&PsLoadedModuleSpinLock, OldIrql);
//     ExReleaseResourceLite(&PsLoadedModuleResource);
//     KeLeaveCriticalRegion();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiUpdateThunks(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                IN PVOID OldBase,
//                IN PVOID NewBase,
//                IN ULONG Size)
// {
//     ULONG_PTR OldBaseTop, Delta;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY NextEntry;
//     ULONG ImportSize;
//     //
//     // FIXME: MINGW-W64 must fix LD to generate drivers that Windows can load,
//     // since a real version of Windows would fail at this point, but they seem
//     // busy implementing features such as "HotPatch" support in GCC 4.6 instead,
//     // a feature which isn't even used by Windows. Priorities, priorities...
//     // Please note that Microsoft WDK EULA and license prohibits using
//     // the information contained within it for the generation of "non-Windows"
//     // drivers, which is precisely what LD will generate, since an LD driver
//     // will not load on Windows.
//     //
// #ifdef _WORKING_LINKER_
//     ULONG i;
// #endif
//     PULONG_PTR ImageThunk;
//     PIMAGE_IMPORT_DESCRIPTOR ImportDescriptor;
//
//     /* Calculate the top and delta */
//     OldBaseTop = (ULONG_PTR)OldBase + Size - 1;
//     Delta = (ULONG_PTR)NewBase - (ULONG_PTR)OldBase;
//
//     /* Loop the loader block */
//     for (NextEntry = LoaderBlock->LoadOrderListHead.Flink;
//          NextEntry != &LoaderBlock->LoadOrderListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the loader entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
// #ifdef _WORKING_LINKER_
//         /* Get the IAT */
//         ImageThunk = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                   TRUE,
//                                                   IMAGE_DIRECTORY_ENTRY_IAT,
//                                                   &ImportSize);
//         if (!ImageThunk) continue;
//
//         /* Make sure we have an IAT */
//         DPRINT("[Mm0]: Updating thunks in: %wZ\n", &LdrEntry->BaseDllName);
//         for (i = 0; i < ImportSize; i++, ImageThunk++)
//         {
//             /* Check if it's within this module */
//             if ((*ImageThunk >= (ULONG_PTR)OldBase) && (*ImageThunk <= OldBaseTop))
//             {
//                 /* Relocate it */
//                 DPRINT("[Mm0]: Updating IAT at: %p. Old Entry: %p. New Entry: %p.\n",
//                         ImageThunk, *ImageThunk, *ImageThunk + Delta);
//                 *ImageThunk += Delta;
//             }
//         }
// #else
//         /* Get the import table */
//         ImportDescriptor = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                         TRUE,
//                                                         IMAGE_DIRECTORY_ENTRY_IMPORT,
//                                                         &ImportSize);
//         if (!ImportDescriptor) continue;
//
//         /* Make sure we have an IAT */
//         DPRINT("[Mm0]: Updating thunks in: %wZ\n", &LdrEntry->BaseDllName);
//         while ((ImportDescriptor->Name) &&
//                (ImportDescriptor->OriginalFirstThunk))
//         {
//             /* Get the image thunk */
//             ImageThunk = (PVOID)((ULONG_PTR)LdrEntry->DllBase +
//                                  ImportDescriptor->FirstThunk);
//             while (*ImageThunk)
//             {
//                 /* Check if it's within this module */
//                 if ((*ImageThunk >= (ULONG_PTR)OldBase) && (*ImageThunk <= OldBaseTop))
//                 {
//                     /* Relocate it */
//                     DPRINT("[Mm0]: Updating IAT at: %p. Old Entry: %p. New Entry: %p.\n",
//                             ImageThunk, *ImageThunk, *ImageThunk + Delta);
//                     *ImageThunk += Delta;
//                 }
//
//                 /* Go to the next thunk */
//                 ImageThunk++;
//             }
//
//             /* Go to the next import */
//             ImportDescriptor++;
//         }
// #endif
//     }
// }
//
// NTSTATUS
// NTAPI
// MiSnapThunk(IN PVOID DllBase,
//             IN PVOID ImageBase,
//             IN PIMAGE_THUNK_DATA Name,
//             IN PIMAGE_THUNK_DATA Address,
//             IN PIMAGE_EXPORT_DIRECTORY ExportDirectory,
//             IN ULONG ExportSize,
//             IN BOOLEAN SnapForwarder,
//             OUT PCHAR *MissingApi)
// {
//     BOOLEAN IsOrdinal;
//     USHORT Ordinal;
//     PULONG NameTable;
//     PUSHORT OrdinalTable;
//     PIMAGE_IMPORT_BY_NAME NameImport;
//     USHORT Hint;
//     NTSTATUS Status;
//     PCHAR MissingForwarder;
//     CHAR NameBuffer[MAXIMUM_FILENAME_LENGTH];
//     PULONG ExportTable;
//     ANSI_STRING DllName;
//     UNICODE_STRING ForwarderName;
//     PLIST_ENTRY NextEntry;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     ULONG ForwardExportSize;
//     PIMAGE_EXPORT_DIRECTORY ForwardExportDirectory;
//     PIMAGE_IMPORT_BY_NAME ForwardName;
//     SIZE_T ForwardLength;
//     IMAGE_THUNK_DATA ForwardThunk;
//
//     PAGED_CODE();
//
//     /* Check if this is an ordinal */
//     IsOrdinal = IMAGE_SNAP_BY_ORDINAL(Name->u1.Ordinal);
//     if ((IsOrdinal) && !(SnapForwarder))
//     {
//         /* Get the ordinal number and set it as missing */
//         Ordinal = (USHORT)(IMAGE_ORDINAL(Name->u1.Ordinal) -
//                            ExportDirectory->Base);
//         *MissingApi = (PCHAR)(ULONG_PTR)Ordinal;
//     }
//     else
//     {
//         /* Get the VA if we don't have to snap */
//         if (!SnapForwarder) Name->u1.AddressOfData += (ULONG_PTR)ImageBase;
//         NameImport = (PIMAGE_IMPORT_BY_NAME)Name->u1.AddressOfData;
//
//         /* Copy the procedure name */
//         RtlStringCbCopyA(*MissingApi,
//                          MAXIMUM_FILENAME_LENGTH,
//                          (PCHAR)NameImport->Name);
//
//         /* Setup name tables */
//         DPRINT("Import name: %s\n", NameImport->Name);
//         NameTable = (PULONG)((ULONG_PTR)DllBase +
//                              ExportDirectory->AddressOfNames);
//         OrdinalTable = (PUSHORT)((ULONG_PTR)DllBase +
//                                  ExportDirectory->AddressOfNameOrdinals);
//
//         /* Get the hint and check if it's valid */
//         Hint = NameImport->Hint;
//         if ((Hint < ExportDirectory->NumberOfNames) &&
//             !(strcmp((PCHAR)NameImport->Name, (PCHAR)DllBase + NameTable[Hint])))
//         {
//             /* We have a match, get the ordinal number from here */
//             Ordinal = OrdinalTable[Hint];
//         }
//         else
//         {
//             /* Do a binary search */
//             Ordinal = NameToOrdinal((PCHAR)NameImport->Name,
//                                     DllBase,
//                                     ExportDirectory->NumberOfNames,
//                                     NameTable,
//                                     OrdinalTable);
//
//             /* Check if we couldn't find it */
//             if (Ordinal == -1)
//             {
//                 DPRINT1("Warning: Driver failed to load, %s not found\n", NameImport->Name);
//                 return STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
//             }
//         }
//     }
//
//     /* Check if the ordinal is valid */
//     if (Ordinal >= ExportDirectory->NumberOfFunctions)
//     {
//         /* It's not, fail */
//         Status = STATUS_DRIVER_ORDINAL_NOT_FOUND;
//     }
//     else
//     {
//         /* In case the forwarder is missing */
//         MissingForwarder = NameBuffer;
//
//         /* Resolve the address and write it */
//         ExportTable = (PULONG)((ULONG_PTR)DllBase +
//                                ExportDirectory->AddressOfFunctions);
//         Address->u1.Function = (ULONG_PTR)DllBase + ExportTable[Ordinal];
//
//         /* Assume success from now on */
//         Status = STATUS_SUCCESS;
//
//         /* Check if the function is actually a forwarder */
//         if ((Address->u1.Function > (ULONG_PTR)ExportDirectory) &&
//             (Address->u1.Function < (ULONG_PTR)ExportDirectory + ExportSize))
//         {
//             /* Now assume failure in case the forwarder doesn't exist */
//             Status = STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
//
//             /* Build the forwarder name */
//             DllName.Buffer = (PCHAR)Address->u1.Function;
//
//             /* lld (llvm linker) prepends '_' to forwarded export DLL names
//              * on i386 (e.g. "_HAL.WRITE_PORT_UCHAR" instead of
//              * "HAL.WRITE_PORT_UCHAR"). Strip it so module lookup succeeds. */
//             if (DllName.Buffer[0] == '_' && strchr(DllName.Buffer + 1, '.'))
//                 DllName.Buffer++;
//
//             DllName.Length = (USHORT)(strchr(DllName.Buffer, '.') -
//                                       DllName.Buffer) +
//                                       sizeof(ANSI_NULL);
//             DllName.MaximumLength = DllName.Length;
//
//             /* Convert it */
//             if (!NT_SUCCESS(RtlAnsiStringToUnicodeString(&ForwarderName,
//                                                          &DllName,
//                                                          TRUE)))
//             {
//                 /* We failed, just return an error */
//                 return Status;
//             }
//
//             /* Loop the module list */
//             NextEntry = PsLoadedModuleList.Flink;
//             while (NextEntry != &PsLoadedModuleList)
//             {
//                 /* Get the loader entry */
//                 LdrEntry = CONTAINING_RECORD(NextEntry,
//                                              LDR_DATA_TABLE_ENTRY,
//                                              InLoadOrderLinks);
//
//                 /* Check if it matches */
//                 if (RtlPrefixUnicodeString(&ForwarderName,
//                                            &LdrEntry->BaseDllName,
//                                            TRUE))
//                 {
//                     /* Get the forwarder export directory */
//                     ForwardExportDirectory =
//                         RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                      TRUE,
//                                                      IMAGE_DIRECTORY_ENTRY_EXPORT,
//                                                      &ForwardExportSize);
//                     if (!ForwardExportDirectory) break;
//
//                     /* Allocate a name entry */
//                     ForwardLength = strlen(DllName.Buffer + DllName.Length) +
//                                     sizeof(ANSI_NULL);
//                     ForwardName = ExAllocatePoolWithTag(PagedPool,
//                                                         sizeof(*ForwardName) +
//                                                         ForwardLength,
//                                                         TAG_LDR_WSTR);
//                     if (!ForwardName) break;
//
//                     /* Copy the data */
//                     RtlCopyMemory(&ForwardName->Name[0],
//                                   DllName.Buffer + DllName.Length,
//                                   ForwardLength);
//                     ForwardName->Hint = 0;
//
//                     /* Set the new address */
//                     ForwardThunk.u1.AddressOfData = (ULONG_PTR)ForwardName;
//
//                     /* Snap the forwarder */
//                     Status = MiSnapThunk(LdrEntry->DllBase,
//                                          ImageBase,
//                                          &ForwardThunk,
//                                          &ForwardThunk,
//                                          ForwardExportDirectory,
//                                          ForwardExportSize,
//                                          TRUE,
//                                          &MissingForwarder);
//
//                     /* Free the forwarder name and set the thunk */
//                     ExFreePoolWithTag(ForwardName, TAG_LDR_WSTR);
//                     Address->u1 = ForwardThunk.u1;
//                     break;
//                 }
//
//                 /* Go to the next entry */
//                 NextEntry = NextEntry->Flink;
//             }
//
//             /* Free the name */
//             RtlFreeUnicodeString(&ForwarderName);
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// MmUnloadSystemImage(IN PVOID ImageHandle)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry = ImageHandle;
//     PVOID BaseAddress = LdrEntry->DllBase;
//     NTSTATUS Status;
//     STRING TempName;
//     BOOLEAN HadEntry = FALSE;
//
//     /* Acquire the loader lock */
//     KeEnterCriticalRegion();
//     KeWaitForSingleObject(&MmSystemLoadLock,
//                           WrVirtualMemory,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     /* Check if this driver was loaded at boot and didn't get imports parsed */
//     if (LdrEntry->LoadedImports == MM_SYSLDR_BOOT_LOADED) goto Done;
//
//     /* We should still be alive */
//     ASSERT(LdrEntry->LoadCount != 0);
//     LdrEntry->LoadCount--;
//
//     /* Check if we're still loaded */
//     if (LdrEntry->LoadCount) goto Done;
//
//     /* We should cleanup... are symbols loaded */
//     if (LdrEntry->Flags & LDRP_DEBUG_SYMBOLS_LOADED)
//     {
//         /* Create the ANSI name */
//         Status = RtlUnicodeStringToAnsiString(&TempName,
//                                               &LdrEntry->BaseDllName,
//                                               TRUE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Unload the symbols */
//             DbgUnLoadImageSymbols(&TempName,
//                                   BaseAddress,
//                                   (ULONG_PTR)PsGetCurrentProcessId());
//             RtlFreeAnsiString(&TempName);
//         }
//     }
//
//     /* FIXME: Free the driver */
//     DPRINT1("Leaking driver: %wZ\n", &LdrEntry->BaseDllName);
//     //MmFreeSection(LdrEntry->DllBase);
//
//     /* Check if we're linked in */
//     if (LdrEntry->InLoadOrderLinks.Flink)
//     {
//         /* Remove us */
//         MiProcessLoaderEntry(LdrEntry, FALSE);
//         HadEntry = TRUE;
//     }
//
//     /* Dereference and clear the imports */
//     MiDereferenceImports(LdrEntry->LoadedImports);
//     MiClearImports(LdrEntry);
//
//     /* Check if the entry needs to go away */
//     if (HadEntry)
//     {
//         /* Check if it had a name */
//         if (LdrEntry->FullDllName.Buffer)
//         {
//             /* Free it */
//             ExFreePoolWithTag(LdrEntry->FullDllName.Buffer, TAG_LDR_WSTR);
//         }
//
//         /* Check if we had a section */
//         if (LdrEntry->SectionPointer)
//         {
//             /* Dereference it */
//             ObDereferenceObject(LdrEntry->SectionPointer);
//         }
//
//         /* Free the entry */
//         ExFreePoolWithTag(LdrEntry, TAG_MODULE_OBJECT);
//     }
//
//     /* Release the system lock and return */
// Done:
//     KeReleaseMutant(&MmSystemLoadLock, MUTANT_INCREMENT, FALSE, FALSE);
//     KeLeaveCriticalRegion();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MiResolveImageReferences(IN PVOID ImageBase,
//                          IN PUNICODE_STRING ImageFileDirectory,
//                          IN PUNICODE_STRING NamePrefix OPTIONAL,
//                          OUT PCHAR *MissingApi,
//                          OUT PWCHAR *MissingDriver,
//                          OUT PLOAD_IMPORTS *LoadImports)
// {
//     static UNICODE_STRING DriversFolderName = RTL_CONSTANT_STRING(L"drivers\\");
//     PCHAR MissingApiBuffer = *MissingApi, ImportName;
//     PIMAGE_IMPORT_DESCRIPTOR ImportDescriptor, CurrentImport;
//     ULONG ImportSize, ImportCount = 0, LoadedImportsSize, ExportSize;
//     PLOAD_IMPORTS LoadedImports, NewImports;
//     ULONG i;
//     BOOLEAN GdiLink, NormalLink;
//     BOOLEAN ReferenceNeeded, Loaded;
//     ANSI_STRING TempString;
//     UNICODE_STRING NameString, DllName;
//     PLDR_DATA_TABLE_ENTRY LdrEntry = NULL, DllEntry, ImportEntry = NULL;
//     PVOID ImportBase, DllBase;
//     PLIST_ENTRY NextEntry;
//     PIMAGE_EXPORT_DIRECTORY ExportDirectory;
//     NTSTATUS Status;
//     PIMAGE_THUNK_DATA OrigThunk, FirstThunk;
//
//     PAGED_CODE();
//
//     DPRINT("%s - ImageBase: %p. ImageFileDirectory: %wZ\n",
//            __FUNCTION__, ImageBase, ImageFileDirectory);
//
//     /* No name string buffer yet */
//     NameString.Buffer = NULL;
//
//     /* Assume no imports */
//     *LoadImports = MM_SYSLDR_NO_IMPORTS;
//
//     /* Get the import descriptor */
//     ImportDescriptor = RtlImageDirectoryEntryToData(ImageBase,
//                                                     TRUE,
//                                                     IMAGE_DIRECTORY_ENTRY_IMPORT,
//                                                     &ImportSize);
//     if (!ImportDescriptor) return STATUS_SUCCESS;
//
//     /* Loop all imports to count them */
//     for (CurrentImport = ImportDescriptor;
//          (CurrentImport->Name) && (CurrentImport->OriginalFirstThunk);
//          CurrentImport++)
//     {
//         /* One more */
//         ImportCount++;
//     }
//
//     /* Make sure we have non-zero imports */
//     if (ImportCount)
//     {
//         /* Calculate and allocate the list we'll need */
//         LoadedImportsSize = ImportCount * sizeof(PVOID) + sizeof(SIZE_T);
//         LoadedImports = ExAllocatePoolWithTag(PagedPool,
//                                               LoadedImportsSize,
//                                               TAG_LDR_IMPORTS);
//         if (LoadedImports)
//         {
//             /* Zero it */
//             RtlZeroMemory(LoadedImports, LoadedImportsSize);
//             LoadedImports->Count = ImportCount;
//         }
//     }
//     else
//     {
//         /* No table */
//         LoadedImports = NULL;
//     }
//
//     /* Reset the import count and loop descriptors again */
//     GdiLink = NormalLink = FALSE;
//     ImportCount = 0;
//     while ((ImportDescriptor->Name) && (ImportDescriptor->OriginalFirstThunk))
//     {
//         /* Get the name */
//         ImportName = (PCHAR)((ULONG_PTR)ImageBase + ImportDescriptor->Name);
//
//         /* Check if this is a GDI driver */
//         GdiLink = GdiLink ||
//                   !(_strnicmp(ImportName, "win32k", sizeof("win32k") - 1));
//
//         /* We can also allow dxapi (for Windows compat, allow IRT and coverage) */
//         NormalLink = NormalLink ||
//                      ((_strnicmp(ImportName, "win32k", sizeof("win32k") - 1)) &&
//                       (_strnicmp(ImportName, "dxapi", sizeof("dxapi") - 1)) &&
//                       (_strnicmp(ImportName, "coverage", sizeof("coverage") - 1)) &&
//                       (_strnicmp(ImportName, "irt", sizeof("irt") - 1)));
//
//         /* Check if this is a valid GDI driver */
//         if (GdiLink && NormalLink)
//         {
//             /* It's not, it's importing stuff it shouldn't be! */
//             Status = STATUS_PROCEDURE_NOT_FOUND;
//             goto Failure;
//         }
//
//         /* Check for user-mode printer or video card drivers, which don't belong */
//         if (!(_strnicmp(ImportName, "ntdll", sizeof("ntdll") - 1)) ||
//             !(_strnicmp(ImportName, "winsrv", sizeof("winsrv") - 1)) ||
//             !(_strnicmp(ImportName, "advapi32", sizeof("advapi32") - 1)) ||
//             !(_strnicmp(ImportName, "kernel32", sizeof("kernel32") - 1)) ||
//             !(_strnicmp(ImportName, "user32", sizeof("user32") - 1)) ||
//             !(_strnicmp(ImportName, "gdi32", sizeof("gdi32") - 1)))
//         {
//             /* This is not kernel code */
//             Status = STATUS_PROCEDURE_NOT_FOUND;
//             goto Failure;
//         }
//
//         /* Check if this is a "core" import, which doesn't get referenced */
//         if (!(_strnicmp(ImportName, "ntoskrnl", sizeof("ntoskrnl") - 1)) ||
//             !(_strnicmp(ImportName, "win32k", sizeof("win32k") - 1)) ||
//             !(_strnicmp(ImportName, "hal", sizeof("hal") - 1)))
//         {
//             /* Don't reference this */
//             ReferenceNeeded = FALSE;
//         }
//         else
//         {
//             /* Reference these modules */
//             ReferenceNeeded = TRUE;
//         }
//
//         /* Now setup a unicode string for the import */
//         RtlInitAnsiString(&TempString, ImportName);
//         Status = RtlAnsiStringToUnicodeString(&NameString, &TempString, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Failed */
//             goto Failure;
//         }
//
//         /* We don't support name prefixes yet */
//         if (NamePrefix) DPRINT1("Name Prefix not yet supported!\n");
//
//         /* Remember that we haven't loaded the import at this point */
// CheckDllState:
//         Loaded = FALSE;
//         ImportBase = NULL;
//
//         /* Loop the driver list */
//         NextEntry = PsLoadedModuleList.Flink;
//         while (NextEntry != &PsLoadedModuleList)
//         {
//             /* Get the loader entry and compare the name */
//             LdrEntry = CONTAINING_RECORD(NextEntry,
//                                          LDR_DATA_TABLE_ENTRY,
//                                          InLoadOrderLinks);
//             if (RtlEqualUnicodeString(&NameString,
//                                       &LdrEntry->BaseDllName,
//                                       TRUE))
//             {
//                 /* Get the base address */
//                 ImportBase = LdrEntry->DllBase;
//
//                 /* Check if we haven't loaded yet, and we need references */
//                 if (!(Loaded) && (ReferenceNeeded))
//                 {
//                     /* Make sure we're not already loading */
//                     if (!(LdrEntry->Flags & LDRP_LOAD_IN_PROGRESS))
//                     {
//                         /* Increase the load count */
//                         LdrEntry->LoadCount++;
//                     }
//                 }
//
//                 /* Done, break out */
//                 break;
//             }
//
//             /* Go to the next entry */
//             NextEntry = NextEntry->Flink;
//         }
//
//         /* Check if we haven't loaded the import yet */
//         if (!ImportBase)
//         {
//             /* Setup the import DLL name */
//             DllName.MaximumLength = NameString.Length +
//                                     ImageFileDirectory->Length +
//                                     sizeof(UNICODE_NULL);
//             DllName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                    DllName.MaximumLength,
//                                                    TAG_LDR_WSTR);
//             if (!DllName.Buffer)
//             {
//                 /* We're out of resources */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Failure;
//             }
//
//             /* Add the import name to the base directory */
//             RtlCopyUnicodeString(&DllName, ImageFileDirectory);
//             RtlAppendUnicodeStringToString(&DllName,
//                                            &NameString);
//
//             /* Load the image */
//             Status = MmLoadSystemImage(&DllName,
//                                        NamePrefix,
//                                        NULL,
//                                        FALSE,
//                                        (PVOID *)&DllEntry,
//                                        &DllBase);
//
//             /* win32k / GDI drivers can also import from system32 folder */
//             if ((Status == STATUS_OBJECT_NAME_NOT_FOUND) &&
//                 (MI_IS_SESSION_ADDRESS(ImageBase) || 1)) // HACK
//             {
//                 /* Free the old name buffer */
//                 ExFreePoolWithTag(DllName.Buffer, TAG_LDR_WSTR);
//
//                 /* Calculate size for a string the adds 'drivers\' */
//                 DllName.MaximumLength += DriversFolderName.Length;
//
//                 /* Allocate the new buffer */
//                 DllName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                        DllName.MaximumLength,
//                                                        TAG_LDR_WSTR);
//                 if (!DllName.Buffer)
//                 {
//                     /* We're out of resources */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto Failure;
//                 }
//
//                 /* Copy image directory and append 'drivers\' folder name */
//                 RtlCopyUnicodeString(&DllName, ImageFileDirectory);
//                 RtlAppendUnicodeStringToString(&DllName, &DriversFolderName);
//
//                 /* Now add the import name */
//                 RtlAppendUnicodeStringToString(&DllName, &NameString);
//
//                 /* Try once again to load the image */
//                 Status = MmLoadSystemImage(&DllName,
//                                            NamePrefix,
//                                            NULL,
//                                            FALSE,
//                                            (PVOID *)&DllEntry,
//                                            &DllBase);
//             }
//
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Fill out the information for the error */
//                 *MissingDriver = DllName.Buffer;
//                 *(PULONG)MissingDriver |= 1;
//                 *MissingApi = NULL;
//
//                 DPRINT1("Failed to load dependency: %wZ\n", &DllName);
//
//                 /* Don't free the name */
//                 DllName.Buffer = NULL;
//
//                 /* Cleanup and return */
//                 goto Failure;
//             }
//
//             /* We can free the DLL Name */
//             ExFreePoolWithTag(DllName.Buffer, TAG_LDR_WSTR);
//             DllName.Buffer = NULL;
//
//             /* We're now loaded */
//             Loaded = TRUE;
//
//             /* Sanity check */
//             ASSERT(DllBase == DllEntry->DllBase);
//
//             /* Call the initialization routines */
//             Status = MmCallDllInitialize(DllEntry, &PsLoadedModuleList);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* We failed, unload the image */
//                 MmUnloadSystemImage(DllEntry);
//                 ERROR_DBGBREAK("MmCallDllInitialize failed with status 0x%x\n", Status);
//                 Loaded = FALSE;
//             }
//
//             /* Loop again to make sure that everything is OK */
//             goto CheckDllState;
//         }
//
//         /* Check if we're support to reference this import */
//         if ((ReferenceNeeded) && (LoadedImports))
//         {
//             /* Make sure we're not already loading */
//             if (!(LdrEntry->Flags & LDRP_LOAD_IN_PROGRESS))
//             {
//                 /* Add the entry */
//                 LoadedImports->Entry[ImportCount] = LdrEntry;
//                 ImportCount++;
//             }
//         }
//
//         /* Free the import name */
//         RtlFreeUnicodeString(&NameString);
//
//         /* Set the missing driver name and get the export directory */
//         *MissingDriver = LdrEntry->BaseDllName.Buffer;
//         ExportDirectory = RtlImageDirectoryEntryToData(ImportBase,
//                                                        TRUE,
//                                                        IMAGE_DIRECTORY_ENTRY_EXPORT,
//                                                        &ExportSize);
//         if (!ExportDirectory)
//         {
//             /* Cleanup and return */
//             DPRINT1("Warning: Driver failed to load, %S not found\n", *MissingDriver);
//             Status = STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
//             goto Failure;
//         }
//
//         /* Make sure we have an IAT */
//         if (ImportDescriptor->OriginalFirstThunk)
//         {
//             /* Get the first thunks */
//             OrigThunk = (PVOID)((ULONG_PTR)ImageBase +
//                                 ImportDescriptor->OriginalFirstThunk);
//             FirstThunk = (PVOID)((ULONG_PTR)ImageBase +
//                                  ImportDescriptor->FirstThunk);
//
//             /* Loop the IAT */
//             while (OrigThunk->u1.AddressOfData)
//             {
//                 /* Snap thunk */
//                 Status = MiSnapThunk(ImportBase,
//                                      ImageBase,
//                                      OrigThunk++,
//                                      FirstThunk++,
//                                      ExportDirectory,
//                                      ExportSize,
//                                      FALSE,
//                                      MissingApi);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Cleanup and return */
//                     goto Failure;
//                 }
//
//                 /* Reset the buffer */
//                 *MissingApi = MissingApiBuffer;
//             }
//         }
//
//         /* Go to the next import */
//         ImportDescriptor++;
//     }
//
//     /* Check if we have an import list */
//     if (LoadedImports)
//     {
//         /* Reset the count again, and loop entries */
//         ImportCount = 0;
//         for (i = 0; i < LoadedImports->Count; i++)
//         {
//             if (LoadedImports->Entry[i])
//             {
//                 /* Got an entry, OR it with 1 in case it's the single entry */
//                 ImportEntry = (PVOID)((ULONG_PTR)LoadedImports->Entry[i] |
//                                       MM_SYSLDR_SINGLE_ENTRY);
//                 ImportCount++;
//             }
//         }
//
//         /* Check if we had no imports */
//         if (!ImportCount)
//         {
//             /* Free the list and set it to no imports */
//             ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
//             LoadedImports = MM_SYSLDR_NO_IMPORTS;
//         }
//         else if (ImportCount == 1)
//         {
//             /* Just one entry, we can free the table and only use our entry */
//             ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
//             LoadedImports = (PLOAD_IMPORTS)ImportEntry;
//         }
//         else if (ImportCount != LoadedImports->Count)
//         {
//             /* Allocate a new list */
//             LoadedImportsSize = ImportCount * sizeof(PVOID) + sizeof(SIZE_T);
//             NewImports = ExAllocatePoolWithTag(PagedPool,
//                                                LoadedImportsSize,
//                                                TAG_LDR_IMPORTS);
//             if (NewImports)
//             {
//                 /* Set count */
//                 NewImports->Count = 0;
//
//                 /* Loop all the imports */
//                 for (i = 0; i < LoadedImports->Count; i++)
//                 {
//                     /* Make sure it's valid */
//                     if (LoadedImports->Entry[i])
//                     {
//                         /* Copy it */
//                         NewImports->Entry[NewImports->Count] = LoadedImports->Entry[i];
//                         NewImports->Count++;
//                     }
//                 }
//
//                 /* Free the old copy */
//                 ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
//                 LoadedImports = NewImports;
//             }
//         }
//
//         /* Return the list */
//         *LoadImports = LoadedImports;
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
//
// Failure:
//
//     /* Cleanup and return */
//     RtlFreeUnicodeString(&NameString);
//
//     if (LoadedImports)
//     {
//         MiDereferenceImports(LoadedImports);
//         ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// MiFreeInitializationCode(IN PVOID InitStart,
//                          IN PVOID InitEnd)
// {
//     PMMPTE PointerPte;
//     PFN_NUMBER PagesFreed;
//
//     /* Get the start PTE */
//     PointerPte = MiAddressToPte(InitStart);
//     ASSERT(MI_IS_PHYSICAL_ADDRESS(InitStart) == FALSE);
//
//     /*  Compute the number of pages we expect to free */
//     PagesFreed = (PFN_NUMBER)(MiAddressToPte(InitEnd) - PointerPte);
//
//     /* Try to actually free them */
//     PagesFreed = MiDeleteSystemPageableVm(PointerPte,
//                                           PagesFreed,
//                                           0,
//                                           NULL);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiFindInitializationCode(OUT PVOID *StartVa,
//                          OUT PVOID *EndVa)
// {
//     ULONG Size, SectionCount, Alignment;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     ULONG_PTR DllBase, InitStart, InitEnd, ImageEnd, InitCode;
//     PLIST_ENTRY NextEntry;
//     PIMAGE_NT_HEADERS NtHeader;
//     PIMAGE_SECTION_HEADER Section, LastSection, InitSection;
//     BOOLEAN InitFound;
//     DBG_UNREFERENCED_LOCAL_VARIABLE(InitSection);
//
//     /* So we don't free our own code yet */
//     InitCode = (ULONG_PTR)&MiFindInitializationCode;
//
//     /* Assume failure */
//     *StartVa = NULL;
//
//     /* Acquire the necessary locks while we loop the list */
//     KeEnterCriticalRegion();
//     KeWaitForSingleObject(&MmSystemLoadLock,
//                           WrVirtualMemory,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//     ExAcquireResourceExclusiveLite(&PsLoadedModuleResource, TRUE);
//
//     /* Loop all loaded modules */
//     NextEntry = PsLoadedModuleList.Flink;
//     while (NextEntry != &PsLoadedModuleList)
//     {
//         /* Get the loader entry and its DLL base */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//         DllBase = (ULONG_PTR)LdrEntry->DllBase;
//
//         /* Only process boot loaded images. Other drivers are processed by
//            MmFreeDriverInitialization */
//         if (LdrEntry->Flags & LDRP_MM_LOADED)
//         {
//             /* Keep going */
//             NextEntry = NextEntry->Flink;
//             continue;
//         }
//
//         /* Get the NT header */
//         NtHeader = RtlImageNtHeader((PVOID)DllBase);
//         if (!NtHeader)
//         {
//             /* Keep going */
//             NextEntry = NextEntry->Flink;
//             continue;
//         }
//
//         /* Get the first section, the section count, and scan them all */
//         Section = IMAGE_FIRST_SECTION(NtHeader);
//         SectionCount = NtHeader->FileHeader.NumberOfSections;
//         InitStart = 0;
//         while (SectionCount > 0)
//         {
//             /* Assume failure */
//             InitFound = FALSE;
//
//             /* Is this the INIT section or a discardable section? */
//             if ((strncmp((PCCH)Section->Name, "INIT", 5) == 0) ||
//                 ((Section->Characteristics & IMAGE_SCN_MEM_DISCARDABLE)))
//             {
//                 /* Remember this */
//                 InitFound = TRUE;
//                 InitSection = Section;
//             }
//
//             if (InitFound)
//             {
//                 /* Pick the biggest size -- either raw or virtual */
//                 Size = max(Section->SizeOfRawData, Section->Misc.VirtualSize);
//
//                 /* Read the section alignment */
//                 Alignment = NtHeader->OptionalHeader.SectionAlignment;
//
//                 /* Get the start and end addresses */
//                 InitStart = DllBase + Section->VirtualAddress;
//                 InitEnd = ALIGN_UP_BY(InitStart + Size, Alignment);
//
//                 /* Align the addresses to PAGE_SIZE */
//                 InitStart = ALIGN_UP_BY(InitStart, PAGE_SIZE);
//                 InitEnd = ALIGN_DOWN_BY(InitEnd, PAGE_SIZE);
//
//                 /* Have we reached the last section? */
//                 if (SectionCount == 1)
//                 {
//                     /* Remember this */
//                     LastSection = Section;
//                 }
//                 else
//                 {
//                     /* We have not, loop all the sections */
//                     LastSection = NULL;
//                     do
//                     {
//                         /* Keep going until we find a non-discardable section range */
//                         SectionCount--;
//                         Section++;
//                         if (Section->Characteristics & IMAGE_SCN_MEM_DISCARDABLE)
//                         {
//                             /* Discardable, so record it, then keep going */
//                             LastSection = Section;
//                         }
//                         else
//                         {
//                             /* Non-contigous discard flag, or no flag, break out */
//                             break;
//                         }
//                     }
//                     while (SectionCount > 1);
//                 }
//
//                 /* Have we found a discardable or init section? */
//                 if (LastSection)
//                 {
//                     /* Pick the biggest size -- either raw or virtual */
//                     Size = max(LastSection->SizeOfRawData, LastSection->Misc.VirtualSize);
//
//                     /* Use this as the end of the section address */
//                     InitEnd = DllBase + LastSection->VirtualAddress + Size;
//
//                     /* Have we reached the last section yet? */
//                     if (SectionCount != 1)
//                     {
//                         /* Then align this accross the session boundary */
//                         InitEnd = ALIGN_UP_BY(InitEnd, Alignment);
//                         InitEnd = ALIGN_DOWN_BY(InitEnd, PAGE_SIZE);
//                     }
//                 }
//
//                 /* Make sure we don't let the init section go past the image */
//                 ImageEnd = DllBase + LdrEntry->SizeOfImage;
//                 if (InitEnd > ImageEnd) InitEnd = ALIGN_UP_BY(ImageEnd, PAGE_SIZE);
//
//                 /* Make sure we have a valid, non-zero init section */
//                 if (InitStart < InitEnd)
//                 {
//                     /* Make sure we are not within this code itself */
//                     if ((InitCode >= InitStart) && (InitCode < InitEnd))
//                     {
//                         /* Return it, we can't free ourselves now */
//                         ASSERT(*StartVa == 0);
//                         *StartVa = (PVOID)InitStart;
//                         *EndVa = (PVOID)InitEnd;
//                     }
//                     else
//                     {
//                         /* This isn't us -- go ahead and free it */
//                         ASSERT(MI_IS_PHYSICAL_ADDRESS((PVOID)InitStart) == FALSE);
//                         DPRINT("Freeing init code: %p-%p ('%wZ' @%p : '%s')\n",
//                                (PVOID)InitStart,
//                                (PVOID)InitEnd,
//                                &LdrEntry->BaseDllName,
//                                LdrEntry->DllBase,
//                                InitSection->Name);
//                         MiFreeInitializationCode((PVOID)InitStart, (PVOID)InitEnd);
//                     }
//                 }
//             }
//
//             /* Move to the next section */
//             SectionCount--;
//             Section++;
//         }
//
//         /* Move to the next module */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the locks and return */
//     ExReleaseResourceLite(&PsLoadedModuleResource);
//     KeReleaseMutant(&MmSystemLoadLock, MUTANT_INCREMENT, FALSE, FALSE);
//     KeLeaveCriticalRegion();
// }
//
// /*
//  * Note: This function assumes that all discardable sections are at the end of
//  * the PE file. It searches backwards until it finds the non-discardable section
//  */
// VOID
// NTAPI
// MmFreeDriverInitialization(IN PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PMMPTE StartPte, EndPte;
//     PFN_NUMBER PageCount;
//     PVOID DllBase;
//     ULONG i;
//     PIMAGE_NT_HEADERS NtHeader;
//     PIMAGE_SECTION_HEADER Section, DiscardSection;
//
//     /* Get the base address and the page count */
//     DllBase = LdrEntry->DllBase;
//     PageCount = LdrEntry->SizeOfImage >> PAGE_SHIFT;
//
//     /* Get the last PTE in this image */
//     EndPte = MiAddressToPte(DllBase) + PageCount;
//
//     /* Get the NT header */
//     NtHeader = RtlImageNtHeader(DllBase);
//     if (!NtHeader) return;
//
//     /* Get the last section and loop each section backwards */
//     Section = IMAGE_FIRST_SECTION(NtHeader) + NtHeader->FileHeader.NumberOfSections;
//     DiscardSection = NULL;
//     for (i = 0; i < NtHeader->FileHeader.NumberOfSections; i++)
//     {
//         /* Go back a section and check if it's discardable */
//         Section--;
//         if (Section->Characteristics & IMAGE_SCN_MEM_DISCARDABLE)
//         {
//             /* It is, select it for freeing */
//             DiscardSection = Section;
//         }
//         else
//         {
//             /* No more discardable sections exist, bail out */
//             break;
//         }
//     }
//
//     /* Bail out if there's nothing to free */
//     if (!DiscardSection) return;
//
//     /* Push the DLL base to the first disacrable section, and get its PTE */
//     DllBase = (PVOID)ROUND_TO_PAGES((ULONG_PTR)DllBase + DiscardSection->VirtualAddress);
//     ASSERT(MI_IS_PHYSICAL_ADDRESS(DllBase) == FALSE);
//     StartPte = MiAddressToPte(DllBase);
//
//     /* Check how many pages to free total */
//     PageCount = (PFN_NUMBER)(EndPte - StartPte);
//     if (!PageCount) return;
//
//     /* Delete this many PTEs */
//     MiDeleteSystemPageableVm(StartPte, PageCount, 0, NULL);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiReloadBootLoadedDrivers(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextEntry;
//     ULONG i = 0;
//     PIMAGE_NT_HEADERS NtHeader;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PIMAGE_FILE_HEADER FileHeader;
//     BOOLEAN ValidRelocs;
//     PIMAGE_DATA_DIRECTORY DataDirectory;
//     PVOID DllBase, NewImageAddress;
//     NTSTATUS Status;
//     PMMPTE PointerPte, StartPte, LastPte;
//     PFN_COUNT PteCount;
//     PMMPFN Pfn1;
//     MMPTE TempPte, OldPte;
//
//     /* Loop driver list */
//     for (NextEntry = LoaderBlock->LoadOrderListHead.Flink;
//          NextEntry != &LoaderBlock->LoadOrderListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the loader entry and NT header */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//         NtHeader = RtlImageNtHeader(LdrEntry->DllBase);
//
//         /* Debug info */
//         DPRINT("[Mm0]: Driver at: %p ending at: %p for module: %wZ\n",
//                 LdrEntry->DllBase,
//                 (ULONG_PTR)LdrEntry->DllBase + LdrEntry->SizeOfImage,
//                 &LdrEntry->FullDllName);
//
//         /* Get the first PTE and the number of PTEs we'll need */
//         PointerPte = StartPte = MiAddressToPte(LdrEntry->DllBase);
//         PteCount = ROUND_TO_PAGES(LdrEntry->SizeOfImage) >> PAGE_SHIFT;
//         LastPte = StartPte + PteCount;
//
// #if MI_TRACE_PFNS
//         /* Loop the PTEs */
//         while (PointerPte < LastPte)
//         {
//             ULONG len;
//             ASSERT(PointerPte->u.Hard.Valid == 1);
//             Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
//             len = wcslen(LdrEntry->BaseDllName.Buffer) * sizeof(WCHAR);
//             snprintf(Pfn1->ProcessName, min(16, len), "%S", LdrEntry->BaseDllName.Buffer);
//             PointerPte++;
//         }
// #endif
//         /* Skip kernel and HAL */
//         /* ROS HACK: Skip BOOTVID/KDCOM too */
//         i++;
//         if (i <= 4) continue;
//
//         /* Skip non-drivers */
//         if (!NtHeader) continue;
//
//         /* Get the file header and make sure we can relocate */
//         FileHeader = &NtHeader->FileHeader;
//         if (FileHeader->Characteristics & IMAGE_FILE_RELOCS_STRIPPED) continue;
//         if (NtHeader->OptionalHeader.NumberOfRvaAndSizes <
//             IMAGE_DIRECTORY_ENTRY_BASERELOC) continue;
//
//         /* Everything made sense until now, check the relocation section too */
//         DataDirectory = &NtHeader->OptionalHeader.
//                         DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC];
//         if (!DataDirectory->VirtualAddress)
//         {
//             /* We don't really have relocations */
//             ValidRelocs = FALSE;
//         }
//         else
//         {
//             /* Make sure the size is valid */
//             if ((DataDirectory->VirtualAddress + DataDirectory->Size) >
//                 LdrEntry->SizeOfImage)
//             {
//                 /* They're not, skip */
//                 continue;
//             }
//
//             /* We have relocations */
//             ValidRelocs = TRUE;
//         }
//
//         /* Remember the original address */
//         DllBase = LdrEntry->DllBase;
//
//         /* Loop the PTEs */
//         PointerPte = StartPte;
//         while (PointerPte < LastPte)
//         {
//             /* Mark the page modified in the PFN database */
//             ASSERT(PointerPte->u.Hard.Valid == 1);
//             Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
//             ASSERT(Pfn1->u3.e1.Rom == 0);
//             Pfn1->u3.e1.Modified = TRUE;
//
//             /* Next */
//             PointerPte++;
//         }
//
//         /* Now reserve system PTEs for the image */
//         PointerPte = MiReserveSystemPtes(PteCount, SystemPteSpace);
//         if (!PointerPte)
//         {
//             /* Shouldn't happen */
//             ERROR_FATAL("[Mm0]: Couldn't allocate driver section!\n");
//             return;
//         }
//
//         /* This is the new virtual address for the module */
//         LastPte = PointerPte + PteCount;
//         NewImageAddress = MiPteToAddress(PointerPte);
//
//         /* Sanity check */
//         DPRINT("[Mm0]: Copying from: %p to: %p\n", DllBase, NewImageAddress);
//         ASSERT(ExpInitializationPhase == 0);
//
//         /* Loop the new driver PTEs */
//         TempPte = ValidKernelPte;
//         while (PointerPte < LastPte)
//         {
//             /* Copy the old data */
//             OldPte = *StartPte;
//             ASSERT((OldPte.valid() as usize) == 1);
//
//             /* Set page number from the loader's memory */
//             TempPte.set_page_frame_number((OldPte.page_frame_number()) as PFN_NUMBER);
//
//             /* Write it */
//             MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//             /* Move on */
//             PointerPte++;
//             StartPte++;
//         }
//
//         /* Update position */
//         PointerPte -= PteCount;
//
//         /* Sanity check */
//         ASSERT(*(PULONG)NewImageAddress == *(PULONG)DllBase);
//
//         /* Set the image base to the address where the loader put it */
//         NtHeader->OptionalHeader.ImageBase = (ULONG_PTR)DllBase;
//
//         /* Check if we had relocations */
//         if (ValidRelocs)
//         {
//             /* Relocate the image */
//             Status = LdrRelocateImageWithBias(NewImageAddress,
//                                               0,
//                                               "SYSLDR",
//                                               STATUS_SUCCESS,
//                                               STATUS_CONFLICTING_ADDRESSES,
//                                               STATUS_INVALID_IMAGE_FORMAT);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* This shouldn't happen */
//                 ERROR_FATAL("Relocations failed!\n");
//                 return;
//             }
//         }
//
//         /* Update the loader entry */
//         LdrEntry->DllBase = NewImageAddress;
//
//         /* Update the thunks */
//         DPRINT("[Mm0]: Updating thunks to: %wZ\n", &LdrEntry->BaseDllName);
//         MiUpdateThunks(LoaderBlock,
//                        DllBase,
//                        NewImageAddress,
//                        LdrEntry->SizeOfImage);
//
//         /* Update the loader entry */
//         LdrEntry->Flags |= LDRP_SYSTEM_MAPPED;
//         LdrEntry->EntryPoint = (PVOID)((ULONG_PTR)NewImageAddress +
//                                 NtHeader->OptionalHeader.AddressOfEntryPoint);
//         LdrEntry->SizeOfImage = PteCount << PAGE_SHIFT;
//
//         /* FIXME: We'll need to fixup the PFN linkage when switching to ARM3 */
//     }
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// MiBuildImportsForBootDrivers(VOID)
// {
//     PLIST_ENTRY NextEntry, NextEntry2;
//     PLDR_DATA_TABLE_ENTRY LdrEntry, KernelEntry, HalEntry, LdrEntry2, LastEntry;
//     PLDR_DATA_TABLE_ENTRY* EntryArray;
//     UNICODE_STRING KernelName = RTL_CONSTANT_STRING(L"ntoskrnl.exe");
//     UNICODE_STRING HalName = RTL_CONSTANT_STRING(L"hal.dll");
//     PLOAD_IMPORTS LoadedImports;
//     ULONG LoadedImportsSize, ImportSize;
//     PULONG_PTR ImageThunk;
//     ULONG_PTR DllBase, DllEnd;
//     ULONG Modules = 0, i, j = 0;
//     PIMAGE_IMPORT_DESCRIPTOR ImportDescriptor;
//
// #ifdef _M_ARM64
//     extern volatile PVOID Arm64UartKernelBase;
// #define _DBG_TRACE(s) do { \
//         volatile ULONG *__u = (volatile ULONG *)Arm64UartKernelBase; \
//         const char *__p = (s); \
//         if (__u) { while (*__p) *__u = (ULONG)*__p++; } \
//     } while(0)
// #define _DBG_HEX(prefix, val) do { \
//         volatile ULONG *__u = (volatile ULONG *)Arm64UartKernelBase; \
//         const char *__p = (prefix); int __i; ULONG_PTR __v = (ULONG_PTR)(val); \
//         if (__u) { \
//             while (*__p) *__u = (ULONG)*__p++; \
//             *__u = (ULONG)'0'; *__u = (ULONG)'x'; \
//             for (__i = 60; __i >= 0; __i -= 4) { \
//                 int __d = (__v >> __i) & 0xF; \
//                 *__u = (ULONG)(__d < 10 ? '0' + __d : 'A' + __d - 10); \
//             } \
//             *__u = (ULONG)'\r'; *__u = (ULONG)'\n'; \
//         } \
//     } while(0)
// #else
// #define _DBG_TRACE(s) ((void)0)
// #define _DBG_HEX(prefix, val) ((void)0)
// #endif
//
//     _DBG_TRACE("[IMPORT] MiBuildImportsForBootDrivers enter\r\n");
//     /* Initialize variables */
//     KernelEntry = HalEntry = LastEntry = NULL;
//
//     /* Loop the loaded module list... we are early enough that no lock is needed */
//     NextEntry = PsLoadedModuleList.Flink;
//     while (NextEntry != &PsLoadedModuleList)
//     {
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* Check if it's the kernel or HAL */
//         if (RtlEqualUnicodeString(&KernelName, &LdrEntry->BaseDllName, TRUE))
//         {
//             /* Found it */
//             KernelEntry = LdrEntry;
//         }
//         else if (RtlEqualUnicodeString(&HalName, &LdrEntry->BaseDllName, TRUE))
//         {
//             /* Found it */
//             HalEntry = LdrEntry;
//         }
//
//         /* Check if this is a driver DLL */
//         if (LdrEntry->Flags & LDRP_DRIVER_DEPENDENT_DLL)
//         {
//             /* Check if this is the HAL or kernel */
//             if ((LdrEntry == HalEntry) || (LdrEntry == KernelEntry))
//             {
//                 /* Add a reference */
//                 LdrEntry->LoadCount = 1;
//             }
//             else
//             {
//                 /* No referencing needed */
//                 LdrEntry->LoadCount = 0;
//             }
//         }
//         else
//         {
//             /* Add a reference for all other modules as well */
//             LdrEntry->LoadCount = 1;
//         }
//
//         /* Remember this came from the loader */
//         LdrEntry->LoadedImports = MM_SYSLDR_BOOT_LOADED;
//
//         /* Keep looping */
//         NextEntry = NextEntry->Flink;
//         Modules++;
//     }
//
//     _DBG_HEX("[IMPORT] modules=", Modules);
//     /* We must have at least found the kernel and HAL */
//     if (!(HalEntry) || (!KernelEntry)) return STATUS_NOT_FOUND;
//
//     /* Allocate the list */
//     EntryArray = ExAllocatePoolWithTag(PagedPool, Modules * sizeof(PVOID), TAG_LDR_IMPORTS);
//     if (!EntryArray) return STATUS_INSUFFICIENT_RESOURCES;
//     _DBG_HEX("[IMPORT] EntryArray=", (ULONG_PTR)EntryArray);
//
//     /* Loop the loaded module list again */
//     NextEntry = PsLoadedModuleList.Flink;
//     while (NextEntry != &PsLoadedModuleList)
//     {
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
// #ifdef _WORKING_LOADER_
//         /* Get its imports */
//         ImageThunk = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                   TRUE,
//                                                   IMAGE_DIRECTORY_ENTRY_IAT,
//                                                   &ImportSize);
//         if (!ImageThunk)
// #else
//         /* Get its imports */
//         _DBG_HEX("[IMPORT] DllBase=", (ULONG_PTR)LdrEntry->DllBase);
//         ImportDescriptor = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                         TRUE,
//                                                         IMAGE_DIRECTORY_ENTRY_IMPORT,
//                                                         &ImportSize);
//         if (!ImportDescriptor)
// #endif
//         {
//             /* None present */
//             LdrEntry->LoadedImports = MM_SYSLDR_NO_IMPORTS;
//             _DBG_TRACE("[IMPORT] no imports\r\n");
//             NextEntry = NextEntry->Flink;
//             continue;
//         }
//
//         /* Clear the list and count the number of IAT thunks */
//         RtlZeroMemory(EntryArray, Modules * sizeof(PVOID));
// #ifdef _WORKING_LOADER_
//         ImportSize /= sizeof(ULONG_PTR);
//
//         /* Scan the thunks */
//         for (i = 0, DllBase = 0, DllEnd = 0; i < ImportSize; i++, ImageThunk++)
// #else
//         DllBase = DllEnd = i = 0;
//         _DBG_TRACE("[IMPORT] processing imports\r\n");
//         while ((ImportDescriptor->Name) &&
//                (ImportDescriptor->OriginalFirstThunk))
//         {
//             /* Get the image thunk */
//             ImageThunk = (PVOID)((ULONG_PTR)LdrEntry->DllBase +
//                                  ImportDescriptor->FirstThunk);
//             while (*ImageThunk)
// #endif
//             {
//             /* Do we already have an address? */
//             if (DllBase)
//             {
//                 /* Is the thunk in the same address? */
//                 if ((*ImageThunk >= DllBase) && (*ImageThunk < DllEnd))
//                 {
//                     /* Skip it, we already have a reference for it */
//                     ASSERT(EntryArray[j]);
//                     ImageThunk++;
//                     continue;
//                 }
//             }
//
//             /* Loop the loaded module list to locate this address owner */
//             j = 0;
//             NextEntry2 = PsLoadedModuleList.Flink;
//             while (NextEntry2 != &PsLoadedModuleList)
//             {
//                 /* Get the entry */
//                 LdrEntry2 = CONTAINING_RECORD(NextEntry2,
//                                               LDR_DATA_TABLE_ENTRY,
//                                               InLoadOrderLinks);
//
//                 /* Get the address range for this module */
//                 DllBase = (ULONG_PTR)LdrEntry2->DllBase;
//                 DllEnd = DllBase + LdrEntry2->SizeOfImage;
//
//                 /* Check if this IAT entry matches it */
//                 if ((*ImageThunk >= DllBase) && (*ImageThunk < DllEnd))
//                 {
//                     /* Save it */
//                     //DPRINT1("Found imported dll: %wZ\n", &LdrEntry2->BaseDllName);
//                     EntryArray[j] = LdrEntry2;
//                     break;
//                 }
//
//                 /* Keep searching */
//                 NextEntry2 = NextEntry2->Flink;
//                 j++;
//             }
//
//             /* Do we have a thunk outside the range? */
//             if ((*ImageThunk < DllBase) || (*ImageThunk >= DllEnd))
//             {
//                 /* Could be 0... */
//                 if (*ImageThunk)
//                 {
//                     /* Should not be happening */
//                     ERROR_FATAL("Broken IAT entry for %p at %p (%lx)\n",
//                                 LdrEntry, ImageThunk, *ImageThunk);
//                 }
//
//                 /* Reset if we hit this */
//                 DllBase = 0;
//             }
// #ifndef _WORKING_LOADER_
//             ImageThunk++;
//             }
//
//             i++;
//             ImportDescriptor++;
// #endif
//         }
//
//         /* Now scan how many imports we really have */
//         for (i = 0, ImportSize = 0; i < Modules; i++)
//         {
//             /* Skip HAL and kernel */
//             if ((EntryArray[i]) &&
//                 (EntryArray[i] != HalEntry) &&
//                 (EntryArray[i] != KernelEntry))
//             {
//                 /* A valid reference */
//                 LastEntry = EntryArray[i];
//                 ImportSize++;
//             }
//         }
//
//         /* Do we have any imports after all? */
//         if (!ImportSize)
//         {
//             /* No */
//             LdrEntry->LoadedImports = MM_SYSLDR_NO_IMPORTS;
//         }
//         else if (ImportSize == 1)
//         {
//             /* A single entry import */
//             LdrEntry->LoadedImports = (PVOID)((ULONG_PTR)LastEntry | MM_SYSLDR_SINGLE_ENTRY);
//             LastEntry->LoadCount++;
//         }
//         else
//         {
//             /* We need an import table */
//             LoadedImportsSize = ImportSize * sizeof(PVOID) + sizeof(SIZE_T);
//             LoadedImports = ExAllocatePoolWithTag(PagedPool,
//                                                   LoadedImportsSize,
//                                                   TAG_LDR_IMPORTS);
//             ASSERT(LoadedImports);
//
//             /* Save the count */
//             LoadedImports->Count = ImportSize;
//
//             /* Now copy all imports */
//             for (i = 0, j = 0; i < Modules; i++)
//             {
//                 /* Skip HAL and kernel */
//                 if ((EntryArray[i]) &&
//                     (EntryArray[i] != HalEntry) &&
//                     (EntryArray[i] != KernelEntry))
//                 {
//                     /* A valid reference */
//                     //DPRINT1("Found valid entry: %p\n", EntryArray[i]);
//                     LoadedImports->Entry[j] = EntryArray[i];
//                     EntryArray[i]->LoadCount++;
//                     j++;
//                 }
//             }
//
//             /* Should had as many entries as we expected */
//             ASSERT(j == ImportSize);
//             LdrEntry->LoadedImports = LoadedImports;
//         }
//
//         /* Next */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Free the initial array */
//     _DBG_TRACE("[IMPORT] loop done, freeing array\r\n");
//     ExFreePoolWithTag(EntryArray, TAG_LDR_IMPORTS);
//     _DBG_TRACE("[IMPORT] ExFreePoolWithTag done\r\n");
//
//     /* FIXME: Might not need to keep the HAL/Kernel imports around */
//
//     /* Kernel and HAL are loaded at boot */
//     KernelEntry->LoadedImports = MM_SYSLDR_BOOT_LOADED;
//     HalEntry->LoadedImports = MM_SYSLDR_BOOT_LOADED;
//     _DBG_TRACE("[IMPORT] MiBuildImportsForBootDrivers returning\r\n");
//
//     /* All worked well */
//     return STATUS_SUCCESS;
// #undef _DBG_TRACE
// #undef _DBG_HEX
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiLocateKernelSections(IN PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     ULONG_PTR DllBase;
//     PIMAGE_NT_HEADERS NtHeaders;
//     PIMAGE_SECTION_HEADER SectionHeader;
//     ULONG Sections, Size;
//
//     /* Get the kernel section header */
//     DllBase = (ULONG_PTR)LdrEntry->DllBase;
//     NtHeaders = RtlImageNtHeader((PVOID)DllBase);
//     SectionHeader = IMAGE_FIRST_SECTION(NtHeaders);
//
//     /* Loop all the sections */
//     for (Sections = NtHeaders->FileHeader.NumberOfSections;
//          Sections > 0; --Sections, ++SectionHeader)
//     {
//         /* Grab the size of the section */
//         Size = max(SectionHeader->SizeOfRawData, SectionHeader->Misc.VirtualSize);
//
//         /* Check for .RSRC section */
//         if (*(PULONG)SectionHeader->Name == 'rsr.')
//         {
//             /* Remember the PTEs so we can modify them later */
//             MiKernelResourceStartPte = MiAddressToPte(DllBase +
//                                                       SectionHeader->VirtualAddress);
//             MiKernelResourceEndPte = MiAddressToPte(ROUND_TO_PAGES(DllBase +
//                                                     SectionHeader->VirtualAddress + Size));
//         }
//         else if (*(PULONG)SectionHeader->Name == 'LOOP')
//         {
//             /* POOLCODE vs. POOLMI */
//             if (*(PULONG)&SectionHeader->Name[4] == 'EDOC')
//             {
//                 /* Found Ex* Pool code */
//                 ExPoolCodeStart = DllBase + SectionHeader->VirtualAddress;
//                 ExPoolCodeEnd = ExPoolCodeStart + Size;
//             }
//             else if (*(PUSHORT)&SectionHeader->Name[4] == 'MI')
//             {
//                 /* Found Mm* Pool code */
//                 MmPoolCodeStart = DllBase + SectionHeader->VirtualAddress;
//                 MmPoolCodeEnd = MmPoolCodeStart + Size;
//             }
//         }
//         else if ((*(PULONG)SectionHeader->Name == 'YSIM') &&
//                  (*(PULONG)&SectionHeader->Name[4] == 'ETPS'))
//         {
//             /* Found MISYSPTE (Mm System PTE code) */
//             MmPteCodeStart = DllBase + SectionHeader->VirtualAddress;
//             MmPteCodeEnd = MmPteCodeStart + Size;
//         }
//     }
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// MiInitializeLoadedModuleList(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry, NewEntry;
//     PLIST_ENTRY ListHead, NextEntry;
//     ULONG EntrySize;
//
//     /* Setup the loaded module list and locks */
//     ExInitializeResourceLite(&PsLoadedModuleResource);
//     KeInitializeSpinLock(&PsLoadedModuleSpinLock);
//     InitializeListHead(&PsLoadedModuleList);
//
//     /* Get loop variables and the kernel entry */
//     ListHead = &LoaderBlock->LoadOrderListHead;
//     NextEntry = ListHead->Flink;
//     LdrEntry = CONTAINING_RECORD(NextEntry,
//                                  LDR_DATA_TABLE_ENTRY,
//                                  InLoadOrderLinks);
//     PsNtosImageBase = (ULONG_PTR)LdrEntry->DllBase;
//
//     /* Locate resource section, pool code, and system pte code */
//     MiLocateKernelSections(LdrEntry);
//
//     /* Loop the loader block */
//     while (NextEntry != ListHead)
//     {
//         /* Get the loader entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* FIXME: ROS HACK. Make sure this is a driver */
//         if (!RtlImageNtHeader(LdrEntry->DllBase))
//         {
//             /* Skip this entry */
//             NextEntry = NextEntry->Flink;
//             continue;
//         }
//
//         /* Calculate the size we'll need and allocate a copy */
//         EntrySize = sizeof(LDR_DATA_TABLE_ENTRY) +
//                     LdrEntry->BaseDllName.MaximumLength +
//                     sizeof(UNICODE_NULL);
//         NewEntry = ExAllocatePoolWithTag(NonPagedPool, EntrySize, TAG_MODULE_OBJECT);
//         if (!NewEntry) return FALSE;
//
//         /* Copy the entry over */
//         *NewEntry = *LdrEntry;
//
//         /* Allocate the name */
//         NewEntry->FullDllName.Buffer =
//             ExAllocatePoolWithTag(PagedPool,
//                                   LdrEntry->FullDllName.MaximumLength +
//                                       sizeof(UNICODE_NULL),
//                                   TAG_LDR_WSTR);
//         if (!NewEntry->FullDllName.Buffer)
//         {
//             ExFreePoolWithTag(NewEntry, TAG_MODULE_OBJECT);
//             return FALSE;
//         }
//
//         /* Set the base name */
//         NewEntry->BaseDllName.Buffer = (PVOID)(NewEntry + 1);
//
//         /* Copy the full and base name */
//         RtlCopyMemory(NewEntry->FullDllName.Buffer,
//                       LdrEntry->FullDllName.Buffer,
//                       LdrEntry->FullDllName.MaximumLength);
//         RtlCopyMemory(NewEntry->BaseDllName.Buffer,
//                       LdrEntry->BaseDllName.Buffer,
//                       LdrEntry->BaseDllName.MaximumLength);
//
//         /* Null-terminate the base name */
//         NewEntry->BaseDllName.Buffer[NewEntry->BaseDllName.Length /
//                                      sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* Insert the entry into the list */
//         InsertTailList(&PsLoadedModuleList, &NewEntry->InLoadOrderLinks);
// #ifdef _M_ARM64
//         /* Trace module base addresses for crash symbolication */
//         ARM64_UART_TRACE_HEX("[MOD] base=", (ULONG_PTR)NewEntry->DllBase);
//         ARM64_UART_TRACE_HEX("[MOD] size=", NewEntry->SizeOfImage);
//         /* Print base name (WCHAR) as ASCII */
//         {
//             volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//             if (_u) {
//                 PWCHAR _n = NewEntry->BaseDllName.Buffer;
//                 USHORT _l = NewEntry->BaseDllName.Length / sizeof(WCHAR);
//                 const char *_p = "[MOD] name=";
//                 while (*_p) *_u = (ULONG)*_p++;
//                 for (USHORT _i = 0; _i < _l && _n[_i]; _i++)
//                     *_u = (ULONG)(char)_n[_i];
//                 *_u = '\r'; *_u = '\n';
//             }
//         }
// #endif
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Build the import lists for the boot drivers */
//     MiBuildImportsForBootDrivers();
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[IMPORT] MiBuildImports returned\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* We're done */
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// MmChangeKernelResourceSectionProtection(IN ULONG_PTR ProtectionMask)
// {
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//
//     /* Don't do anything if the resource section is already writable */
//     if (MiKernelResourceStartPte == NULL || MiKernelResourceEndPte == NULL)
//         return FALSE;
//
//     /* If the resource section is physical, we cannot change its protection */
//     if (MI_IS_PHYSICAL_ADDRESS(MiPteToAddress(MiKernelResourceStartPte)))
//         return FALSE;
//
//     /* Loop the PTEs */
//     for (PointerPte = MiKernelResourceStartPte; PointerPte < MiKernelResourceEndPte; ++PointerPte)
//     {
//         /* Read the PTE */
//         TempPte = *PointerPte;
//
//         /* Update the protection */
//         MI_MAKE_HARDWARE_PTE_KERNEL(&TempPte, PointerPte, ProtectionMask, TempPte.page_frame_number());
//         MI_UPDATE_VALID_PTE(PointerPte, TempPte);
//     }
//
//     /* Only flush the current processor's TLB */
//     KeFlushCurrentTb();
//     return TRUE;
// }
//
// VOID
// NTAPI
// MmMakeKernelResourceSectionWritable(VOID)
// {
//     /* Don't do anything if the resource section is already writable */
//     if (MiKernelResourceStartPte == NULL || MiKernelResourceEndPte == NULL)
//         return;
//
//     /* If the resource section is physical, we cannot change its protection */
//     if (MI_IS_PHYSICAL_ADDRESS(MiPteToAddress(MiKernelResourceStartPte)))
//         return;
//
//     if (MmChangeKernelResourceSectionProtection(MM_READWRITE))
//     {
//         /*
//          * Invalidate the cached resource section PTEs
//          * so as to not change its protection again later.
//          */
//         MiKernelResourceStartPte = NULL;
//         MiKernelResourceEndPte = NULL;
//     }
// }
//
// LOGICAL
// NTAPI
// MiUseLargeDriverPage(IN ULONG NumberOfPtes,
//                      IN OUT PVOID *ImageBaseAddress,
//                      IN PUNICODE_STRING BaseImageName,
//                      IN BOOLEAN BootDriver)
// {
//     PLIST_ENTRY NextEntry;
//     BOOLEAN DriverFound = FALSE;
//     PMI_LARGE_PAGE_DRIVER_ENTRY LargePageDriverEntry;
//     ASSERT(KeGetCurrentIrql () <= APC_LEVEL);
//     ASSERT(*ImageBaseAddress >= MmSystemRangeStart);
//
// #ifdef _X86_
//     if (!(KeFeatureBits & KF_LARGE_PAGE)) return FALSE;
//     if (!(__readcr4() & CR4_PSE)) return FALSE;
// #endif
//
//     /* Make sure there's enough system PTEs for a large page driver */
//     if (MmTotalFreeSystemPtes[SystemPteSpace] < (16 * (PDE_MAPPED_VA >> PAGE_SHIFT)))
//     {
//         return FALSE;
//     }
//
//     /* This happens if the registry key had a "*" (wildcard) in it */
//     if (MiLargePageAllDrivers == 0)
//     {
//         /* It didn't, so scan the list */
//         NextEntry = MiLargePageDriverList.Flink;
//         while (NextEntry != &MiLargePageDriverList)
//         {
//             /* Check if the driver name matches */
//             LargePageDriverEntry = CONTAINING_RECORD(NextEntry,
//                                                      MI_LARGE_PAGE_DRIVER_ENTRY,
//                                                      Links);
//             if (RtlEqualUnicodeString(BaseImageName,
//                                       &LargePageDriverEntry->BaseName,
//                                       TRUE))
//             {
//                 /* Enable large pages for this driver */
//                 DriverFound = TRUE;
//                 break;
//             }
//
//             /* Keep trying */
//             NextEntry = NextEntry->Flink;
//         }
//
//         /* If we didn't find the driver, it doesn't need large pages */
//         if (DriverFound == FALSE) return FALSE;
//     }
//
//     /* Nothing to do yet */
//     DPRINT1("Large pages not supported!\n");
//     return FALSE;
// }
//
// VOID
// NTAPI
// MiSetSystemCodeProtection(
//     _In_ PMMPTE FirstPte,
//     _In_ PMMPTE LastPte,
//     _In_ ULONG Protection)
// {
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//
//     /* Loop the PTEs */
//     for (PointerPte = FirstPte; PointerPte <= LastPte; PointerPte++)
//     {
//         /* Read the PTE */
//         TempPte = *PointerPte;
//
//         /* Make sure it's valid */
//         if ((TempPte.valid() as usize) != 1)
//         {
//             DPRINT1("CORE-16449: FirstPte=%p, LastPte=%p, Protection=%lx\n", FirstPte, LastPte, Protection);
//             DPRINT1("CORE-16449: PointerPte=%p, TempPte=%lx\n", PointerPte, TempPte.Long);
//             DPRINT1("CORE-16449: Please issue the 'mod' and 'bt' (KDBG) or 'lm' and 'kp' (WinDbg) commands. Then report this in Jira.\n");
//             ASSERT((TempPte.valid() as usize) == 1);
//             break;
//         }
//
//         /* Update the protection */
// #ifdef _M_ARM64
//         TempPte.u.Hard.Writable = BooleanFlagOn(Protection, IMAGE_SCN_MEM_WRITE);
// #else
//         TempPte.write() = BooleanFlagOn(Protection, IMAGE_SCN_MEM_WRITE);
// #endif
// #if _MI_HAS_NO_EXECUTE
// #ifdef _M_ARM64
//         TempPte.u.Hard.UserNoExecute = !BooleanFlagOn(Protection, IMAGE_SCN_MEM_EXECUTE);
// #else
//         TempPte.u.Hard.NoExecute = !BooleanFlagOn(Protection, IMAGE_SCN_MEM_EXECUTE);
// #endif
// #endif
//
//         MI_UPDATE_VALID_PTE(PointerPte, TempPte);
//     }
//
//     /* Flush it all */
//     KeFlushEntireTb(TRUE, TRUE);
// }
//
// VOID
// NTAPI
// MiWriteProtectSystemImage(
//     _In_ PVOID ImageBase)
// {
//     PIMAGE_NT_HEADERS NtHeaders;
//     PIMAGE_SECTION_HEADER SectionHeaders, Section;
//     ULONG i;
//     PVOID SectionBase, SectionEnd;
//     ULONG SectionSize;
//     ULONG Protection;
//     PMMPTE FirstPte, LastPte;
//
//     /* Check if the registry setting is on or not */
//     if (MmEnforceWriteProtection == FALSE)
//     {
//         /* Ignore section protection */
//         return;
//     }
//
//     /* Large page mapped images are not supported */
//     NT_ASSERT(!MI_IS_PHYSICAL_ADDRESS(ImageBase));
//
//     /* Session images are not yet supported */
//     NT_ASSERT(!MI_IS_SESSION_ADDRESS(ImageBase));
//
//     /* Get the NT headers */
//     NtHeaders = RtlImageNtHeader(ImageBase);
//     if (NtHeaders == NULL)
//     {
//         DPRINT1("Failed to get NT headers for image @ %p\n", ImageBase);
//         return;
//     }
//
//     /* Don't touch NT4 drivers */
//     if ((NtHeaders->OptionalHeader.MajorOperatingSystemVersion < 5) ||
//         (NtHeaders->OptionalHeader.MajorSubsystemVersion < 5))
//     {
//         DPRINT1("Skipping NT 4 driver @ %p\n", ImageBase);
//         return;
//     }
//
//     /* Get the section headers */
//     SectionHeaders = IMAGE_FIRST_SECTION(NtHeaders);
//
//     /* Get the base address of the first section */
//     SectionBase = Add2Ptr(ImageBase, SectionHeaders[0].VirtualAddress);
//
//     /* Start protecting the image header as R/W */
//     FirstPte = MiAddressToPte(ImageBase);
//     LastPte = MiAddressToPte(SectionBase) - 1;
//     Protection = IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
//     if (LastPte >= FirstPte)
//     {
//         MiSetSystemCodeProtection(FirstPte, LastPte, Protection);
//     }
//
//     /* Loop the sections */
//     for (i = 0; i < NtHeaders->FileHeader.NumberOfSections; i++)
//     {
//         /* Get the section base address and size */
//         Section = &SectionHeaders[i];
//         SectionBase = Add2Ptr(ImageBase, Section->VirtualAddress);
//         SectionSize = max(Section->SizeOfRawData, Section->Misc.VirtualSize);
//
//         /* Get the first PTE of this section */
//         FirstPte = MiAddressToPte(SectionBase);
//
//         /* Check for overlap with the previous range */
//         if (FirstPte == LastPte)
//         {
//             /* Combine the old and new protection by ORing them */
//             Protection |= (Section->Characteristics & IMAGE_SCN_PROTECTION_MASK);
//
//             /* Update the protection for this PTE */
//             MiSetSystemCodeProtection(FirstPte, FirstPte, Protection);
//
//             /* Skip this PTE */
//             FirstPte++;
//         }
//
//         /* There can not be gaps! */
//         NT_ASSERT(FirstPte == (LastPte + 1));
//
//         /* Get the end of the section and the last PTE */
//         SectionEnd = Add2Ptr(SectionBase, SectionSize - 1);
//         NT_ASSERT(SectionEnd < Add2Ptr(ImageBase, NtHeaders->OptionalHeader.SizeOfImage));
//         LastPte = MiAddressToPte(SectionEnd);
//
//         /* If there are no more pages (after an overlap), skip this section */
//         if (LastPte < FirstPte)
//         {
//             NT_ASSERT(FirstPte == (LastPte + 1));
//             continue;
//         }
//
//         /* Get the section protection */
//         Protection = (Section->Characteristics & IMAGE_SCN_PROTECTION_MASK);
//
//         /* Update the protection for this section */
//         MiSetSystemCodeProtection(FirstPte, LastPte, Protection);
//     }
//
//     /* Image should end with the last section */
//     if (ALIGN_UP_POINTER_BY(SectionEnd, PAGE_SIZE) !=
//         Add2Ptr(ImageBase, NtHeaders->OptionalHeader.SizeOfImage))
//     {
//         DPRINT1("ImageBase 0x%p ImageSize 0x%lx Section %u VA 0x%lx Raw 0x%lx virt 0x%lx\n",
//             ImageBase,
//             NtHeaders->OptionalHeader.SizeOfImage,
//             i,
//             Section->VirtualAddress,
//             Section->SizeOfRawData,
//             Section->Misc.VirtualSize);
//     }
// }
//
// VOID
// NTAPI
// MiSetPagingOfDriver(IN PMMPTE PointerPte,
//                     IN PMMPTE LastPte)
// {
// #ifdef ENABLE_MISETPAGINGOFDRIVER
//     PVOID ImageBase;
//     PETHREAD CurrentThread = PsGetCurrentThread();
//     PFN_COUNT PageCount = 0;
//     PFN_NUMBER PageFrameIndex;
//     PMMPFN Pfn1;
// #endif // ENABLE_MISETPAGINGOFDRIVER
//
//     PAGED_CODE();
//
// #ifndef ENABLE_MISETPAGINGOFDRIVER
//     /* The page fault handler is broken and doesn't page back in! */
//     DPRINT1("WARNING: MiSetPagingOfDriver() called, but paging is broken! ignoring!\n");
// #else  // ENABLE_MISETPAGINGOFDRIVER
//     /* Get the driver's base address */
//     ImageBase = MiPteToAddress(PointerPte);
//     ASSERT(MI_IS_SESSION_IMAGE_ADDRESS(ImageBase) == FALSE);
//
//     /* If this is a large page, it's stuck in physical memory */
//     if (MI_IS_PHYSICAL_ADDRESS(ImageBase)) return;
//
//     /* Lock the working set */
//     MiLockWorkingSet(CurrentThread, &MmSystemCacheWs);
//
//     /* Loop the PTEs */
//     while (PointerPte <= LastPte)
//     {
//         /* Check for valid PTE */
//         if (PointerPte->u.Hard.Valid == 1)
//         {
//             PageFrameIndex = PFN_FROM_PTE(PointerPte);
//             Pfn1 = MiGetPfnEntry(PageFrameIndex);
//             ASSERT(Pfn1->u2.ShareCount == 1);
//
//             /* No working sets in ReactOS yet */
//             PageCount++;
//         }
//
//         ImageBase = (PVOID)((ULONG_PTR)ImageBase + PAGE_SIZE);
//         PointerPte++;
//     }
//
//     /* Release the working set */
//     MiUnlockWorkingSet(CurrentThread, &MmSystemCacheWs);
//
//     /* Do we have any driver pages? */
//     if (PageCount)
//     {
//         /* Update counters */
//         InterlockedExchangeAdd((PLONG)&MmTotalSystemDriverPages, PageCount);
//     }
// #endif // ENABLE_MISETPAGINGOFDRIVER
// }
//
// VOID
// NTAPI
// MiEnablePagingOfDriver(IN PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     ULONG_PTR ImageBase;
//     PIMAGE_NT_HEADERS NtHeaders;
//     ULONG Sections, Alignment, Size;
//     PIMAGE_SECTION_HEADER Section;
//     PMMPTE PointerPte = NULL, LastPte = NULL;
//     if (MmDisablePagingExecutive) return;
//
//     /* Get the driver base address and its NT header */
//     ImageBase = (ULONG_PTR)LdrEntry->DllBase;
//     NtHeaders = RtlImageNtHeader((PVOID)ImageBase);
//     if (!NtHeaders) return;
//
//     /* Get the sections and their alignment */
//     Sections = NtHeaders->FileHeader.NumberOfSections;
//     Alignment = NtHeaders->OptionalHeader.SectionAlignment - 1;
//
//     /* Loop each section */
//     Section = IMAGE_FIRST_SECTION(NtHeaders);
//     while (Sections)
//     {
//         /* Find PAGE or .edata */
//         if ((*(PULONG)Section->Name == 'EGAP') ||
//             (*(PULONG)Section->Name == 'ade.'))
//         {
//             /* Had we already done some work? */
//             if (!PointerPte)
//             {
//                 /* Nope, setup the first PTE address */
//                 PointerPte = MiAddressToPte(ROUND_TO_PAGES(ImageBase +
//                                                            Section->VirtualAddress));
//             }
//
//             /* Compute the size */
//             Size = max(Section->SizeOfRawData, Section->Misc.VirtualSize);
//
//             /* Find the last PTE that maps this section */
//             LastPte = MiAddressToPte(ImageBase +
//                                      Section->VirtualAddress +
//                                      Alignment + Size - PAGE_SIZE);
//         }
//         else
//         {
//             /* Had we found a section before? */
//             if (PointerPte)
//             {
//                 /* Mark it as pageable */
//                 MiSetPagingOfDriver(PointerPte, LastPte);
//                 PointerPte = NULL;
//             }
//         }
//
//         /* Keep searching */
//         Sections--;
//         Section++;
//     }
//
//     /* Handle the straggler */
//     if (PointerPte) MiSetPagingOfDriver(PointerPte, LastPte);
// }
//
// #ifdef CONFIG_SMP
// FORCEINLINE
// BOOLEAN
// MiVerifyImageIsOkForMpUse(
//     _In_ PIMAGE_NT_HEADERS NtHeaders)
// {
//     /* Fail if we have more than one CPU, but the image is only safe for UP */
//     if ((KeNumberProcessors > 1) &&
//         (NtHeaders->FileHeader.Characteristics & IMAGE_FILE_UP_SYSTEM_ONLY))
//     {
//         return FALSE;
//     }
//     /* Otherwise, it's safe to use */
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// MmVerifyImageIsOkForMpUse(
//     _In_ PVOID BaseAddress)
// {
//     PIMAGE_NT_HEADERS NtHeaders;
//     PAGED_CODE();
//
//     /* Get the NT headers. If none, suppose the image is safe
//      * to use on an MP system, otherwise invoke the helper. */
//     NtHeaders = RtlImageNtHeader(BaseAddress);
//     if (!NtHeaders)
//         return TRUE;
//     return MiVerifyImageIsOkForMpUse(NtHeaders);
// }
// #endif // CONFIG_SMP
//
// NTSTATUS
// NTAPI
// MmCheckSystemImage(
//     _In_ HANDLE ImageHandle)
// {
//     NTSTATUS Status;
//     HANDLE SectionHandle;
//     PVOID ViewBase = NULL;
//     SIZE_T ViewSize = 0;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_STANDARD_INFORMATION FileStandardInfo;
//     KAPC_STATE ApcState;
//     PIMAGE_NT_HEADERS NtHeaders;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PAGED_CODE();
//
//     /* Setup the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Create a section for the DLL */
//     Status = ZwCreateSection(&SectionHandle,
//                              SECTION_MAP_EXECUTE,
//                              &ObjectAttributes,
//                              NULL,
//                              PAGE_EXECUTE,
//                              SEC_IMAGE,
//                              ImageHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwCreateSection failed with status 0x%x\n", Status);
//         return Status;
//     }
//
//     /* Make sure we're in the system process */
//     KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//
//     /* Map it */
//     Status = ZwMapViewOfSection(SectionHandle,
//                                 NtCurrentProcess(),
//                                 &ViewBase,
//                                 0,
//                                 0,
//                                 NULL,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0,
//                                 PAGE_EXECUTE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, close the handle and return */
//         DPRINT1("ZwMapViewOfSection failed with status 0x%x\n", Status);
//         KeUnstackDetachProcess(&ApcState);
//         ZwClose(SectionHandle);
//         return Status;
//     }
//
//     /* Now query image information */
//     Status = ZwQueryInformationFile(ImageHandle,
//                                     &IoStatusBlock,
//                                     &FileStandardInfo,
//                                     sizeof(FileStandardInfo),
//                                     FileStandardInformation);
//     if (NT_SUCCESS(Status))
//     {
//         /* First, verify the checksum */
//         if (!LdrVerifyMappedImageMatchesChecksum(ViewBase,
//                                                  ViewSize,
//                                                  FileStandardInfo.
//                                                  EndOfFile.LowPart))
//         {
//             /* Set checksum failure */
//             Status = STATUS_IMAGE_CHECKSUM_MISMATCH;
//             goto Fail;
//         }
//
//         /* Make sure it's a real image */
//         NtHeaders = RtlImageNtHeader(ViewBase);
//         if (!NtHeaders)
//         {
//             /* Set checksum failure */
//             Status = STATUS_IMAGE_CHECKSUM_MISMATCH;
//             goto Fail;
//         }
//
//         /* Make sure it's for the correct architecture */
//         if ((NtHeaders->FileHeader.Machine != IMAGE_FILE_MACHINE_NATIVE) ||
//             (NtHeaders->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR_MAGIC))
//         {
//             /* Set protection failure */
//             Status = STATUS_INVALID_IMAGE_PROTECT;
//             goto Fail;
//         }
//
// #ifdef CONFIG_SMP
//         /* Check that the image is safe to use if we have more than one CPU */
//         if (!MiVerifyImageIsOkForMpUse(NtHeaders))
//         {
//             /* Otherwise it's not the right image */
//             Status = STATUS_IMAGE_MP_UP_MISMATCH;
//         }
// #endif // CONFIG_SMP
//     }
//
//     /* Unmap the section, close the handle, and return status */
// Fail:
//     ZwUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//     KeUnstackDetachProcess(&ApcState);
//     ZwClose(SectionHandle);
//     return Status;
// }
//
//
// PVOID
// NTAPI
// LdrpFetchAddressOfSecurityCookie(PVOID BaseAddress, ULONG SizeOfImage)
// {
//     PIMAGE_LOAD_CONFIG_DIRECTORY ConfigDir;
//     ULONG DirSize;
//     PULONG_PTR Cookie = NULL;
//
//     /* Get the pointer to the config directory */
//     ConfigDir = RtlImageDirectoryEntryToData(BaseAddress,
//                                              TRUE,
//                                              IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
//                                              &DirSize);
//
//     /* Check for sanity */
//     if (!ConfigDir ||
//         DirSize < RTL_SIZEOF_THROUGH_FIELD(IMAGE_LOAD_CONFIG_DIRECTORY, SecurityCookie))
//     {
//         /* Invalid directory*/
//         return NULL;
//     }
//
//     /* Now get the cookie */
//     Cookie = (PULONG_PTR)ConfigDir->SecurityCookie;
//
//     /* Check this cookie */
//     if ((PCHAR)Cookie <= (PCHAR)BaseAddress ||
//         (PCHAR)Cookie >= (PCHAR)BaseAddress + SizeOfImage - sizeof(*Cookie))
//     {
//         Cookie = NULL;
//     }
//
//     /* Return validated security cookie */
//     return Cookie;
// }
//
// PVOID
// NTAPI
// LdrpInitSecurityCookie(PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PULONG_PTR Cookie;
//     ULONG_PTR NewCookie;
//
//     /* Fetch address of the cookie */
//     Cookie = LdrpFetchAddressOfSecurityCookie(LdrEntry->DllBase, LdrEntry->SizeOfImage);
//
//     if (!Cookie)
//         return NULL;
//     
//     /* Check if it's a default one */
//     if ((*Cookie == DEFAULT_SECURITY_COOKIE) ||
//         (*Cookie == 0))
//     {
//         LARGE_INTEGER Counter = KeQueryPerformanceCounter(NULL);
//         /* The address should be unique */
//         NewCookie = (ULONG_PTR)Cookie;
//
//         /* We just need a simple tick, don't care about precision and whatnot */
//         NewCookie ^= (ULONG_PTR)Counter.LowPart;
// #ifdef _WIN64
//         /* Some images expect first 16 bits to be kept clean (like in default cookie) */
//         if (NewCookie > COOKIE_MAX)
//         {
//             NewCookie >>= 16;
//         }
// #endif
//         /* If the result is 0 or the same as we got, just add one to the default value */
//         if ((NewCookie == 0) || (NewCookie == *Cookie))
//         {
//             NewCookie = DEFAULT_SECURITY_COOKIE + 1;
//         }
//
//         /* Set the new cookie value */
//         *Cookie = NewCookie; 
//     }
//
//     return Cookie;
// }
//
// NTSTATUS
// NTAPI
// MmLoadSystemImage(IN PUNICODE_STRING FileName,
//                   IN PUNICODE_STRING NamePrefix OPTIONAL,
//                   IN PUNICODE_STRING LoadedName OPTIONAL,
//                   IN ULONG Flags,
//                   OUT PVOID *ModuleObject,
//                   OUT PVOID *ImageBaseAddress)
// {
//     PVOID ModuleLoadBase = NULL;
//     NTSTATUS Status;
//     HANDLE FileHandle = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIMAGE_NT_HEADERS NtHeader;
//     UNICODE_STRING BaseName, BaseDirectory, PrefixName;
//     PLDR_DATA_TABLE_ENTRY LdrEntry = NULL;
//     ULONG EntrySize, DriverSize;
//     PLOAD_IMPORTS LoadedImports = MM_SYSLDR_NO_IMPORTS;
//     PCHAR MissingApiName, Buffer;
//     PWCHAR MissingDriverName, PrefixedBuffer = NULL;
//     HANDLE SectionHandle;
//     ACCESS_MASK DesiredAccess;
//     PSECTION Section = NULL;
//     BOOLEAN LockOwned = FALSE;
//     PLIST_ENTRY NextEntry;
//     IMAGE_INFO ImageInfo;
//
//     PAGED_CODE();
//
//     /* Detect session-load */
//     if (Flags)
//     {
//         /* Sanity checks */
//         ASSERT(NamePrefix == NULL);
//         ASSERT(LoadedName == NULL);
//
//         /* Make sure the process is in session too */
//         if (!PsGetCurrentProcess()->ProcessInSession) return STATUS_NO_MEMORY;
//     }
//
//     /* Allocate a buffer we'll use for names */
//     Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                    MAXIMUM_FILENAME_LENGTH,
//                                    TAG_LDR_WSTR);
//     if (!Buffer) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Check for a separator */
//     if (FileName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//     {
//         PWCHAR p;
//         ULONG BaseLength;
//
//         /* Loop the path until we get to the base name */
//         p = &FileName->Buffer[FileName->Length / sizeof(WCHAR)];
//         while (*(p - 1) != OBJ_NAME_PATH_SEPARATOR) p--;
//
//         /* Get the length */
//         BaseLength = (ULONG)(&FileName->Buffer[FileName->Length / sizeof(WCHAR)] - p);
//         BaseLength *= sizeof(WCHAR);
//
//         /* Setup the string */
//         BaseName.Length = (USHORT)BaseLength;
//         BaseName.Buffer = p;
//     }
//     else
//     {
//         /* Otherwise, we already have a base name */
//         BaseName.Length = FileName->Length;
//         BaseName.Buffer = FileName->Buffer;
//     }
//
//     /* Setup the maximum length */
//     BaseName.MaximumLength = BaseName.Length;
//
//     /* Now compute the base directory */
//     BaseDirectory = *FileName;
//     BaseDirectory.Length -= BaseName.Length;
//     BaseDirectory.MaximumLength = BaseDirectory.Length;
//
//     /* And the prefix, which for now is just the name itself */
//     PrefixName = *FileName;
//
//     /* Check if we have a prefix */
//     if (NamePrefix)
//     {
//         /* Check if "directory + prefix" is too long for the string */
//         Status = RtlUShortAdd(BaseDirectory.Length,
//                               NamePrefix->Length,
//                               &PrefixName.MaximumLength);
//         if (!NT_SUCCESS(Status))
//         {
//             Status = STATUS_INVALID_PARAMETER;
//             goto Quickie;
//         }
//
//         /* Check if "directory + prefix + basename" is too long for the string */
//         Status = RtlUShortAdd(PrefixName.MaximumLength,
//                               BaseName.Length,
//                               &PrefixName.MaximumLength);
//         if (!NT_SUCCESS(Status))
//         {
//             Status = STATUS_INVALID_PARAMETER;
//             goto Quickie;
//         }
//
//         /* Allocate the buffer exclusively used for prefixed name */
//         PrefixedBuffer = ExAllocatePoolWithTag(PagedPool,
//                                                PrefixName.MaximumLength,
//                                                TAG_LDR_WSTR);
//         if (!PrefixedBuffer)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quickie;
//         }
//
//         /* Clear out the prefixed name string */
//         PrefixName.Buffer = PrefixedBuffer;
//         PrefixName.Length = 0;
//
//         /* Concatenate the strings */
//         RtlAppendUnicodeStringToString(&PrefixName, &BaseDirectory);
//         RtlAppendUnicodeStringToString(&PrefixName, NamePrefix);
//         RtlAppendUnicodeStringToString(&PrefixName, &BaseName);
//
//         /* Now the base name of the image becomes the prefixed version */
//         BaseName.Buffer = &(PrefixName.Buffer[BaseDirectory.Length / sizeof(WCHAR)]);
//         BaseName.Length += NamePrefix->Length;
//         BaseName.MaximumLength = (PrefixName.MaximumLength - BaseDirectory.Length);
//     }
//
//     /* Check if we already have a name, use it instead */
//     if (LoadedName) BaseName = *LoadedName;
//
//     /* Check for loader snap debugging */
//     if (NtGlobalFlag & FLG_SHOW_LDR_SNAPS)
//     {
//         /* Print out standard string */
//         DPRINT1("MM:SYSLDR Loading %wZ (%wZ) %s\n",
//                 &PrefixName, &BaseName, Flags ? "in session space" : "");
//     }
//
//     /* Acquire the load lock */
// LoaderScan:
//     ASSERT(LockOwned == FALSE);
//     LockOwned = TRUE;
//     KeEnterCriticalRegion();
//     KeWaitForSingleObject(&MmSystemLoadLock,
//                           WrVirtualMemory,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     /* Scan the module list */
//     NextEntry = PsLoadedModuleList.Flink;
//     while (NextEntry != &PsLoadedModuleList)
//     {
//         /* Get the entry and compare the names */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//         if (RtlEqualUnicodeString(&PrefixName, &LdrEntry->FullDllName, TRUE))
//         {
//             /* Found it, break out */
//             break;
//         }
//
//         /* Keep scanning */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Check if we found the image */
//     if (NextEntry != &PsLoadedModuleList)
//     {
//         /* Check if we had already mapped a section */
//         if (Section)
//         {
//             /* Dereference and clear */
//             ObDereferenceObject(Section);
//             Section = NULL;
//         }
//
//         /* Check if this was supposed to be a session load */
//         if (!Flags)
//         {
//             /* It wasn't, so just return the data */
//             *ModuleObject = LdrEntry;
//             *ImageBaseAddress = LdrEntry->DllBase;
//             Status = STATUS_IMAGE_ALREADY_LOADED;
//         }
//         else
//         {
//             /* We don't support session loading yet */
//             UNIMPLEMENTED_DBGBREAK("Unsupported Session-Load!\n");
//             Status = STATUS_NOT_IMPLEMENTED;
//         }
//
//         /* Do cleanup */
//         goto Quickie;
//     }
//     else if (!Section)
//     {
//         /* It wasn't loaded, and we didn't have a previous attempt */
//         KeReleaseMutant(&MmSystemLoadLock, MUTANT_INCREMENT, FALSE, FALSE);
//         KeLeaveCriticalRegion();
//         LockOwned = FALSE;
//
//         /* Check if KD is enabled */
//         if ((KdDebuggerEnabled) && !(KdDebuggerNotPresent))
//         {
//             /* FIXME: Attempt to get image from KD */
//         }
//
//         /* We don't have a valid entry */
//         LdrEntry = NULL;
//
//         /* Setup image attributes */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    FileName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    NULL);
//
//         /* Open the image */
//         Status = ZwOpenFile(&FileHandle,
//                             FILE_EXECUTE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_DELETE,
//                             0);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwOpenFile failed for '%wZ' with status 0x%x\n",
//                     FileName, Status);
//             goto Quickie;
//         }
//
//         /* Validate it */
// #ifdef _M_ARM64
//         /*
//          * Skip MmCheckSystemImage on ARM64: it maps via SEC_IMAGE into user
//          * space which returns corrupted data for large images (>1MB), and
//          * ZwUnmapViewOfSection hangs for the resulting ~17MB mapping.
//          * The actual loading via MiLoadImageSection uses kernel-space mapping.
//          */
// #else
//         Status = MmCheckSystemImage(FileHandle);
//         if ((Status == STATUS_IMAGE_CHECKSUM_MISMATCH) ||
//             (Status == STATUS_IMAGE_MP_UP_MISMATCH) ||
//             (Status == STATUS_INVALID_IMAGE_PROTECT))
//         {
//             /* Fail loading */
//             goto Quickie;
//         }
// #endif
//
//         /* Check if this is a session-load */
//         if (Flags)
//         {
//             /* Then we only need read and execute */
//             DesiredAccess = SECTION_MAP_READ | SECTION_MAP_EXECUTE;
//         }
//         else
//         {
//             /* Otherwise, we can allow write access */
//             DesiredAccess = SECTION_ALL_ACCESS;
//         }
//
//         /* Initialize the attributes for the section */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    NULL,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    NULL);
//
//         /* Create the section */
//         Status = ZwCreateSection(&SectionHandle,
//                                  DesiredAccess,
//                                  &ObjectAttributes,
//                                  NULL,
//                                  PAGE_EXECUTE,
//                                  SEC_IMAGE,
//                                  FileHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwCreateSection failed with status 0x%x\n", Status);
//             goto Quickie;
//         }
//
//         /* Now get the section pointer */
//         Status = ObReferenceObjectByHandle(SectionHandle,
//                                            SECTION_MAP_EXECUTE,
//                                            MmSectionObjectType,
//                                            KernelMode,
//                                            (PVOID*)&Section,
//                                            NULL);
//         ZwClose(SectionHandle);
//         if (!NT_SUCCESS(Status)) goto Quickie;
//
//         /* Check if this was supposed to be a session-load */
//         if (Flags)
//         {
//             /* We don't support session loading yet */
//             UNIMPLEMENTED_DBGBREAK("Unsupported Session-Load!\n");
//             goto Quickie;
//         }
//
//         /* Check the loader list again, we should end up in the path below */
//         goto LoaderScan;
//     }
//     else
//     {
//         /* We don't have a valid entry */
//         LdrEntry = NULL;
//     }
//
//     /* Load the image */
//     Status = MiLoadImageSection(&Section,
//                                 &ModuleLoadBase,
//                                 FileName,
//                                 FALSE,
//                                 NULL);
//     ASSERT(Status != STATUS_ALREADY_COMMITTED);
//
//     /* Get the size of the driver */
//     DriverSize = ((PMM_IMAGE_SECTION_OBJECT)Section->Segment)->ImageInformation.ImageFileSize;
//
//     /* Make sure we're not being loaded into session space */
//     if (!Flags)
//     {
//         /* Check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Support large pages for drivers */
//             MiUseLargeDriverPage(DriverSize / PAGE_SIZE,
//                                  &ModuleLoadBase,
//                                  &BaseName,
//                                  TRUE);
//         }
//
//         /* Dereference the section */
//         ObDereferenceObject(Section);
//         Section = NULL;
//     }
//
//     /* Check for failure of the load earlier */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MiLoadImageSection failed with status 0x%x\n", Status);
//         goto Quickie;
//     }
//
//     /* Relocate the driver */
//     Status = LdrRelocateImageWithBias(ModuleLoadBase,
//                                       0,
//                                       "SYSLDR",
//                                       STATUS_SUCCESS,
//                                       STATUS_CONFLICTING_ADDRESSES,
//                                       STATUS_INVALID_IMAGE_FORMAT);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LdrRelocateImageWithBias failed with status 0x%x\n", Status);
//         goto Quickie;
//     }
//
//     /* Get the NT Header */
//     NtHeader = RtlImageNtHeader(ModuleLoadBase);
//
//     /* Calculate the size we'll need for the entry and allocate it */
//     EntrySize = sizeof(LDR_DATA_TABLE_ENTRY) +
//                 BaseName.Length +
//                 sizeof(UNICODE_NULL);
//
//     /* Allocate the entry */
//     LdrEntry = ExAllocatePoolWithTag(NonPagedPool, EntrySize, TAG_MODULE_OBJECT);
//     if (!LdrEntry)
//     {
//         /* Fail */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Setup the entry */
//     LdrEntry->Flags = LDRP_LOAD_IN_PROGRESS;
//     LdrEntry->LoadCount = 1;
//     LdrEntry->LoadedImports = LoadedImports;
//     LdrEntry->PatchInformation = NULL;
//
//     /* Check the version */
//     if ((NtHeader->OptionalHeader.MajorOperatingSystemVersion >= 5) &&
//         (NtHeader->OptionalHeader.MajorImageVersion >= 5))
//     {
//         /* Mark this image as a native image */
//         LdrEntry->Flags |= LDRP_ENTRY_NATIVE;
//     }
//
//     /* Setup the rest of the entry */
//     LdrEntry->DllBase = ModuleLoadBase;
//     LdrEntry->EntryPoint = (PVOID)((ULONG_PTR)ModuleLoadBase +
//                                    NtHeader->OptionalHeader.AddressOfEntryPoint);
//     LdrEntry->SizeOfImage = DriverSize;
//     LdrEntry->CheckSum = NtHeader->OptionalHeader.CheckSum;
//     LdrEntry->SectionPointer = Section;
//
//     /* Now write the DLL name */
//     LdrEntry->BaseDllName.Buffer = (PVOID)(LdrEntry + 1);
//     LdrEntry->BaseDllName.Length = BaseName.Length;
//     LdrEntry->BaseDllName.MaximumLength = BaseName.Length;
//
//     /* Copy and null-terminate it */
//     RtlCopyMemory(LdrEntry->BaseDllName.Buffer,
//                   BaseName.Buffer,
//                   BaseName.Length);
//     LdrEntry->BaseDllName.Buffer[BaseName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Now allocate the full name */
//     LdrEntry->FullDllName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                          PrefixName.Length +
//                                                          sizeof(UNICODE_NULL),
//                                                          TAG_LDR_WSTR);
//     if (!LdrEntry->FullDllName.Buffer)
//     {
//         /* Don't fail, just set it to zero */
//         LdrEntry->FullDllName.Length = 0;
//         LdrEntry->FullDllName.MaximumLength = 0;
//     }
//     else
//     {
//         /* Set it up */
//         LdrEntry->FullDllName.Length = PrefixName.Length;
//         LdrEntry->FullDllName.MaximumLength = PrefixName.Length;
//
//         /* Copy and null-terminate */
//         RtlCopyMemory(LdrEntry->FullDllName.Buffer,
//                       PrefixName.Buffer,
//                       PrefixName.Length);
//         LdrEntry->FullDllName.Buffer[PrefixName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* Add the entry */
//     MiProcessLoaderEntry(LdrEntry, TRUE);
//
//     /* Resolve imports */
//     MissingApiName = Buffer;
//     MissingDriverName = NULL;
//     Status = MiResolveImageReferences(ModuleLoadBase,
//                                       &BaseDirectory,
//                                       NULL,
//                                       &MissingApiName,
//                                       &MissingDriverName,
//                                       &LoadedImports);
//     if (!NT_SUCCESS(Status))
//     {
//         BOOLEAN NeedToFreeString = FALSE;
//
//         /* If the lowest bit is set to 1, this is a hint that we need to free */
//         if (*(ULONG_PTR*)&MissingDriverName & 1)
//         {
//             NeedToFreeString = TRUE;
//             *(ULONG_PTR*)&MissingDriverName &= ~1;
//         }
//
//         DPRINT1("MiResolveImageReferences failed with status 0x%x\n", Status);
//         DPRINT1(" Missing driver '%ls', missing API '%s'\n",
//                 MissingDriverName, MissingApiName);
//
//         if (NeedToFreeString)
//         {
//             ExFreePoolWithTag(MissingDriverName, TAG_LDR_WSTR);
//         }
//
//         /* Fail */
//         MiProcessLoaderEntry(LdrEntry, FALSE);
//
//         /* Check if we need to free the name */
//         if (LdrEntry->FullDllName.Buffer)
//         {
//             /* Free it */
//             ExFreePoolWithTag(LdrEntry->FullDllName.Buffer, TAG_LDR_WSTR);
//         }
//
//         /* Free the entry itself */
//         ExFreePoolWithTag(LdrEntry, TAG_MODULE_OBJECT);
//         LdrEntry = NULL;
//         goto Quickie;
//     }
//
//     /* Update the loader entry */
//     LdrEntry->Flags |= (LDRP_SYSTEM_MAPPED |
//                         LDRP_ENTRY_PROCESSED |
//                         LDRP_MM_LOADED);
//     LdrEntry->Flags &= ~LDRP_LOAD_IN_PROGRESS;
//     LdrEntry->LoadedImports = LoadedImports;
//
//     /* FIXME: Call driver verifier's loader function */
//
//     /* Write-protect the system image */
//     MiWriteProtectSystemImage(LdrEntry->DllBase);
//
//     /* Initialize the security cookie (Win7 is not doing this yet!) */
//     LdrpInitSecurityCookie(LdrEntry);
//
//     /* Check if notifications are enabled */
//     if (PsImageNotifyEnabled)
//     {
//         /* Fill out the notification data */
//         ImageInfo.Properties = 0;
//         ImageInfo.ImageAddressingMode = IMAGE_ADDRESSING_MODE_32BIT;
//         ImageInfo.SystemModeImage = TRUE;
//         ImageInfo.ImageSize = LdrEntry->SizeOfImage;
//         ImageInfo.ImageBase = LdrEntry->DllBase;
//         ImageInfo.ImageSectionNumber = ImageInfo.ImageSelector = 0;
//
//         /* Send the notification */
//         PspRunLoadImageNotifyRoutines(FileName, NULL, &ImageInfo);
//     }
//
// #ifdef __ROS_ROSSYM__
//     /* MiCacheImageSymbols doesn't detect rossym */
//     if (TRUE)
// #else
//     /* Check if there's symbols */
//     if (MiCacheImageSymbols(LdrEntry->DllBase))
// #endif
//     {
//         UNICODE_STRING UnicodeTemp;
//         STRING AnsiTemp;
//
//         /* Check if the system root is present */
//         if ((PrefixName.Length > (11 * sizeof(WCHAR))) &&
//             !(_wcsnicmp(PrefixName.Buffer, L"\\SystemRoot", 11)))
//         {
//             /* Add the system root */
//             UnicodeTemp = PrefixName;
//             UnicodeTemp.Buffer += 11;
//             UnicodeTemp.Length -= (11 * sizeof(WCHAR));
//             RtlStringCbPrintfA(Buffer,
//                                MAXIMUM_FILENAME_LENGTH,
//                                "%ws%wZ",
//                                &SharedUserData->NtSystemRoot[2],
//                                &UnicodeTemp);
//         }
//         else
//         {
//             /* Build the name */
//             RtlStringCbPrintfA(Buffer, MAXIMUM_FILENAME_LENGTH,
//                                "%wZ", &BaseName);
//         }
//
//         /* Setup the ANSI string */
//         RtlInitString(&AnsiTemp, Buffer);
//
//         /* Notify the debugger */
//         DbgLoadImageSymbols(&AnsiTemp,
//                             LdrEntry->DllBase,
//                             (ULONG_PTR)PsGetCurrentProcessId());
//         LdrEntry->Flags |= LDRP_DEBUG_SYMBOLS_LOADED;
//     }
//
//     /* Page the driver */
//     ASSERT(Section == NULL);
//     MiEnablePagingOfDriver(LdrEntry);
//
//     /* Return pointers */
//     *ModuleObject = LdrEntry;
//     *ImageBaseAddress = LdrEntry->DllBase;
//
// Quickie:
//     /* Check if we have the lock acquired */
//     if (LockOwned)
//     {
//         /* Release the lock */
//         KeReleaseMutant(&MmSystemLoadLock, MUTANT_INCREMENT, FALSE, FALSE);
//         KeLeaveCriticalRegion();
//         LockOwned = FALSE;
//     }
//
//     /* If we have a file handle, close it */
//     if (FileHandle) ZwClose(FileHandle);
//
//     /* If we have allocated a prefixed name buffer, free it */
//     if (PrefixedBuffer) ExFreePoolWithTag(PrefixedBuffer, TAG_LDR_WSTR);
//
//     /* Free the name buffer and return status */
//     ExFreePoolWithTag(Buffer, TAG_LDR_WSTR);
//     return Status;
// }
//
// PLDR_DATA_TABLE_ENTRY
// NTAPI
// MiLookupDataTableEntry(IN PVOID Address)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry, FoundEntry = NULL;
//     PLIST_ENTRY NextEntry;
//     PAGED_CODE();
//
//     /* Loop entries */
//     NextEntry = PsLoadedModuleList.Flink;
//     do
//     {
//         /* Get the loader entry */
//         LdrEntry =  CONTAINING_RECORD(NextEntry,
//                                       LDR_DATA_TABLE_ENTRY,
//                                       InLoadOrderLinks);
//
//         /* Check if the address matches */
//         if ((Address >= LdrEntry->DllBase) &&
//             (Address < (PVOID)((ULONG_PTR)LdrEntry->DllBase +
//                                LdrEntry->SizeOfImage)))
//         {
//             /* Found a match */
//             FoundEntry = LdrEntry;
//             break;
//         }
//
//         /* Move on */
//         NextEntry = NextEntry->Flink;
//     } while(NextEntry != &PsLoadedModuleList);
//
//     /* Return the entry */
//     return FoundEntry;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmPageEntireDriver(IN PVOID AddressWithinSection)
// {
//     PMMPTE StartPte, EndPte;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PAGED_CODE();
//
//     /* Get the loader entry */
//     LdrEntry = MiLookupDataTableEntry(AddressWithinSection);
//     if (!LdrEntry) return NULL;
//
//     /* Check if paging of kernel mode is disabled or if the driver is mapped as an image */
//     if ((MmDisablePagingExecutive) || (LdrEntry->SectionPointer))
//     {
//         /* Don't do anything, just return the base address */
//         return LdrEntry->DllBase;
//     }
//
//     /* Wait for active DPCs to finish before we page out the driver */
//     KeFlushQueuedDpcs();
//
//     /* Get the PTE range for the whole driver image */
//     StartPte = MiAddressToPte((ULONG_PTR)LdrEntry->DllBase);
//     EndPte = MiAddressToPte((ULONG_PTR)LdrEntry->DllBase + LdrEntry->SizeOfImage);
//
//     /* Enable paging for the PTE range */
//     ASSERT(MI_IS_SESSION_IMAGE_ADDRESS(AddressWithinSection) == FALSE);
//     MiSetPagingOfDriver(StartPte, EndPte);
//
//     /* Return the base address */
//     return LdrEntry->DllBase;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// MmResetDriverPaging(IN PVOID AddressWithinSection)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmGetSystemRoutineAddress(IN PUNICODE_STRING SystemRoutineName)
// {
//     PVOID ProcAddress = NULL;
//     ANSI_STRING AnsiRoutineName;
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     BOOLEAN Found = FALSE;
//     UNICODE_STRING KernelName = RTL_CONSTANT_STRING(L"ntoskrnl.exe");
//     UNICODE_STRING HalName = RTL_CONSTANT_STRING(L"hal.dll");
//     ULONG Modules = 0;
//
//     /* Convert routine to ANSI name */
//     Status = RtlUnicodeStringToAnsiString(&AnsiRoutineName,
//                                           SystemRoutineName,
//                                           TRUE);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Lock the list */
//     KeEnterCriticalRegion();
//     ExAcquireResourceSharedLite(&PsLoadedModuleResource, TRUE);
//
//     /* Loop the loaded module list */
//     NextEntry = PsLoadedModuleList.Flink;
//     while (NextEntry != &PsLoadedModuleList)
//     {
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* Check if it's the kernel or HAL */
//         if (RtlEqualUnicodeString(&KernelName, &LdrEntry->BaseDllName, TRUE))
//         {
//             /* Found it */
//             Found = TRUE;
//             Modules++;
//         }
//         else if (RtlEqualUnicodeString(&HalName, &LdrEntry->BaseDllName, TRUE))
//         {
//             /* Found it */
//             Found = TRUE;
//             Modules++;
//         }
//
//         /* Check if we found a valid binary */
//         if (Found)
//         {
//             /* Find the procedure name */
//             ProcAddress = RtlFindExportedRoutineByName(LdrEntry->DllBase,
//                                                        AnsiRoutineName.Buffer);
//
//             /* Break out if we found it or if we already tried both modules */
//             if (ProcAddress) break;
//             if (Modules == 2) break;
//         }
//
//         /* Keep looping */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the lock */
//     ExReleaseResourceLite(&PsLoadedModuleResource);
//     KeLeaveCriticalRegion();
//
//     /* Free the string and return */
//     RtlFreeAnsiString(&AnsiRoutineName);
//     return ProcAddress;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
