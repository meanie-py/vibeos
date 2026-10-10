use crate::apphelp::sdbapi::SdbCloseDatabase;
use crate::apphelp::sdbstringtable::SdbpAddStringToTable;
use crate::apphelp::sdbapi::SdbpCheckTagIDType;
use crate::apphelp::sdbapi::SdbpCheckTagType;
use crate::apphelp::sdbapi::SdbpCloseMemMappedFile;
use crate::apphelp::sdbapi::SdbpCreate;
use crate::apphelp::sdbapi::SdbpFlush;
use crate::apphelp::sdbapi::SdbpOpenMemMappedFile;
use crate::apphelp::sdbapi::SdbpStrsize;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn strlen(s: *const i8) -> usize;
}
extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

// ---- file-scope preamble (translated) ----
// No file-scope data declarations (types, constants, or tables) in this preamble. The prototypes are functions and the `SdbWriteBinaryTagFromFile` definition is a function, so both are handled per-function.
unsafe extern "system" fn SdbpWrite(pdb: PDB, data: *const ::core::ffi::c_void, size: DWORD) {
    if (*pdb).write_iter.wrapping_add(size) > (*pdb).size {
        let mut oldSize: DWORD = (*pdb).size;
        while (*pdb).size < (*pdb).write_iter.wrapping_add(size) {
            (*pdb).size <<= 1;
        }
        (*pdb).data = SdbReAlloc!(
            (*pdb).data as LPVOID,
            (*pdb).size as SIZE_T,
            oldSize as SIZE_T,
        ) as *mut BYTE;
    }
    memcpy(
        (*pdb).data.add((*pdb).write_iter as usize) as *mut ::core::ffi::c_void,
        data,
        size as usize,
    );
    (*pdb).write_iter = (*pdb).write_iter.wrapping_add(size);
}
unsafe extern "system" fn SdbpGetOrAddStringRef(
    mut pdb: PDB,
    mut string: LPCWSTR,
    mut tagid: *mut TAGID,
) -> BOOL {
    let mut buf: PDB = (*pdb).string_buffer as PDB;
    if (*pdb).for_write == 0 {
        RtlAssert(
            b"pdb->for_write\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"dll/appcompat/apphelp/sdbwrite.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            58 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*pdb).string_buffer.is_null() {
        buf = SdbAlloc!((core::mem::size_of::<DB>()) as SIZE_T) as PDB;
        (*pdb).string_buffer = buf as *mut _DB;
        if buf.is_null() {
            return (FALSE) as BOOL;
        }
        (*buf).size = 128 as DWORD;
        (*buf).data = SdbAlloc!((*buf).size as SIZE_T) as *mut BYTE;
        (*buf).for_write = TRUE as BOOL;
        if (*buf).data.is_null() {
            return (FALSE) as BOOL;
        }
    }
    *tagid = ((*buf).write_iter as usize)
        .wrapping_add(core::mem::size_of::<TAG>() as usize)
        .wrapping_add(core::mem::size_of::<DWORD>() as usize) as TAGID;
    if SdbpAddStringToTable(
        core::ptr::addr_of_mut!((*pdb).string_lookup),
        string as *const WCHAR,
        tagid,
    ) != 0
    {
        return SdbWriteStringTag(buf, TAG_STRINGTABLE_ITEM as TAG, string);
    }
    return ((*pdb).string_lookup != core::ptr::null_mut()) as BOOL;
}
unsafe extern "system" fn SdbpWriteStringtable(mut pdb: PDB) -> BOOL {
    let mut table: TAGID = 0;
    let mut buf: PDB = (*pdb).string_buffer as PDB;
    if buf.is_null() || (*pdb).string_lookup.is_null() {
        return (FALSE) as BOOL;
    }
    table = SdbBeginWriteListTag(pdb, TAG_STRINGTABLE as TAG);
    SdbpWrite(
        pdb,
        (*buf).data as *const ::core::ffi::c_void,
        (*buf).write_iter,
    );
    return SdbEndWriteListTag(pdb, table);
}

#[no_mangle]
pub unsafe extern "system" fn SdbCreateDatabase(path: LPCWSTR, type_: PATH_TYPE) -> PDB {
    static mut version_major: DWORD = 2;
    static mut version_minor: DWORD = 1;
    static mut magic: *const i8 = b"sdbf\0".as_ptr() as *const i8;
    let mut pdb: PDB;

    pdb = SdbpCreate(path, type_, TRUE as BOOL);
    if pdb.is_null() {
        return core::ptr::null_mut();
    }

    (*pdb).size = (core::mem::size_of::<DWORD>() as DWORD)
        .wrapping_add(core::mem::size_of::<DWORD>() as DWORD)
        .wrapping_add(strlen(core::ptr::addr_of!(magic).read()) as DWORD);
    (*pdb).data = SdbAlloc!((*pdb).size as SIZE_T) as *mut BYTE;

    SdbpWrite(
        pdb,
        core::ptr::addr_of!(version_major) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of!(version_minor) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of!(magic).read() as *const core::ffi::c_void,
        strlen(core::ptr::addr_of!(magic).read()) as DWORD,
    );

    return pdb;
}

#[no_mangle]
pub unsafe extern "system" fn SdbCloseDatabaseWrite(pdb: PDB) {
    if (*pdb).for_write == 0 {
        RtlAssert(
            b"pdb->for_write\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"dll/appcompat/apphelp/sdbwrite.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            131 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    SdbpWriteStringtable(pdb);
    SdbpFlush(pdb);
    SdbCloseDatabase(pdb);
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteNULLTag(mut pdb: PDB, mut tag: TAG) -> BOOL {
    if SdbpCheckTagType(tag, (TAG_TYPE_NULL) as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteWORDTag(mut pdb: PDB, mut tag: TAG, mut data: WORD) -> BOOL {
    if SdbpCheckTagType(tag, TAG_TYPE_WORD as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(data) as *const core::ffi::c_void,
        core::mem::size_of::<WORD>() as DWORD,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteDWORDTag(pdb: PDB, tag: TAG, data: DWORD) -> BOOL {
    let mut tag = tag;
    let mut data = data;
    if SdbpCheckTagType(tag, (TAG_TYPE_DWORD) as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(data) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteQWORDTag(pdb: PDB, tag: TAG, data: QWORD) -> BOOL {
    if SdbpCheckTagType(tag, (TAG_TYPE_QWORD) as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpWrite(
        pdb,
        core::ptr::addr_of!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of!(data) as *const core::ffi::c_void,
        core::mem::size_of::<QWORD>() as DWORD,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteStringTag(mut pdb: PDB, mut tag: TAG, mut string: LPCWSTR) -> BOOL {
    let mut size: DWORD = 0;
    if SdbpCheckTagType(tag, (TAG_TYPE_STRINGREF) as WORD) != 0 {
        let mut tagid: TAGID = 0;
        if SdbpGetOrAddStringRef(pdb, string, core::ptr::addr_of_mut!(tagid)) == 0 {
            return (FALSE) as BOOL;
        }
        return SdbWriteStringRefTag(pdb, tag, tagid);
    }
    if SdbpCheckTagType(tag, (TAG_TYPE_STRING) as WORD) == 0 {
        return (FALSE) as BOOL;
    }
    size = SdbpStrsize(string);
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const ::core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(size) as *const ::core::ffi::c_void,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    SdbpWrite(pdb, string as *const ::core::ffi::c_void, size);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteStringRefTag(mut pdb: PDB, mut tag: TAG, mut tagid: TAGID) -> BOOL {
    if SdbpCheckTagType(tag, (TAG_TYPE_STRINGREF) as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tagid) as *const core::ffi::c_void,
        core::mem::size_of::<TAGID>() as DWORD,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteBinaryTag(mut pdb: PDB, mut tag: TAG, mut data: *const BYTE, mut size: DWORD) -> BOOL {
    if SdbpCheckTagType(tag, (TAG_TYPE_BINARY) as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(size) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    SdbpWrite(pdb, data as *const core::ffi::c_void, size);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbWriteBinaryTagFromFile(pdb: PDB, tag: TAG, path: LPCWSTR) -> BOOL {
    let mut mapped: MEMMAPPED = core::mem::zeroed();

    if SdbpCheckTagType(tag, (TAG_TYPE_BINARY) as WORD) == 0 {
        return (FALSE) as BOOL;
    }

    if SdbpOpenMemMappedFile(path, core::ptr::addr_of_mut!(mapped)) == 0 {
        return (FALSE) as BOOL;
    }

    SdbWriteBinaryTag(pdb, tag, mapped.view as *const BYTE, mapped.size as DWORD);
    SdbpCloseMemMappedFile(core::ptr::addr_of_mut!(mapped));
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbBeginWriteListTag(mut pdb: PDB, mut tag: TAG) -> TAGID {
    let mut list_id: TAGID;
    let mut dum: DWORD = 0 as DWORD;

    if SdbpCheckTagType(tag, TAG_TYPE_LIST as WORD) == 0 {
        return TAGID_NULL as TAGID;
    }

    list_id = (*pdb).write_iter as TAGID;
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(tag) as *const core::ffi::c_void,
        core::mem::size_of::<TAG>() as DWORD,
    );
    SdbpWrite(
        pdb,
        core::ptr::addr_of_mut!(dum) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as DWORD,
    ); /* reserve some memory for storing list size */
    return list_id;
}

#[no_mangle]
pub unsafe extern "system" fn SdbEndWriteListTag(mut pdb: PDB, mut tagid: TAGID) -> BOOL {
    if (*pdb).for_write == 0 {
        RtlAssert(
            b"pdb->for_write\0" as *const u8 as *const CHAR as PVOID,
            b"dll/appcompat/apphelp/sdbwrite.c\0"
                as *const u8 as *const CHAR as PVOID,
            345 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    if SdbpCheckTagIDType(pdb, tagid, TAG_TYPE_LIST as WORD) == FALSE as BOOL {
        return FALSE as BOOL;
    }

    /* Write size of list to list tag header */
    *((*pdb)
        .data
        .offset((tagid as usize).wrapping_add(core::mem::size_of::<TAG>() as usize) as isize)
        as *mut BYTE as *mut DWORD) = (((*pdb).write_iter as TAGID).wrapping_sub(tagid) as usize)
        .wrapping_sub(core::mem::size_of::<TAG>() as usize)
        .wrapping_sub(core::mem::size_of::<TAGID>() as usize)
        as DWORD;
    return (TRUE) as BOOL;
}
