//! [correctness-review 2026-06-07 · sonnet; FIXED 2026-06-07 · opus] Reviewed vs sdk/lib/rossym/fromfile.c.
//! Risk: HIGH → RESOLVED. All findings below are FIXED: `.rossym` section name corrected (was the
//! always-FALSE bug); the macro phantoms (RosSym{Read,Seek,Alloc,Free}/IS_VALID_*) are now local
//! `#[inline] unsafe fn` wrappers = the rossympriv.h macro expansions, so widths/ABI are correct.
//! Behavior: broken — section name never matches; function always returns FALSE. [original finding]
//! Functions reviewed: RosSymCreateFromFile (1/1).
//! Findings:
//!   BUG CRITICAL — `rossym_section_name = b"rossym\0"` is missing the leading dot (line 29).
//!     C uses `ROSSYM_SECTION_NAME = ".rossym"` (7 chars with dot, null-padded to 8).
//!     After `strncpy`, section_name = ['r','o','s','s','y','m','\0','\0'].
//!     Real section headers have Name = ['.','r','o','s','s','y','m','\0'].
//!     `memcmp` always fails ('r' != '.'), so `RosSymCreateFromFile` always returns 0 (FALSE).
//!     Fix: `let rossym_section_name = b".rossym\0".as_ptr() as *const i8;`
//!   BUG LINKAGE — `extern "C"` declarations for `RosSymReadFile`, `RosSymSeekFile`,
//!     `RosSymAllocMem`, `RosSymFreeMem`, `ROSSYM_IS_VALID_DOS_HEADER`,
//!     `ROSSYM_IS_VALID_NT_HEADERS` (lines 9-18). All six are C preprocessor macros
//!     in rossympriv.h, not exported functions. No symbol with these names exists in any
//!     object file. These will produce undefined-symbol link errors when integrated.
//!     Fix: call through `crate::init::RosSymCallbacks` function pointers directly, and
//!     inline the IS_VALID checks (IMAGE_DOS_SIGNATURE/e_magic/e_lfanew comparisons).
//!   NOTE — `RosSymSeekFile` declared with `Offset: i64` (line 10) but the real callback
//!     is `SeekFileProc(PVOID, ULONG_PTR)`. ULONG_PTR = usize (unsigned, pointer-sized).
//!     i64 is always 8 bytes; on i386 ULONG_PTR is 4 bytes — wrong width AND sign.
//!   NOTE — `RosSymReadFile` declared with `Size: usize` (line 9) but the real callback
//!     has `Size: ULONG` (u32). On amd64/arm64, usize is 8 bytes; u32 is 4 bytes.
//!     The mismatch is benign in practice (values fit in 32 bits) but is technically wrong.
//!   NOTE — `section_index` is u32 (line 35) and `header_count` is u16 (USHORT).
//!     Comparison `header_count as u32 <= section_index` is correct.
//!   NOTE — seek_offset computation (lines 65-67) correctly reproduces the
//!     `IMAGE_FIRST_SECTION` macro arithmetic. Allocation, read sizes, pointer arithmetic
//!     for ROSSYM_INFO Symbols/Strings fields, and the null-terminator write all match C.
//!   NOTE — ROSSYM_HEADER validation condition (3-way check, lines 112-114) correctly
//!     matches fromfile.c (which has 3 conditions vs fromraw.c's 4).
//! Unverified: Whether a cmake-level C shim will provide the extern "C" callback wrappers;
//!   if so, the ABI annotations (i64/usize types) must still be corrected.

use crate::types::*;
use ntkrnl_sys::ddk::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS, IMAGE_SECTION_HEADER};

extern "C" {
    pub fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    pub fn strncpy(dst: *mut core::ffi::c_char, src: *const core::ffi::c_char, n: usize) -> *mut core::ffi::c_char;
}

// PE signature constants. The ROSSYM_IS_VALID_* macros expand against these; they're not in the
// kernel ddk bindgen, so they're defined locally with the standard PE values.
const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D; // 'MZ'
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550; // 'PE\0\0'
#[cfg(target_arch = "x86")]
const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x010B; // PE32
#[cfg(not(target_arch = "x86"))]
const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x020B; // PE32+

// rossympriv.h defines these as function-like macros over the global RosSymCallbacks / inline
// header checks. Translated as local wrappers (= the macro expansions) so every call site below
// is unchanged. NOT exported symbols — the original "extern" decls were a mechanical-translation
// artifact that would have produced undefined-symbol link errors.
#[inline]
unsafe fn RosSymReadFile(FileContext: *mut core::ffi::c_void, Buffer: *mut core::ffi::c_void, Size: usize) -> u8 {
    (crate::init::RosSymCallbacks.ReadFileProc.unwrap())(FileContext, Buffer, Size as u32)
}
#[inline]
unsafe fn RosSymSeekFile(FileContext: *mut core::ffi::c_void, Offset: i64) -> u8 {
    (crate::init::RosSymCallbacks.SeekFileProc.unwrap())(FileContext, Offset as usize)
}
#[inline]
unsafe fn RosSymAllocMem(Size: usize) -> *mut core::ffi::c_void {
    (crate::init::RosSymCallbacks.AllocMemProc.unwrap())(Size)
}
#[inline]
unsafe fn RosSymFreeMem(Mem: *mut core::ffi::c_void) {
    (crate::init::RosSymCallbacks.FreeMemProc.unwrap())(Mem)
}
#[inline]
unsafe fn ROSSYM_IS_VALID_DOS_HEADER(DosHeader: *const IMAGE_DOS_HEADER) -> u8 {
    ((*DosHeader).e_magic == IMAGE_DOS_SIGNATURE && (*DosHeader).e_lfanew != 0) as u8
}
#[inline]
unsafe fn ROSSYM_IS_VALID_NT_HEADERS(NtHeaders: *const IMAGE_NT_HEADERS) -> u8 {
    ((*NtHeaders).Signature == IMAGE_NT_SIGNATURE
        && (*NtHeaders).OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR_MAGIC) as u8
}

// ---- file-scope preamble (translated) ----
// This file has no corresponding Rust declarations because the only
// file-scope C declaration is the object-like macro "#define NDEBUG",
// which has no value and thus no direct Rust equivalent. Per the
// translation rules, such macros are skipped.
#[no_mangle]
pub unsafe extern "C" fn RosSymCreateFromFile(FileContext: *mut core::ffi::c_void, RosSymInfo: *mut *mut ROSSYM_INFO) -> u8 {
    const IMAGE_SIZEOF_SHORT_NAME: usize = 8;
    let rossym_section_name = b".rossym\0".as_ptr() as *const i8;

    let mut dos_header: IMAGE_DOS_HEADER = core::mem::zeroed();
    let mut nt_headers: IMAGE_NT_HEADERS = core::mem::zeroed();
    let section_headers: *mut IMAGE_SECTION_HEADER;
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
