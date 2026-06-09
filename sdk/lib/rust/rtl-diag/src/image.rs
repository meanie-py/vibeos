use crate::shim::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{IMAGE_BASE_RELOCATION, IMAGE_DOS_HEADER, IMAGE_NT_HEADERS, PCSTR, PIMAGE_BASE_RELOCATION, PIMAGE_DATA_DIRECTORY, PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS, PIMAGE_OPTIONAL_HEADER32, PIMAGE_OPTIONAL_HEADER64, PIMAGE_SECTION_HEADER, PUINT64, PULONGLONG, RtlImageNtHeaderEx};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.
#[no_mangle]
pub unsafe extern "C" fn ChkSum(mut Sum: ULONG, Src: PUSHORT, Len: ULONG) -> USHORT {
    let mut i: ULONG = 0;
    while i < Len {
        Sum += *Src.add(i as usize) as ULONG;
        Sum = (Sum & 0xFFFF) + (Sum >> 16);
        i += 1;
    }
    ((Sum + (Sum >> 16)) & 0xFFFF) as USHORT
}

#[no_mangle]
pub unsafe extern "system" fn LdrVerifyMappedImageMatchesChecksum(
    BaseAddress: *mut core::ffi::c_void,
    ImageSize: usize,
    FileLength: u32,
) -> u8 {
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpImageNtHeaderEx(
    Flags: ULONG,
    Base: PVOID,
    Size: ULONG64,
    OutHeaders: *mut PIMAGE_NT_HEADERS,
) -> NTSTATUS {
    let NtHeaders: PIMAGE_NT_HEADERS;
    let DosHeader: PIMAGE_DOS_HEADER;
    let WantsRangeCheck: bool;
    let NtHeaderOffset: ULONG;

    if OutHeaders.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    core::ptr::write_volatile(OutHeaders, core::ptr::null_mut());

    if (Flags & !RTL_IMAGE_NT_HEADER_EX_FLAG_NO_RANGE_CHECK) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if Base.is_null() || Base == (-1isize as PVOID) {
        return STATUS_INVALID_PARAMETER;
    }

    WantsRangeCheck = (Flags & RTL_IMAGE_NT_HEADER_EX_FLAG_NO_RANGE_CHECK) == 0;
    if WantsRangeCheck {
        if Size < core::mem::size_of::<IMAGE_DOS_HEADER>() as ULONG64 {
            return STATUS_INVALID_IMAGE_FORMAT;
        }
    }

    DosHeader = Base as PIMAGE_DOS_HEADER;
    if (*DosHeader).e_magic != IMAGE_DOS_SIGNATURE as u16 {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    NtHeaderOffset = (*DosHeader).e_lfanew as ULONG;

    if NtHeaderOffset >= (256 * 1024 * 1024) {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    if WantsRangeCheck {
        if (NtHeaderOffset as ULONG64
            + RTL_SIZEOF_THROUGH_FIELD!(IMAGE_NT_HEADERS, FileHeader) as ULONG64)
            >= Size
        {
            return STATUS_INVALID_IMAGE_FORMAT;
        }
    }

    NtHeaders = ((Base as ULONG_PTR) + NtHeaderOffset as ULONG_PTR) as PIMAGE_NT_HEADERS;

    if Base <= MmHighestUserAddress {
        if (NtHeaders.add(1) as PVOID) > MmHighestUserAddress {
            return STATUS_INVALID_IMAGE_FORMAT;
        }
    }

    if (*NtHeaders).Signature != IMAGE_NT_SIGNATURE {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    core::ptr::write_volatile(OutHeaders, NtHeaders);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlImageNtHeader(Base: *mut core::ffi::c_void) -> PIMAGE_NT_HEADERS {
    let mut NtHeader: PIMAGE_NT_HEADERS = core::ptr::null_mut();

    RtlImageNtHeaderEx(
        0x00000001,
        Base,
        0,
        &mut NtHeader,
    );
    NtHeader
}

#[no_mangle]
pub unsafe extern "system" fn RtlImageDirectoryEntryToData(
    mut BaseAddress: *mut core::ffi::c_void,
    mut MappedAsImage: u8,
    Directory: u16,
    Size: *mut u32,
) -> *mut core::ffi::c_void {
    let NtHeader: PIMAGE_NT_HEADERS;
    let Va: u32;

    if (BaseAddress as usize) & 1 != 0 {
        BaseAddress = ((BaseAddress as usize) & !1usize) as *mut core::ffi::c_void;
        MappedAsImage = 0u8;
    }

    NtHeader = RtlImageNtHeader(BaseAddress);
    if NtHeader.is_null() {
        return core::ptr::null_mut();
    }

    if (*NtHeader).OptionalHeader.Magic == 0x20b {
        let OptionalHeader =
            core::ptr::addr_of_mut!((*NtHeader).OptionalHeader) as PIMAGE_OPTIONAL_HEADER64;

        if Directory as u32 >= (*OptionalHeader).NumberOfRvaAndSizes {
            return core::ptr::null_mut();
        }

        Va = (*OptionalHeader).DataDirectory[Directory as usize].VirtualAddress;
        if Va == 0 {
            return core::ptr::null_mut();
        }

        core::ptr::write_volatile(Size, (*OptionalHeader).DataDirectory[Directory as usize].Size);

        if MappedAsImage != 0 || Va < (*OptionalHeader).SizeOfHeaders {
            return ((BaseAddress as usize) + Va as usize) as *mut core::ffi::c_void;
        }
    } else {
        let OptionalHeader =
            core::ptr::addr_of_mut!((*NtHeader).OptionalHeader) as PIMAGE_OPTIONAL_HEADER32;

        if Directory as u32 >= (*OptionalHeader).NumberOfRvaAndSizes {
            return core::ptr::null_mut();
        }

        Va = (*OptionalHeader).DataDirectory[Directory as usize].VirtualAddress;
        if Va == 0 {
            return core::ptr::null_mut();
        }

        core::ptr::write_volatile(Size, (*OptionalHeader).DataDirectory[Directory as usize].Size);

        if MappedAsImage != 0 || Va < (*OptionalHeader).SizeOfHeaders {
            return ((BaseAddress as usize) + Va as usize) as *mut core::ffi::c_void;
        }
    }

    RtlImageRvaToVa(NtHeader, BaseAddress, Va, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn RtlImageRvaToSection(
    NtHeader: PIMAGE_NT_HEADERS,
    BaseAddress: *mut core::ffi::c_void,
    Rva: ULONG,
) -> PIMAGE_SECTION_HEADER {
    let mut Section: PIMAGE_SECTION_HEADER;
    let mut Va: ULONG;
    let mut Count: ULONG;

    Count = SWAPW((*NtHeader).FileHeader.NumberOfSections) as ULONG;
    Section = IMAGE_FIRST_SECTION(NtHeader);

    while Count != 0 {
        Count -= 1;
        Va = SWAPD((*Section).VirtualAddress);
        if (Va <= Rva) && (Rva < Va + SWAPD((*Section).SizeOfRawData)) {
            return Section;
        }
        Section = Section.add(1);
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlImageRvaToVa(
    NtHeader: PIMAGE_NT_HEADERS,
    BaseAddress: *mut core::ffi::c_void,
    Rva: ULONG,
    SectionHeader: *mut PIMAGE_SECTION_HEADER,
) -> *mut core::ffi::c_void {
    let mut Section: PIMAGE_SECTION_HEADER = core::ptr::null_mut();

    if !SectionHeader.is_null() {
        Section = *SectionHeader;
    }

    if Section.is_null()
        || Rva < SWAPD((*Section).VirtualAddress)
        || Rva >= SWAPD((*Section).VirtualAddress) + SWAPD((*Section).Misc.VirtualSize)
    {
        Section = RtlImageRvaToSection(NtHeader, BaseAddress, Rva);
        if Section.is_null() {
            return core::ptr::null_mut();
        }

        if !SectionHeader.is_null() {
            core::ptr::write_volatile(SectionHeader, Section);
        }
    }

    (BaseAddress as usize
        + Rva as usize
        + SWAPD((*Section).PointerToRawData) as usize
        - SWAPD((*Section).VirtualAddress) as usize) as *mut core::ffi::c_void
}

#[no_mangle]
pub unsafe extern "system" fn LdrProcessRelocationBlockLongLong(
    Address: ULONG_PTR,
    Count: ULONG,
    mut TypeOffset: PUSHORT,
    Delta: LONGLONG,
) -> PIMAGE_BASE_RELOCATION {
    let mut Offset: SHORT;
    let mut Type: USHORT;
    let mut i: ULONG;
    let ShortPtr: PUSHORT;
    let LongPtr: PULONG;
    let LongLongPtr: PULONGLONG;

    i = 0;
    while i < Count {
        Offset = (SWAPW(*TypeOffset) & 0xFFF) as SHORT;
        Type = SWAPW(*TypeOffset) >> 12;
        let ShortPtr = ((Address as ULONG_PTR).wrapping_add(Offset as i16 as isize as ULONG_PTR)) as PUSHORT;

        match Type as u32 {
            IMAGE_REL_BASED_ABSOLUTE => {}

            IMAGE_REL_BASED_HIGH => {
                *ShortPtr = HIWORD(
                    (MAKELONG(0, *ShortPtr) as i64 + (Delta & 0xFFFFFFFF)) as ULONG,
                ) as USHORT;
            }

            IMAGE_REL_BASED_LOW => {
                *ShortPtr =
                    (SWAPW(*ShortPtr) as u32 + LOWORD((Delta & 0xFFFF) as ULONG) as u32) as USHORT;
            }

            IMAGE_REL_BASED_HIGHLOW => {
                let LongPtr = ((Address as ULONG_PTR)
                    .wrapping_add(Offset as i16 as isize as ULONG_PTR))
                    as PULONG;
                *LongPtr = (SWAPD(*LongPtr) as i64 + (Delta & 0xFFFFFFFF)) as ULONG;
            }

            IMAGE_REL_BASED_DIR64 => {
                let LongLongPtr = ((Address as ULONG_PTR)
                    .wrapping_add(Offset as i16 as isize as ULONG_PTR))
                    as PUINT64;
                *LongLongPtr = (SWAPQ(*LongLongPtr) as i64 + Delta) as u64;
            }

            _ => {
                return core::ptr::null_mut();
            }
        }

        TypeOffset = TypeOffset.add(1);
        i += 1;
    }

    TypeOffset as PIMAGE_BASE_RELOCATION
}

#[no_mangle]
pub unsafe extern "system" fn LdrRelocateImage(
    BaseAddress: PVOID,
    LoaderName: PCSTR,
    Success: ULONG,
    Conflict: ULONG,
    Invalid: ULONG,
) -> ULONG {
    LdrRelocateImageWithBias(BaseAddress, 0, LoaderName, Success, Conflict, Invalid)
}

#[no_mangle]
pub unsafe extern "system" fn LdrRelocateImageWithBias(
    BaseAddress: PVOID,
    AdditionalBias: LONGLONG,
    LoaderName: PCSTR,
    Success: ULONG,
    Conflict: ULONG,
    Invalid: ULONG,
) -> ULONG {
    let NtHeaders: PIMAGE_NT_HEADERS;
    let RelocationDDir: PIMAGE_DATA_DIRECTORY;
    let mut RelocationDir: PIMAGE_BASE_RELOCATION;
    let RelocationEnd: PIMAGE_BASE_RELOCATION;
    let mut Count: ULONG;
    let mut Address: ULONG_PTR;
    let mut TypeOffset: PUSHORT;
    let Delta: LONGLONG;

    let _ = LoaderName;

    NtHeaders = RtlImageNtHeader(BaseAddress);

    if NtHeaders.is_null() {
        return Invalid;
    }

    if ((*NtHeaders).FileHeader.Characteristics as ULONG & IMAGE_FILE_RELOCS_STRIPPED as ULONG) != 0 {
        return Conflict;
    }

    RelocationDDir = core::ptr::addr_of_mut!(
        (*NtHeaders).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize]
    );

    if (*RelocationDDir).VirtualAddress == 0 || (*RelocationDDir).Size == 0 {
        return Success;
    }

    Delta = (BaseAddress as ULONG_PTR as LONGLONG)
        - (*NtHeaders).OptionalHeader.ImageBase as LONGLONG
        + AdditionalBias;
    RelocationDir = ((BaseAddress as ULONG_PTR) + (*RelocationDDir).VirtualAddress as ULONG_PTR)
        as PIMAGE_BASE_RELOCATION;
    RelocationEnd = ((RelocationDir as ULONG_PTR) + (*RelocationDDir).Size as ULONG_PTR)
        as PIMAGE_BASE_RELOCATION;

    while RelocationDir < RelocationEnd && (*RelocationDir).SizeOfBlock > 0 {
        Count = ((*RelocationDir).SizeOfBlock as ULONG
            - core::mem::size_of::<IMAGE_BASE_RELOCATION>() as ULONG)
            / core::mem::size_of::<USHORT>() as ULONG;
        Address = (BaseAddress as ULONG_PTR) + (*RelocationDir).VirtualAddress as ULONG_PTR;
        TypeOffset = RelocationDir.add(1) as PUSHORT;

        RelocationDir = LdrProcessRelocationBlockLongLong(Address, Count, TypeOffset, Delta);

        if RelocationDir.is_null() {
            return Invalid;
        }
    }

    Success
}
