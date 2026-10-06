use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{IMAGE_RESOURCE_DATA_ENTRY, IMAGE_RESOURCE_DIRECTORY, IMAGE_RESOURCE_DIRECTORY_ENTRY, LDR_ENUM_RESOURCE_INFO, LPCWSTR, PIMAGE_RESOURCE_DATA_ENTRY, PIMAGE_RESOURCE_DIRECTORY, PIMAGE_RESOURCE_DIRECTORY_ENTRY, PLDR_RESOURCE_INFO, RtlImageDirectoryEntryToData, RtlImageNtHeader, RtlImageRvaToVa, wchar_t};

// find_entry is provided per-environment by libsupp (dll/ntdll/rtl/libsupp.c
// for ntdll, the kernel libsupp for ntoskrnl) — declared `NTSTATUS find_entry`
// with NO NTAPI, i.e. __cdecl → must be `extern "C"` (extern "system" would be
// stdcall on i386 and link-fail / corrupt the stack).
extern "C" {
    fn find_entry(
        BaseAddress: *mut core::ffi::c_void,
        info: PLDR_RESOURCE_INFO,
        level: ULONG,
        ret: *mut *mut core::ffi::c_void,
        want_dir: i32,
    ) -> NTSTATUS;
}

const IMAGE_DIRECTORY_ENTRY_RESOURCE: ULONG = 2;
const STATUS_RESOURCE_DATA_NOT_FOUND: NTSTATUS = 0xC0000089u32 as NTSTATUS;
const EXCEPTION_ACCESS_VIOLATION: ULONG = 0xC0000005;
const EXCEPTION_PRIV_INSTRUCTION: ULONG = 0xC0000096;
const EXCEPTION_EXECUTE_HANDLER: i32 = 1;
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;

#[repr(C)]
pub struct IMAGE_RESOURCE_DIR_STRING_U {
    pub Length: u16,
    pub NameString: [u16; 1],
}
type _IMAGE_RESOURCE_DIR_STRING_U = IMAGE_RESOURCE_DIR_STRING_U;

// ---- file-scope preamble (translated) ----
pub const RESOURCE_TYPE_LEVEL: ULONG = 0;
pub const RESOURCE_NAME_LEVEL: ULONG = 1;
pub const RESOURCE_LANGUAGE_LEVEL: ULONG = 2;
pub const RESOURCE_DATA_LEVEL: ULONG = 3;
#[no_mangle]
pub unsafe extern "C" fn page_fault(ExceptionCode: ULONG) -> i32 {
    if ExceptionCode == EXCEPTION_ACCESS_VIOLATION ||
       ExceptionCode == EXCEPTION_PRIV_INSTRUCTION {
        return EXCEPTION_EXECUTE_HANDLER;
    }
    EXCEPTION_CONTINUE_SEARCH
}

unsafe fn is_data_file_module(BaseAddress: *mut core::ffi::c_void) -> i32 {
    ((BaseAddress as usize) & 1) as i32
}
#[no_mangle]
pub unsafe extern "C" fn push_language(list: *mut u16, pos: u32, lang: u16) -> i32 {
    let mut i: u32 = 0;
    while i < pos {
        if *list.add(i as usize) == lang {
            return pos as i32;
        }
        i += 1;
    }
    let mut new_pos = pos;
    *list.add(new_pos as usize) = lang;
    new_pos += 1;
    new_pos as i32
}
#[no_mangle]
pub unsafe extern "C" fn find_first_entry(
    dir: *mut IMAGE_RESOURCE_DIRECTORY,
    root: *mut core::ffi::c_void,
    want_dir: core::ffi::c_int,
) -> *mut IMAGE_RESOURCE_DIRECTORY {
    let entry = dir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let count = (*dir).NumberOfNamedEntries as core::ffi::c_int
        + (*dir).NumberOfIdEntries as core::ffi::c_int;

    let mut pos: core::ffi::c_int = 0;
    while pos < count {
        let e = &*entry.offset(pos as isize);
        let data_is_dir = e.__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory();
        if (data_is_dir == 0) == (want_dir == 0) {
            return (root as *mut u8).add(e.__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize)
                as *mut IMAGE_RESOURCE_DIRECTORY;
        }
        pos += 1;
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn find_entry_by_id(
    dir: PIMAGE_RESOURCE_DIRECTORY,
    id: u16,
    root: *mut core::ffi::c_void,
    want_dir: i32,
) -> PIMAGE_RESOURCE_DIRECTORY {
    let entry = dir.add(1) as PIMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut min: i32 = (*dir).NumberOfNamedEntries as i32;
    let mut max: i32 = min + (*dir).NumberOfIdEntries as i32 - 1;
    while min <= max {
        let pos = (min + max) / 2;
        let e = entry.offset(pos as isize);
        if (*e).__bindgen_anon_1.Id == id {
            let is_dir = (*e).__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory();
            if (is_dir == 0) == (want_dir == 0) {
                return ((root as *mut u8)
                    .offset((*e).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as isize))
                    as PIMAGE_RESOURCE_DIRECTORY;
            }
            break;
        }
        if (*e).__bindgen_anon_1.Id > id {
            max = pos - 1;
        } else {
            min = pos + 1;
        }
    }
    core::ptr::null_mut()
}

unsafe fn CompareResourceString(
    SearchString: *const wchar_t,
    ResourceString: *const IMAGE_RESOURCE_DIR_STRING_U,
) -> i32 {
    let mut p1 = SearchString;
    let mut p2 = core::ptr::addr_of!((*ResourceString).NameString) as *const wchar_t;
    let mut remaining = (*ResourceString).Length as usize;
    let mut chr1: wchar_t;
    let mut chr2: wchar_t;

    while remaining != 0 {
        remaining -= 1;

        chr1 = *p1;
        p1 = p1.add(1);
        chr2 = *p2;
        p2 = p2.add(1);

        if chr1 != chr2 {
            if chr1 >= b'a' as wchar_t && chr1 <= b'z' as wchar_t {
                chr1 -= (b'a' - b'A') as wchar_t;
            }
            if chr2 >= b'a' as wchar_t && chr2 <= b'z' as wchar_t {
                chr2 -= (b'a' - b'A') as wchar_t;
            }

            if chr1 != chr2 {
                return chr1 as i32 - chr2 as i32;
            }
        }
    }

    if *p1 != 0 {
        return 1;
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn find_entry_by_name(
    dir: PIMAGE_RESOURCE_DIRECTORY,
    name: LPCWSTR,
    root: *mut core::ffi::c_void,
    want_dir: i32,
) -> PIMAGE_RESOURCE_DIRECTORY {
    // Exceeds the C (`name & 0xFFFF0000`), which on 64-bit tests bits 16-31
    // only: a name string at ...0000_xxxx was looked up as numeric id xxxx.
    if (name as usize) & !0xFFFF == 0 {
        return find_entry_by_id(dir, ((name as usize) & 0xFFFF) as u16, root, want_dir);
    }

    let entry = dir.add(1) as PIMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut min: i32 = 0;
    let mut max: i32 = (*dir).NumberOfNamedEntries as i32 - 1;

    while min <= max {
        let pos = (min + max) / 2;
        let cur = entry.offset(pos as isize);
        let str = (root as *const u8).add((*cur).__bindgen_anon_1.__bindgen_anon_1.NameOffset() as usize)
            as *const IMAGE_RESOURCE_DIR_STRING_U;
        let res = CompareResourceString(name as *const wchar_t, str);
        if res == 0 {
            if ((*cur).__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory() != 0) == (want_dir != 0) {
                return (root as *mut u8).add((*cur).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize)
                    as PIMAGE_RESOURCE_DIRECTORY;
            }
            break;
        }
        if res < 0 {
            max = pos - 1;
        } else {
            min = pos + 1;
        }
    }

    core::ptr::null_mut()
}
unsafe fn LdrpAccessResource(
    BaseAddress: *mut core::ffi::c_void,
    entry: *mut IMAGE_RESOURCE_DATA_ENTRY,
    ptr: *mut *mut core::ffi::c_void,
    size: *mut u32,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;

    let mut dirsize: u32 = 0;

    if RtlImageDirectoryEntryToData(
        BaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_RESOURCE as u16,
        &mut dirsize,
    )
    .is_null()
    {
        status = STATUS_RESOURCE_DATA_NOT_FOUND;
    } else {
        if !ptr.is_null() {
            if is_data_file_module(BaseAddress) != 0 {
                let mod_: *mut core::ffi::c_void =
                    ((BaseAddress as usize) & !1usize) as *mut core::ffi::c_void;
                *ptr = RtlImageRvaToVa(
                    RtlImageNtHeader(mod_),
                    mod_,
                    (*entry).OffsetToData,
                    core::ptr::null_mut(),
                );
            } else {
                *ptr = (BaseAddress as *mut u8).add((*entry).OffsetToData as usize)
                    as *mut core::ffi::c_void;
            }
        }
        if !size.is_null() {
            *size = (*entry).Size;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn LdrFindResource_U(
    BaseAddress: PVOID,
    ResourceInfo: PLDR_RESOURCE_INFO,
    Level: ULONG,
    ResourceDataEntry: *mut PIMAGE_RESOURCE_DATA_ENTRY,
) -> NTSTATUS {
    let mut res: PVOID = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    status = find_entry(BaseAddress, ResourceInfo, Level, &mut res, 0);
    if NT_SUCCESS(status) {
        core::ptr::write_volatile(ResourceDataEntry, res as PIMAGE_RESOURCE_DATA_ENTRY);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn LdrAccessResource(
    BaseAddress: *mut core::ffi::c_void,
    ResourceDataEntry: PIMAGE_RESOURCE_DATA_ENTRY,
    Resource: *mut *mut core::ffi::c_void,
    Size: *mut u32,
) -> i32 {
    LdrpAccessResource(BaseAddress, ResourceDataEntry, Resource, Size)
}

#[no_mangle]
pub unsafe extern "system" fn LdrFindResourceDirectory_U(
    BaseAddress: PVOID,
    info: PLDR_RESOURCE_INFO,
    level: ULONG,
    addr: *mut PIMAGE_RESOURCE_DIRECTORY,
) -> NTSTATUS {
    let mut res: PVOID = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    status = find_entry(BaseAddress, info, level, &mut res, 1);
    if NT_SUCCESS(status) {
        core::ptr::write_volatile(addr, res as PIMAGE_RESOURCE_DIRECTORY);
    }

    status
}

unsafe fn LdrpCompareResourceNames_U(
    ResourceData: *mut u8,
    Entry: PIMAGE_RESOURCE_DIRECTORY_ENTRY,
    CompareName: usize,
) -> i32 {
    if CompareName <= 0xFFFF {
        (CompareName as i32) - ((*Entry).__bindgen_anon_1.Id as i32)
    } else {
        let ResourceString = ResourceData
            .add((*Entry).__bindgen_anon_1.__bindgen_anon_1.NameOffset() as usize)
            as *mut _IMAGE_RESOURCE_DIR_STRING_U;

        let mut ResourceStringLength = (*ResourceString).Length;

        let mut String1 = core::ptr::addr_of_mut!((*ResourceString).NameString) as *mut u16;
        let mut String2 = CompareName as *mut u16;

        while ResourceStringLength != 0 {
            ResourceStringLength -= 1;

            let Char1 = *String1;
            String1 = String1.add(1);
            let Char2 = *String2;
            String2 = String2.add(1);

            if (Char1 != Char2) || (Char2 == 0) {
                return (Char2 as i32) - (Char1 as i32);
            }
        }

        if *String2 == 0 {
            0
        } else {
            1
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrEnumResources(
    ImageBase: *mut core::ffi::c_void,
    ResourceInfo: PLDR_RESOURCE_INFO,
    Level: u32,
    ResourceCount: *mut u32,
    Resources: *mut LDR_ENUM_RESOURCE_INFO,
) -> i32 {
    const RESOURCE_TYPE_LEVEL: u32 = 0;
    const RESOURCE_NAME_LEVEL: u32 = 1;
    const RESOURCE_LANGUAGE_LEVEL: u32 = 2;
    const IMAGE_DIRECTORY_ENTRY_RESOURCE: u16 = 2;
    const STATUS_RESOURCE_DATA_NOT_FOUND: i32 = 0xC0000089u32 as i32;
    const STATUS_INVALID_IMAGE_FORMAT: i32 = 0xC000007Bu32 as i32;

    let MaxResourceCount: u32 = if !Resources.is_null() { *ResourceCount } else { 0 };

    core::ptr::write_volatile(ResourceCount, 0);

    let mut Size: u32 = 0;
    let ResourceData = RtlImageDirectoryEntryToData(
        ImageBase,
        1u8,
        IMAGE_DIRECTORY_ENTRY_RESOURCE,
        &mut Size,
    ) as *mut u8;

    if ResourceData.is_null() {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }

    let TypeDirectory = ResourceData as PIMAGE_RESOURCE_DIRECTORY;
    let mut TypeEntry = TypeDirectory.add(1) as PIMAGE_RESOURCE_DIRECTORY_ENTRY;

    let NumberOfTypeEntries = (*TypeDirectory).NumberOfNamedEntries as u32
        + (*TypeDirectory).NumberOfIdEntries as u32;

    let mut Status: i32 = STATUS_SUCCESS;
    let mut Count: u32 = 0;

    let mut i: u32 = 0;
    while i < NumberOfTypeEntries {
        if Level > RESOURCE_TYPE_LEVEL {
            let Result = LdrpCompareResourceNames_U(
                ResourceData,
                TypeEntry,
                (*ResourceInfo).Type as usize,
            );
            if Result != 0 {
                i += 1;
                TypeEntry = TypeEntry.add(1);
                continue;
            }
        }

        if (*TypeEntry).__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory() == 0 {
            return STATUS_INVALID_IMAGE_FORMAT;
        }

        let NameDirectory = ResourceData
            .add((*TypeEntry).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize)
            as PIMAGE_RESOURCE_DIRECTORY;
        let mut NameEntry = NameDirectory.add(1) as PIMAGE_RESOURCE_DIRECTORY_ENTRY;

        let NumberOfNameEntries = (*NameDirectory).NumberOfNamedEntries as u32
            + (*NameDirectory).NumberOfIdEntries as u32;

        let mut j: u32 = 0;
        while j < NumberOfNameEntries {
            if Level > RESOURCE_NAME_LEVEL {
                let Result = LdrpCompareResourceNames_U(
                    ResourceData,
                    NameEntry,
                    (*ResourceInfo).Name as usize,
                );
                if Result != 0 {
                    j += 1;
                    NameEntry = NameEntry.add(1);
                    continue;
                }
            }

            if (*NameEntry).__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory() == 0 {
                return STATUS_INVALID_IMAGE_FORMAT;
            }

            let LangDirectory = ResourceData
                .add((*NameEntry).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize)
                as PIMAGE_RESOURCE_DIRECTORY;
            let mut LangEntry = LangDirectory.add(1) as PIMAGE_RESOURCE_DIRECTORY_ENTRY;

            let NumberOfLangEntries = (*LangDirectory).NumberOfNamedEntries as u32
                + (*LangDirectory).NumberOfIdEntries as u32;

            let mut k: u32 = 0;
            while k < NumberOfLangEntries {
                if Level > RESOURCE_LANGUAGE_LEVEL {
                    let Result = LdrpCompareResourceNames_U(
                        ResourceData,
                        LangEntry,
                        (*ResourceInfo).Language as usize,
                    );
                    if Result != 0 {
                        k += 1;
                        LangEntry = LangEntry.add(1);
                        continue;
                    }
                }

                if (*LangEntry).__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory() != 0 {
                    return STATUS_INVALID_IMAGE_FORMAT;
                }

                let DataEntry = ResourceData
                    .add((*LangEntry).__bindgen_anon_2.OffsetToData as usize)
                    as PIMAGE_RESOURCE_DATA_ENTRY;

                if Count < MaxResourceCount {
                    let res = Resources.add(Count as usize);

                    (*res).Type = (if (*TypeEntry).__bindgen_anon_1.__bindgen_anon_1.NameIsString() != 0 {
                        ResourceData as usize
                            + (*TypeEntry).__bindgen_anon_1.__bindgen_anon_1.NameOffset() as usize
                    } else {
                        (*TypeEntry).__bindgen_anon_1.Id as usize
                    }) as _;

                    (*res).Name = (if (*NameEntry).__bindgen_anon_1.__bindgen_anon_1.NameIsString() != 0 {
                        ResourceData as usize
                            + (*NameEntry).__bindgen_anon_1.__bindgen_anon_1.NameOffset() as usize
                    } else {
                        (*NameEntry).__bindgen_anon_1.Id as usize
                    }) as _;

                    (*res).Language = (if (*LangEntry).__bindgen_anon_1.__bindgen_anon_1.NameIsString() != 0 {
                        ResourceData as usize
                            + (*LangEntry).__bindgen_anon_1.__bindgen_anon_1.NameOffset() as usize
                    } else {
                        (*LangEntry).__bindgen_anon_1.Id as usize
                    }) as _;

                    (*res).Data = (ImageBase as *mut u8).add((*DataEntry).OffsetToData as usize) as *mut core::ffi::c_void;
                    (*res).Reserved = 0;
                    (*res).Size = (*DataEntry).Size;
                } else {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                }

                Count += 1;
                k += 1;
                LangEntry = LangEntry.add(1);
            }

            j += 1;
            NameEntry = NameEntry.add(1);
        }

        i += 1;
        TypeEntry = TypeEntry.add(1);
    }

    core::ptr::write_volatile(ResourceCount, Count);
    Status
}
