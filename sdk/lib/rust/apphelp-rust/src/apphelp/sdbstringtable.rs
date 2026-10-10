use crate::apphelp::sdbapi::SdbpStrlen;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct SdbHashEntry {
    pub Next: *mut SdbHashEntry,
    pub Tagid: TAGID,
    pub Name: [WCHAR; 1],
}

pub type PSdbHashEntry = *mut SdbHashEntry;

// sdbstringtable.c:39 `struct SdbStringHashTable` is private to that TU (the header only
// forward-declares it), so the island's version is opaque; this is the real layout.
#[repr(C)]
pub struct HashTable {
    pub Size: DWORD,
    pub Entries: *mut *mut SdbHashEntry,
}

pub const DEFAULT_TABLE_SIZE: u32 = 0x100;

unsafe extern "C" fn HashCreate() -> *mut HashTable {
    let mut tab: *mut HashTable =
        SdbAlloc!(core::mem::size_of::<HashTable>() as SIZE_T) as *mut HashTable;
    if tab.is_null() {
        // SHIM_ERR is a log-level constant, not a callable; the message is carried alongside it.
        SHIM_ERR!(b"Failed to allocate 8 bytes.\r\n\0".as_ptr() as *const i8);
        return tab;
    }
    (*tab).Size = DEFAULT_TABLE_SIZE as DWORD;
    (*tab).Entries = SdbAlloc!(
        ((*tab).Size as SIZE_T).wrapping_mul(core::mem::size_of::<SdbHashEntry>() as SIZE_T),
    ) as *mut *mut SdbHashEntry;
    return tab;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpTableDestroy(pTable: *mut *mut SdbStringHashTable) {
    let pTable = pTable as *mut *mut HashTable;
    let mut table: *mut HashTable = *pTable;
    let mut entry: *mut SdbHashEntry = core::ptr::null_mut();
    let mut next: *mut SdbHashEntry = core::ptr::null_mut();
    let mut n: DWORD = 0;
    let mut depth: DWORD = 0;
    let mut once: DWORD = 1;

    *pTable = core::ptr::null_mut();
    n = 0;
    while n < (*table).Size {
        depth = 0;
        next = *(*table).Entries.offset(n as isize);
        entry = next;
        while !entry.is_null() {
            next = (*entry).Next;
            SdbFree!(entry as LPVOID);
            entry = next;
            depth = depth.wrapping_add(1);
        }
        if once != 0 && depth > 3 {
            once = 0;
        }
        n = n.wrapping_add(1);
    }
    SdbFree!((*table).Entries as LPVOID);
    SdbFree!(table as LPVOID);
}

unsafe extern "C" fn StringHash(mut str: *const WCHAR) -> DWORD {
    let mut hash: DWORD = 0 as DWORD;
    while *str != 0 {
        hash = (65599 as ULONG)
            .wrapping_mul(hash as ULONG)
            .wrapping_add(*str as ULONG) as DWORD;
        str = str.offset(1);
    }
    return hash;
}
#[no_mangle]
pub unsafe extern "C" fn Sdbwcscmp(mut s1: *const WCHAR, mut s2: *const WCHAR) -> ::core::ffi::c_int {
    while *s1 == *s2 {
        if *s1 == 0 {
            return 0;
        }
        s1 = s1.offset(1);
        s2 = s2.offset(1);
    }
    return *s1 as ::core::ffi::c_int - *s2 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Sdbwcscpy(
    mut wcDest: *mut WCHAR,
    mut numElement: size_t,
    mut wcSrc: *const WCHAR,
) -> INT {
    let mut size: size_t = 0 as size_t;
    if wcDest.is_null() || numElement == 0 {
        return 22 as INT;
    }
    *wcDest.offset(0 as isize) = 0 as WCHAR;
    if wcSrc.is_null() {
        return 22 as INT;
    }
    size = (SdbpStrlen(wcSrc as PCWSTR) as size_t).wrapping_add(1 as size_t);
    if size > numElement {
        return 34 as INT;
    }
    memcpy(
        wcDest as *mut core::ffi::c_void,
        wcSrc as *const core::ffi::c_void,
        size.wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    return 0 as INT;
}

unsafe extern "C" fn TableFindPtr(
    table: *mut HashTable,
    str: *const WCHAR,
) -> *mut *mut SdbHashEntry {
    let hash: DWORD = StringHash(str);
    let mut entry: *mut *mut SdbHashEntry =
        (*table).Entries.offset((hash % (*table).Size) as isize) as *mut *mut SdbHashEntry;
    while !(*entry).is_null() {
        if Sdbwcscmp(core::ptr::addr_of_mut!((**entry).Name) as *mut WCHAR, str) == 0 {
            return entry;
        }
        entry = core::ptr::addr_of_mut!((**entry).Next);
    }
    return entry;
}

unsafe extern "C" fn HashAddString(mut table: *mut HashTable, mut position: *mut *mut SdbHashEntry, mut str: *const WCHAR, mut tagid: TAGID) -> BOOL {
    let mut entry: *mut SdbHashEntry = core::ptr::null_mut();
    let mut size: SIZE_T = 0;
    let mut len: SIZE_T = 0;
    if position.is_null() {
        position = TableFindPtr(table, str);
    }
    len = (SdbpStrlen(str) as SIZE_T) + 1;
    size = (core::mem::offset_of!(SdbHashEntry, Name) as SIZE_T) + (len * (core::mem::size_of::<WCHAR>() as SIZE_T));
    *position = SdbAlloc!(size) as *mut SdbHashEntry;
    entry = *position;
    if entry.is_null() {
        SHIM_ERR!(b"Failed to allocate %u bytes.\n\0".as_ptr() as *const i8, size);
        return FALSE as BOOL;
    }
    (*entry).Tagid = tagid;
    Sdbwcscpy(core::ptr::addr_of_mut!((*entry).Name) as *mut WCHAR, len as size_t, str);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpAddStringToTable(table: *mut *mut SdbStringHashTable, mut str: *const WCHAR, mut tagid: *mut TAGID) -> BOOL {
    let table = table as *mut *mut HashTable;
    let mut entry: *mut *mut SdbHashEntry = ::core::ptr::null_mut::<*mut SdbHashEntry>();
    if (*table).is_null() {
        *table = HashCreate();
        if (*table).is_null() {
            if g_ShimDebugLevel != 0 {
                crate::apphelp::apphelp::ShimDbgPrint(
                    SHIM_ERR,
                    b"SdbpAddStringToTable\0" as *const u8 as PCSTR,
                    b"Error creating hash table\n\0" as *const u8 as PCSTR,
                );
            }
            return FALSE as BOOL;
        }
    }
    entry = TableFindPtr(*table, str);
    if !(*entry).is_null() {
        *tagid = (**entry).Tagid;
        return (FALSE) as BOOL;
    }
    return HashAddString(*table, entry, str, *tagid);
}
