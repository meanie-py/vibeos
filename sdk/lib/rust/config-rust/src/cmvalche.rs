use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    CmpCleanUpKcbValueCache, CmpCompareCompressedName, CmpCompressedNameSize,
    CmpCopyCompressedName, CmpFree, CmpGetValueData, CmpTryToConvertKcbSharedToExclusive,
    HCELL_INDEX, KEY_VALUE_BASIC_INFORMATION, KEY_VALUE_FULL_INFORMATION,
    KEY_VALUE_PARTIAL_INFORMATION, KEY_VALUE_PARTIAL_INFORMATION_ALIGN64,
    KEY_VALUE_INFORMATION_CLASS, PCACHED_CHILD_LIST, PCELL_DATA, PCM_CACHED_VALUE,
    PCM_KEY_CONTROL_BLOCK, PCM_KEY_NODE, PCM_KEY_VALUE, PCUNICODE_STRING, PHCELL_INDEX,
    PHHIVE, PKEY_VALUE_INFORMATION, PNTSTATUS, PUNICODE_STRING, RtlCompareMemory,
    RtlCompareUnicodeString, VALUE_SEARCH_RETURN_TYPE,
    _UNICODE_STRING,
    _VALUE_SEARCH_RETURN_TYPE_SearchSuccess as SearchSuccess,
    _VALUE_SEARCH_RETURN_TYPE_SearchFail as SearchFail,
    _VALUE_SEARCH_RETURN_TYPE_SearchNeedExclusiveLock as SearchNeedExclusiveLock,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueBasicInformation as KeyValueBasicInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as KeyValueFullInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformationAlign64 as KeyValueFullInformationAlign64,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformationAlign64 as KeyValuePartialInformationAlign64,
};
use core::ffi::c_void;

use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
// C macros / constants used by this module — defined locally because they are
// not part of the bindgen surface (cmdata.h / hivedata.h).
const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
const VALUE_COMP_NAME: u16 = 0x0001;
const KEY_SYM_LINK: u32 = 0x0010;
const CM_KEY_VALUE_SMALL: u32 = 0x4;
const CM_KEY_VALUE_SPECIAL_SIZE: u32 = 0x80000000;

#[inline]
unsafe fn ASSERT_VALUE_CACHE() {}

#[inline]
fn ALIGN_UP(value: ULONG, align: ULONG) -> ULONG {
    (value + align - 1) & !(align - 1)
}

extern "C" {
    fn CmpIsKeyValueSmall(KeySize: *mut ULONG, DataLength: ULONG) -> BOOLEAN;
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    fn CmpIsKcbLockedExclusive(Kcb: PCM_KEY_CONTROL_BLOCK) -> u8;
    fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn CMP_IS_CELL_CACHED(list: HCELL_INDEX) -> u8;
}

#[inline(always)]

unsafe fn CmpIsValueCached(CellIndex: HCELL_INDEX) -> u8 {
    if CellIndex == HCELL_NIL {
        return 0u8;
    }
    
    if (CellIndex & 1) != 0 {
        return 1u8;
    }
    
    0u8
}

unsafe fn CmpSetValueCached(CellIndex: PHCELL_INDEX) {
    *CellIndex |= 1;
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetValueListFromCache(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    CellData: *mut PCELL_DATA,
    IndexIsCached: *mut u8,
    ValueListToRelease: PHCELL_INDEX
) -> VALUE_SEARCH_RETURN_TYPE {
    let Kcb = opaque_ptr(Kcb);
    let CellData = opaque_ptr(CellData);
    let IndexIsCached = opaque_ptr(IndexIsCached);
    let ValueListToRelease = opaque_ptr(ValueListToRelease);
    let Hive: PHHIVE;
    let ChildList: PCACHED_CHILD_LIST;
    let CellToRelease: HCELL_INDEX;

    *ValueListToRelease = HCELL_NIL;
    *IndexIsCached = 0u8;

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    ChildList = core::ptr::addr_of_mut!((*Kcb).ValueCache);

    if CmpIsValueCached((*ChildList).__bindgen_anon_1.ValueList) != 0 {
        ASSERT_VALUE_CACHE();
        *IndexIsCached = 1u8;
        *CellData = core::ptr::null_mut();
    } else {
        if CmpIsKcbLockedExclusive(Kcb) == 0 &&
            CmpTryToConvertKcbSharedToExclusive(Kcb) == 0 {
            return SearchNeedExclusiveLock;
        }

        CellToRelease = (*ChildList).__bindgen_anon_1.ValueList;
        *CellData = HvGetCell(Hive, CellToRelease) as PCELL_DATA;
        if (*CellData).is_null() {
            return SearchFail;
        }

        *ValueListToRelease = CellToRelease;
    }

    SearchSuccess
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetValueKeyFromCache(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    CellData: PCELL_DATA,
    Index: u32,
    CachedValue: *mut PCM_CACHED_VALUE,
    Value: *mut PCM_KEY_VALUE,
    IndexIsCached: u8,
    ValueIsCached: *mut u8,
    CellToRelease: PHCELL_INDEX,
) -> VALUE_SEARCH_RETURN_TYPE {
    let Kcb = opaque_ptr(Kcb);
    let CellData = opaque_ptr(CellData);
    let CachedValue = opaque_ptr(CachedValue);
    let Value = opaque_ptr(Value);
    let ValueIsCached = opaque_ptr(ValueIsCached);
    let CellToRelease = opaque_ptr(CellToRelease);

    let Hive: PHHIVE;
    let KeyValue: PCM_KEY_VALUE;
    let Cell: HCELL_INDEX;

    *CellToRelease = 0xFFFFFFFF;
    *Value = core::ptr::null_mut();
    *ValueIsCached = 0u8;

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;

    if IndexIsCached != 0 {
        ASSERT_VALUE_CACHE();
        *ValueIsCached = 1u8;
    } else {
        Cell = *(core::ptr::addr_of!((*CellData).u.KeyList) as *const HCELL_INDEX).add(Index as usize);
        KeyValue = HvGetCell(Hive, Cell) as PCM_KEY_VALUE;
        if KeyValue.is_null() {
            return SearchFail;
        }

        *CellToRelease = Cell;
        *Value = KeyValue;
    }

    return SearchSuccess;
}

unsafe fn CmpGetValueDataFromCache(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    CachedValue: *mut PCM_CACHED_VALUE,
    ValueKey: PCELL_DATA,
    ValueIsCached: BOOLEAN,
    DataPointer: *mut PVOID,
    Allocated: PBOOLEAN,
    CellToRelease: PHCELL_INDEX
) -> VALUE_SEARCH_RETURN_TYPE {
    let Hive: PHHIVE;
    let mut Length: ULONG = 0;

    *DataPointer = core::ptr::null_mut();
    *Allocated = 0u8;
    *CellToRelease = HCELL_NIL;

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;

    if ValueIsCached != 0 {
        ASSERT_VALUE_CACHE();
    } else {
        if CmpGetValueData(
            Hive,
            core::ptr::addr_of_mut!((*ValueKey).u.KeyValue),
            core::ptr::addr_of_mut!(Length),
            DataPointer,
            Allocated,
            CellToRelease
        ) == 0 {
            return SearchFail;
        }
    }

    return SearchSuccess;
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindValueByNameFromCache(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    Name: PCUNICODE_STRING,
    CachedValue: *mut PCM_CACHED_VALUE,
    Index: *mut u32,
    Value: *mut PCM_KEY_VALUE,
    ValueIsCached: *mut u8,
    CellToRelease: PHCELL_INDEX,
) -> VALUE_SEARCH_RETURN_TYPE {
    let Kcb = opaque_ptr(Kcb);
    let Name = opaque_ptr_const(Name);
    let CachedValue = opaque_ptr(CachedValue);
    let Index = opaque_ptr(Index);
    let Value = opaque_ptr(Value);
    let ValueIsCached = opaque_ptr(ValueIsCached);
    let CellToRelease = opaque_ptr(CellToRelease);

    let Hive: PHHIVE;
    let mut SearchResult: VALUE_SEARCH_RETURN_TYPE = SearchFail;
    let mut Result: i32;
    let mut SearchName: _UNICODE_STRING = core::mem::zeroed();
    let mut CellData: PCELL_DATA = core::ptr::null_mut();
    let ChildList: PCACHED_CHILD_LIST;
    let mut KeyValue: PCM_KEY_VALUE;
    let mut IndexIsCached: u8 = 0;
    let mut i: u32 = 0;
    let mut Cell: HCELL_INDEX = HCELL_NIL;

    *CellToRelease = HCELL_NIL;
    *Value = core::ptr::null_mut();

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    ChildList = core::ptr::addr_of_mut!((*Kcb).ValueCache);

    if (*ChildList).Count != 0 {
        SearchResult = CmpGetValueListFromCache(
            Kcb,
            core::ptr::addr_of_mut!(CellData),
            core::ptr::addr_of_mut!(IndexIsCached),
            core::ptr::addr_of_mut!(Cell),
        );
        if SearchResult != SearchSuccess {
            return SearchResult;
        }

        if IndexIsCached != 0 {
            ASSERT_VALUE_CACHE();
        }

        loop {
            if *CellToRelease != HCELL_NIL {
                HvReleaseCell(Hive, *CellToRelease);
                *CellToRelease = HCELL_NIL;
            }

            SearchResult = CmpGetValueKeyFromCache(
                Kcb,
                CellData,
                i,
                CachedValue,
                Value,
                IndexIsCached,
                ValueIsCached,
                CellToRelease,
            );
            if SearchResult != SearchSuccess {
                return SearchResult;
            }

            if IndexIsCached != 0 && *ValueIsCached != 0 {
                ASSERT_VALUE_CACHE();
                Result = -1;
            } else {
                KeyValue = *Value;
                let key_value_name_ptr = core::ptr::addr_of_mut!((*KeyValue).Name) as *mut u16;
                let key_value_name_length = core::ptr::addr_of!((*KeyValue).NameLength).read_unaligned();
                let key_value_flags = core::ptr::addr_of!((*KeyValue).Flags).read_unaligned();
                if key_value_flags & VALUE_COMP_NAME != 0 {
                    Result = CmpCompareCompressedName(
                        Name,
                        key_value_name_ptr,
                        key_value_name_length as ULONG,
                    );
                } else {
                    SearchName.Length = key_value_name_length;
                    SearchName.MaximumLength = SearchName.Length;
                    SearchName.Buffer = key_value_name_ptr;
                    Result = RtlCompareUnicodeString(Name, core::ptr::addr_of_mut!(SearchName) as PCUNICODE_STRING, 1u8);
                }
            }

            if Result == 0 {
                *Index = i;
                SearchResult = SearchSuccess;
                break;
            }

            i += 1;
            if i == (*ChildList).Count {
                *Value = core::ptr::null_mut();
                SearchResult = SearchFail;
                break;
            }
        }
    }

    if Cell != HCELL_NIL {
        HvReleaseCell(Hive, Cell);
    }
    SearchResult
}

#[no_mangle]
pub unsafe extern "system" fn CmpQueryKeyValueData(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    CachedValue: *mut PCM_CACHED_VALUE,
    ValueKey: PCM_KEY_VALUE,
    ValueIsCached: BOOLEAN,
    KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
    KeyValueInformation: PVOID,
    Length: ULONG,
    ResultLength: PULONG,
    Status: PNTSTATUS
) -> VALUE_SEARCH_RETURN_TYPE {
    let Kcb = opaque_ptr(Kcb);
    let CachedValue = opaque_ptr(CachedValue);
    let ValueKey = opaque_ptr(ValueKey);
    let KeyValueInformation = opaque_ptr(KeyValueInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let Status = opaque_ptr(Status);
    let Info: PKEY_VALUE_INFORMATION = KeyValueInformation as PKEY_VALUE_INFORMATION;
    let CellData: PCELL_DATA;
    let mut NameSize: USHORT;
    let mut Size: ULONG;
    let MinimumSize: ULONG;
    let mut SizeLeft: ULONG;
    let KeySize: ULONG;
    let mut AlignedData: ULONG = 0;
    let DataOffset: ULONG;
    let mut Buffer: PVOID = core::ptr::null_mut();
    let IsSmall: BOOLEAN;
    let mut BufferAllocated: BOOLEAN = 0u8;
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;
    let mut Result: VALUE_SEARCH_RETURN_TYPE = SearchSuccess;

    CellData = ValueKey as PCELL_DATA;

    let cell_kv_ptr = core::ptr::addr_of_mut!((*CellData).u.KeyValue);
    let cell_name_ptr = core::ptr::addr_of_mut!((*cell_kv_ptr).Name) as *mut u16;
    let cell_flags = core::ptr::addr_of!((*cell_kv_ptr).Flags).read_unaligned();
    let cell_name_length = core::ptr::addr_of!((*cell_kv_ptr).NameLength).read_unaligned();
    let cell_type = core::ptr::addr_of!((*cell_kv_ptr).Type).read_unaligned();
    let cell_data_length = core::ptr::addr_of!((*cell_kv_ptr).DataLength).read_unaligned();

    if cell_flags & VALUE_COMP_NAME != 0 {
        NameSize = CmpCompressedNameSize(
            cell_name_ptr,
            cell_name_length as ULONG,
        );
    } else {
        NameSize = cell_name_length;
    }

    match KeyValueInformationClass {
        KeyValueBasicInformation => {
            Size = core::mem::offset_of!(KEY_VALUE_BASIC_INFORMATION, Name) as ULONG
                + NameSize as ULONG;
            let MinimumSize = core::mem::offset_of!(KEY_VALUE_BASIC_INFORMATION, Name) as ULONG;

            *ResultLength = Size;
            *Status = STATUS_SUCCESS;

            if Length < MinimumSize {
                *Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*Info).__bindgen_anon_1.KeyValueBasicInformation.TitleIndex = 0;
                (*Info).__bindgen_anon_1.KeyValueBasicInformation.Type = cell_type;
                (*Info).__bindgen_anon_1.KeyValueBasicInformation.NameLength = NameSize as ULONG;

                SizeLeft = Length - MinimumSize;
                Size = NameSize as ULONG;

                if SizeLeft < Size {
                    Size = SizeLeft;
                    *Status = STATUS_BUFFER_OVERFLOW;
                }

                if cell_flags & VALUE_COMP_NAME != 0 {
                    CmpCopyCompressedName(
                        (*Info).__bindgen_anon_1.KeyValueBasicInformation.Name.as_mut_ptr(),
                        Size,
                        cell_name_ptr,
                        cell_name_length as ULONG,
                    );
                } else {
                    // RtlCopyMemory: Size is a BYTE count (a u16 copy would move 2x).
                    core::ptr::copy_nonoverlapping(
                        cell_name_ptr as *const u8,
                        (*Info).__bindgen_anon_1.KeyValueBasicInformation.Name.as_mut_ptr() as *mut u8,
                        Size as usize
                    );
                }
            }
        }

        KeyValueFullInformation | KeyValueFullInformationAlign64 => {

            let mut KeySize: ULONG = 0;
            IsSmall = CmpIsKeyValueSmall(&mut KeySize, cell_data_length);

            Size = core::mem::offset_of!(KEY_VALUE_FULL_INFORMATION, Name) as ULONG
                + NameSize as ULONG
                + KeySize;

            let MinimumSize = core::mem::offset_of!(KEY_VALUE_FULL_INFORMATION, Name) as ULONG;

            if KeySize > 0 {
                DataOffset = Size - KeySize;

                #[cfg(target_pointer_width = "64")]
                {
                    AlignedData = ALIGN_UP(DataOffset, core::mem::size_of::<u64>() as ULONG);
                }
                #[cfg(target_pointer_width = "32")]
                {
                    if KeyValueInformationClass == KeyValueFullInformationAlign64 {
                        AlignedData = ALIGN_UP(DataOffset, core::mem::size_of::<u64>() as ULONG);
                    } else {
                        AlignedData = ALIGN_UP(DataOffset, core::mem::size_of::<ULONG>() as ULONG);
                    }
                }

                if AlignedData > DataOffset {
                    Size += AlignedData - DataOffset;
                }
            }

            *ResultLength = Size;
            *Status = STATUS_SUCCESS;

            if Length < MinimumSize {
                *Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*Info).__bindgen_anon_1.KeyValueFullInformation.TitleIndex = 0;
                (*Info).__bindgen_anon_1.KeyValueFullInformation.Type = cell_type;
                (*Info).__bindgen_anon_1.KeyValueFullInformation.DataLength = KeySize;
                (*Info).__bindgen_anon_1.KeyValueFullInformation.NameLength = NameSize as ULONG;

                SizeLeft = Length - MinimumSize;
                Size = NameSize as ULONG;

                if SizeLeft < Size {
                    Size = SizeLeft;
                    *Status = STATUS_BUFFER_OVERFLOW;
                }

                if cell_flags & VALUE_COMP_NAME != 0 {
                    CmpCopyCompressedName(
                        (*Info).__bindgen_anon_1.KeyValueFullInformation.Name.as_mut_ptr(),
                        Size,
                        cell_name_ptr,
                        cell_name_length as ULONG,
                    );
                } else {
                    // RtlCopyMemory: Size is a BYTE count (a u16 copy would move 2x).
                    core::ptr::copy_nonoverlapping(
                        cell_name_ptr as *const u8,
                        (*Info).__bindgen_anon_1.KeyValueFullInformation.Name.as_mut_ptr() as *mut u8,
                        Size as usize
                    );
                }

                if KeySize > 0 {
                    if IsSmall != 0 {
                        Buffer = core::ptr::addr_of_mut!((*CellData).u.KeyValue.Data) as PVOID;
                    } else {
                        Result = CmpGetValueDataFromCache(
                            Kcb,
                            CachedValue,
                            CellData,
                            ValueIsCached,
                            &mut Buffer,
                            &mut BufferAllocated,
                            &mut CellToRelease
                        );
                        if Result != SearchSuccess {
                            *Status = STATUS_INSUFFICIENT_RESOURCES;
                        }
                    }

                    (*Info).__bindgen_anon_1.KeyValueFullInformation.DataOffset = AlignedData;

                    SizeLeft = if (Length as LONG) < (AlignedData as LONG) {
                        0
                    } else {
                        Length - AlignedData
                    };
                    Size = KeySize;

                    if SizeLeft < Size {
                        Size = SizeLeft;
                        *Status = STATUS_BUFFER_OVERFLOW;
                    }

                    if !Buffer.is_null() {
                        core::ptr::copy_nonoverlapping(
                            Buffer as *const u8,
                            (Info as ULONG_PTR + AlignedData as ULONG_PTR) as *mut u8,
                            Size as usize
                        );
                    }
                } else {
                    (*Info).__bindgen_anon_1.KeyValueFullInformation.DataOffset = 0xFFFFFFFF;
                }
            }
        }

        KeyValuePartialInformation | KeyValuePartialInformationAlign64 => {

            let mut KeySize: ULONG = 0;
            IsSmall = CmpIsKeyValueSmall(&mut KeySize, cell_data_length);

            let MinimumSize: ULONG;
            if KeyValueInformationClass == KeyValuePartialInformationAlign64 {
                Size = core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION_ALIGN64, Data) as ULONG
                    + KeySize;
                MinimumSize = core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION_ALIGN64, Data) as ULONG;
            } else {
                Size = core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG
                    + KeySize;
                MinimumSize = core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG;
            }

            *ResultLength = Size;
            *Status = STATUS_SUCCESS;

            if Length < MinimumSize {
                *Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                if KeyValueInformationClass == KeyValuePartialInformationAlign64 {
                    (*Info).__bindgen_anon_1.KeyValuePartialInformationAlign64.Type = cell_type;
                    (*Info).__bindgen_anon_1.KeyValuePartialInformationAlign64.DataLength = KeySize;
                } else {
                    (*Info).__bindgen_anon_1.KeyValuePartialInformation.TitleIndex = 0;
                    (*Info).__bindgen_anon_1.KeyValuePartialInformation.Type = cell_type;
                    (*Info).__bindgen_anon_1.KeyValuePartialInformation.DataLength = KeySize;
                }

                if KeySize > 0 {
                    if IsSmall != 0 {
                        Buffer = core::ptr::addr_of_mut!((*CellData).u.KeyValue.Data) as PVOID;
                    } else {
                        Result = CmpGetValueDataFromCache(
                            Kcb,
                            CachedValue,
                            CellData,
                            ValueIsCached,
                            &mut Buffer,
                            &mut BufferAllocated,
                            &mut CellToRelease
                        );
                        if Result != SearchSuccess {
                            *Status = STATUS_INSUFFICIENT_RESOURCES;
                        }
                    }

                    SizeLeft = Length - MinimumSize;
                    Size = KeySize;

                    if SizeLeft < Size {
                        Size = SizeLeft;
                        *Status = STATUS_BUFFER_OVERFLOW;
                    }

                    if !Buffer.is_null() {
                        if KeyValueInformationClass == KeyValuePartialInformationAlign64 {
                            core::ptr::copy_nonoverlapping(
                                Buffer as *const u8,
                                (*Info).__bindgen_anon_1.KeyValuePartialInformationAlign64.Data.as_mut_ptr(),
                                Size as usize
                            );
                        } else {
                            core::ptr::copy_nonoverlapping(
                                Buffer as *const u8,
                                (*Info).__bindgen_anon_1.KeyValuePartialInformation.Data.as_mut_ptr(),
                                Size as usize
                            );
                        }
                    }
                }
            }
        }

        _ => {
            crate::dbg_print!(
                b"Caller requested unknown class: %lx\n\0",
                KeyValueInformationClass
            );
            *Status = STATUS_INVALID_PARAMETER;
        }
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn CmpCompareNewValueDataAgainstKCBCache(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    ValueName: PUNICODE_STRING,
    Type: u32,
    Data: *mut core::ffi::c_void,
    DataSize: u32,
) -> VALUE_SEARCH_RETURN_TYPE {
    let Kcb = opaque_ptr(Kcb);
    let ValueName = opaque_ptr(ValueName);
    let Data = opaque_ptr(Data);

    let mut SearchResult: VALUE_SEARCH_RETURN_TYPE;
    let KeyNode: PCM_KEY_NODE;
    let mut CachedValue: PCM_CACHED_VALUE = core::ptr::null_mut();
    let mut Index: u32 = 0;
    let mut Value: PCM_KEY_VALUE = core::ptr::null_mut();
    let mut ValueCached: u8 = 0;
    let mut BufferAllocated: u8 = 0u8;
    let mut Buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ValueCellToRelease: HCELL_INDEX = HCELL_NIL;
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;
    let IsSmall: u8;
    let mut CompareResult: usize;

    'Quickie: {
        if ((*Kcb).__bindgen_anon_5.Flags() & KEY_SYM_LINK) != 0 {
            if CmpIsKcbLockedExclusive(Kcb) == 0 &&
               CmpTryToConvertKcbSharedToExclusive(Kcb) == 0 {
                return SearchNeedExclusiveLock;
            }

            KeyNode = HvGetCell(
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
            ) as PCM_KEY_NODE;
            if KeyNode.is_null() {
                return SearchFail;
            }

            CmpCleanUpKcbValueCache(Kcb);

            (*Kcb).ValueCache.Count = (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count;
            (*Kcb).ValueCache.__bindgen_anon_1.ValueList = (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.List;

            HvReleaseCell(
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
            );
        }

        SearchResult = CmpFindValueByNameFromCache(
            Kcb,
            ValueName,
            core::ptr::addr_of_mut!(CachedValue),
            core::ptr::addr_of_mut!(Index),
            core::ptr::addr_of_mut!(Value),
            core::ptr::addr_of_mut!(ValueCached),
            core::ptr::addr_of_mut!(ValueCellToRelease),
        );

        if SearchResult == SearchNeedExclusiveLock {
            break 'Quickie;
        } else if SearchResult == SearchSuccess {
            if (Type == (*Value).Type) &&
               (DataSize == ((*Value).DataLength & !CM_KEY_VALUE_SPECIAL_SIZE)) {
                IsSmall = if DataSize <= CM_KEY_VALUE_SMALL { 1u8 } else { 0u8 };
                if IsSmall != 0 {
                    Buffer = core::ptr::addr_of_mut!((*Value).Data) as *mut core::ffi::c_void;
                } else {
                    SearchResult = CmpGetValueDataFromCache(
                        Kcb,
                        core::ptr::addr_of_mut!(CachedValue),
                        Value as PCELL_DATA,
                        ValueCached,
                        core::ptr::addr_of_mut!(Buffer),
                        core::ptr::addr_of_mut!(BufferAllocated),
                        core::ptr::addr_of_mut!(CellToRelease),
                    );
                    if SearchResult != SearchSuccess {
                        break 'Quickie;
                    }
                }

                if DataSize != 0 {
                    CompareResult = RtlCompareMemory(
                        Buffer,
                        Data,
                        (DataSize & !CM_KEY_VALUE_SPECIAL_SIZE) as _,
                    ) as usize;
                } else {
                    CompareResult = 0;
                }

                if CompareResult != DataSize as usize {
                    SearchResult = SearchFail;
                }
            } else {
                SearchResult = SearchFail;
            }
        }
    }

    if ValueCellToRelease != HCELL_NIL {
        HvReleaseCell((*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive, ValueCellToRelease);
    }

    if BufferAllocated != 0 {
        CmpFree(Buffer, 0);
    }

    if CellToRelease != HCELL_NIL {
        HvReleaseCell((*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive, CellToRelease);
    }

    SearchResult
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmvalche.c
//  * PURPOSE:         Configuration Manager - Value Cell Cache
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// FORCEINLINE
// BOOLEAN
// CmpIsValueCached(IN HCELL_INDEX CellIndex)
// {
//     /* Make sure that the cell is valid in the first place */
//     if (CellIndex == HCELL_NIL) return FALSE;
//
//     /*Is this cell actually a pointer to the cached value data? */
//     if (CellIndex & 1) return TRUE;
//
//     /* This is a regular cell */
//     return FALSE;
// }
//
// FORCEINLINE
// VOID
// CmpSetValueCached(IN PHCELL_INDEX CellIndex)
// {
//     /* Set the cached bit */
//     *CellIndex |= 1;
// }
//
// #define ASSERT_VALUE_CACHE() \
//     ASSERTMSG("Cached Values Not Yet Supported!\n", FALSE);
//
// /* FUNCTIONS *****************************************************************/
//
// VALUE_SEARCH_RETURN_TYPE
// NTAPI
// CmpGetValueListFromCache(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                          OUT PCELL_DATA *CellData,
//                          OUT BOOLEAN *IndexIsCached,
//                          OUT PHCELL_INDEX ValueListToRelease)
// {
//     PHHIVE Hive;
//     PCACHED_CHILD_LIST ChildList;
//     HCELL_INDEX CellToRelease;
//
//     /* Set defaults */
//     *ValueListToRelease = HCELL_NIL;
//     *IndexIsCached = FALSE;
//
//     /* Get the hive and value cache */
//     Hive = Kcb->KeyHive;
//     ChildList = &Kcb->ValueCache;
//
//     /* Check if the value is cached */
//     if (CmpIsValueCached(ChildList->ValueList))
//     {
//         /* It is: we don't expect this yet! */
//         ASSERT_VALUE_CACHE();
//         *IndexIsCached = TRUE;
//         *CellData = NULL;
//     }
//     else
//     {
//         /* Make sure the KCB is locked exclusive */
//         if (!CmpIsKcbLockedExclusive(Kcb) &&
//             !CmpTryToConvertKcbSharedToExclusive(Kcb))
//         {
//             /* We need the exclusive lock */
//             return SearchNeedExclusiveLock;
//         }
//
//         /* Select the value list as our cell, and get the actual list array */
//         CellToRelease = ChildList->ValueList;
//         *CellData = (PCELL_DATA)HvGetCell(Hive, CellToRelease);
//         if (!*CellData) return SearchFail;
//
//         /* FIXME: Here we would cache the value */
//
//         /* Return the cell to be released */
//         *ValueListToRelease = CellToRelease;
//     }
//
//     /* If we got here, then the value list was found */
//     return SearchSuccess;
// }
//
// VALUE_SEARCH_RETURN_TYPE
// NTAPI
// CmpGetValueKeyFromCache(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                         IN PCELL_DATA CellData,
//                         IN ULONG Index,
//                         OUT PCM_CACHED_VALUE **CachedValue,
//                         OUT PCM_KEY_VALUE *Value,
//                         IN BOOLEAN IndexIsCached,
//                         OUT BOOLEAN *ValueIsCached,
//                         OUT PHCELL_INDEX CellToRelease)
// {
//     PHHIVE Hive;
//     PCM_KEY_VALUE KeyValue;
//     HCELL_INDEX Cell;
//
//     /* Set defaults */
//     *CellToRelease = HCELL_NIL;
//     *Value = NULL;
//     *ValueIsCached = FALSE;
//
//     /* Get the hive */
//     Hive = Kcb->KeyHive;
//
//     /* Check if the index was cached */
//     if (IndexIsCached)
//     {
//         /* Not expected yet! */
//         ASSERT_VALUE_CACHE();
//         *ValueIsCached = TRUE;
//     }
//     else
//     {
//         /* Get the cell index and the key value associated to it */
//         Cell = CellData->u.KeyList[Index];
//         KeyValue = (PCM_KEY_VALUE)HvGetCell(Hive, Cell);
//         if (!KeyValue) return SearchFail;
//
//         /* Return the cell and the actual key value */
//         *CellToRelease = Cell;
//         *Value = KeyValue;
//     }
//
//     /* If we got here, then we found the key value */
//     return SearchSuccess;
// }
//
// VALUE_SEARCH_RETURN_TYPE
// NTAPI
// CmpGetValueDataFromCache(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                          IN PCM_CACHED_VALUE *CachedValue,
//                          IN PCELL_DATA ValueKey,
//                          IN BOOLEAN ValueIsCached,
//                          OUT PVOID *DataPointer,
//                          OUT PBOOLEAN Allocated,
//                          OUT PHCELL_INDEX CellToRelease)
// {
//     PHHIVE Hive;
//     ULONG Length;
//
//     /* Sanity checks */
//     ASSERT(MAXIMUM_CACHED_DATA < CM_KEY_VALUE_BIG);
//     ASSERT((ValueKey->u.KeyValue.DataLength & CM_KEY_VALUE_SPECIAL_SIZE) == 0);
//
//     /* Set defaults */
//     *DataPointer = NULL;
//     *Allocated = FALSE;
//     *CellToRelease = HCELL_NIL;
//
//     /* Get the hive */
//     Hive = Kcb->KeyHive;
//
//     /* Check it the value is cached */
//     if (ValueIsCached)
//     {
//         /* This isn't expected! */
//         ASSERT_VALUE_CACHE();
//     }
//     else
//     {
//         /* It's not, get the value data using the typical routine */
//         if (!CmpGetValueData(Hive,
//                              &ValueKey->u.KeyValue,
//                              &Length,
//                              DataPointer,
//                              Allocated,
//                              CellToRelease))
//         {
//             /* Nothing found: make sure no data was allocated */
//             ASSERT(*Allocated == FALSE);
//             ASSERT(*DataPointer == NULL);
//             return SearchFail;
//         }
//     }
//
//     /* We found the actual data, return success */
//     return SearchSuccess;
// }
//
// VALUE_SEARCH_RETURN_TYPE
// NTAPI
// CmpFindValueByNameFromCache(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                             IN PCUNICODE_STRING Name,
//                             OUT PCM_CACHED_VALUE **CachedValue,
//                             OUT ULONG *Index,
//                             OUT PCM_KEY_VALUE *Value,
//                             OUT BOOLEAN *ValueIsCached,
//                             OUT PHCELL_INDEX CellToRelease)
// {
//     PHHIVE Hive;
//     VALUE_SEARCH_RETURN_TYPE SearchResult = SearchFail;
//     LONG Result;
//     UNICODE_STRING SearchName;
//     PCELL_DATA CellData;
//     PCACHED_CHILD_LIST ChildList;
//     PCM_KEY_VALUE KeyValue;
//     BOOLEAN IndexIsCached;
//     ULONG i = 0;
//     HCELL_INDEX Cell = HCELL_NIL;
//
//     /* Set defaults */
//     *CellToRelease = HCELL_NIL;
//     *Value = NULL;
//
//     /* Get the hive and child list */
//     Hive = Kcb->KeyHive;
//     ChildList = &Kcb->ValueCache;
//
//     /* Check if the child list has any entries */
//     if (ChildList->Count != 0)
//     {
//         /* Get the value list associated to this child list */
//         SearchResult = CmpGetValueListFromCache(Kcb,
//                                                 &CellData,
//                                                 &IndexIsCached,
//                                                 &Cell);
//         if (SearchResult != SearchSuccess)
//         {
//             /* We either failed or need the exclusive lock */
//             ASSERT((SearchResult == SearchFail) || !CmpIsKcbLockedExclusive(Kcb));
//             ASSERT(Cell == HCELL_NIL);
//             return SearchResult;
//         }
//
//         /* The index shouldn't be cached right now */
//         if (IndexIsCached) ASSERT_VALUE_CACHE();
//
//         /* Loop every value */
//         while (TRUE)
//         {
//             /* Check if there's any cell to release */
//             if (*CellToRelease != HCELL_NIL)
//             {
//                 /* Release it now */
//                 HvReleaseCell(Hive, *CellToRelease);
//                 *CellToRelease = HCELL_NIL;
//             }
//
//             /* Get the key value for this index */
//             SearchResult = CmpGetValueKeyFromCache(Kcb,
//                                                    CellData,
//                                                    i,
//                                                    CachedValue,
//                                                    Value,
//                                                    IndexIsCached,
//                                                    ValueIsCached,
//                                                    CellToRelease);
//             if (SearchResult != SearchSuccess)
//             {
//                 /* We either failed or need the exclusive lock */
//                 ASSERT((SearchResult == SearchFail) || !CmpIsKcbLockedExclusive(Kcb));
//                 ASSERT(Cell == HCELL_NIL);
//                 return SearchResult;
//             }
//
//             /* Check if the both the index and the value are cached */
//             if (IndexIsCached && *ValueIsCached)
//             {
//                 /* We don't expect this yet */
//                 ASSERT_VALUE_CACHE();
//                 Result = -1;
//             }
//             else
//             {
//                 /* No cache, so try to compare the name. Is it compressed? */
//                 KeyValue = *Value;
//                 if (KeyValue->Flags & VALUE_COMP_NAME)
//                 {
//                     /* It is, do a compressed name comparison */
//                     Result = CmpCompareCompressedName(Name,
//                                                       KeyValue->Name,
//                                                       KeyValue->NameLength);
//                 }
//                 else
//                 {
//                     /* It's not compressed, so do a standard comparison */
//                     SearchName.Length = KeyValue->NameLength;
//                     SearchName.MaximumLength = SearchName.Length;
//                     SearchName.Buffer = KeyValue->Name;
//                     Result = RtlCompareUnicodeString(Name, &SearchName, TRUE);
//                 }
//             }
//
//             /* Check if we found the value data */
//             if (!Result)
//             {
//                 /* We have, return the index of the value and success */
//                 *Index = i;
//                 SearchResult = SearchSuccess;
//                 goto Quickie;
//             }
//
//             /* We didn't find it, try the next entry */
//             if (++i == ChildList->Count)
//             {
//                 /* The entire list was parsed, fail */
//                 *Value = NULL;
//                 SearchResult = SearchFail;
//                 goto Quickie;
//             }
//         }
//     }
//
//     /* We should only get here if the child list is empty */
//     ASSERT(ChildList->Count == 0);
//
// Quickie:
//     /* Release the value list cell if required, and return search result */
//     if (Cell != HCELL_NIL) HvReleaseCell(Hive, Cell);
//     return SearchResult;
// }
//
// VALUE_SEARCH_RETURN_TYPE
// NTAPI
// CmpQueryKeyValueData(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                      IN PCM_CACHED_VALUE *CachedValue,
//                      IN PCM_KEY_VALUE ValueKey,
//                      IN BOOLEAN ValueIsCached,
//                      IN KEY_VALUE_INFORMATION_CLASS KeyValueInformationClass,
//                      IN PVOID KeyValueInformation,
//                      IN ULONG Length,
//                      OUT PULONG ResultLength,
//                      OUT PNTSTATUS Status)
// {
//     PKEY_VALUE_INFORMATION Info = (PKEY_VALUE_INFORMATION)KeyValueInformation;
//     PCELL_DATA CellData;
//     USHORT NameSize;
//     ULONG Size, MinimumSize, SizeLeft, KeySize, AlignedData = 0, DataOffset;
//     PVOID Buffer;
//     BOOLEAN IsSmall, BufferAllocated = FALSE;
//     HCELL_INDEX CellToRelease = HCELL_NIL;
//     VALUE_SEARCH_RETURN_TYPE Result = SearchSuccess;
//
//     /* Get the value data */
//     CellData = (PCELL_DATA)ValueKey;
//
//     /* Check if the value is compressed */
//     if (CellData->u.KeyValue.Flags & VALUE_COMP_NAME)
//     {
//         /* Get the compressed name size */
//         NameSize = CmpCompressedNameSize(CellData->u.KeyValue.Name,
//                                          CellData->u.KeyValue.NameLength);
//     }
//     else
//     {
//         /* Get the real size */
//         NameSize = CellData->u.KeyValue.NameLength;
//     }
//
//     /* Check what kind of information the caller is requesting */
//     switch (KeyValueInformationClass)
//     {
//         /* Basic information */
//         case KeyValueBasicInformation:
//
//             /* This is how much size we'll need */
//             Size = FIELD_OFFSET(KEY_VALUE_BASIC_INFORMATION, Name) + NameSize;
//
//             /* This is the minimum we can work with */
//             MinimumSize = FIELD_OFFSET(KEY_VALUE_BASIC_INFORMATION, Name);
//
//             /* Return the size we'd like, and assume success */
//             *ResultLength = Size;
//             *Status = STATUS_SUCCESS;
//
//             /* Check if the caller gave us below our minimum */
//             if (Length < MinimumSize)
//             {
//                 /* Then we must fail */
//                 *Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* Fill out the basic information */
//             Info->KeyValueBasicInformation.TitleIndex = 0;
//             Info->KeyValueBasicInformation.Type = CellData->u.KeyValue.Type;
//             Info->KeyValueBasicInformation.NameLength = NameSize;
//
//             /* Now only the name is left */
//             SizeLeft = Length - MinimumSize;
//             Size = NameSize;
//
//             /* Check if the remaining buffer is too small for the name */
//             if (SizeLeft < Size)
//             {
//                 /* Copy only as much as can fit, and tell the caller */
//                 Size = SizeLeft;
//                 *Status = STATUS_BUFFER_OVERFLOW;
//             }
//
//             /* Check if this is a compressed name */
//             if (CellData->u.KeyValue.Flags & VALUE_COMP_NAME)
//             {
//                 /* Copy as much as we can of the compressed name */
//                 CmpCopyCompressedName(Info->KeyValueBasicInformation.Name,
//                                       Size,
//                                       CellData->u.KeyValue.Name,
//                                       CellData->u.KeyValue.NameLength);
//             }
//             else
//             {
//                 /* Copy as much as we can of the raw name */
//                 RtlCopyMemory(Info->KeyValueBasicInformation.Name,
//                               CellData->u.KeyValue.Name,
//                               Size);
//             }
//
//             /* We're all done */
//             break;
//
//         /* Full key information */
//         case KeyValueFullInformation:
//         case KeyValueFullInformationAlign64:
//
//             /* Check if this is a small key and compute key size */
//             IsSmall = CmpIsKeyValueSmall(&KeySize,
//                                          CellData->u.KeyValue.DataLength);
//
//             /* Calculate the total size required */
//             Size = FIELD_OFFSET(KEY_VALUE_FULL_INFORMATION, Name) +
//                    NameSize +
//                    KeySize;
//
//             /* And this is the least we can work with */
//             MinimumSize = FIELD_OFFSET(KEY_VALUE_FULL_INFORMATION, Name);
//
//             /* Check if there's any key data */
//             if (KeySize > 0)
//             {
//                 /* Calculate the data offset */
//                 DataOffset = Size - KeySize;
//
// #ifdef _WIN64
//                 /* On 64-bit, always align to 8 bytes */
//                 AlignedData = ALIGN_UP(DataOffset, ULONGLONG);
// #else
//                 /* On 32-bit, align the offset to 4 or 8 bytes */
//                 if (KeyValueInformationClass == KeyValueFullInformationAlign64)
//                 {
//                     AlignedData = ALIGN_UP(DataOffset, ULONGLONG);
//                 }
//                 else
//                 {
//                     AlignedData = ALIGN_UP(DataOffset, ULONG);
//                 }
// #endif
//                 /* If alignment was required, we'll need more space */
//                 if (AlignedData > DataOffset) Size += (AlignedData-DataOffset);
//             }
//
//             /* Tell the caller the size we'll finally need, and set success */
//             *ResultLength = Size;
//             *Status = STATUS_SUCCESS;
//
//             /* Check if the caller is giving us too little */
//             if (Length < MinimumSize)
//             {
//                 /* Then fail right now */
//                 *Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* Fill out the basic information */
//             Info->KeyValueFullInformation.TitleIndex = 0;
//             Info->KeyValueFullInformation.Type = CellData->u.KeyValue.Type;
//             Info->KeyValueFullInformation.DataLength = KeySize;
//             Info->KeyValueFullInformation.NameLength = NameSize;
//
//             /* Only the name is left now */
//             SizeLeft = Length - MinimumSize;
//             Size = NameSize;
//
//             /* Check if the name fits */
//             if (SizeLeft < Size)
//             {
//                 /* It doesn't, truncate what we'll copy, and tell the caller */
//                 Size = SizeLeft;
//                 *Status = STATUS_BUFFER_OVERFLOW;
//             }
//
//             /* Check if this key value is compressed */
//             if (CellData->u.KeyValue.Flags & VALUE_COMP_NAME)
//             {
//                 /* It is, copy the compressed name */
//                 CmpCopyCompressedName(Info->KeyValueFullInformation.Name,
//                                       Size,
//                                       CellData->u.KeyValue.Name,
//                                       CellData->u.KeyValue.NameLength);
//             }
//             else
//             {
//                 /* It's not, copy the raw name */
//                 RtlCopyMemory(Info->KeyValueFullInformation.Name,
//                               CellData->u.KeyValue.Name,
//                               Size);
//             }
//
//             /* Now check if the key had any data */
//             if (KeySize > 0)
//             {
//                 /* Was it a small key? */
//                 if (IsSmall)
//                 {
//                     /* Then the data is directly into the cell */
//                     Buffer = &CellData->u.KeyValue.Data;
//                 }
//                 else
//                 {
//                     /* Otherwise, we must retrieve it from the value cache */
//                     Result = CmpGetValueDataFromCache(Kcb,
//                                                       CachedValue,
//                                                       CellData,
//                                                       ValueIsCached,
//                                                       &Buffer,
//                                                       &BufferAllocated,
//                                                       &CellToRelease);
//                     if (Result != SearchSuccess)
//                     {
//                         /* We failed, nothing should be allocated */
//                         ASSERT(Buffer == NULL);
//                         ASSERT(BufferAllocated == FALSE);
//                         *Status = STATUS_INSUFFICIENT_RESOURCES;
//                     }
//                 }
//
//                 /* Now that we know we truly have data, set its offset */
//                 Info->KeyValueFullInformation.DataOffset = AlignedData;
//
//                 /* Only the data remains to be copied */
//                 SizeLeft = (((LONG)Length - (LONG)AlignedData) < 0) ?
//                            0 : (Length - AlignedData);
//                 Size = KeySize;
//
//                 /* Check if the caller has no space for it */
//                 if (SizeLeft < Size)
//                 {
//                     /* Truncate what we'll copy, and tell the caller */
//                     Size = SizeLeft;
//                     *Status = STATUS_BUFFER_OVERFLOW;
//                 }
//
//                 /* Sanity check */
//                 ASSERT(IsSmall ? (Size <= CM_KEY_VALUE_SMALL) : TRUE);
//
//                 /* Make sure we have a valid buffer */
//                 if (Buffer)
//                 {
//                     /* Copy the data into the aligned offset */
//                     RtlCopyMemory((PVOID)((ULONG_PTR)Info + AlignedData),
//                                   Buffer,
//                                   Size);
//                 }
//             }
//             else
//             {
//                 /* We don't have any data, set the offset to -1, not 0! */
//                 Info->KeyValueFullInformation.DataOffset = 0xFFFFFFFF;
//             }
//
//             /* We're done! */
//             break;
//
//         /* Partial information requested (no name or alignment!) */
//         case KeyValuePartialInformation:
//         case KeyValuePartialInformationAlign64:
//
//             /* Check if this is a small key and compute key size */
//             IsSmall = CmpIsKeyValueSmall(&KeySize,
//                                          CellData->u.KeyValue.DataLength);
//
//             /* Calculate the total size required and the least we can work with */
//             if (KeyValueInformationClass == KeyValuePartialInformationAlign64)
//             {
//                 Size = FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION_ALIGN64, Data) + KeySize;
//                 MinimumSize = FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION_ALIGN64, Data);
//             }
//             else
//             {
//                 Size = FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data) + KeySize;
//                 MinimumSize = FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data);
//             }
//
//             /* Tell the caller the size we'll finally need, and set success */
//             *ResultLength = Size;
//             *Status = STATUS_SUCCESS;
//
//             /* Check if the caller is giving us too little */
//             if (Length < MinimumSize)
//             {
//                 /* Then fail right now */
//                 *Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* Fill out the basic information */
//             if (KeyValueInformationClass == KeyValuePartialInformationAlign64)
//             {
//                 Info->KeyValuePartialInformationAlign64.Type = CellData->u.KeyValue.Type;
//                 Info->KeyValuePartialInformationAlign64.DataLength = KeySize;
//             }
//             else
//             {
//                 Info->KeyValuePartialInformation.TitleIndex = 0;
//                 Info->KeyValuePartialInformation.Type = CellData->u.KeyValue.Type;
//                 Info->KeyValuePartialInformation.DataLength = KeySize;
//             }
//
//             /* Now check if the key had any data */
//             if (KeySize > 0)
//             {
//                 /* Was it a small key? */
//                 if (IsSmall)
//                 {
//                     /* Then the data is directly into the cell */
//                     Buffer = &CellData->u.KeyValue.Data;
//                 }
//                 else
//                 {
//                     /* Otherwise, we must retrieve it from the value cache */
//                     Result = CmpGetValueDataFromCache(Kcb,
//                                                       CachedValue,
//                                                       CellData,
//                                                       ValueIsCached,
//                                                       &Buffer,
//                                                       &BufferAllocated,
//                                                       &CellToRelease);
//                     if (Result != SearchSuccess)
//                     {
//                         /* We failed, nothing should be allocated */
//                         ASSERT(Buffer == NULL);
//                         ASSERT(BufferAllocated == FALSE);
//                         *Status = STATUS_INSUFFICIENT_RESOURCES;
//                     }
//                 }
//
//                 /* Only the data remains to be copied */
//                 SizeLeft = Length - MinimumSize;
//                 Size = KeySize;
//
//                 /* Check if the caller has no space for it */
//                 if (SizeLeft < Size)
//                 {
//                     /* Truncate what we'll copy, and tell the caller */
//                     Size = SizeLeft;
//                     *Status = STATUS_BUFFER_OVERFLOW;
//                 }
//
//                 /* Sanity check */
//                 ASSERT(IsSmall ? (Size <= CM_KEY_VALUE_SMALL) : TRUE);
//
//                 /* Make sure we have a valid buffer */
//                 if (Buffer)
//                 {
//                     /* Copy the data into the aligned offset */
//                     if (KeyValueInformationClass == KeyValuePartialInformationAlign64)
//                     {
//                         RtlCopyMemory(Info->KeyValuePartialInformationAlign64.Data,
//                                       Buffer,
//                                       Size);
//                     }
//                     else
//                     {
//                         RtlCopyMemory(Info->KeyValuePartialInformation.Data,
//                                       Buffer,
//                                       Size);
//                     }
//                 }
//             }
//
//             /* We're done! */
//             break;
//
//         /* Other information class */
//         default:
//
//             /* We got some class that we don't support */
//             DPRINT1("Caller requested unknown class: %lx\n", KeyValueInformationClass);
//             *Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     /* Return the search result as well */
//     return Result;
// }
//
// VALUE_SEARCH_RETURN_TYPE
// NTAPI
// CmpCompareNewValueDataAgainstKCBCache(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                                       IN PUNICODE_STRING ValueName,
//                                       IN ULONG Type,
//                                       IN PVOID Data,
//                                       IN ULONG DataSize)
// {
//     VALUE_SEARCH_RETURN_TYPE SearchResult;
//     PCM_KEY_NODE KeyNode;
//     PCM_CACHED_VALUE *CachedValue;
//     ULONG Index;
//     PCM_KEY_VALUE Value;
//     BOOLEAN ValueCached, BufferAllocated = FALSE;
//     PVOID Buffer;
//     HCELL_INDEX ValueCellToRelease = HCELL_NIL, CellToRelease = HCELL_NIL;
//     BOOLEAN IsSmall;
//     ULONG_PTR CompareResult;
//     PAGED_CODE();
//
//     /* Check if this is a symlink */
//     if (Kcb->Flags & KEY_SYM_LINK)
//     {
//         /* We need the exclusive lock */
//         if (!CmpIsKcbLockedExclusive(Kcb) &&
//             !CmpTryToConvertKcbSharedToExclusive(Kcb))
//         {
//             /* We need the exclusive lock */
//             return SearchNeedExclusiveLock;
//         }
//
//         /* Otherwise, get the key node */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Kcb->KeyHive, Kcb->KeyCell);
//         if (!KeyNode) return SearchFail;
//
//         /* Cleanup the KCB cache */
//         CmpCleanUpKcbValueCache(Kcb);
//
//         /* Sanity checks */
//         ASSERT(!CMP_IS_CELL_CACHED(Kcb->ValueCache.ValueList));
//         ASSERT(!(Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND));
//
//         /* Set the value cache */
//         Kcb->ValueCache.Count = KeyNode->ValueList.Count;
//         Kcb->ValueCache.ValueList = KeyNode->ValueList.List;
//
//         /* Release the cell */
//         HvReleaseCell(Kcb->KeyHive, Kcb->KeyCell);
//     }
//
//     /* Do the search */
//     SearchResult = CmpFindValueByNameFromCache(Kcb,
//                                                ValueName,
//                                                &CachedValue,
//                                                &Index,
//                                                &Value,
//                                                &ValueCached,
//                                                &ValueCellToRelease);
//     if (SearchResult == SearchNeedExclusiveLock)
//     {
//         /* We need the exclusive lock */
//         ASSERT(!CmpIsKcbLockedExclusive(Kcb));
//         ASSERT(ValueCellToRelease == HCELL_NIL);
//         ASSERT(Value == NULL);
//         goto Quickie;
//     }
//     else if (SearchResult == SearchSuccess)
//     {
//         /* Sanity check */
//         ASSERT(Value);
//
//         /* First of all, check if the key size and type matches */
//         if ((Type == Value->Type) &&
//             (DataSize == (Value->DataLength & ~CM_KEY_VALUE_SPECIAL_SIZE)))
//         {
//             /* Check if this is a small key */
//             IsSmall = (DataSize <= CM_KEY_VALUE_SMALL) ? TRUE: FALSE;
//             if (IsSmall)
//             {
//                 /* Compare against the data directly */
//                 Buffer = &Value->Data;
//             }
//             else
//             {
//                 /* Do a search */
//                 SearchResult = CmpGetValueDataFromCache(Kcb,
//                                                         CachedValue,
//                                                         (PCELL_DATA)Value,
//                                                         ValueCached,
//                                                         &Buffer,
//                                                         &BufferAllocated,
//                                                         &CellToRelease);
//                 if (SearchResult != SearchSuccess)
//                 {
//                     /* Sanity checks */
//                     ASSERT(Buffer == NULL);
//                     ASSERT(BufferAllocated == FALSE);
//                     goto Quickie;
//                 }
//             }
//
//             /* Now check the data size */
//             if (DataSize)
//             {
//                 /* Do the compare */
//                 CompareResult = RtlCompareMemory(Buffer,
//                                                  Data,
//                                                  DataSize &
//                                                  ~CM_KEY_VALUE_SPECIAL_SIZE);
//             }
//             else
//             {
//                 /* It's equal */
//                 CompareResult = 0;
//             }
//
//             /* Now check if the compare wasn't equal */
//             if (CompareResult != DataSize) SearchResult = SearchFail;
//         }
//         else
//         {
//             /* The length or type isn't equal */
//             SearchResult = SearchFail;
//         }
//     }
//
// Quickie:
//     /* Release the value cell */
//     if (ValueCellToRelease) HvReleaseCell(Kcb->KeyHive, ValueCellToRelease);
//
//     /* Free the buffer */
//     if (BufferAllocated) CmpFree(Buffer, 0);
//
//     /* Free the cell */
//     if (CellToRelease) HvReleaseCell(Kcb->KeyHive, CellToRelease);
//
//     /* Return the search result */
//     return SearchResult;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
