use core::mem::{align_of, offset_of, size_of};
use core::ptr;

use ntkrnl_sys::ddk::{
    BOOLEAN, IMAGE_DOS_HEADER, IMAGE_FILE_HEADER, IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64,
    IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, IMAGE_SECTION_HEADER, LARGE_INTEGER,
    NTSTATUS, PMM_IMAGE_SECTION_OBJECT, PMM_SECTION_SEGMENT, PVOID, SIZE_T, ULONG, ULONGLONG,
    ULONG_PTR, USHORT,
};
use ntkrnl_sys::mm::{
    PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_NOACCESS, PAGE_READONLY,
};

type PEXEFMT_CB_READ_FILE = unsafe extern "system" fn(
    PVOID,
    *mut LARGE_INTEGER,
    ULONG,
    *mut PVOID,
    *mut PVOID,
    *mut ULONG,
) -> NTSTATUS;
type PEXEFMT_CB_ALLOCATE_SEGMENTS = unsafe extern "system" fn(ULONG) -> PMM_SECTION_SEGMENT;

extern "system" {
    fn ExFreePool(P: PVOID);
}

const TRUE: BOOLEAN = 1;
const FALSE: BOOLEAN = 0;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as NTSTATUS;
const STATUS_END_OF_FILE: NTSTATUS = 0xC0000011_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007B_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_PROTECT: NTSTATUS = 0xC0000130_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_WIN_64: NTSTATUS = 0xC000035A_u32 as NTSTATUS;
const STATUS_ROS_EXEFMT_UNKNOWN_FORMAT: NTSTATUS = 0xA0100001_u32 as NTSTATUS;

const EXEFMT_LOAD_ASSUME_SEGMENTS_SORTED: ULONG = 1 << 0;
const EXEFMT_LOAD_ASSUME_SEGMENTS_NO_OVERLAP: ULONG = 1 << 1;
const EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED: ULONG = 1 << 2;
const EXEFMT_LOAD_ASSUME_SEGMENTS_OK: ULONG = EXEFMT_LOAD_ASSUME_SEGMENTS_SORTED
    | EXEFMT_LOAD_ASSUME_SEGMENTS_NO_OVERLAP
    | EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED;

const EXEFMT_LOAD_HEADER_SIZE: usize = 0x2000;
const PAGE_SIZE_ULONG: ULONG = ntkrnl_sys::base::PAGE_SIZE as ULONG;
const PAGE_READWRITE: ULONG = ntkrnl_sys::base::PAGE_READWRITE as ULONG;

const IMAGE_DOS_SIGNATURE: USHORT = 0x5A4D;
const IMAGE_NT_SIGNATURE: ULONG = 0x00004550;
const IMAGE_NT_OPTIONAL_HDR32_MAGIC: USHORT = 0x10b;
const IMAGE_NT_OPTIONAL_HDR64_MAGIC: USHORT = 0x20b;
const IMAGE_DLLCHARACTERISTICS_NO_ISOLATION: USHORT = 0x0200;

const IMAGE_SCN_CNT_CODE: ULONG = 0x00000020;
const IMAGE_SCN_CNT_INITIALIZED_DATA: ULONG = 0x00000040;
const IMAGE_SCN_CNT_UNINITIALIZED_DATA: ULONG = 0x00000080;
const IMAGE_SCN_MEM_SHARED: ULONG = 0x10000000;
const IMAGE_SCN_MEM_EXECUTE: ULONG = 0x20000000;
const IMAGE_SCN_MEM_READ: ULONG = 0x40000000;
const IMAGE_SCN_MEM_WRITE: ULONG = 0x80000000;

const RTL_SIZEOF_THROUGH_IMAGE_NT_HEADERS32_FILE_HEADER: ULONG =
    (offset_of!(IMAGE_NT_HEADERS32, FileHeader) + size_of::<IMAGE_FILE_HEADER>()) as ULONG;
const FIELD_OFFSET_IMAGE_NT_HEADERS32_OPTIONAL_HEADER: ULONG =
    offset_of!(IMAGE_NT_HEADERS32, OptionalHeader) as ULONG;

const _: () = assert!(EXEFMT_LOAD_HEADER_SIZE >= size_of::<IMAGE_DOS_HEADER>());
const _: () = assert!(size_of::<IMAGE_NT_HEADERS32>() <= size_of::<IMAGE_NT_HEADERS64>());
const _: () = assert!(align_of::<IMAGE_NT_HEADERS32>() == align_of::<IMAGE_NT_HEADERS64>());
const _: () = assert!(
    (offset_of!(IMAGE_NT_HEADERS32, FileHeader) + size_of::<IMAGE_FILE_HEADER>())
        == (offset_of!(IMAGE_NT_HEADERS64, FileHeader) + size_of::<IMAGE_FILE_HEADER>())
);
const _: () = assert!(
    offset_of!(IMAGE_NT_HEADERS32, OptionalHeader)
        == offset_of!(IMAGE_NT_HEADERS64, OptionalHeader)
);
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    USHORT,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, Magic),
    offset_of!(IMAGE_OPTIONAL_HEADER64, Magic),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    ULONG,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, SectionAlignment),
    offset_of!(IMAGE_OPTIONAL_HEADER64, SectionAlignment),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    ULONG,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, FileAlignment),
    offset_of!(IMAGE_OPTIONAL_HEADER64, FileAlignment),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    USHORT,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, Subsystem),
    offset_of!(IMAGE_OPTIONAL_HEADER64, Subsystem),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    USHORT,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, MinorSubsystemVersion),
    offset_of!(IMAGE_OPTIONAL_HEADER64, MinorSubsystemVersion),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    USHORT,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, MajorSubsystemVersion),
    offset_of!(IMAGE_OPTIONAL_HEADER64, MajorSubsystemVersion),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    ULONG,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, AddressOfEntryPoint),
    offset_of!(IMAGE_OPTIONAL_HEADER64, AddressOfEntryPoint),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    ULONG,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfCode),
    offset_of!(IMAGE_OPTIONAL_HEADER64, SizeOfCode),
));
const _: () = assert!(PEFMT_FIELDS_EQUAL::<
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    ULONG,
>(
    offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfHeaders),
    offset_of!(IMAGE_OPTIONAL_HEADER64, SizeOfHeaders),
));

const fn PEFMT_FIELDS_EQUAL<T1, T2, F>(Offset1: usize, Offset2: usize) -> bool {
    let _ = core::mem::size_of::<T1>();
    let _ = core::mem::size_of::<T2>();
    Offset1 == Offset2 && core::mem::size_of::<F>() == core::mem::size_of::<F>()
}

static SectionCharacteristicsToProtect: [ULONG; 16] = [
    PAGE_NOACCESS,
    PAGE_NOACCESS,
    PAGE_EXECUTE,
    PAGE_EXECUTE,
    PAGE_READONLY,
    PAGE_READONLY,
    PAGE_EXECUTE_READ,
    PAGE_EXECUTE_READ,
    PAGE_READWRITE,
    PAGE_READWRITE,
    PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_READWRITE,
    PAGE_READWRITE,
    PAGE_READWRITE,
    PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_READWRITE,
];

macro_rules! read_field {
    ($ptr:expr, $($field:tt)+) => {{
        ptr::read_unaligned(ptr::addr_of!((*$ptr).$($field)+))
    }};
}

macro_rules! write_out_field {
    ($ptr:expr, $field:ident, $value:expr) => {{
        ptr::write_volatile(ptr::addr_of_mut!((*$ptr).$field), $value);
    }};
    ($ptr:expr, $field1:ident.$field2:ident, $value:expr) => {{
        ptr::write_volatile(ptr::addr_of_mut!((*$ptr).$field1.$field2), $value);
    }};
    ($ptr:expr, $field1:ident.$field2:ident.$field3:ident.$field4:ident, $value:expr) => {{
        ptr::write_volatile(
            ptr::addr_of_mut!((*$ptr).$field1.$field2.$field3.$field4),
            $value,
        );
    }};
}

#[inline]
fn RTL_CONTAINS_FIELD(Size: ULONG, FieldOffset: usize, FieldSize: usize) -> bool {
    FieldOffset + FieldSize <= Size as usize
}

#[inline]
fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
}

#[inline]
fn Intsafe_CanAddULongPtr(Addend1: ULONG_PTR, Addend2: ULONG_PTR) -> bool {
    Addend1 <= ULONG_PTR::MAX - Addend2
}

#[inline]
fn Intsafe_CanAddLong64(Addend1: i64, Addend2: i64) -> bool {
    match i64::MAX.checked_sub(Addend2) {
        Some(MaxAddend1) => Addend1 <= MaxAddend1,
        None => false,
    }
}

#[inline]
fn Intsafe_CanAddULong32(Addend1: ULONG, Addend2: ULONG) -> bool {
    Addend1 <= ULONG::MAX - Addend2
}

#[inline]
unsafe fn Intsafe_AddULong32(Result: *mut ULONG, Addend1: ULONG, Addend2: ULONG) -> bool {
    if !Intsafe_CanAddULong32(Addend1, Addend2) {
        return false;
    }

    *Result = Addend1 + Addend2;
    true
}

#[inline]
fn Intsafe_CanMulULong32(Factor1: ULONG, Factor2: ULONG) -> bool {
    if Factor2 == 0 {
        return true;
    }

    Factor1 <= ULONG::MAX / Factor2
}

#[inline]
fn Intsafe_CanOffsetPointer(Pointer: *const core::ffi::c_void, Offset: SIZE_T) -> bool {
    Intsafe_CanAddULongPtr(Pointer as ULONG_PTR, Offset as ULONG_PTR)
}

#[inline]
fn IsPowerOf2(Number: ULONG) -> bool {
    if Number == 0 {
        return false;
    }

    (Number & (Number - 1)) == 0
}

#[inline]
fn ModPow2(Address: ULONG, Alignment: ULONG) -> ULONG {
    ASSERT!(IsPowerOf2(Alignment));
    Address & (Alignment - 1)
}

#[inline]
fn IsAligned(Address: ULONG, Alignment: ULONG) -> bool {
    ModPow2(Address, Alignment) == 0
}

#[inline]
unsafe fn AlignUp(AlignedAddress: *mut ULONG, Address: ULONG, Alignment: ULONG) -> bool {
    let nExcess = ModPow2(Address, Alignment);

    if nExcess == 0 {
        *AlignedAddress = Address;
        nExcess == 0
    } else {
        Intsafe_AddULong32(AlignedAddress, Address, Alignment - nExcess)
    }
}

#[inline]
fn ALIGN_UP_BY_ULONG_PTR(Value: ULONG_PTR, Alignment: ULONG) -> ULONG_PTR {
    let Alignment = Alignment as ULONG_PTR;
    Value.wrapping_add(Alignment - 1) & !(Alignment - 1)
}

#[inline]
fn ALIGN_UP_BY_ULONG(Value: ULONG, Alignment: ULONG) -> ULONG {
    Value.wrapping_add(Alignment - 1) & !(Alignment - 1)
}

/*
 References:
  [1] Microsoft Corporation, "Microsoft Portable Executable and Common Object
      File Format Specification", revision 6.0 (February 1999)
*/
#[no_mangle]
pub unsafe extern "system" fn PeFmtCreateSection(
    FileHeader: *const core::ffi::c_void,
    FileHeaderSize: SIZE_T,
    File: PVOID,
    ImageSectionObject: PMM_IMAGE_SECTION_OBJECT,
    Flags: *mut ULONG,
    ReadFileCb: PEXEFMT_CB_READ_FILE,
    AllocateSegmentsCb: PEXEFMT_CB_ALLOCATE_SEGMENTS,
) -> NTSTATUS {
    let mut nStatus: NTSTATUS;
    let mut cbFileHeaderOffsetSize: ULONG = 0;
    let mut cbSectionHeadersOffset: ULONG = 0;
    let mut cbSectionHeadersSize: ULONG;
    let mut cbSectionHeadersOffsetSize: ULONG = 0;
    let mut cbOptHeaderSize: ULONG;
    let mut cbHeadersSize: ULONG = 0;
    let mut nSectionAlignment: ULONG;
    let mut nFileAlignment: ULONG;
    let mut ImageBase: ULONG_PTR = 0;
    let pidhDosHeader: *const IMAGE_DOS_HEADER;
    let mut pinhNtHeader: *const IMAGE_NT_HEADERS32;
    let piohOptHeader: *const IMAGE_OPTIONAL_HEADER32;
    let mut pishSectionHeaders: *const IMAGE_SECTION_HEADER;
    let mut pssSegments: PMM_SECTION_SEGMENT;
    let mut lnOffset = LARGE_INTEGER { QuadPart: 0 };
    let mut pBuffer: PVOID;
    let mut nPrevVirtualEndOfSegment: ULONG_PTR = 0;
    let mut nFileSizeOfHeaders: ULONG = 0;
    let mut i: ULONG;
    let mut AlignedLength: ULONG;

    ASSERT!(!FileHeader.is_null());
    ASSERT!(FileHeaderSize > 0);
    ASSERT!(!File.is_null());
    ASSERT!(!ImageSectionObject.is_null());
    ASSERT!(!Flags.is_null());
    ASSERT!(Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize));
    ASSERT!((FileHeader as ULONG_PTR % align_of::<IMAGE_DOS_HEADER>() as ULONG_PTR) == 0);

    pBuffer = ptr::null_mut();
    pidhDosHeader = FileHeader as *const IMAGE_DOS_HEADER;

    'l_Return: loop {
        /* DOS HEADER */
        nStatus = STATUS_ROS_EXEFMT_UNKNOWN_FORMAT;

        /* image too small to be an MZ executable */
        if (FileHeaderSize as ULONG_PTR) < size_of::<IMAGE_DOS_HEADER>() as ULONG_PTR {
            // DPRINT!("Too small to be an MZ executable, size is %lu\n", FileHeaderSize)
            break 'l_Return;
        }

        let e_magic = read_field!(pidhDosHeader, e_magic);
        let e_lfanew = read_field!(pidhDosHeader, e_lfanew);

        /* no MZ signature */
        if e_magic != IMAGE_DOS_SIGNATURE {
            // DPRINT!("No MZ signature found, e_magic is %hX\n", e_magic)
            break 'l_Return;
        }

        /* NT HEADER */
        nStatus = STATUS_INVALID_IMAGE_PROTECT;

        /* not a Windows executable */
        if e_lfanew <= 0 {
            // DPRINT!("Not a Windows executable, e_lfanew is %d\n", e_lfanew)
            break 'l_Return;
        }

        if !Intsafe_AddULong32(
            &mut cbFileHeaderOffsetSize,
            e_lfanew as ULONG,
            RTL_SIZEOF_THROUGH_IMAGE_NT_HEADERS32_FILE_HEADER,
        ) {
            // DPRINT!("The DOS stub is too large, e_lfanew is %X\n", e_lfanew)
            break 'l_Return;
        }

        if (FileHeaderSize as ULONG_PTR) < cbFileHeaderOffsetSize as ULONG_PTR {
            pinhNtHeader = ptr::null();
        } else {
            /*
             * we already know that Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize),
             * and FileHeaderSize >= cbFileHeaderOffsetSize, so this holds true too
             */
            ASSERT!(Intsafe_CanOffsetPointer(FileHeader, e_lfanew as SIZE_T));
            pinhNtHeader =
                (FileHeader as *const u8).add(e_lfanew as usize) as *const IMAGE_NT_HEADERS32;
        }

        /*
         * the buffer doesn't contain the NT file header, or the alignment is wrong: we
         * need to read the header from the file
         */
        let mut ReadHeaderFromFile = (FileHeaderSize as ULONG_PTR)
            < cbFileHeaderOffsetSize as ULONG_PTR
            || (pinhNtHeader as ULONG_PTR % align_of::<IMAGE_NT_HEADERS32>() as ULONG_PTR) != 0;

        if !ReadHeaderFromFile {
            let mut cbOptHeaderOffsetSize: ULONG = 0;

            nStatus = STATUS_INVALID_IMAGE_PROTECT;

            /* don't trust an invalid NT header */
            if read_field!(pinhNtHeader, Signature) != IMAGE_NT_SIGNATURE {
                // DPRINT!("The file isn't a PE executable, Signature is %X\n", pinhNtHeader->Signature)
                break 'l_Return;
            }

            if !Intsafe_AddULong32(
                &mut cbOptHeaderOffsetSize,
                e_lfanew as ULONG,
                FIELD_OFFSET_IMAGE_NT_HEADERS32_OPTIONAL_HEADER,
            ) {
                // DPRINT!("The DOS stub is too large, e_lfanew is %X\n", e_lfanew)
                break 'l_Return;
            }

            nStatus = STATUS_INVALID_IMAGE_FORMAT;

            if !Intsafe_AddULong32(
                &mut cbOptHeaderOffsetSize,
                cbOptHeaderOffsetSize,
                read_field!(pinhNtHeader, FileHeader.SizeOfOptionalHeader) as ULONG,
            ) {
                // DPRINT!("The NT header is too large, SizeOfOptionalHeader is %X\n", pinhNtHeader->FileHeader.SizeOfOptionalHeader)
                break 'l_Return;
            }

            /* the buffer doesn't contain the whole NT header: read it from the file */
            if cbOptHeaderOffsetSize as ULONG_PTR > FileHeaderSize as ULONG_PTR {
                ReadHeaderFromFile = true;
            }
        }

        if ReadHeaderFromFile {
            let mut cbNtHeaderSize: ULONG = 0;
            let mut cbReadSize: ULONG = 0;
            let mut pData: PVOID = ptr::null_mut();

            lnOffset.QuadPart = e_lfanew as i64;

            /* read the header from the file */
            nStatus = ReadFileCb(
                File,
                &mut lnOffset,
                size_of::<IMAGE_NT_HEADERS64>() as ULONG,
                &mut pData,
                &mut pBuffer,
                &mut cbReadSize,
            );

            if !NT_SUCCESS(nStatus) {
                let ReturnedStatus = nStatus;

                /* If it attempted to read past the end of the file, it means e_lfanew is invalid */
                if ReturnedStatus == STATUS_END_OF_FILE {
                    nStatus = STATUS_INVALID_IMAGE_PROTECT;
                }

                // DPRINT!("ReadFile failed, status %08X\n", ReturnedStatus)
                break 'l_Return;
            }

            ASSERT!(!pData.is_null());
            ASSERT!(!pBuffer.is_null());
            ASSERT!(cbReadSize > 0);

            nStatus = STATUS_INVALID_IMAGE_FORMAT;

            /* the buffer doesn't contain the file header */
            if cbReadSize < RTL_SIZEOF_THROUGH_IMAGE_NT_HEADERS32_FILE_HEADER {
                // DPRINT!("The file doesn't contain the PE file header\n")
                break 'l_Return;
            }

            pinhNtHeader = pData as *const IMAGE_NT_HEADERS32;

            /* object still not aligned: copy it to the beginning of the buffer */
            if (pinhNtHeader as ULONG_PTR % align_of::<IMAGE_NT_HEADERS32>() as ULONG_PTR) != 0 {
                ASSERT!(
                    (pBuffer as ULONG_PTR % align_of::<IMAGE_NT_HEADERS32>() as ULONG_PTR) == 0
                );
                ptr::copy(pData as *const u8, pBuffer as *mut u8, cbReadSize as usize);
                pinhNtHeader = pBuffer as *const IMAGE_NT_HEADERS32;
            }

            /* invalid NT header */
            nStatus = STATUS_INVALID_IMAGE_PROTECT;

            if read_field!(pinhNtHeader, Signature) != IMAGE_NT_SIGNATURE {
                // DPRINT!("The file isn't a PE executable, Signature is %X\n", pinhNtHeader->Signature)
                break 'l_Return;
            }

            nStatus = STATUS_INVALID_IMAGE_FORMAT;

            if !Intsafe_AddULong32(
                &mut cbNtHeaderSize,
                read_field!(pinhNtHeader, FileHeader.SizeOfOptionalHeader) as ULONG,
                FIELD_OFFSET_IMAGE_NT_HEADERS32_OPTIONAL_HEADER,
            ) {
                // DPRINT!("The full NT header is too large\n")
                break 'l_Return;
            }

            /* the buffer doesn't contain the whole NT header */
            if cbReadSize < cbNtHeaderSize {
                // DPRINT!("The file doesn't contain the full NT header\n")
                break 'l_Return;
            }
        }

        /* read information from the NT header */
        piohOptHeader = (pinhNtHeader as *const u8)
            .add(FIELD_OFFSET_IMAGE_NT_HEADERS32_OPTIONAL_HEADER as usize)
            as *const IMAGE_OPTIONAL_HEADER32;
        cbOptHeaderSize = read_field!(pinhNtHeader, FileHeader.SizeOfOptionalHeader) as ULONG;

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        if !RTL_CONTAINS_FIELD(
            cbOptHeaderSize,
            offset_of!(IMAGE_OPTIONAL_HEADER32, Magic),
            size_of::<USHORT>(),
        ) {
            // DPRINT!("The optional header doesn't contain the Magic field, SizeOfOptionalHeader is %X\n", cbOptHeaderSize)
            break 'l_Return;
        }

        /* ASSUME: RtlZeroMemory(ImageSectionObject, sizeof(*ImageSectionObject)); */

        let OptionalHeaderMagic = read_field!(piohOptHeader, Magic);
        match OptionalHeaderMagic {
            IMAGE_NT_OPTIONAL_HDR64_MAGIC => {
                #[cfg(target_pointer_width = "32")]
                {
                    nStatus = STATUS_INVALID_IMAGE_WIN_64;
                    // DPRINT!("Win64 optional header, unsupported\n")
                    break 'l_Return;
                }
            }
            IMAGE_NT_OPTIONAL_HDR32_MAGIC => {}
            _ => {
                // DPRINT!("Unrecognized optional header, Magic is %X\n", piohOptHeader->Magic)
                break 'l_Return;
            }
        }

        if RTL_CONTAINS_FIELD(
            cbOptHeaderSize,
            offset_of!(IMAGE_OPTIONAL_HEADER32, SectionAlignment),
            size_of::<ULONG>(),
        ) && RTL_CONTAINS_FIELD(
            cbOptHeaderSize,
            offset_of!(IMAGE_OPTIONAL_HEADER32, FileAlignment),
            size_of::<ULONG>(),
        ) {
            let SectionAlignment = read_field!(piohOptHeader, SectionAlignment);
            let FileAlignment = read_field!(piohOptHeader, FileAlignment);

            /* See [1], section 3.4.2 */
            if SectionAlignment < PAGE_SIZE_ULONG {
                if FileAlignment != SectionAlignment {
                    // DPRINT!("Sections aren't page-aligned and the file alignment isn't the same\n")
                    break 'l_Return;
                }
            } else if SectionAlignment < FileAlignment {
                // DPRINT!("The section alignment is smaller than the file alignment\n")
                break 'l_Return;
            }

            nSectionAlignment = SectionAlignment;
            nFileAlignment = FileAlignment;

            if !IsPowerOf2(nSectionAlignment) || !IsPowerOf2(nFileAlignment) {
                // DPRINT!("The section alignment (%u) and file alignment (%u) aren't both powers of 2\n", nSectionAlignment, nFileAlignment)
                break 'l_Return;
            }
        } else {
            nSectionAlignment = PAGE_SIZE_ULONG;
            nFileAlignment = PAGE_SIZE_ULONG;
        }

        ASSERT!(IsPowerOf2(nSectionAlignment));
        ASSERT!(IsPowerOf2(nFileAlignment));

        match OptionalHeaderMagic {
            /* PE32 */
            IMAGE_NT_OPTIONAL_HDR32_MAGIC => {
                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, ImageBase),
                    size_of::<ULONG>(),
                ) {
                    ImageBase = read_field!(piohOptHeader, ImageBase) as ULONG_PTR;
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfImage),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.ImageFileSize,
                        read_field!(piohOptHeader, SizeOfImage)
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfStackReserve),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.MaximumStackSize,
                        read_field!(piohOptHeader, SizeOfStackReserve) as SIZE_T
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfStackCommit),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.CommittedStackSize,
                        read_field!(piohOptHeader, SizeOfStackCommit) as SIZE_T
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, Subsystem),
                    size_of::<USHORT>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.SubSystemType,
                        read_field!(piohOptHeader, Subsystem) as ULONG
                    );

                    if RTL_CONTAINS_FIELD(
                        cbOptHeaderSize,
                        offset_of!(IMAGE_OPTIONAL_HEADER32, MinorSubsystemVersion),
                        size_of::<USHORT>(),
                    ) && RTL_CONTAINS_FIELD(
                        cbOptHeaderSize,
                        offset_of!(IMAGE_OPTIONAL_HEADER32, MajorSubsystemVersion),
                        size_of::<USHORT>(),
                    ) {
                        write_out_field!(
                            ImageSectionObject,
                            ImageInformation
                                .__bindgen_anon_1
                                .__bindgen_anon_1
                                .SubSystemMinorVersion,
                            read_field!(piohOptHeader, MinorSubsystemVersion)
                        );
                        write_out_field!(
                            ImageSectionObject,
                            ImageInformation
                                .__bindgen_anon_1
                                .__bindgen_anon_1
                                .SubSystemMajorVersion,
                            read_field!(piohOptHeader, MajorSubsystemVersion)
                        );
                    }
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, AddressOfEntryPoint),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.TransferAddress,
                        ImageBase.wrapping_add(
                            read_field!(piohOptHeader, AddressOfEntryPoint) as ULONG_PTR
                        ) as PVOID
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfCode),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.ImageContainsCode,
                        (read_field!(piohOptHeader, SizeOfCode) != 0) as BOOLEAN
                    );
                } else {
                    write_out_field!(ImageSectionObject, ImageInformation.ImageContainsCode, TRUE);
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, AddressOfEntryPoint),
                    size_of::<ULONG>(),
                ) {
                    if read_field!(piohOptHeader, AddressOfEntryPoint) == 0 {
                        write_out_field!(
                            ImageSectionObject,
                            ImageInformation.ImageContainsCode,
                            FALSE
                        );
                    }
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, LoaderFlags),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.LoaderFlags,
                        read_field!(piohOptHeader, LoaderFlags)
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER32, DllCharacteristics),
                    size_of::<USHORT>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.DllCharacteristics,
                        read_field!(piohOptHeader, DllCharacteristics)
                    );

                    /*
                     * Since we don't really implement SxS yet and LD doesn't supoprt /ALLOWISOLATION:NO, hard-code
                     * this flag here, which will prevent the loader and other code from doing any .manifest or SxS
                     * magic to any binary.
                     *
                     * This will break applications that depend on SxS when running with real Windows Kernel32/SxS/etc
                     * but honestly that's not tested. It will also break them when running no ReactOS once we implement
                     * the SxS support -- at which point, duh, this should be removed.
                     *
                     * But right now, any app depending on SxS is already broken anyway, so this flag only helps.
                     */
                    let DllCharacteristics = ptr::read_volatile(ptr::addr_of!(
                        (*ImageSectionObject).ImageInformation.DllCharacteristics
                    )) | IMAGE_DLLCHARACTERISTICS_NO_ISOLATION;
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.DllCharacteristics,
                        DllCharacteristics
                    );
                }
            }
            /* PE64 */
            #[cfg(target_pointer_width = "64")]
            IMAGE_NT_OPTIONAL_HDR64_MAGIC => {
                let pioh64OptHeader = piohOptHeader as *const IMAGE_OPTIONAL_HEADER64;

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, ImageBase),
                    size_of::<ULONGLONG>(),
                ) {
                    let ImageBase64 = read_field!(pioh64OptHeader, ImageBase);
                    ImageBase = ImageBase64 as ULONG_PTR;
                    if ImageBase64 > ULONG_PTR::MAX as ULONGLONG {
                        // DPRINT!("ImageBase exceeds the address space\n")
                        break 'l_Return;
                    }
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, SizeOfImage),
                    size_of::<ULONG>(),
                ) {
                    let SizeOfImage = read_field!(pioh64OptHeader, SizeOfImage);
                    if SizeOfImage as ULONGLONG > ULONG_PTR::MAX as ULONGLONG {
                        // DPRINT!("SizeOfImage exceeds the address space\n")
                        break 'l_Return;
                    }

                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.ImageFileSize,
                        SizeOfImage
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, SizeOfStackReserve),
                    size_of::<ULONGLONG>(),
                ) {
                    let SizeOfStackReserve = read_field!(pioh64OptHeader, SizeOfStackReserve);
                    if SizeOfStackReserve > ULONG_PTR::MAX as ULONGLONG {
                        // DPRINT!("SizeOfStackReserve exceeds the address space\n")
                        break 'l_Return;
                    }

                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.MaximumStackSize,
                        SizeOfStackReserve as SIZE_T
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, SizeOfStackCommit),
                    size_of::<ULONGLONG>(),
                ) {
                    let SizeOfStackCommit = read_field!(pioh64OptHeader, SizeOfStackCommit);
                    if SizeOfStackCommit > ULONG_PTR::MAX as ULONGLONG {
                        // DPRINT!("SizeOfStackCommit exceeds the address space\n")
                        break 'l_Return;
                    }

                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.CommittedStackSize,
                        SizeOfStackCommit as SIZE_T
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, Subsystem),
                    size_of::<USHORT>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.SubSystemType,
                        read_field!(pioh64OptHeader, Subsystem) as ULONG
                    );

                    if RTL_CONTAINS_FIELD(
                        cbOptHeaderSize,
                        offset_of!(IMAGE_OPTIONAL_HEADER64, MinorSubsystemVersion),
                        size_of::<USHORT>(),
                    ) && RTL_CONTAINS_FIELD(
                        cbOptHeaderSize,
                        offset_of!(IMAGE_OPTIONAL_HEADER64, MajorSubsystemVersion),
                        size_of::<USHORT>(),
                    ) {
                        write_out_field!(
                            ImageSectionObject,
                            ImageInformation
                                .__bindgen_anon_1
                                .__bindgen_anon_1
                                .SubSystemMinorVersion,
                            read_field!(pioh64OptHeader, MinorSubsystemVersion)
                        );
                        write_out_field!(
                            ImageSectionObject,
                            ImageInformation
                                .__bindgen_anon_1
                                .__bindgen_anon_1
                                .SubSystemMajorVersion,
                            read_field!(pioh64OptHeader, MajorSubsystemVersion)
                        );
                    }
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, AddressOfEntryPoint),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.TransferAddress,
                        ImageBase.wrapping_add(
                            read_field!(pioh64OptHeader, AddressOfEntryPoint) as ULONG_PTR
                        ) as PVOID
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, SizeOfCode),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.ImageContainsCode,
                        (read_field!(pioh64OptHeader, SizeOfCode) != 0) as BOOLEAN
                    );
                } else {
                    write_out_field!(ImageSectionObject, ImageInformation.ImageContainsCode, TRUE);
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, AddressOfEntryPoint),
                    size_of::<ULONG>(),
                ) {
                    if read_field!(pioh64OptHeader, AddressOfEntryPoint) == 0 {
                        write_out_field!(
                            ImageSectionObject,
                            ImageInformation.ImageContainsCode,
                            FALSE
                        );
                    }
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, LoaderFlags),
                    size_of::<ULONG>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.LoaderFlags,
                        read_field!(pioh64OptHeader, LoaderFlags)
                    );
                }

                if RTL_CONTAINS_FIELD(
                    cbOptHeaderSize,
                    offset_of!(IMAGE_OPTIONAL_HEADER64, DllCharacteristics),
                    size_of::<USHORT>(),
                ) {
                    write_out_field!(
                        ImageSectionObject,
                        ImageInformation.DllCharacteristics,
                        read_field!(pioh64OptHeader, DllCharacteristics)
                    );
                }
            }
            _ => {}
        }

        /* [1], section 3.4.2 */
        if (ImageBase % 0x10000) != 0 {
            // DPRINT!("ImageBase is not aligned on a 64KB boundary")
            break 'l_Return;
        }

        write_out_field!(
            ImageSectionObject,
            ImageInformation.ImageCharacteristics,
            read_field!(pinhNtHeader, FileHeader.Characteristics)
        );
        write_out_field!(
            ImageSectionObject,
            ImageInformation.Machine,
            read_field!(pinhNtHeader, FileHeader.Machine)
        );
        write_out_field!(ImageSectionObject, ImageInformation.GpValue, 0);
        write_out_field!(ImageSectionObject, ImageInformation.ZeroBits, 0);
        write_out_field!(ImageSectionObject, BasedAddress, ImageBase as PVOID);

        /* SECTION HEADERS */
        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        let NumberOfSections = read_field!(pinhNtHeader, FileHeader.NumberOfSections) as ULONG;

        /* see [1], section 3.3 */
        if NumberOfSections > 96 {
            // DPRINT!("Too many sections, NumberOfSections is %u\n", pinhNtHeader->FileHeader.NumberOfSections)
            break 'l_Return;
        }

        /*
         * the additional segment is for the file's headers. They need to be present for
         * the benefit of the dynamic loader (to locate exports, defaults for thread
         * parameters, resources, etc.)
         */
        let NrSegments = NumberOfSections + 1;
        write_out_field!(ImageSectionObject, NrSegments, NrSegments);

        /* file offset for the section headers */
        if !Intsafe_AddULong32(
            &mut cbSectionHeadersOffset,
            e_lfanew as ULONG,
            FIELD_OFFSET_IMAGE_NT_HEADERS32_OPTIONAL_HEADER,
        ) {
            // DPRINT!("Offset overflow\n")
            break 'l_Return;
        }

        if !Intsafe_AddULong32(
            &mut cbSectionHeadersOffset,
            cbSectionHeadersOffset,
            read_field!(pinhNtHeader, FileHeader.SizeOfOptionalHeader) as ULONG,
        ) {
            // DPRINT!("Offset overflow\n")
            break 'l_Return;
        }

        /* size of the section headers */
        ASSERT!(Intsafe_CanMulULong32(
            NumberOfSections,
            size_of::<IMAGE_SECTION_HEADER>() as ULONG
        ));
        cbSectionHeadersSize = NumberOfSections * size_of::<IMAGE_SECTION_HEADER>() as ULONG;

        if !Intsafe_AddULong32(
            &mut cbSectionHeadersOffsetSize,
            cbSectionHeadersOffset,
            cbSectionHeadersSize,
        ) {
            // DPRINT!("Section headers too large\n")
            break 'l_Return;
        }

        /* size of the executable's headers */
        if RTL_CONTAINS_FIELD(
            cbOptHeaderSize,
            offset_of!(IMAGE_OPTIONAL_HEADER32, SizeOfHeaders),
            size_of::<ULONG>(),
        ) {
            // if(!IsAligned(piohOptHeader->SizeOfHeaders, nFileAlignment))
            //     DIE(("SizeOfHeaders is not aligned\n"));

            let SizeOfHeaders = read_field!(piohOptHeader, SizeOfHeaders);
            if cbSectionHeadersSize > SizeOfHeaders {
                // DPRINT!("The section headers overflow SizeOfHeaders\n")
                break 'l_Return;
            }

            cbHeadersSize = SizeOfHeaders;
        } else if !AlignUp(
            &mut cbHeadersSize,
            cbSectionHeadersOffsetSize,
            nFileAlignment,
        ) {
            // DPRINT!("Overflow aligning the size of headers\n")
            break 'l_Return;
        }

        if !pBuffer.is_null() {
            ExFreePool(pBuffer);
            pBuffer = ptr::null_mut();
        }
        /* WARNING: pinhNtHeader IS NO LONGER USABLE */
        /* WARNING: piohOptHeader IS NO LONGER USABLE */
        /* WARNING: pioh64OptHeader IS NO LONGER USABLE */

        if (FileHeaderSize as ULONG_PTR) < cbSectionHeadersOffsetSize as ULONG_PTR {
            pishSectionHeaders = ptr::null();
        } else {
            /*
             * we already know that Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize),
             * and FileHeaderSize >= cbSectionHeadersOffsetSize, so this holds true too
             */
            ASSERT!(Intsafe_CanOffsetPointer(
                FileHeader,
                cbSectionHeadersOffset as SIZE_T
            ));
            pishSectionHeaders = (FileHeader as *const u8).add(cbSectionHeadersOffset as usize)
                as *const IMAGE_SECTION_HEADER;
        }

        /*
         * the buffer doesn't contain the section headers, or the alignment is wrong:
         * read the headers from the file
         */
        if (FileHeaderSize as ULONG_PTR) < cbSectionHeadersOffsetSize as ULONG_PTR
            || (pishSectionHeaders as ULONG_PTR % align_of::<IMAGE_SECTION_HEADER>() as ULONG_PTR)
                != 0
        {
            let mut pData: PVOID = ptr::null_mut();
            let mut cbReadSize: ULONG = 0;

            lnOffset.QuadPart = cbSectionHeadersOffset as i64;

            /* read the header from the file */
            nStatus = ReadFileCb(
                File,
                &mut lnOffset,
                cbSectionHeadersSize,
                &mut pData,
                &mut pBuffer,
                &mut cbReadSize,
            );

            if !NT_SUCCESS(nStatus) {
                // DPRINT!("ReadFile failed with status %08X\n", nStatus)
                break 'l_Return;
            }

            ASSERT!(!pData.is_null());
            ASSERT!(!pBuffer.is_null());
            ASSERT!(cbReadSize > 0);

            nStatus = STATUS_INVALID_IMAGE_FORMAT;

            /* the buffer doesn't contain all the section headers */
            if cbReadSize < cbSectionHeadersSize {
                // DPRINT!("The file doesn't contain all of the section headers\n")
                break 'l_Return;
            }

            pishSectionHeaders = pData as *const IMAGE_SECTION_HEADER;

            /* object still not aligned: copy it to the beginning of the buffer */
            if (pishSectionHeaders as ULONG_PTR % align_of::<IMAGE_SECTION_HEADER>() as ULONG_PTR)
                != 0
            {
                ASSERT!(
                    (pBuffer as ULONG_PTR % align_of::<IMAGE_SECTION_HEADER>() as ULONG_PTR) == 0
                );
                ptr::copy(pData as *const u8, pBuffer as *mut u8, cbReadSize as usize);
                pishSectionHeaders = pBuffer as *const IMAGE_SECTION_HEADER;
            }
        }

        /* SEGMENTS */
        /* allocate the segments */
        nStatus = STATUS_INSUFFICIENT_RESOURCES;
        let Segments = AllocateSegmentsCb(NrSegments);
        write_out_field!(ImageSectionObject, Segments, Segments);

        if Segments.is_null() {
            // DPRINT!("AllocateSegments failed\n")
            break 'l_Return;
        }

        /* initialize the headers segment */
        pssSegments = Segments;

        // ASSERT(IsAligned(cbHeadersSize, nFileAlignment));

        if !AlignUp(&mut nFileSizeOfHeaders, cbHeadersSize, nFileAlignment) {
            // DPRINT!("Cannot align the size of the section headers\n")
            break 'l_Return;
        }

        nPrevVirtualEndOfSegment =
            ALIGN_UP_BY_ULONG_PTR(cbHeadersSize as ULONG_PTR, nSectionAlignment);
        if nPrevVirtualEndOfSegment < cbHeadersSize as ULONG_PTR {
            // DPRINT!("Cannot align the size of the section headers\n")
            break 'l_Return;
        }

        (*pssSegments).Image.FileOffset = 0;
        (*pssSegments).Protection = PAGE_READONLY;
        (*pssSegments).Length.QuadPart = nPrevVirtualEndOfSegment as i64;
        (*pssSegments).RawLength.QuadPart = nFileSizeOfHeaders as i64;
        (*pssSegments).Image.VirtualAddress = 0;
        (*pssSegments).Image.Characteristics = 0;
        (*pssSegments).WriteCopy = TRUE;

        /* skip the headers segment */
        pssSegments = pssSegments.add(1);

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        ASSERT!((*ImageSectionObject).RefCount > 0);

        /* convert the executable sections into segments. See also [1], section 4 */
        i = 0;
        while i < NrSegments - 1 {
            let pishSectionHeader = pishSectionHeaders.add(i as usize);
            let pssSegment = pssSegments.add(i as usize);
            let mut nCharacteristics: ULONG;
            let SectionVirtualAddress = read_field!(pishSectionHeader, VirtualAddress);
            let SectionPointerToRawData = read_field!(pishSectionHeader, PointerToRawData);
            let SectionSizeOfRawData = read_field!(pishSectionHeader, SizeOfRawData);
            let SectionCharacteristics = read_field!(pishSectionHeader, Characteristics);
            let SectionMisc = read_field!(pishSectionHeader, Misc);
            let SectionVirtualSize = SectionMisc.VirtualSize;

            /* validate the alignment */
            if !IsAligned(SectionVirtualAddress, nSectionAlignment) {
                // DPRINT!("Image.VirtualAddress[%u] is not aligned\n", i)
                break 'l_Return;
            }

            /* sections must be contiguous, ordered by base address and non-overlapping */
            if SectionVirtualAddress as ULONG_PTR != nPrevVirtualEndOfSegment {
                // DPRINT!("Memory gap between section %u and the previous\n", i)
                break 'l_Return;
            }

            /* ignore explicit BSS sections */
            if SectionPointerToRawData != 0 && SectionSizeOfRawData != 0 {
                /* validate the alignment */
                /*
                 * Yes, this should be a multiple of FileAlignment, but there's
                 * stuff out there that isn't. We can cope with that
                 */
                // if(!IsAligned(pishSectionHeaders[i].SizeOfRawData, nFileAlignment))
                //     DIE(("SizeOfRawData[%u] is not aligned\n", i));

                // if(!IsAligned(pishSectionHeaders[i].PointerToRawData, nFileAlignment))
                //     DIE(("PointerToRawData[%u] is not aligned\n", i));

                if !Intsafe_CanAddULong32(SectionPointerToRawData, SectionSizeOfRawData) {
                    // DPRINT!("SizeOfRawData[%u] too large\n", i)
                    break 'l_Return;
                }

                /* conversion */
                (*pssSegment).Image.FileOffset = SectionPointerToRawData as ULONGLONG;
                (*pssSegment).RawLength.QuadPart = SectionSizeOfRawData as i64;
            } else {
                /* FIXME: Should reset PointerToRawData to 0 in the image mapping */
                ASSERT!((*pssSegment).Image.FileOffset == 0);
                ASSERT!((*pssSegment).RawLength.QuadPart == 0);
            }

            ASSERT!(Intsafe_CanAddLong64(
                (*pssSegment).Image.FileOffset as i64,
                (*pssSegment).RawLength.QuadPart
            ));

            nCharacteristics = SectionCharacteristics;

            /* no explicit protection */
            if (nCharacteristics
                & (IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE))
                == 0
            {
                if (nCharacteristics & IMAGE_SCN_CNT_CODE) != 0 {
                    nCharacteristics |= IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ;
                }

                if (nCharacteristics & IMAGE_SCN_CNT_INITIALIZED_DATA) != 0 {
                    nCharacteristics |= IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
                }

                if (nCharacteristics & IMAGE_SCN_CNT_UNINITIALIZED_DATA) != 0 {
                    nCharacteristics |= IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
                }
            }

            /* see table above */
            (*pssSegment).Protection =
                SectionCharacteristicsToProtect[(nCharacteristics >> 28) as usize];
            (*pssSegment).WriteCopy = if (nCharacteristics & IMAGE_SCN_MEM_SHARED) == 0 {
                TRUE
            } else {
                FALSE
            };

            if SectionVirtualSize == 0 {
                (*pssSegment).Length.QuadPart = SectionSizeOfRawData as i64;
            } else {
                (*pssSegment).Length.QuadPart = SectionVirtualSize as i64;
            }

            AlignedLength = ALIGN_UP_BY_ULONG((*pssSegment).Length.u.LowPart, nSectionAlignment);
            if AlignedLength < (*pssSegment).Length.u.LowPart {
                // DPRINT!("Cannot align the virtual size of section %u\n", i)
                break 'l_Return;
            }

            (*pssSegment).Length.u.LowPart = AlignedLength;

            if (*pssSegment).Length.QuadPart == 0 {
                // DPRINT!("Virtual size of section %u is null\n", i)
                break 'l_Return;
            }

            (*pssSegment).Image.VirtualAddress = SectionVirtualAddress as ULONG_PTR;
            (*pssSegment).Image.Characteristics = SectionCharacteristics;

            /* ensure the memory image is no larger than 4GB */
            nPrevVirtualEndOfSegment = (SectionVirtualAddress as ULONG_PTR)
                .wrapping_add((*pssSegment).Length.QuadPart as ULONG_PTR);
            if nPrevVirtualEndOfSegment < SectionVirtualAddress as ULONG_PTR {
                // DPRINT!("The image is too large\n")
                break 'l_Return;
            }

            i += 1;
        }

        if nSectionAlignment >= PAGE_SIZE_ULONG {
            let OldFlags = ptr::read_volatile(Flags);
            ptr::write_volatile(Flags, OldFlags | EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED);
        }

        /* Success */
        nStatus = STATUS_SUCCESS; // STATUS_ROS_EXEFMT_LOADED_FORMAT | EXEFMT_LOADED_PE32;
        break 'l_Return;
    }

    if !pBuffer.is_null() {
        ExFreePool(pBuffer);
    }

    nStatus
}
