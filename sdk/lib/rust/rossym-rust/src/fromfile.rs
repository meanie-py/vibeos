use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS, IMAGE_SECTION_HEADER, PIMAGE_SECTION_HEADER};
use core::ffi::c_void;

extern "C" {
    fn RosSymReadFile(FileContext: *mut core::ffi::c_void, Buffer: *mut core::ffi::c_void, Size: usize) -> u8;
    fn RosSymSeekFile(FileContext: *mut core::ffi::c_void, Offset: i64) -> u8;
    fn RosSymAllocMem(Size: usize) -> *mut core::ffi::c_void;
    fn RosSymFreeMem(Mem: *mut core::ffi::c_void);
    fn ROSSYM_IS_VALID_DOS_HEADER(DosHeader: *const IMAGE_DOS_HEADER) -> u8;
    fn ROSSYM_IS_VALID_NT_HEADERS(NtHeaders: *const IMAGE_NT_HEADERS) -> u8;
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn strncpy(dst: *mut core::ffi::c_char, src: *const core::ffi::c_char, n: usize) -> *mut core::ffi::c_char;
    fn DPRINT1(format: *const core::ffi::c_char, ...);
    fn DPRINT(format: *const core::ffi::c_char, ...);
}

// ---- file-scope preamble (translated) ----
// This file has no corresponding Rust declarations because the only
// file-scope C declaration is the object-like macro "#define NDEBUG",
// which has no value and thus no direct Rust equivalent. Per the
// translation rules, such macros are skipped.
#[no_mangle]
pub unsafe extern "C" fn RosSymCreateFromFile(FileContext: *mut core::ffi::c_void, RosSymInfo: *mut *mut ROSSYM_INFO) -> u8 {
    const IMAGE_SIZEOF_SHORT_NAME: usize = 8;
    let rossym_section_name = b"rossym\0".as_ptr() as *const i8;

    let mut dos_header: IMAGE_DOS_HEADER = core::mem::zeroed();
    let mut nt_headers: IMAGE_NT_HEADERS = core::mem::zeroed();
    let mut section_headers: *mut IMAGE_SECTION_HEADER;
    let mut section_header: *mut IMAGE_SECTION_HEADER;
    let mut section_index: u32;
    let mut section_name: [i8; IMAGE_SIZEOF_SHORT_NAME] = [0; IMAGE_SIZEOF_SHORT_NAME];
    let mut rossym_header: ROSSYM_HEADER = core::mem::zeroed();

    /* Load DOS header */
    if RosSymReadFile(FileContext, core::ptr::addr_of_mut!(dos_header) as *mut core::ffi::c_void, core::mem::size_of::<IMAGE_DOS_HEADER>()) == 0 {
        // DPRINT removed
        return 0u8;
    }
    if ROSSYM_IS_VALID_DOS_HEADER(core::ptr::addr_of!(dos_header)) == 0 {
        // DPRINT removed
        return 0u8;
    }

    /* Load NT headers */
    if RosSymSeekFile(FileContext, dos_header.e_lfanew as i64) == 0 {
        // DPRINT removed
        return 0u8;
    }
    if RosSymReadFile(FileContext, core::ptr::addr_of_mut!(nt_headers) as *mut core::ffi::c_void, core::mem::size_of::<IMAGE_NT_HEADERS>()) == 0 {
        // DPRINT removed
        return 0u8;
    }
    if ROSSYM_IS_VALID_NT_HEADERS(core::ptr::addr_of!(nt_headers)) == 0 {
        // DPRINT removed
        return 0u8;
    }

    /* Load section headers */
    let nt_headers_ptr = core::ptr::addr_of_mut!(nt_headers);
    let first_section_ptr = (core::ptr::addr_of!((*nt_headers_ptr).OptionalHeader) as *const u8)
        .add(nt_headers.FileHeader.SizeOfOptionalHeader as usize) as *mut IMAGE_SECTION_HEADER;
    let seek_offset = (first_section_ptr as usize) - (nt_headers_ptr as usize) + dos_header.e_lfanew as usize;
    if RosSymSeekFile(FileContext, seek_offset as i64) == 0 {
        // DPRINT removed
        return 0u8;
    }
    let header_count = nt_headers.FileHeader.NumberOfSections;
    section_headers = RosSymAllocMem(header_count as usize * core::mem::size_of::<IMAGE_SECTION_HEADER>()) as *mut IMAGE_SECTION_HEADER;
    if section_headers.is_null() {
        // DPRINT removed
        return 0u8;
    }
    if RosSymReadFile(FileContext, section_headers as *mut core::ffi::c_void, header_count as usize * core::mem::size_of::<IMAGE_SECTION_HEADER>()) == 0 {
        RosSymFreeMem(section_headers as *mut core::ffi::c_void);
        // DPRINT removed
        return 0u8;
    }

    /* Search for the section header */
    strncpy(section_name.as_mut_ptr(), rossym_section_name, IMAGE_SIZEOF_SHORT_NAME);
    section_header = section_headers;
    section_index = 0;
    while section_index < header_count as u32 {
        if memcmp(section_name.as_ptr() as *const core::ffi::c_void, (*section_header).Name.as_ptr() as *const core::ffi::c_void, IMAGE_SIZEOF_SHORT_NAME) == 0 {
            break;
        }
        section_index += 1;
        section_header = section_header.add(1);
    }
    if (header_count as u32) <= section_index {
        RosSymFreeMem(section_headers as *mut core::ffi::c_void);
        // DPRINT removed
        return 0u8;
    }

    /* Load rossym header */
    if RosSymSeekFile(FileContext, (*section_header).PointerToRawData as i64) == 0 {
        RosSymFreeMem(section_headers as *mut core::ffi::c_void);
        // DPRINT removed
        return 0u8;
    }
    RosSymFreeMem(section_headers as *mut core::ffi::c_void);
    if RosSymReadFile(FileContext, core::ptr::addr_of_mut!(rossym_header) as *mut core::ffi::c_void, core::mem::size_of::<ROSSYM_HEADER>()) == 0 {
        // DPRINT removed
        return 0u8;
    }
    if rossym_header.SymbolsOffset < core::mem::size_of::<ROSSYM_HEADER>() as u32
        || rossym_header.StringsOffset < rossym_header.SymbolsOffset + rossym_header.SymbolsLength
        || 0 != (rossym_header.SymbolsLength % core::mem::size_of::<ROSSYM_ENTRY>() as u32)
    {
        // DPRINT removed
        return 0u8;
    }

    let alloc_size = core::mem::size_of::<ROSSYM_INFO>() - core::mem::size_of::<ROSSYM_HEADER>()
        + rossym_header.StringsOffset as usize + rossym_header.StringsLength as usize + 1;
    *RosSymInfo = RosSymAllocMem(alloc_size) as *mut ROSSYM_INFO;
    if (*RosSymInfo).is_null() {
        // DPRINT removed
        return 0u8;
    }
    let info_ptr = *RosSymInfo;
    let base_addr = info_ptr as usize;
    (*info_ptr).Symbols = (base_addr + core::mem::size_of::<ROSSYM_INFO>() - core::mem::size_of::<ROSSYM_HEADER>()
        + rossym_header.SymbolsOffset as usize) as *mut ROSSYM_ENTRY;
    (*info_ptr).SymbolsCount = rossym_header.SymbolsLength / core::mem::size_of::<ROSSYM_ENTRY>() as u32;
    (*info_ptr).Strings = (base_addr + core::mem::size_of::<ROSSYM_INFO>() - core::mem::size_of::<ROSSYM_HEADER>()
        + rossym_header.StringsOffset as usize) as *mut i8;
    (*info_ptr).StringsLength = rossym_header.StringsLength;
    let read_size = rossym_header.StringsOffset as usize + rossym_header.StringsLength as usize
        - core::mem::size_of::<ROSSYM_HEADER>();
    if RosSymReadFile(FileContext, info_ptr.add(1) as *mut core::ffi::c_void, read_size) == 0 {
        // DPRINT removed
        return 0u8;
    }
    /* Make sure the last string is null terminated, we allocated an extra byte for that */
    *((*info_ptr).Strings.add((*info_ptr).StringsLength as usize)) = 0;

    return 1u8;
}
