// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7A63DD943648: [u16; 5] = [0x0025u16, 0x002E, 0x002A, 0x0053, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn IniCacheFreeKey(Key: PINI_KEYWORD) {
    RemoveEntryList(&raw mut (*Key).ListEntry);

    if !(*Key).Name.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Key).Name as PVOID);
    }
    if !(*Key).Data.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Key).Data as PVOID);
    }
    RtlFreeHeap(ProcessHeap, 0, Key as PVOID);
}

unsafe extern "C" fn IniCacheFreeSection(mut Section: PINI_SECTION) {
    RemoveEntryList(&raw mut (*Section).ListEntry);
    while IsListEmpty(&raw mut (*Section).KeyList) == false {
        let mut Entry: PLIST_ENTRY = RemoveHeadList(&raw mut (*Section).KeyList);
        let mut Key: PINI_KEYWORD = CONTAINING_RECORD!(Entry, INI_KEYWORD, ListEntry);
        IniCacheFreeKey(Key);
    }
    if !(*Section).Name.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Section).Name as PVOID);
    }
    RtlFreeHeap(ProcessHeap, 0, Section as PVOID);
}

unsafe extern "C" fn IniCacheFindSection(
    mut Cache: PINICACHE,
    mut Name: PCWSTR,
) -> PINI_SECTION {
    let mut Entry: PLIST_ENTRY = (*Cache).SectionList.Flink;
    while Entry != &raw mut (*Cache).SectionList {
        let Section: PINI_SECTION = CONTAINING_RECORD!(Entry, INI_SECTION, ListEntry);
        if _wcsicmp((*Section).Name, Name) == 0 {
            return Section;
        }
        Entry = (*Entry).Flink;
    }
    return core::ptr::null_mut();
}
unsafe extern "C" fn IniCacheFindKey(Section: PINI_SECTION, Name: PCWSTR) -> PINI_KEYWORD {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    Entry = (*Section).KeyList.Flink as PLIST_ENTRY;
    while Entry != core::ptr::addr_of_mut!((*Section).KeyList) {
        let Key: PINI_KEYWORD = CONTAINING_RECORD!(Entry, INI_KEYWORD, ListEntry);
        if _wcsicmp((*Key).Name as PCWCH, Name as PCWCH) == 0 {
            return Key;
        }
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn IniCacheAddKeyAorW(
    Section: PINI_SECTION,
    AnchorKey: PINI_KEYWORD,
    InsertionType: INSERTION_TYPE,
    Name: *const core::ffi::c_void,
    NameLength: ULONG,
    Data: *const core::ffi::c_void,
    DataLength: ULONG,
    IsUnicode: BOOLEAN,
) -> PINI_KEYWORD {
    let mut Key: PINI_KEYWORD = core::ptr::null_mut();
    let mut NameU: PWSTR = core::ptr::null_mut();
    let mut DataU: PWSTR = core::ptr::null_mut();

    if Section.is_null()
        || Name.is_null()
        || NameLength == 0
        || Data.is_null()
        || DataLength == 0
    {
        DPRINT!("Invalid parameter\n");
        return core::ptr::null_mut();
    }

    NameU = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((NameLength + 1) * core::mem::size_of::<u16>() as ULONG) as SIZE_T,
    ) as PWSTR;
    if NameU.is_null() {
        DPRINT!("RtlAllocateHeap() failed\n");
        return core::ptr::null_mut();
    }

    if IsUnicode != 0 {
        wcsncpy(NameU, Name as PCWCH, (NameLength) as _);
    } else {
        _snwprintf(NameU, (NameLength) as _, RUSTCONV_WSTR_7A63DD943648.as_ptr(), NameLength, Name as PCCH);
    }
    *NameU.offset(NameLength as isize) = UNICODE_NULL;

    Key = IniCacheFindKey(Section, NameU);
    if !Key.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (NameU).cast());

        DataU = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((DataLength + 1) * core::mem::size_of::<u16>() as ULONG) as SIZE_T,
        ) as PWSTR;
        if DataU.is_null() {
            DPRINT!("RtlAllocateHeap() failed\n");
            return core::ptr::null_mut();
        }

        if IsUnicode != 0 {
            wcsncpy(DataU, Data as PCWCH, (DataLength) as _);
        } else {
            _snwprintf(DataU, (DataLength) as _, RUSTCONV_WSTR_7A63DD943648.as_ptr(), DataLength, Data as PCCH);
        }
        *DataU.offset(DataLength as isize) = UNICODE_NULL;

        RtlFreeHeap(ProcessHeap, 0, ((*Key).Data).cast());
        (*Key).Data = DataU;

        return Key;
    }

    Key = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<INI_KEYWORD>()) as SIZE_T,
    ) as PINI_KEYWORD;
    if Key.is_null() {
        DPRINT!("RtlAllocateHeap() failed\n");
        RtlFreeHeap(ProcessHeap, 0, (NameU).cast());
        return core::ptr::null_mut();
    }
    (*Key).Name = NameU;

    DataU = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((DataLength + 1) * core::mem::size_of::<u16>() as ULONG) as SIZE_T,
    ) as PWSTR;
    if DataU.is_null() {
        DPRINT!("RtlAllocateHeap() failed\n");
        RtlFreeHeap(ProcessHeap, 0, (NameU).cast());
        RtlFreeHeap(ProcessHeap, 0, (Key).cast());
        return core::ptr::null_mut();
    }

    if IsUnicode != 0 {
        wcsncpy(DataU, Data as PCWCH, (DataLength) as _);
    } else {
        _snwprintf(DataU, (DataLength) as _, RUSTCONV_WSTR_7A63DD943648.as_ptr(), DataLength, Data as PCCH);
    }
    *DataU.offset(DataLength as isize) = UNICODE_NULL;
    (*Key).Data = DataU;

    if IsListEmpty(&raw mut (*Section).KeyList) != false {
        InsertHeadList(&raw mut (*Section).KeyList, &raw mut (*Key).ListEntry);
    } else if InsertionType == INSERT_FIRST
        || (InsertionType == INSERT_BEFORE
            && (AnchorKey.is_null() || &raw mut (*AnchorKey).ListEntry == (*Section).KeyList.Flink))
    {
        InsertHeadList(&raw mut (*Section).KeyList, &raw mut (*Key).ListEntry);
    } else if InsertionType == INSERT_BEFORE && !AnchorKey.is_null() {
        InsertTailList(&raw mut (*AnchorKey).ListEntry, &raw mut (*Key).ListEntry);
    } else if InsertionType == INSERT_LAST
        || (InsertionType == INSERT_AFTER
            && (AnchorKey.is_null() || &raw mut (*AnchorKey).ListEntry == (*Section).KeyList.Blink))
    {
        InsertTailList(&raw mut (*Section).KeyList, &raw mut (*Key).ListEntry);
    } else if InsertionType == INSERT_AFTER && !AnchorKey.is_null() {
        InsertHeadList(&raw mut (*AnchorKey).ListEntry, &raw mut (*Key).ListEntry);
    }

    Key
}

unsafe extern "C" fn IniCacheAddSectionAorW(
    Cache: PINICACHE,
    Name: *const core::ffi::c_void,
    NameLength: u32,
    IsUnicode: u8,
) -> PINI_SECTION {
    let mut Section: PINI_SECTION = core::ptr::null_mut();
    let mut NameU: PWSTR = core::ptr::null_mut();

    if Cache.is_null() || Name.is_null() || NameLength == 0 {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    NameU = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((NameLength + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as PWSTR;
    if NameU.is_null() {
        DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    if IsUnicode != 0 {
        wcsncpy(NameU, Name as *const u16, (NameLength) as _);
    } else {
        _snwprintf(NameU, (NameLength) as _, RUSTCONV_WSTR_7A63DD943648.as_ptr(), NameLength, Name as *const i8);
    }
    *NameU.offset(NameLength as isize) = UNICODE_NULL;

    Section = IniCacheFindSection(Cache, NameU);
    if !Section.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (NameU).cast());
        return Section;
    }

    Section = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<INI_SECTION>()) as SIZE_T,
    ) as PINI_SECTION;
    if Section.is_null() {
        DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
        RtlFreeHeap(ProcessHeap, 0, (NameU).cast());
        return core::ptr::null_mut();
    }

    (*Section).Name = NameU;
    InitializeListHead(&raw mut (*Section).KeyList);

    InsertTailList(&raw mut (*Cache).SectionList, &raw mut (*Section).ListEntry);

    Section
}

unsafe extern "C" fn IniCacheSkipWhitespace(mut Ptr: PCHAR) -> PCHAR {
    while *Ptr as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && isspace(*Ptr as ::core::ffi::c_int) != 0
    {
        Ptr = Ptr.offset(1);
    }
    return if *Ptr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        ::core::ptr::null_mut::<CHAR>()
    } else {
        Ptr
    };
}

unsafe extern "C" fn IniCacheSkipToNextSection(mut Ptr: PCHAR) -> PCHAR {
    while *Ptr as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && *Ptr as ::core::ffi::c_int != '[' as i32
    {
        while *Ptr as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && *Ptr as ::core::ffi::c_int != 0x000Au16 as ::core::ffi::c_int
        {
            Ptr = Ptr.offset(1);
        }
        Ptr = Ptr.offset(1);
    }
    return if *Ptr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        ::core::ptr::null_mut::<CHAR>()
    } else {
        Ptr
    };
}

unsafe extern "C" fn IniCacheGetSectionName(
    mut Ptr: PCHAR,
    mut NamePtr: *mut PCHAR,
    mut NameSize: PULONG,
) -> PCHAR {
    let mut Size: ULONG = 0;
    *NamePtr = core::ptr::null_mut();
    *NameSize = 0;
    while *Ptr as i32 != 0 && isspace(*Ptr as i32) != 0 {
        Ptr = Ptr.offset(1);
    }
    *NamePtr = Ptr;
    while *Ptr as i32 != 0 && *Ptr as i32 != ']' as i32 {
        Size = Size.wrapping_add(1);
        Ptr = Ptr.offset(1);
    }
    Ptr = Ptr.offset(1);
    while *Ptr as i32 != 0 && *Ptr as i32 != 0x000Au16 as i32 {
        Ptr = Ptr.offset(1);
    }
    Ptr = Ptr.offset(1);
    *NameSize = Size;
    DPRINT!(b"SectionName: '%.*s'\n\0".as_ptr() as PCCH, Size, *NamePtr);
    return Ptr;
}

unsafe extern "C" fn IniCacheGetKeyName(
    mut Ptr: PCHAR,
    NamePtr: *mut PCHAR,
    NameSize: PULONG,
) -> PCHAR {
    let mut Size: ULONG = 0;

    *NamePtr = core::ptr::null_mut();
    *NameSize = 0;

    while !Ptr.is_null() && *Ptr != 0 {
        *NamePtr = core::ptr::null_mut();
        *NameSize = 0;
        Size = 0;

        while isspace(*Ptr as i32) != 0 || *Ptr == '\n' as i8 || *Ptr == '\r' as i8 {
            Ptr = Ptr.offset(1);
        }
        if *Ptr == 0 {
            continue;
        }

        *NamePtr = Ptr;

        while *Ptr != 0 && isspace(*Ptr as i32) == 0 && *Ptr != '=' as i8 && *Ptr != ';' as i8 {
            Size = Size.wrapping_add(1);
            Ptr = Ptr.offset(1);
        }
        if *Ptr == ';' as i8 {
            while *Ptr != 0 && *Ptr != '\r' as i8 && *Ptr != '\n' as i8 {
                Ptr = Ptr.offset(1);
            }
        } else {
            *NameSize = Size;
            break;
        }
    }

    return Ptr;
}

unsafe extern "C" fn IniCacheGetKeyValue(
    mut Ptr: PCHAR,
    DataPtr: *mut PCHAR,
    DataSize: PULONG,
    String: BOOLEAN,
) -> PCHAR {
    let mut Size: ULONG = 0;

    *DataPtr = core::ptr::null_mut();
    *DataSize = 0;

    while *Ptr as i32 != 0 && isspace(*Ptr as i32) != 0 {
        Ptr = Ptr.offset(1);
    }

    if *Ptr as i32 != '=' as i32 {
        return core::ptr::null_mut();
    }
    Ptr = Ptr.offset(1);

    while *Ptr as i32 != 0 && isspace(*Ptr as i32) != 0 {
        Ptr = Ptr.offset(1);
    }

    if *Ptr as i32 == '"' as i32 && String != 0 {
        Ptr = Ptr.offset(1);

        *DataPtr = Ptr;
        while *Ptr as i32 != '"' as i32 {
            Ptr = Ptr.offset(1);
            Size = Size.wrapping_add(1);
        }
        Ptr = Ptr.offset(1);

        while *Ptr as i32 != 0 && *Ptr as i32 != '\r' as i32 && *Ptr as i32 != '\n' as i32 {
            Ptr = Ptr.offset(1);
        }
    } else {
        *DataPtr = Ptr;
        while *Ptr as i32 != 0 && *Ptr as i32 != '\r' as i32 && *Ptr as i32 != ';' as i32 {
            Ptr = Ptr.offset(1);
            Size = Size.wrapping_add(1);
        }
    }

    if *Ptr as i32 == '\r' as i32 {
        Ptr = Ptr.offset(1);
    }
    if *Ptr as i32 == '\n' as i32 {
        Ptr = Ptr.offset(1);
    }

    *DataSize = Size;

    return Ptr;
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheLoadFromMemory(
    Cache: *mut PINICACHE,
    FileBuffer: PCHAR,
    FileLength: ULONG,
    String: BOOLEAN,
) -> NTSTATUS {
    let mut Ptr: PCHAR;

    let mut Section: PINI_SECTION;
    let mut Key: PINI_KEYWORD;

    let mut SectionName: PCHAR = core::mem::zeroed();
    let mut SectionNameSize: ULONG = core::mem::zeroed();

    let mut KeyName: PCHAR = core::mem::zeroed();
    let mut KeyNameSize: ULONG = core::mem::zeroed();

    let mut KeyValue: PCHAR = core::mem::zeroed();
    let mut KeyValueSize: ULONG = core::mem::zeroed();

    /* Allocate inicache header */
    *Cache = IniCacheCreate();
    if (*Cache).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Parse ini file */
    Section = core::ptr::null_mut();
    Ptr = FileBuffer;
    while !Ptr.is_null() && *Ptr as i32 != 0 {
        Ptr = IniCacheSkipWhitespace(Ptr);
        if Ptr.is_null() {
            continue;
        }

        if *Ptr as i32 == '[' as i32 {
            Section = core::ptr::null_mut();
            Ptr = Ptr.add(1);

            Ptr = IniCacheGetSectionName(Ptr, &mut SectionName, &mut SectionNameSize);

            DPRINT!(b"[%.*s]\n\0".as_ptr() as *const i8, SectionNameSize, SectionName);

            Section = IniCacheAddSectionAorW(
                *Cache,
                SectionName as *const core::ffi::c_void,
                SectionNameSize,
                (FALSE) as u8,
            );
            if Section.is_null() {
                DPRINT!(b"IniCacheAddSectionAorW() failed\n\0".as_ptr() as *const i8);
                Ptr = IniCacheSkipToNextSection(Ptr);
                continue;
            }
        } else {
            if Section.is_null() {
                Ptr = IniCacheSkipToNextSection(Ptr);
                continue;
            }

            Ptr = IniCacheGetKeyName(Ptr, &mut KeyName, &mut KeyNameSize);

            Ptr = IniCacheGetKeyValue(Ptr, &mut KeyValue, &mut KeyValueSize, String);

            DPRINT!(
                b"'%.*s' = '%.*s'\n\0".as_ptr() as *const i8,
                KeyNameSize,
                KeyName,
                KeyValueSize,
                KeyValue,
            );

            Key = IniCacheAddKeyAorW(
                Section,
                core::ptr::null_mut(),
                INSERT_LAST,
                KeyName as *const core::ffi::c_void,
                KeyNameSize,
                KeyValue as *const core::ffi::c_void,
                KeyValueSize,
                (FALSE) as BOOLEAN,
            );
            if Key.is_null() {
                DPRINT!(b"IniCacheAddKeyAorW() failed\n\0".as_ptr() as *const i8);
            }
        }
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheLoadByHandle(
    Cache: *mut PINICACHE,
    FileHandle: HANDLE,
    String: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut FileBuffer: PCHAR;
    let mut FileLength: ULONG;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();

    *Cache = core::ptr::null_mut();

    Status = NtQueryInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut FileInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        FileStandardInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtQueryInformationFile() failed (Status %lx)\n", Status);
        return Status;
    }

    FileLength = FileInfo.EndOfFile.u.LowPart;

    DPRINT!("File size: %lu\n", FileLength);

    FileBuffer = RtlAllocateHeap(
        ProcessHeap,
        0,
        (FileLength + 1) as SIZE_T,
    ) as PCHAR;
    if FileBuffer.is_null() {
        DPRINT1!("RtlAllocateHeap() failed\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    FileOffset.QuadPart = 0;
    Status = NtReadFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        FileBuffer as *mut core::ffi::c_void,
        FileLength,
        &mut FileOffset,
        core::ptr::null_mut(),
    );

    *FileBuffer.offset(FileLength as isize) = 0;

    if NT_SUCCESS!(Status) != 0 {
        Status = IniCacheLoadFromMemory(Cache, FileBuffer, FileLength, String);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("IniCacheLoadFromMemory() failed (Status %lx)\n", Status);
        }
    }

    RtlFreeHeap(ProcessHeap, 0, (FileBuffer).cast());
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheLoad(
    Cache: *mut PINICACHE,
    FileName: PWSTR,
    String: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileHandle: HANDLE = core::ptr::null_mut();

    *Cache = core::ptr::null_mut();

    RtlInitUnicodeString(&mut Name, FileName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        FILE_GENERIC_READ | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtOpenFile() failed (Status %lx)\n", Status);
        return Status;
    }

    DPRINT!("NtOpenFile() successful\n");

    Status = IniCacheLoadByHandle(Cache, FileHandle, String);

    NtClose(FileHandle);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheDestroy(Cache: PINICACHE) {
    if Cache.is_null() {
        return;
    }

    while IsListEmpty(&raw mut (*Cache).SectionList) == false {
        let Entry: PLIST_ENTRY = RemoveHeadList(&raw mut (*Cache).SectionList);
        let Section: PINI_SECTION = CONTAINING_RECORD!(Entry, INI_SECTION, ListEntry);
        IniCacheFreeSection(Section);
    }

    RtlFreeHeap(ProcessHeap, 0, (Cache).cast());
}
#[no_mangle]
pub unsafe extern "C" fn IniGetSection(
    Cache: PINICACHE,
    Name: PCWSTR,
) -> PINI_SECTION {
    if Cache.is_null() || Name.is_null() {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }
    return IniCacheFindSection(Cache, Name);
}
#[no_mangle]
pub unsafe extern "C" fn IniGetKey(
    Section: PINI_SECTION,
    KeyName: PCWSTR,
    KeyData: *mut PCWSTR,
) -> PINI_KEYWORD {
    let mut Key: PINI_KEYWORD = core::ptr::null_mut();

    if Section.is_null() || KeyName.is_null() || KeyData.is_null() {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    *KeyData = core::ptr::null_mut();

    Key = IniCacheFindKey(Section, KeyName);
    if Key.is_null() {
        return core::ptr::null_mut();
    }

    *KeyData = (*Key).Data;

    return Key;
}
#[no_mangle]
pub unsafe extern "C" fn IniFindFirstValue(
    Section: PINI_SECTION,
    KeyName: *mut PCWSTR,
    KeyData: *mut PCWSTR,
) -> PINICACHEITERATOR {
    let mut Iterator: PINICACHEITERATOR;
    let mut Entry: PLIST_ENTRY;
    let mut Key: PINI_KEYWORD;

    if Section.is_null() || KeyName.is_null() || KeyData.is_null() {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as PCCH);
        return core::ptr::null_mut();
    }

    Entry = (*Section).KeyList.Flink;
    if Entry == core::ptr::addr_of_mut!((*Section).KeyList) {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as PCCH);
        return core::ptr::null_mut();
    }
    Key = CONTAINING_RECORD!(Entry, INI_KEYWORD, ListEntry);

    Iterator = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<INICACHEITERATOR>()) as SIZE_T,
    ) as PINICACHEITERATOR;
    if Iterator.is_null() {
        DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as PCCH);
        return core::ptr::null_mut();
    }
    (*Iterator).Section = Section;
    (*Iterator).Key = Key;

    *KeyName = (*Key).Name;
    *KeyData = (*Key).Data;

    return Iterator;
}
#[no_mangle]
pub unsafe extern "C" fn IniFindNextValue(
    Iterator: PINICACHEITERATOR,
    KeyName: *mut PCWSTR,
    KeyData: *mut PCWSTR,
) -> BOOLEAN {
    let mut Entry: PLIST_ENTRY;
    let mut Key: PINI_KEYWORD;

    if Iterator.is_null() || KeyName.is_null() || KeyData.is_null() {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return (FALSE) as u8;
    }

    Entry = (*(*Iterator).Key).ListEntry.Flink;
    if Entry == &raw mut (*(*Iterator).Section).KeyList {
        DPRINT!(b"No more entries\n\0".as_ptr() as *const i8);
        return (FALSE) as u8;
    }
    Key = CONTAINING_RECORD!(Entry, INI_KEYWORD, ListEntry);

    (*Iterator).Key = Key;

    *KeyName = (*Key).Name;
    *KeyData = (*Key).Data;

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn IniFindClose(Iterator: PINICACHEITERATOR) {
    if Iterator.is_null() {
        return;
    }
    RtlFreeHeap(ProcessHeap, 0, (Iterator).cast());
}
#[no_mangle]
pub unsafe extern "C" fn IniAddSection(
    Cache: PINICACHE,
    Name: PCWSTR,
) -> PINI_SECTION {
    if Cache.is_null() || Name.is_null() || *Name == 0 {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }
    return IniCacheAddSectionAorW(
        Cache,
        Name as *const core::ffi::c_void,
        wcslen(Name) as u32,
        TRUE as u8,
    );
}
#[no_mangle]
pub unsafe extern "C" fn IniRemoveSection(Section: PINI_SECTION) {
    if Section.is_null() {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return;
    }
    IniCacheFreeSection(Section);
}
#[no_mangle]
pub unsafe extern "C" fn IniInsertKey(
    Section: PINI_SECTION,
    AnchorKey: PINI_KEYWORD,
    InsertionType: INSERTION_TYPE,
    Name: PCWSTR,
    Data: PCWSTR,
) -> PINI_KEYWORD {
    if Section.is_null() || Name.is_null() || *Name == 0 || Data.is_null() || *Data == 0 {
        DPRINT!(b"Invalid parameter\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }
    return IniCacheAddKeyAorW(
        Section,
        AnchorKey,
        InsertionType,
        Name as *const core::ffi::c_void,
        wcslen(Name) as ULONG,
        Data as *const core::ffi::c_void,
        wcslen(Data) as ULONG,
        (TRUE) as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "C" fn IniAddKey(
    Section: PINI_SECTION,
    Name: PCWSTR,
    Data: PCWSTR,
) -> PINI_KEYWORD {
    return IniInsertKey(Section, core::ptr::null_mut(), INSERT_LAST, Name, Data);
}
#[no_mangle]
pub unsafe extern "C" fn IniRemoveKeyByName(
    Section: PINI_SECTION,
    KeyName: PCWSTR,
) {
    let mut Key: PINI_KEYWORD = core::ptr::null_mut();
    UNREFERENCED_PARAMETER!(Section);
    Key = IniCacheFindKey(Section, KeyName);
    if !Key.is_null() {
        IniCacheFreeKey(Key);
    }
}
#[no_mangle]
pub unsafe extern "C" fn IniRemoveKey(
    Section: PINI_SECTION,
    Key: PINI_KEYWORD,
) {
    UNREFERENCED_PARAMETER!(Section);
    if Key.is_null() {
        DPRINT!("Invalid parameter\n");
        return;
    }
    IniCacheFreeKey(Key);
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheCreate() -> *mut INICACHE {
    let mut Cache: *mut INICACHE;

    Cache = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<INICACHE>()) as SIZE_T,
    ) as *mut INICACHE;
    if Cache.is_null() {
        DPRINT!(b"RtlAllocateHeap() failed\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }
    InitializeListHead(&mut (*Cache).SectionList);

    return Cache;
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheSaveByHandle(
    Cache: PINICACHE,
    FileHandle: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Entry1: PLIST_ENTRY;
    let mut Entry2: PLIST_ENTRY;
    let mut Section: PINI_SECTION;
    let mut Key: PINI_KEYWORD;
    let mut BufferSize: ULONG;
    let mut Buffer: PCHAR;
    let mut Ptr: PCHAR;
    let mut Len: ULONG;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();

    BufferSize = 0;
    Entry1 = (*Cache).SectionList.Flink;
    while Entry1 != &raw mut (*Cache).SectionList {
        Section = CONTAINING_RECORD!(Entry1, INI_SECTION, ListEntry);
        BufferSize = BufferSize.wrapping_add(
            ((if !(*Section).Name.is_null() {
                wcslen((*Section).Name)
            } else {
                0
            })
            .wrapping_add(4)) as _,
        );

        Entry2 = (*Section).KeyList.Flink;
        while Entry2 != &raw mut (*Section).KeyList {
            Key = CONTAINING_RECORD!(Entry2, INI_KEYWORD, ListEntry);
            BufferSize = BufferSize.wrapping_add(
                (wcslen((*Key).Name)
                    .wrapping_add(if !(*Key).Data.is_null() {
                        wcslen((*Key).Data)
                    } else {
                        0
                    })
                    .wrapping_add(3)) as _,
            );
            Entry2 = (*Entry2).Flink;
        }

        Entry1 = (*Entry1).Flink;
        if Entry1 != &raw mut (*Cache).SectionList {
            BufferSize = BufferSize.wrapping_add(2);
        }
    }

    DPRINT!("BufferSize: %lu\n", BufferSize);

    Buffer = RtlAllocateHeap(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (BufferSize.wrapping_add(1)) as SIZE_T,
    ) as PCHAR;
    if Buffer.is_null() {
        DPRINT1!("RtlAllocateHeap() failed\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Ptr = Buffer;
    Entry1 = (*Cache).SectionList.Flink;
    while Entry1 != &raw mut (*Cache).SectionList {
        Section = CONTAINING_RECORD!(Entry1, INI_SECTION, ListEntry);
        Len = (sprintf(Ptr, c"[%S]\r\n".as_ptr(), (*Section).Name)) as u32;
        Ptr = Ptr.offset(Len as isize);

        Entry2 = (*Section).KeyList.Flink;
        while Entry2 != &raw mut (*Section).KeyList {
            Key = CONTAINING_RECORD!(Entry2, INI_KEYWORD, ListEntry);
            Len = (sprintf(Ptr, c"%S=%S\r\n".as_ptr(), (*Key).Name, (*Key).Data)) as u32;
            Ptr = Ptr.offset(Len as isize);
            Entry2 = (*Entry2).Flink;
        }

        Entry1 = (*Entry1).Flink;
        if Entry1 != &raw mut (*Cache).SectionList {
            Len = (sprintf(Ptr, c"\r\n".as_ptr())) as u32;
            Ptr = Ptr.offset(Len as isize);
        }
    }

    Offset.QuadPart = 0;
    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &raw mut IoStatusBlock,
        Buffer as PVOID,
        BufferSize,
        &raw mut Offset,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
        RtlFreeHeap(ProcessHeap, 0, (Buffer).cast());
        return Status;
    }

    RtlFreeHeap(ProcessHeap, 0, (Buffer).cast());
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn IniCacheSave(
    Cache: PINICACHE,
    FileName: PWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileHandle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(&mut Name, FileName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &mut FileHandle,
        FILE_GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_SUPERSEDE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_SEQUENTIAL_ONLY | FILE_NON_DIRECTORY_FILE,
        core::ptr::null_mut(),
        0,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT!("NtCreateFile() failed (Status %lx)\n", Status);
        return Status;
    }

    Status = IniCacheSaveByHandle(Cache, FileHandle);

    NtClose(FileHandle);
    return Status;
}
