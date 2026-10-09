use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{HCELL_INDEX, PCELL_DATA, PCHILD_LIST, PCM_KEY_VALUE, PCUNICODE_STRING, PHCELL_INDEX, PHHIVE, RtlCompareUnicodeString, RtlUpcaseUnicodeChar, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell}; // cdecl Hv* used by CmpFindNameInList

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
// The file contains only a preprocessor directive (#define NDEBUG),
// which is not translated to Rust as it's a build/debug control flag.

#[no_mangle]
pub unsafe extern "system" fn CmpCopyName(
    Hive: PHHIVE,
    Destination: *mut u16,
    Source: PCUNICODE_STRING,
) -> u16 {
    let mut i: u32;

    if (*Hive).Version == 1 {
        core::ptr::copy_nonoverlapping(
            (*Source).Buffer,
            Destination,
            (*Source).Length as usize / core::mem::size_of::<u16>(),
        );
        return (*Source).Length;
    }

    i = 0;
    while i < ((*Source).Length as u32 / core::mem::size_of::<u16>() as u32) {
        if *(*Source).Buffer.offset(i as isize) > 0xFF {
            core::ptr::copy_nonoverlapping(
                (*Source).Buffer,
                Destination,
                (*Source).Length as usize / core::mem::size_of::<u16>(),
            );
            return (*Source).Length;
        }

        *(Destination as *mut u8).offset(i as isize) =
            (*(*Source).Buffer.offset(i as isize)) as u8;
        i += 1;
    }

    (*Source).Length / core::mem::size_of::<u16>() as u16
}

#[no_mangle]
pub unsafe extern "system" fn CmpCopyCompressedName(
    Destination: PWCHAR,
    DestinationLength: ULONG,
    Source: PWCHAR,
    SourceLength: ULONG,
) {
    let mut i: ULONG;
    let Length: ULONG;

    Length = core::cmp::min(DestinationLength / core::mem::size_of::<WCHAR>() as ULONG, SourceLength);
    i = 0;
    while i < Length {
        *Destination.offset(i as isize) = *((Source as PUCHAR).offset(i as isize)) as WCHAR;
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpNameSize(
    Hive: PHHIVE,
    Name: PCUNICODE_STRING,
) -> u16 {
    let mut i: u32;

    if (*Hive).Version == 1 {
        return (*Name).Length;
    }

    i = 0;
    while i < ((*Name).Length as u32 / core::mem::size_of::<u16>() as u32) {
        if (*Name).Buffer.offset(i as isize).read() > 0xFF {
            return (*Name).Length;
        }
        i += 1;
    }

    ((*Name).Length as u32 / core::mem::size_of::<u16>() as u32) as u16
}

#[no_mangle]
pub unsafe extern "system" fn CmpCompressedNameSize(
    _Name: PWCHAR,
    Length: ULONG,
) -> USHORT {
    (Length * core::mem::size_of::<WCHAR>() as ULONG) as USHORT
}

#[no_mangle]
pub unsafe extern "system" fn CmpCompareCompressedName(
    SearchName: PCUNICODE_STRING,
    CompressedName: *mut u16,
    NameLength: u32,
) -> i32 {
    let mut p: *mut u16;
    let mut pp: *mut u8;
    let mut chr1: u16;
    let mut chr2: u16;
    let mut SearchLength: u16;
    let _Result: i32;

    p = (*SearchName).Buffer;
    pp = CompressedName as *mut u8;
    SearchLength = (*SearchName).Length / core::mem::size_of::<u16>() as u16;
    let mut NameLength = NameLength;
    
    while SearchLength > 0 && NameLength > 0 {
        chr1 = *p;
        p = p.add(1);
        chr2 = *pp as u16;
        pp = pp.add(1);

        if chr1 != chr2 {
            let Result = RtlUpcaseUnicodeChar(chr1) as i32 - 
                        RtlUpcaseUnicodeChar(chr2) as i32;
            if Result != 0 {
                return Result;
            }
        }

        SearchLength -= 1;
        NameLength -= 1;
    }

    SearchLength as i32 - NameLength as i32
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindNameInList(
    Hive: PHHIVE,
    ChildList: PCHILD_LIST,
    Name: PCUNICODE_STRING,
    ChildIndex: *mut u32,
    CellIndex: PHCELL_INDEX,
) -> u8 {
    let CellData: PCELL_DATA;
    let mut CellToRelease: HCELL_INDEX = 0xFFFFFFFF;
    let mut i: u32;
    let mut KeyValue: PCM_KEY_VALUE;
    let mut Result: i32;
    let mut SearchName: _UNICODE_STRING = core::mem::zeroed();
    let mut Success: u8 = 0u8;

    if (*ChildList).Count != 0 {
        CellData = HvGetCell(Hive, (*ChildList).List) as PCELL_DATA;
        if CellData.is_null() {
            *CellIndex = 0xFFFFFFFF;
            return 0u8;
        }

        i = 0;
        while i < (*ChildList).Count {
            if CellToRelease != 0xFFFFFFFF {
                HvReleaseCell(Hive, CellToRelease);
                CellToRelease = 0xFFFFFFFF;
            }

            KeyValue = HvGetCell(Hive, *(core::ptr::addr_of!((*CellData).u.KeyList) as *const HCELL_INDEX).add(i as usize)) as PCM_KEY_VALUE;
            if KeyValue.is_null() {
                *CellIndex = 0xFFFFFFFF;
                Success = 0u8;
                break;
            }

            CellToRelease = *(core::ptr::addr_of!((*CellData).u.KeyList) as *const HCELL_INDEX).add(i as usize);

            if (*KeyValue).Flags & 0x0001 != 0 {
                Result = CmpCompareCompressedName(
                    Name,
                    core::ptr::addr_of_mut!((*KeyValue).Name) as *mut u16,
                    (*KeyValue).NameLength as u32,
                );
            } else {
                SearchName.Length = (*KeyValue).NameLength;
                SearchName.MaximumLength = SearchName.Length;
                SearchName.Buffer = core::ptr::addr_of_mut!((*KeyValue).Name) as *mut u16;
                Result = RtlCompareUnicodeString(Name, &SearchName, 1u8);
            }

            if Result == 0 {
                if !ChildIndex.is_null() {
                    *ChildIndex = i;
                }
                *CellIndex = *(core::ptr::addr_of!((*CellData).u.KeyList) as *const HCELL_INDEX).add(i as usize);
                Success = 1u8;
                break;
            }

            i += 1;
        }

        if i == (*ChildList).Count {
            if !ChildIndex.is_null() {
                *ChildIndex = i;
            }
            *CellIndex = 0xFFFFFFFF;
            Success = 1u8;
        }
    } else {
        if !ChildIndex.is_null() {
            *ChildIndex = 0;
        }
        *CellIndex = 0xFFFFFFFF;
        return 1u8;
    }

    if !CellData.is_null() {
        HvReleaseCell(Hive, (*ChildList).List);
    }

    if CellToRelease != 0xFFFFFFFF {
        HvReleaseCell(Hive, CellToRelease);
    }
    Success
}


// ============================================================================
