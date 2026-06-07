use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS, IMAGE_SECTION_HEADER, PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS, PIMAGE_SECTION_HEADER};
use core::ffi::c_void;

extern "C" {
    fn ROSSYM_IS_VALID_DOS_HEADER(DosHeader: *const IMAGE_DOS_HEADER) -> i32;
    fn ROSSYM_IS_VALID_NT_HEADERS(NtHeaders: *const IMAGE_NT_HEADERS) -> i32;
    fn IMAGE_FIRST_SECTION(NtHeaders: *const IMAGE_NT_HEADERS) -> *mut IMAGE_SECTION_HEADER;
    fn RosSymCreateFromRaw(RawData: *const u8, Size: u32, RosSymInfo: *mut PROSSYM_INFO) -> i32;
    fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
    fn strncpy(dest: *mut u8, src: *const u8, count: usize) -> *mut u8;
    fn DPRINT1(message: *const i8);
    fn DPRINT(format: *const i8, ...) -> i32;
}

const IMAGE_SIZEOF_SHORT_NAME: usize = 8;
const ROSSYM_SECTION_NAME: &[u8; IMAGE_SIZEOF_SHORT_NAME] = b".rossym\0";
#[no_mangle]
pub unsafe extern "C" fn RosSymCreateFromMem(
    ImageStart: *mut core::ffi::c_void,
    ImageSize: usize,
    RosSymInfo: *mut PROSSYM_INFO,
) -> i32 {
    unsafe {
        let DosHeader: *mut IMAGE_DOS_HEADER;
        let NtHeaders: *mut IMAGE_NT_HEADERS;
        let mut SectionHeader: *mut IMAGE_SECTION_HEADER;
        let mut SectionIndex: u32;
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
            SectionName.as_mut_ptr() as *mut u8,
            ROSSYM_SECTION_NAME.as_ptr() as *const u8,
            IMAGE_SIZEOF_SHORT_NAME as usize,
        );

        for SectionIndex in 0..((*NtHeaders).FileHeader.NumberOfSections as u32) {
            if memcmp(
                SectionName.as_ptr() as *const u8,
                (*SectionHeader).Name.as_ptr() as *const u8,
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

        RosSymCreateFromRaw(
            (ImageStart as *mut u8).add((*SectionHeader).VirtualAddress as usize) as *const u8,
            (*SectionHeader).SizeOfRawData,
            RosSymInfo,
        )
    }
}
