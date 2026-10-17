//! [correctness-review 2026-06-07 · sonnet; FIXED 2026-06-07 · opus] Reviewed vs sdk/lib/rossym/frommem.c.
//! Risk: MED → RESOLVED. FIXED: IMAGE_FIRST_SECTION + ROSSYM_IS_VALID_* are now local wrapper fns
//! (the macro expansions); RosSymCreateFromRaw is called directly as the sibling crate fn (correct
//! usize width); return type i32 → u8 (BOOLEAN).
//! Behavior: logic correct; two linkage bugs will surface at integration. [original finding]
//! Functions reviewed: RosSymCreateFromMem (1/1).
//! Findings:
//!   BUG LINKAGE — `extern "C" fn IMAGE_FIRST_SECTION(...)` (line 11). `IMAGE_FIRST_SECTION`
//!     is a C preprocessor macro in ntimage.h, not an exported function. No object symbol
//!     with this name exists. Link error at integration. Fix: compute inline as
//!     `(NtHeaders as *const u8).add(FIELD_OFFSET(OptionalHeader) + SizeOfOptionalHeader)`,
//!     matching the pattern fromfile.rs already uses correctly (lines 65-67 of fromfile.rs).
//!   BUG LINKAGE — `extern "C" fn ROSSYM_IS_VALID_DOS_HEADER` and `ROSSYM_IS_VALID_NT_HEADERS`
//!     (lines 9-10). Both are C preprocessor macros in rossympriv.h, not exported functions.
//!     Link error at integration. Fix: inline the checks (e_magic == IMAGE_DOS_SIGNATURE &&
//!     e_lfanew != 0; Signature == IMAGE_NT_SIGNATURE && OptionalHeader.Magic == arch magic).
//!   BUG RETURN TYPE — Return type `i32` (line 26). C declares `BOOLEAN` (unsigned char, 1 byte).
//!     On all supported ABIs the return register (AL/EAX) means the value is observable, but
//!     callers expecting BOOLEAN and receiving i32 see wrong type in the ABI contract.
//!     Fix: change to `-> u8` to match fromfile.rs (which correctly uses u8 for BOOLEAN).
//!   BUG LINKAGE (minor) — `extern "C" fn RosSymCreateFromRaw(..., Size: u32, ...)` (line 12).
//!     C declares `ULONG_PTR DataSize`. fromraw.rs exports `DataSize: usize`. On amd64/arm64 the
//!     zero-extended u32 matches the register-wide usize, so the passed value is correct in
//!     practice. On i386 both are 4 bytes, also fine. Benign in practice but wrong declaration.
//!   NOTE — `ROSSYM_SECTION_NAME = b".rossym\0"` (line 20) is CORRECT (contrast: fromfile.rs
//!     uses the wrong `b"rossym\0"` missing the dot). The strncpy + memcmp section-search
//!     logic faithfully reproduces the C behavior.
//!   NOTE — All bounds checks (ImageSize vs DOS/NT header sizes, section header end address,
//!     PointerToRawData+SizeOfRawData, VirtualAddress+VirtualSize) match frommem.c exactly.
//!   NOTE — `RosSymSectionFound` as i32 flag (vs C BOOLEAN) is an internal-only variable
//!     with no ABI impact; semantics are identical.
//! Unverified: IMAGE_NT_OPTIONAL_HDR_MAGIC value used by the inlined IS_VALID check
//!   (arch-conditional 32/64 bit magic) — the fix must use the correct arch-specific constant.

use crate::types::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::fromfile::memcmp;
use crate::fromfile::strncpy;
use ntkrnl_sys::ddk::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS, IMAGE_SECTION_HEADER};

extern "C" {
}

// PE signature constants (defined locally; see fromfile.rs).
const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
#[cfg(target_arch = "x86")]
const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x010B;
#[cfg(not(target_arch = "x86"))]
const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x020B;

// rossympriv.h / ntimage.h function-like macros → local wrappers (call sites unchanged).
#[inline]
unsafe fn ROSSYM_IS_VALID_DOS_HEADER(DosHeader: *const IMAGE_DOS_HEADER) -> i32 {
    ((*DosHeader).e_magic == IMAGE_DOS_SIGNATURE && (*DosHeader).e_lfanew != 0) as i32
}
#[inline]
unsafe fn ROSSYM_IS_VALID_NT_HEADERS(NtHeaders: *const IMAGE_NT_HEADERS) -> i32 {
    ((*NtHeaders).Signature == IMAGE_NT_SIGNATURE
        && (*NtHeaders).OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR_MAGIC) as i32
}
#[inline]
unsafe fn IMAGE_FIRST_SECTION(NtHeaders: *const IMAGE_NT_HEADERS) -> *mut IMAGE_SECTION_HEADER {
    ((NtHeaders as usize)
        + core::mem::offset_of!(IMAGE_NT_HEADERS, OptionalHeader)
        + (*NtHeaders).FileHeader.SizeOfOptionalHeader as usize) as *mut IMAGE_SECTION_HEADER
}

const IMAGE_SIZEOF_SHORT_NAME: usize = 8;
const ROSSYM_SECTION_NAME: &[u8; IMAGE_SIZEOF_SHORT_NAME] = b".rossym\0";
#[no_mangle]
pub unsafe extern "C" fn RosSymCreateFromMem(
    ImageStart: *mut core::ffi::c_void,
    ImageSize: usize,
    RosSymInfo: *mut PROSSYM_INFO,
) -> u8 {
    unsafe {
        let DosHeader: *mut IMAGE_DOS_HEADER;
        let NtHeaders: *mut IMAGE_NT_HEADERS;
        let mut SectionHeader: *mut IMAGE_SECTION_HEADER;
        let _SectionIndex: u32;
        let mut RosSymSectionFound: i32 = 0;
        let mut SectionName: [u8; IMAGE_SIZEOF_SHORT_NAME] = [0u8; IMAGE_SIZEOF_SHORT_NAME];

        DosHeader = ImageStart as *mut IMAGE_DOS_HEADER;
        if ImageSize < core::mem::size_of::<IMAGE_DOS_HEADER>()
            || ROSSYM_IS_VALID_DOS_HEADER(DosHeader) == 0 {
            // DPRINT removed
            return 0;
        }

        NtHeaders = (ImageStart as *mut u8).add((*DosHeader).e_lfanew as usize) as *mut IMAGE_NT_HEADERS;
        if ImageSize < ((*DosHeader).e_lfanew as usize + core::mem::size_of::<IMAGE_NT_HEADERS>())
            || ROSSYM_IS_VALID_NT_HEADERS(NtHeaders) == 0 {
            // DPRINT removed
            return 0;
        }

        SectionHeader = IMAGE_FIRST_SECTION(NtHeaders);
        let end_addr = (SectionHeader as *mut u8).add(
            ((*NtHeaders).FileHeader.NumberOfSections as usize) * core::mem::size_of::<IMAGE_SECTION_HEADER>()
        ) as usize;
        let start_addr = ImageStart as *mut u8 as usize;
        if ImageSize < end_addr - start_addr {
            // DPRINT removed
            return 0;
        }

        strncpy(
            (SectionName.as_mut_ptr() as *mut u8).cast(),
            (ROSSYM_SECTION_NAME.as_ptr() as *const u8).cast(),
            IMAGE_SIZEOF_SHORT_NAME as usize,
        );

        for _SectionIndex in 0..((*NtHeaders).FileHeader.NumberOfSections as u32) {
            if memcmp(
                (SectionName.as_ptr() as *const u8).cast(),
                ((*SectionHeader).Name.as_ptr() as *const u8).cast(),
                IMAGE_SIZEOF_SHORT_NAME as usize,
            ) == 0 {
                RosSymSectionFound = 1;
                break;
            }
            SectionHeader = SectionHeader.add(1);
        }

        if RosSymSectionFound == 0 {
            // DPRINT removed
            return 0;
        }

        if ImageSize < ((*SectionHeader).PointerToRawData as usize + (*SectionHeader).SizeOfRawData as usize)
            || ((*SectionHeader).SizeOfRawData as usize) < core::mem::size_of::<ROSSYM_HEADER>() {
            // DPRINT removed
            return 0;
        }

        if ((*SectionHeader).VirtualAddress as usize + (*SectionHeader).Misc.VirtualSize as usize) > ImageSize {
            // DPRINT removed
            return 0;
        }

        crate::fromraw::RosSymCreateFromRaw(
            (ImageStart as *mut u8).add((*SectionHeader).VirtualAddress as usize) as *mut core::ffi::c_void,
            (*SectionHeader).SizeOfRawData as usize,
            RosSymInfo,
        )
    }
}
