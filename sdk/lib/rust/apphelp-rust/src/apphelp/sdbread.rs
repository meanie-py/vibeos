use crate::apphelp::sdbapi::SdbIsNullGUID;
use crate::apphelp::sdbapi::SdbpCheckTagIDType;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn SdbpReadData(mut pdb: PDB, mut dest: PVOID, mut offset: DWORD, mut num: DWORD) -> BOOL {
    let mut size: DWORD = offset.wrapping_add(num);
    /* Either overflow or no data to read */
    if size <= offset {
        return FALSE as BOOL;
    }
    /* Overflow */
    if (*pdb).size < size {
        return FALSE as BOOL;
    }
    memcpy(
        dest as *mut core::ffi::c_void,
        (*pdb).data.offset(offset as isize) as *const core::ffi::c_void,
        num as usize,
    );
    return (TRUE) as BOOL;
}
unsafe extern "system" fn SdbpGetTagSize(pdb: PDB, tagid: TAGID) -> DWORD {
    let mut type_: WORD;
    let mut size: DWORD;
    type_ = ((SdbGetTagFromTagID(pdb, tagid) as ::core::ffi::c_int)
        & (TAG_TYPE_MASK as ::core::ffi::c_int)) as WORD;
    if type_ as ::core::ffi::c_int == (TAG_NULL as ::core::ffi::c_int) {
        return 0;
    }

    size = SdbGetTagDataSize(pdb, tagid);
    if (type_ as ::core::ffi::c_int) <= (TAG_TYPE_STRINGREF as ::core::ffi::c_int) {
        size = size.wrapping_add(core::mem::size_of::<TAG>() as DWORD);
        return size;
    } else {
        size = size.wrapping_add(
            (core::mem::size_of::<TAG>()
                + core::mem::size_of::<DWORD>()) as DWORD,
        );
    }

    return size;
}
#[no_mangle]
pub unsafe extern "system" fn SdbpGetString(mut pdb: PDB, mut tagid: TAGID, mut size: PDWORD) -> LPWSTR {
    let mut tag: TAG = 0;
    let mut offset: TAGID = 0;
    tag = SdbGetTagFromTagID(pdb, tagid);
    if tag as ::core::ffi::c_int == (TAG_NULL as i32) {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    if (tag as ::core::ffi::c_int & (TAG_TYPE_MASK as i32)) == (TAG_TYPE_STRINGREF as i32) {
        if (*pdb).stringtable == TAGID_NULL {
            return ::core::ptr::null_mut::<WCHAR>();
        }
        if SdbpReadData(
            pdb,
            ::core::ptr::addr_of_mut!(tagid) as PVOID,
            (tagid as usize).wrapping_add(::core::mem::size_of::<TAG>() as usize) as DWORD,
            ::core::mem::size_of::<TAGID>() as DWORD,
        ) == 0
        {
            return ::core::ptr::null_mut::<WCHAR>();
        }
        offset = ((*pdb).stringtable.wrapping_add(tagid) as usize)
            .wrapping_add(::core::mem::size_of::<TAG>() as usize)
            .wrapping_add(::core::mem::size_of::<TAGID>() as usize) as TAGID;
    } else if (tag as ::core::ffi::c_int & (TAG_TYPE_MASK as i32)) == (TAG_TYPE_STRING as i32) {
        offset = (tagid as usize)
            .wrapping_add(::core::mem::size_of::<TAG>() as usize)
            .wrapping_add(::core::mem::size_of::<TAGID>() as usize) as TAGID;
    } else {
        if g_ShimDebugLevel != 0 {
            crate::apphelp::apphelp::ShimDbgPrint(
                SHIM_ERR,
                b"SdbpGetString\0" as *const u8 as PCSTR,
                b"Tag 0x%u at tagid %u is neither a string or reference to string\n\0" as *const u8
                    as PCSTR,
                tag as ::core::ffi::c_int,
                tagid,
            );
        }
        return ::core::ptr::null_mut::<WCHAR>();
    }
    if !size.is_null()
        && SdbpReadData(
            pdb,
            size as PVOID,
            (offset as usize).wrapping_sub(::core::mem::size_of::<TAGID>() as usize) as DWORD,
            ::core::mem::size_of::<DWORD>() as DWORD,
        ) == 0
    {
        return FALSE as LPWSTR;
    }
    return (*pdb).data.offset(offset as isize) as *mut BYTE as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetTagFromTagID(pdb: PDB, tagid: TAGID) -> TAG {
    let mut data: TAG = 0;
    if SdbpReadData(
        pdb,
        core::ptr::addr_of_mut!(data) as PVOID,
        tagid as DWORD,
        core::mem::size_of::<TAG>() as DWORD,
    ) == 0
    {
        return TAG_NULL as TAG;
    }
    return data;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetTagDataSize(pdb: PDB, tagid: TAGID) -> DWORD {
    /* sizes of data types with fixed size */
    static sizes: [SIZE_T; 6] = [
        0, /* NULL  */ 1, /* BYTE      */
        2, /* WORD  */ 4, /* DWORD     */
        8, /* QWORD */ 4  /* STRINGREF */
    ];
    let mut type_: WORD;
    let mut size: DWORD = 0;

    type_ = (((SdbGetTagFromTagID(pdb, tagid)) as u32) & TAG_TYPE_MASK) as WORD;
    if u32::from(type_) == TAG_NULL {
        return 0;
    }

    if u32::from(type_) <= TAG_TYPE_STRINGREF {
        return sizes[((type_ >> 12) - 1) as usize] as DWORD;
    }

    /* tag with dynamic size (e.g. list): must read size */
    if SdbpReadData(
        pdb,
        core::ptr::addr_of_mut!(size) as PVOID,
        (tagid as DWORD).wrapping_add(core::mem::size_of::<TAG>() as DWORD),
        core::mem::size_of::<DWORD>() as DWORD,
    ) == 0
    {
        return 0;
    }

    return size;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetFirstChild(mut pdb: PDB, mut parent: TAGID) -> TAGID {
    /* if we are at beginning of database */
    if parent == TAGID_ROOT as TAGID {
        /* header only database: no tags */
        if (*pdb).size <= _TAGID_ROOT as DWORD {
            return TAGID_NULL as TAGID;
        }
        /* return *real* root tagid */
        else {
            return _TAGID_ROOT as TAGID;
        }
    }

    /* only list tag can have children */
    if (SdbGetTagFromTagID(pdb, parent) & TAG_TYPE_MASK as TAG) != TAG_TYPE_LIST as TAG {
        return TAGID_NULL as TAGID;
    }

    /* first child is sizeof(core::mem::size_of::<TAG>()) + sizeof(DWORD) bytes after beginning of list */
    return parent
        .wrapping_add(::core::mem::size_of::<TAG>() as TAGID)
        .wrapping_add(::core::mem::size_of::<DWORD>() as TAGID);
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetNextChild(pdb: PDB, parent: TAGID, prev_child: TAGID) -> TAGID {
    let mut next_child: TAGID;
    let mut prev_child_size: DWORD;
    let mut parent_size: DWORD;

    prev_child_size = SdbpGetTagSize(pdb, prev_child);
    if prev_child_size == 0 {
        return TAGID_NULL;
    }

    /* Bound check */
    next_child = (prev_child as DWORD).wrapping_add(prev_child_size) as TAGID;
    if next_child >= (*pdb).size {
        return TAGID_NULL;
    }

    if parent == TAGID_ROOT {
        return next_child;
    }

    parent_size = SdbpGetTagSize(pdb, parent);
    if parent_size == 0 {
        return TAGID_NULL;
    }

    /* Specified parent has no more children */
    if next_child >= (parent as DWORD).wrapping_add(parent_size) {
        return TAGID_NULL;
    }

    return next_child;
}

#[no_mangle]
pub unsafe extern "system" fn SdbFindFirstTag(pdb: PDB, parent: TAGID, tag: TAG) -> TAGID {
    let mut iter: TAGID;

    iter = SdbGetFirstChild(pdb, parent);
    while iter != TAGID_NULL {
        if SdbGetTagFromTagID(pdb, iter) == tag {
            return iter;
        }
        iter = SdbGetNextChild(pdb, parent, iter);
    }
    return TAGID_NULL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbFindNextTag(pdb: PDB, parent: TAGID, prev_child: TAGID) -> TAGID {
    let mut tag: TAG;
    let mut iter: TAGID;

    tag = SdbGetTagFromTagID(pdb, prev_child);
    iter = SdbGetNextChild(pdb, parent, prev_child);

    while iter != TAGID_NULL {
        if SdbGetTagFromTagID(pdb, iter) == tag {
            return iter;
        }
        iter = SdbGetNextChild(pdb, parent, iter);
    }
    return TAGID_NULL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbReadStringTag(pdb: PDB, tagid: TAGID, buffer: LPWSTR, size: DWORD) -> BOOL {
    let mut string: LPWSTR = core::ptr::null_mut();
    let mut string_size: DWORD = 0;

    string = SdbpGetString(pdb, tagid, core::ptr::addr_of_mut!(string_size));
    if string.is_null() {
        return FALSE as BOOL;
    }

    /* Check if buffer is too small */
    if (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) < string_size as usize {
        return FALSE as BOOL;
    }

    memcpy(
        buffer as *mut core::ffi::c_void,
        string as *const core::ffi::c_void,
        string_size as usize,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbReadWORDTag(pdb: PDB, tagid: TAGID, mut ret: WORD) -> WORD {
    if SdbpCheckTagIDType(pdb, tagid, TAG_TYPE_WORD as WORD) != 0 {
        SdbpReadData(
            pdb,
            core::ptr::addr_of_mut!(ret) as PVOID,
            (tagid as DWORD).wrapping_add(2 as DWORD),
            core::mem::size_of::<WORD>() as DWORD,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SdbReadDWORDTag(mut pdb: PDB, mut tagid: TAGID, mut ret: DWORD) -> DWORD {
    if SdbpCheckTagIDType(pdb, tagid, TAG_TYPE_DWORD as WORD) != 0 {
        SdbpReadData(
            pdb,
            core::ptr::addr_of_mut!(ret) as PVOID,
            (tagid as DWORD).wrapping_add(2 as DWORD),
            core::mem::size_of::<DWORD>() as DWORD,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SdbReadQWORDTag(mut pdb: PDB, mut tagid: TAGID, mut ret: QWORD) -> QWORD {
    if SdbpCheckTagIDType(pdb, tagid, TAG_TYPE_QWORD as WORD) != 0 {
        SdbpReadData(
            pdb,
            core::ptr::addr_of_mut!(ret) as PVOID,
            (tagid as usize).wrapping_add(core::mem::size_of::<TAG>() as usize) as DWORD,
            core::mem::size_of::<QWORD>() as DWORD,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SdbReadBinaryTag(pdb: PDB, tagid: TAGID, buffer: PBYTE, size: DWORD) -> BOOL {
    let mut data_size: DWORD = 0;

    if SdbpCheckTagIDType(pdb, tagid, TAG_TYPE_BINARY as WORD) != 0 {
        SdbpReadData(
            pdb,
            core::ptr::addr_of_mut!(data_size) as PVOID,
            (tagid as usize).wrapping_add(core::mem::size_of::<TAG>() as usize) as DWORD,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if size >= data_size {
            return SdbpReadData(
                pdb,
                buffer as PVOID,
                (tagid as usize)
                    .wrapping_add(core::mem::size_of::<TAG>() as usize)
                    .wrapping_add(core::mem::size_of::<DWORD>() as usize) as DWORD,
                data_size,
            );
        }
    }

    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetBinaryTagData(pdb: PDB, tagid: TAGID) -> PVOID {
    if SdbpCheckTagIDType(pdb, tagid, TAG_TYPE_BINARY as WORD) == 0 {
        return core::ptr::null_mut();
    }
    return (*pdb).data.offset(
        (tagid as usize)
            .wrapping_add(core::mem::size_of::<TAG>())
            .wrapping_add(core::mem::size_of::<DWORD>()) as isize,
    ) as *mut BYTE as PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetStringTagPtr(pdb: PDB, tagid: TAGID) -> LPWSTR {
    SdbpGetString(pdb, tagid, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetDatabaseID(pdb: PDB, Guid: *mut GUID) -> BOOL {
    if SdbIsNullGUID(core::ptr::addr_of_mut!((*pdb).database_id)) != 0 {
        let root: TAGID = SdbFindFirstTag(pdb, TAGID_ROOT, TAG_DATABASE as TAG);
        if root != TAGID_NULL {
            let id: TAGID = SdbFindFirstTag(pdb, root, TAG_DATABASE_ID as TAG);
            if id != TAGID_NULL {
                if SdbReadBinaryTag(
                    pdb,
                    id,
                    core::ptr::addr_of_mut!((*pdb).database_id) as PBYTE,
                    core::mem::size_of::<GUID>() as DWORD,
                ) == 0
                {
                    memset(
                        core::ptr::addr_of_mut!((*pdb).database_id) as *mut core::ffi::c_void,
                        0,
                        core::mem::size_of::<GUID>() as usize,
                    );
                }
            } else {
                /* Should we silence this if we are opening a system db? */
                SHIM_ERR!(b"Failed to get the database id\n\0".as_ptr() as *const i8);
            }
        } else {
            /* Should we silence this if we are opening a system db? */
            SHIM_ERR!(b"Failed to get root tag\n\0".as_ptr() as *const i8);
        }
    }
    if SdbIsNullGUID(core::ptr::addr_of_mut!((*pdb).database_id)) == 0 {
        memcpy(
            Guid as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*pdb).database_id) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>() as usize,
        );
        return TRUE as BOOL;
    }
    return (FALSE) as BOOL;
}
