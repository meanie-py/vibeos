use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut first_hlpfile: *mut HLPFILE = ::core::ptr::null::<HLPFILE>() as *mut HLPFILE;

// WINE_DEFAULT_DEBUG_CHANNEL(winhelp): handled by the crate's DPRINT1!/WARN!/ERR! macros.
// HLPFILE_AllocLink prototype: not emitted here (fn declaration, emitted with the function bodies).

unsafe extern "C" fn GET_USHORT(mut buffer: *const BYTE, mut i: ::core::ffi::c_uint) -> ::core::ffi::c_ushort {
    return ((*buffer.offset(i as isize) as ::core::ffi::c_int)
        + 0x100
            * (*buffer.offset(i.wrapping_add(1) as isize) as ::core::ffi::c_int))
        as ::core::ffi::c_ushort;
}

unsafe extern "C" fn GET_SHORT(buffer: *const BYTE, i: ::core::ffi::c_uint) -> ::core::ffi::c_short {
    return ((*buffer.offset(i as isize) as ::core::ffi::c_int)
        + (0x100 as ::core::ffi::c_int)
            * ((*buffer.offset(i.wrapping_add(1 as ::core::ffi::c_uint) as isize))
                as ::core::ffi::c_schar as ::core::ffi::c_int)) as ::core::ffi::c_short;
}

unsafe extern "C" fn GET_UINT(buffer: *const BYTE, i: u32) -> u32 {
    return (GET_USHORT(buffer, i) as u32
        + 0x10000 * GET_USHORT(buffer, i.wrapping_add(2)) as u32);
}

unsafe extern "C" fn HLPFILE_BPTreeSearch(buf: *mut BYTE, key: *const ::core::ffi::c_void, comp: HLPFILE_BPTreeCompare) -> *mut ::core::ffi::c_void {
    let mut magic: ::core::ffi::c_uint = 0;
    let mut page_size: ::core::ffi::c_uint = 0;
    let mut cur_page: ::core::ffi::c_uint = 0;
    let mut level: ::core::ffi::c_uint = 0;
    let mut pages: *mut BYTE = core::ptr::null_mut();
    let mut ptr: *mut BYTE = core::ptr::null_mut();
    let mut newptr: *mut BYTE = core::ptr::null_mut();
    let mut i: ::core::ffi::c_int = 0;
    let mut entries: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = 0;

    magic = GET_USHORT(buf, 9) as ::core::ffi::c_uint;
    if magic != 0x293B {
        WINE_ERR!(b"Invalid magic in B+ tree: 0x%x\n\0".as_ptr() as *const i8, magic);
        return core::ptr::null_mut();
    }
    page_size = GET_USHORT(buf, 9 + 4) as ::core::ffi::c_uint;
    cur_page = GET_USHORT(buf, 9 + 26) as ::core::ffi::c_uint;
    level = GET_USHORT(buf, 9 + 32) as ::core::ffi::c_uint;
    pages = buf.offset(9 as isize).offset(38 as isize);
    loop {
        level = level.wrapping_sub(1);
        if !(level > 0 as ::core::ffi::c_uint) {
            break;
        }
        ptr = pages.offset(cur_page.wrapping_mul(page_size) as isize);
        entries = GET_SHORT(ptr, 2) as ::core::ffi::c_int;
        ptr = ptr.offset(6 as isize);
        i = 0 as ::core::ffi::c_int;
        while i < entries {
            if (comp.unwrap_unchecked())(
                ptr as *mut ::core::ffi::c_void,
                key,
                0 as ::core::ffi::c_int,
                &raw mut newptr as *mut *mut ::core::ffi::c_void,
            ) > 0 {
                break;
            }
            ptr = newptr;
            i += 1;
        }
        cur_page = GET_USHORT(
            ptr.offset(-(2 as isize)),
            0,
        ) as ::core::ffi::c_uint;
    }
    ptr = pages.offset(cur_page.wrapping_mul(page_size) as isize);
    entries = GET_SHORT(ptr, 2) as ::core::ffi::c_int;
    ptr = ptr.offset(8 as isize);
    i = 0 as ::core::ffi::c_int;
    while i < entries {
        ret = (comp.unwrap_unchecked())(
            ptr as *mut ::core::ffi::c_void,
            key,
            1 as ::core::ffi::c_int,
            &raw mut newptr as *mut *mut ::core::ffi::c_void,
        );
        if ret == 0 {
            return ptr as *mut ::core::ffi::c_void;
        }
        if ret > 0 {
            return core::ptr::null_mut();
        }
        ptr = newptr;
        i += 1;
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn HLPFILE_BPTreeEnum(
    mut buf: *mut BYTE,
    mut cb: HLPFILE_BPTreeCallback,
    mut cookie: *mut ::core::ffi::c_void,
) {
    let mut magic: ::core::ffi::c_uint = 0;
    let mut page_size: ::core::ffi::c_uint = 0;
    let mut cur_page: ::core::ffi::c_uint = 0;
    let mut level: ::core::ffi::c_uint = 0;
    let mut pages: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut ptr: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut newptr: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut i: ::core::ffi::c_int = 0;
    let mut entries: ::core::ffi::c_int = 0;
    magic = GET_USHORT(buf, 9) as ::core::ffi::c_uint;
    if magic != 0x293B as ::core::ffi::c_uint {
        WINE_ERR!("Invalid magic in B+ tree: 0x%x\n", magic);
        return;
    }
    page_size = GET_USHORT(
        buf,
        (9 + 4) as ::core::ffi::c_uint,
    ) as ::core::ffi::c_uint;
    cur_page = GET_USHORT(
        buf,
        (9 + 26) as ::core::ffi::c_uint,
    ) as ::core::ffi::c_uint;
    level = GET_USHORT(
        buf,
        (9 + 32) as ::core::ffi::c_uint,
    ) as ::core::ffi::c_uint;
    pages = buf
        .add((9 + 38) as usize);
    loop {
        level = level.wrapping_sub(1);
        if !(level > 0 as ::core::ffi::c_uint) {
            break;
        }
        ptr = pages.add(cur_page.wrapping_mul(page_size) as usize);
        cur_page = GET_USHORT(ptr, 4 as ::core::ffi::c_uint)
            as ::core::ffi::c_uint;
    }
    while cur_page != 0xFFFF as ::core::ffi::c_uint {
        ptr = pages.add(cur_page.wrapping_mul(page_size) as usize);
        entries = GET_SHORT(ptr, 2 as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
        ptr = ptr.add(8 as usize);
        i = 0 as ::core::ffi::c_int;
        while i < entries {
            cb.unwrap_unchecked()(
                ptr as *mut ::core::ffi::c_void,
                &raw mut newptr as *mut *mut ::core::ffi::c_void,
                cookie,
            );
            ptr = newptr;
            i += 1;
        }
        cur_page = GET_USHORT(
            pages.add(cur_page.wrapping_mul(page_size) as usize),
            6 as ::core::ffi::c_uint,
        ) as ::core::ffi::c_uint;
    }
}

unsafe extern "C" fn HLPFILE_UncompressedLZ77_Size(mut ptr: *const BYTE, end: *const BYTE) -> INT {
    let mut i: ::core::ffi::c_int;
    let mut newsize: ::core::ffi::c_int = 0;
    while ptr < end {
        let mut mask: ::core::ffi::c_int = *ptr as ::core::ffi::c_int;
        ptr = ptr.add(1);
        i = 0;
        while i < 8 && ptr < end {
            if (mask & 1) != 0 {
                let mut code: ::core::ffi::c_int =
                    GET_USHORT(ptr, 0) as ::core::ffi::c_int;
                let mut len: ::core::ffi::c_int =
                    3 + (code >> 12);
                newsize += len;
                ptr = ptr.add(2 as usize);
            } else {
                newsize += 1;
                ptr = ptr.add(1);
            }
            i += 1;
            mask >>= 1;
        }
    }
    return newsize as INT;
}

unsafe extern "C" fn HLPFILE_UncompressLZ77(
    mut ptr: *const BYTE,
    mut end: *const BYTE,
    mut newptr: *mut BYTE,
) -> *mut BYTE {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while ptr < end {
        let fresh27 = ptr;
        ptr = ptr.offset(1);
        let mut mask: ::core::ffi::c_int = *fresh27 as ::core::ffi::c_int;
        i = 0 as ::core::ffi::c_int;
        while i < 8 as ::core::ffi::c_int && ptr < end {
            if (mask & 1 as ::core::ffi::c_int) != 0 {
                let mut code: ::core::ffi::c_int = GET_USHORT(
                    ptr,
                    0 as ::core::ffi::c_uint,
                ) as ::core::ffi::c_int;
                let mut len: ::core::ffi::c_int = 3 as ::core::ffi::c_int
                    + (code >> 12 as ::core::ffi::c_int);
                let mut offset: ::core::ffi::c_int =
                    code & 0xfff as ::core::ffi::c_int;
                while len > 0 as ::core::ffi::c_int {
                    *newptr = *newptr
                        .offset(-(offset as isize))
                        .offset(-(1 as isize));
                    len -= 1;
                    newptr = newptr.offset(1);
                }
                ptr = ptr.offset(2 as isize);
            } else {
                let fresh28 = ptr;
                ptr = ptr.offset(1);
                let fresh29 = newptr;
                newptr = newptr.offset(1);
                *fresh29 = *fresh28;
            }
            i += 1;
            mask >>= 1 as ::core::ffi::c_int;
        }
    }
    return newptr;
}

unsafe extern "C" fn HLPFILE_Uncompress2(hlpfile: *mut HLPFILE, mut ptr: *const BYTE, end: *const BYTE, mut newptr: *mut BYTE, newend: *const BYTE) {
    let mut phptr: *mut BYTE = core::ptr::null_mut();
    let mut phend: *mut BYTE = core::ptr::null_mut();
    let mut code: UINT = 0;
    let mut index: UINT = 0;
    while ptr < end && newptr < newend as *mut BYTE {
        if *ptr == 0 || *ptr as i32 >= 0x10 {
            *newptr = *ptr;
            ptr = ptr.add(1);
            newptr = newptr.add(1);
        } else {
            code = (0x100 * (*ptr.add(0) as UINT)
                + (*ptr.add(1) as UINT)) as UINT;
            index = code.wrapping_sub(0x100) / 2;

            phptr = ((*hlpfile).phrases_buffer as *mut BYTE)
                .offset(*(*hlpfile).phrases_offsets.offset(index as isize) as isize);
            phend = ((*hlpfile).phrases_buffer as *mut BYTE).offset(
                *(*hlpfile).phrases_offsets.offset(index.wrapping_add(1) as isize) as isize,
            );

            if newptr.offset(phend.offset_from(phptr) as isize) > newend as *mut BYTE {
                WINE_FIXME!(
                    b"buffer overflow %p > %p for %lu bytes\n\0".as_ptr() as *const i8,
                    newptr,
                    newend,
                    phend.offset_from(phptr) as SIZE_T,
                );
                return;
            }
            memcpy(
                newptr as *mut core::ffi::c_void,
                phptr as *const core::ffi::c_void,
                phend.offset_from(phptr) as usize,
            );
            newptr = newptr.offset(phend.offset_from(phptr) as isize);
            if (code & 1) != 0 {
                *newptr = b' ';
                newptr = newptr.add(1);
            }
            ptr = ptr.add(2);
        }
    }
    if newptr > newend as *mut BYTE {
        WINE_FIXME!(
            b"buffer overflow %p > %p\n\0".as_ptr() as *const i8,
            newptr,
            newend,
        );
    }
}

unsafe extern "C" fn HLPFILE_Uncompress3(
    mut hlpfile: *mut HLPFILE,
    mut dst: *mut ::core::ffi::c_char,
    mut dst_end: *const ::core::ffi::c_char,
    mut src: *const BYTE,
    mut src_end: *const BYTE,
) -> BOOL {
    let mut idx: ::core::ffi::c_uint = 0;
    let mut len: ::core::ffi::c_uint = 0;
    while src < src_end {
        if (*src as ::core::ffi::c_int & 1) == 0 {
            idx = (*src as ::core::ffi::c_int / 2) as ::core::ffi::c_uint;
            if idx > (*hlpfile).num_phrases {
                WINE_ERR!(
                    b"index in phrases %d/%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    idx,
                    (*hlpfile).num_phrases,
                );
                len = 0 as ::core::ffi::c_uint;
            } else {
                len = (*(*hlpfile)
                    .phrases_offsets
                    .offset(idx.wrapping_add(1 as ::core::ffi::c_uint) as isize))
                .wrapping_sub(*(*hlpfile).phrases_offsets.offset(idx as isize));
                if dst.offset(len as isize) <= dst_end as *mut ::core::ffi::c_char {
                    memcpy(
                        dst as *mut ::core::ffi::c_void,
                        (*hlpfile)
                            .phrases_buffer
                            .offset(*(*hlpfile).phrases_offsets.offset(idx as isize) as isize)
                            as *mut ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        len as usize,
                    );
                }
            }
        } else if (*src as ::core::ffi::c_int & 0x03) == 0x01 {
            idx = ((*src as ::core::ffi::c_int + 1)
                * 64) as ::core::ffi::c_uint;
            src = src.offset(1);
            idx = idx.wrapping_add(*src as ::core::ffi::c_uint);
            if idx > (*hlpfile).num_phrases {
                WINE_ERR!(
                    b"index in phrases %d/%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    idx,
                    (*hlpfile).num_phrases,
                );
                len = 0 as ::core::ffi::c_uint;
            } else {
                len = (*(*hlpfile)
                    .phrases_offsets
                    .offset(idx.wrapping_add(1 as ::core::ffi::c_uint) as isize))
                .wrapping_sub(*(*hlpfile).phrases_offsets.offset(idx as isize));
                if dst.offset(len as isize) <= dst_end as *mut ::core::ffi::c_char {
                    memcpy(
                        dst as *mut ::core::ffi::c_void,
                        (*hlpfile)
                            .phrases_buffer
                            .offset(*(*hlpfile).phrases_offsets.offset(idx as isize) as isize)
                            as *mut ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        len as usize,
                    );
                }
            }
        } else if (*src as ::core::ffi::c_int & 0x07) == 0x03 {
            len = ((*src as ::core::ffi::c_int / 8)
                + 1) as ::core::ffi::c_uint;
            if dst.offset(len as isize) <= dst_end as *mut ::core::ffi::c_char {
                memcpy(
                    dst as *mut ::core::ffi::c_void,
                    src.offset(1 as isize) as *const ::core::ffi::c_void,
                    len as usize,
                );
            }
            src = src.offset(len as isize);
        } else {
            len = ((*src as ::core::ffi::c_int / 16)
                + 1) as ::core::ffi::c_uint;
            if dst.offset(len as isize) <= dst_end as *mut ::core::ffi::c_char {
                memset(
                    dst as *mut ::core::ffi::c_void,
                    if (*src as ::core::ffi::c_int & 0x0F) == 0x07 {
                        ' ' as ::core::ffi::c_int
                    } else {
                        0
                    },
                    len as usize,
                );
            }
        }
        dst = dst.offset(len as isize);
        src = src.offset(1);
    }
    if dst > dst_end as *mut ::core::ffi::c_char {
        WINE_ERR!(
            b"buffer overflow (%p > %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            dst,
            dst_end,
        );
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_UncompressRLE(mut src: *const BYTE, mut end: *const BYTE, mut dst: *mut BYTE, dstsz: u32) {
    let mut ch: BYTE;
    let sdst: *mut BYTE = dst.add(dstsz as usize);

    while src < end {
        ch = *src;
        src = src.add(1);
        if ch & 0x80 != 0 {
            ch &= 0x7F;
            if dst.add(ch as usize) <= sdst {
                memcpy(dst as *mut core::ffi::c_void, src as *const core::ffi::c_void, ch as usize);
            }
            src = src.add(ch as usize);
        } else {
            if dst.add(ch as usize) <= sdst {
                memset(dst as *mut core::ffi::c_void, (*src as core::ffi::c_char) as core::ffi::c_int, ch as usize);
            }
            src = src.add(1);
        }
        dst = dst.add(ch as usize);
    }
    if dst != sdst {
        WINE_WARN!(
            b"Buffer X-flow: d(%lu) instead of d(%u)\n\0".as_ptr() as *const core::ffi::c_char,
            dst.offset_from(sdst.sub(dstsz as usize)) as usize,
            dstsz,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn HLPFILE_PageByOffset(hlpfile: *mut HLPFILE, offset: LONG, relative: *mut ULONG) -> *mut HLPFILE_PAGE {
    let mut page: *mut HLPFILE_PAGE = core::ptr::null_mut();
    let mut found: *mut HLPFILE_PAGE = core::ptr::null_mut();

    if hlpfile.is_null() {
        return core::ptr::null_mut();
    }

    WINE_TRACE!(
        b"<%s>[%x]\n\0".as_ptr() as *const i8,
        debugstr_a((*hlpfile).lpszPath as *const i8),
        offset,
    );

    if offset as u32 == 0xFFFFFFFF {  // C: LONG vs unsigned literal -> unsigned compare, i.e. offset == -1
        return core::ptr::null_mut();
    }
    page = core::ptr::null_mut();

    found = core::ptr::null_mut();
    page = (*hlpfile).first_page as *mut HLPFILE_PAGE;
    while !page.is_null() {
        if (*page).offset <= (offset) as u32 && (found.is_null() || (*found).offset < (*page).offset) {
            *relative = offset.wrapping_sub(((*page).offset) as i32) as ULONG;
            found = page;
        }
        page = (*page).next as *mut HLPFILE_PAGE;
    }
    if found.is_null() {
        WINE_ERR!(
            b"Page of offset %u not found in file %s\n\0".as_ptr() as *const i8,
            offset,
            debugstr_a((*hlpfile).lpszPath as *const i8),
        );
    }
    return found;
}

unsafe extern "C" fn HLPFILE_Contents(hlpfile: *mut HLPFILE, relative: *mut ULONG) -> *mut HLPFILE_PAGE {
    let mut page: *mut HLPFILE_PAGE = core::ptr::null_mut();

    if hlpfile.is_null() {
        return core::ptr::null_mut();
    }

    page = HLPFILE_PageByOffset(hlpfile, (*hlpfile).contents_start as LONG, relative);
    if page.is_null() {
        page = (*hlpfile).first_page;
        *relative = 0;
    }
    return page;
}

unsafe extern "C" fn comp_PageByHash(
    p: *mut ::core::ffi::c_void,
    key: *const ::core::ffi::c_void,
    leaf: ::core::ffi::c_int,
    next: *mut *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let lKey: LONG = key as LONG_PTR as LONG;
    let lTest: LONG = GET_UINT(p as *const BYTE, 0) as INT as LONG;
    *next = (p as *mut ::core::ffi::c_char).offset(
        (if leaf != 0 {
            8
        } else {
            6
        }) as isize,
    ) as *mut ::core::ffi::c_void;
    WINE_TRACE!(
        b"Comparing '%d' with '%d'\n\0".as_ptr() as *const ::core::ffi::c_char,
        lKey,
        lTest,
    );
    if lTest < lKey {
        return -(1);
    }
    if lTest > lKey {
        return 1;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn HLPFILE_PageByHash(hlpfile: *mut HLPFILE, lHash: LONG, relative: *mut ULONG) -> *mut HLPFILE_PAGE {
    let mut ptr: *mut BYTE = core::ptr::null_mut();

    if hlpfile.is_null() {
        return core::ptr::null_mut();
    }
    if lHash == 0 {
        return HLPFILE_Contents(hlpfile, relative);
    }

    WINE_TRACE!(
        b"<%s>[%x]\n\0".as_ptr() as *const i8,
        debugstr_a((*hlpfile).lpszPath),
        lHash,
    );

    /* For win 3.0 files hash values are really page numbers */
    if ((*hlpfile).version as LONG) <= 16 {
        if (lHash as ULONG) >= ((*hlpfile).wTOMapLen as ULONG) {
            return core::ptr::null_mut();
        }
        return HLPFILE_PageByOffset(
            hlpfile,
            *(*hlpfile).TOMap.offset(lHash as isize) as LONG,
            relative,
        );
    }

    ptr = HLPFILE_BPTreeSearch(
        (*hlpfile).Context,
        LongToPtr!(lHash),
        Some(comp_PageByHash),
    ) as *mut BYTE;
    if ptr.is_null() {
        WINE_ERR!(
            b"Page of hash %x not found in file %s\n\0".as_ptr() as *const i8,
            lHash,
            debugstr_a((*hlpfile).lpszPath),
        );
        return core::ptr::null_mut();
    }

    return HLPFILE_PageByOffset(
        hlpfile,
        GET_UINT(ptr, 4) as LONG,
        relative,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HLPFILE_PageByMap(mut hlpfile: *mut HLPFILE, mut lMap: LONG, mut relative: *mut ULONG) -> *mut HLPFILE_PAGE {
    let mut i: ::core::ffi::c_uint;

    if hlpfile.is_null() {
        return core::ptr::null_mut();
    }

    WINE_TRACE!(
        b"<%s>[%x]\n\0" as *const u8 as *const ::core::ffi::c_char,
        debugstr_a((*hlpfile).lpszPath as *const ::core::ffi::c_char),
        lMap,
    );

    i = 0;
    while i < (*hlpfile).wMapLen {
        if (*(*hlpfile).Map.add(i as usize)).lMap == lMap {
            return HLPFILE_PageByOffset(
                hlpfile,
                (*(*hlpfile).Map.add(i as usize)).offset as LONG,
                relative,
            );
        }
        i = i.wrapping_add(1);
    }

    WINE_ERR!(
        b"Page of Map %x not found in file %s\n\0" as *const u8 as *const ::core::ffi::c_char,
        lMap,
        debugstr_a((*hlpfile).lpszPath as *const ::core::ffi::c_char),
    );
    return core::ptr::null_mut();
}

unsafe extern "C" fn comp_FindSubFile(
    p: *mut ::core::ffi::c_void,
    key: *const ::core::ffi::c_void,
    leaf: ::core::ffi::c_int,
    next: *mut *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    *next = (p as *mut ::core::ffi::c_char)
        .offset(strlen(p as *const ::core::ffi::c_char) as isize)
        .offset(
            (if leaf != 0 {
                5
            } else {
                3
            }) as isize,
        ) as *mut ::core::ffi::c_void;
    WINE_TRACE!(
        b"Comparing %s with %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        debugstr_a(p as *mut ::core::ffi::c_char),
        debugstr_a(key as *const ::core::ffi::c_char),
    );
    return strcmp(
        p as *const ::core::ffi::c_char,
        key as *const ::core::ffi::c_char,
    );
}

unsafe extern "C" fn HLPFILE_FindSubFile(mut hlpfile: *mut HLPFILE, mut name: LPCSTR, subbuf: *mut *mut BYTE, subend: *mut *mut BYTE) -> BOOL {
    let mut ptr: *mut BYTE = core::ptr::null_mut();
    WINE_TRACE!(b"looking for file %s\n\0".as_ptr() as *const i8, debugstr_a(name as *const i8));
    ptr = HLPFILE_BPTreeSearch(
        (*hlpfile).file_buffer.offset(GET_UINT((*hlpfile).file_buffer as *const BYTE, 4) as isize),
        name as *const core::ffi::c_void,
        Some(comp_FindSubFile),
    ) as *mut BYTE;
    if ptr.is_null() {
        /* Subfiles with bitmap images are usually prefixed with '|', but sometimes not.
           Unfortunately, there is no consensus among different pieces of unofficial
           documentation. So remove leading '|' and try again. */
        let c: CHAR = *name;
        name = name.add(1);
        if c as i32 == '|' as i32 {
            WINE_TRACE!(b"not found. try %s\n\0".as_ptr() as *const i8, debugstr_a(name as *const i8));
            ptr = HLPFILE_BPTreeSearch(
                (*hlpfile).file_buffer.offset(GET_UINT((*hlpfile).file_buffer as *const BYTE, 4) as isize),
                name as *const core::ffi::c_void,
                Some(comp_FindSubFile),
            ) as *mut BYTE;
        }
    }
    if ptr.is_null() {
        return (FALSE) as BOOL;
    }
    *subbuf = (*hlpfile).file_buffer.offset(
        GET_UINT(ptr as *const BYTE, (strlen(name as *const i8) + 1) as u32) as isize,
    );
    if *subbuf >= (*hlpfile).file_buffer.offset((*hlpfile).file_buffer_size as isize) {
        WINE_ERR!(b"internal file %s does not fit\n\0".as_ptr() as *const i8, debugstr_a(name as *const i8));
        return (FALSE) as BOOL;
    }
    *subend = (*subbuf).offset(GET_UINT(*subbuf as *const BYTE, 0) as isize);
    if *subend > (*hlpfile).file_buffer.offset((*hlpfile).file_buffer_size as isize) {
        WINE_ERR!(b"internal file %s does not fit\n\0".as_ptr() as *const i8, debugstr_a(name as *const i8));
        return (FALSE) as BOOL;
    }
    if GET_UINT(*subbuf as *const BYTE, 0)
        < GET_UINT(*subbuf as *const BYTE, 4).wrapping_add(9)
    {
        WINE_ERR!(b"invalid size provided for internal file %s\n\0".as_ptr() as *const i8, debugstr_a(name as *const i8));
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn HLPFILE_Hash(mut lpszContext: LPCSTR) -> LONG {
    let mut lHash: LONG = 0;
    let mut c: CHAR;
    loop {
        let fresh31 = lpszContext;
        lpszContext = lpszContext.offset(1);
        c = *fresh31;
        if !(c != 0) {
            break;
        }
        let mut x: CHAR = 0 as CHAR;
        if c as ::core::ffi::c_int >= 'A' as i32 && c as ::core::ffi::c_int <= 'Z' as i32 {
            x = (c as ::core::ffi::c_int - 'A' as i32 + 17) as CHAR;
        }
        if c as ::core::ffi::c_int >= 'a' as i32 && c as ::core::ffi::c_int <= 'z' as i32 {
            x = (c as ::core::ffi::c_int - 'a' as i32 + 17) as CHAR;
        }
        if c as ::core::ffi::c_int >= '1' as i32 && c as ::core::ffi::c_int <= '9' as i32 {
            x = (c as ::core::ffi::c_int - '0' as i32) as CHAR;
        }
        if c as ::core::ffi::c_int == '0' as i32 {
            x = 10 as CHAR;
        }
        if c as ::core::ffi::c_int == '.' as i32 {
            x = 12 as CHAR;
        }
        if c as ::core::ffi::c_int == '_' as i32 {
            x = 13 as CHAR;
        }
        if x != 0 {
            lHash = lHash.wrapping_mul(43 as LONG).wrapping_add(x as LONG);
        }
    }
    return lHash;
}

// The HLP stream is byte-packed: the C derefs these multi-byte fields unaligned (UB in
// Rust, and a panic under debug assertions), so read them with read_unaligned.
unsafe extern "C" fn fetch_long(ptr: *mut *const BYTE) -> LONG {
    let mut ret: LONG;
    if (**ptr as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0 {
        ret = (((*ptr as *const ULONG).read_unaligned()).wrapping_sub(0x80000000 as ULONG)
            / 2 as ULONG) as LONG;
        *ptr = (*ptr).offset(4 as isize);
    } else {
        ret = ((((*ptr as *const USHORT).read_unaligned()) as ::core::ffi::c_int
            - 0x8000 as ::core::ffi::c_int)
            / 2 as ::core::ffi::c_int) as LONG;
        *ptr = (*ptr).offset(2 as isize);
    }

    return ret;
}

unsafe extern "C" fn fetch_ulong(ptr: *mut *const BYTE) -> ULONG {
    let mut ret: ULONG;

    if (**ptr as i32) & 1 != 0 {
        ret = (*ptr as *const ULONG).read_unaligned() / 2;
        *ptr = (*ptr).add(4);
    } else {
        ret = (((*ptr as *const USHORT).read_unaligned() as i32) / 2) as ULONG;
        *ptr = (*ptr).add(2);
    }
    return ret;
}

unsafe extern "C" fn fetch_short(ptr: *mut *const BYTE) -> i16 {
    let mut ret: i16 = 0;
    if ((**ptr as i32) & 1) != 0 {
        ret = (((*ptr as *const u16).read_unaligned() as i32 - 0x8000) / 2) as i16;
        *ptr = (*ptr).add(2 as usize);
    } else {
        ret = ((**ptr as i32 - 0x80) / 2) as i16;
        *ptr = (*ptr).add(1);
    }
    return ret;
}

unsafe extern "C" fn fetch_ushort(mut ptr: *mut *const BYTE) -> ::core::ffi::c_ushort {
    let mut ret: ::core::ffi::c_ushort = 0;
    if ((**ptr as ::core::ffi::c_int) & 1) != 0 {
        ret = ((*ptr as *const ::core::ffi::c_ushort).read_unaligned() as ::core::ffi::c_int
            / 2) as ::core::ffi::c_ushort;
        *ptr = (*ptr).offset(2 as isize);
    } else {
        ret = (*(*ptr as *const ::core::ffi::c_uchar) as ::core::ffi::c_int
            / 2) as ::core::ffi::c_ushort;
        *ptr = (*ptr).offset(1);
    }
    return ret;
}

unsafe extern "C" fn HLPFILE_DecompressGfx(src: *const BYTE, csz: ::core::ffi::c_uint, sz: ::core::ffi::c_uint, packing: BYTE, alloc: *mut *mut BYTE) -> *const BYTE {
    let mut dst: *const BYTE = core::ptr::null();
    let mut tmp: *mut BYTE;
    let mut sz77: ::core::ffi::c_uint;

    WINE_TRACE!(b"Unpacking (%d) from %u bytes to %u bytes\n\0".as_ptr() as *const ::core::ffi::c_char, packing as ::core::ffi::c_int, csz, sz);

    match packing {
        0 => {
            /* uncompressed */
            if sz != csz {
                WINE_WARN!(b"Bogus gfx sizes (uncompressed): %u / %u\n\0".as_ptr() as *const ::core::ffi::c_char, sz, csz);
            }
            dst = src;
            *alloc = core::ptr::null_mut();
        }
        1 => {
            /* RunLen */
            *alloc = HeapAlloc(GetProcessHeap(), 0, sz as SIZE_T) as *mut BYTE;
            dst = *alloc;
            if dst.is_null() {
                return core::ptr::null();
            }
            HLPFILE_UncompressRLE(src, src.offset(csz as isize), *alloc, sz);
        }
        2 => {
            /* LZ77 */
            sz77 = HLPFILE_UncompressedLZ77_Size(src, src.offset(csz as isize)) as ::core::ffi::c_uint;
            *alloc = HeapAlloc(GetProcessHeap(), 0, sz77 as SIZE_T) as *mut BYTE;
            dst = *alloc;
            if dst.is_null() {
                return core::ptr::null();
            }
            HLPFILE_UncompressLZ77(src, src.offset(csz as isize), *alloc);
            if sz77 != sz {
                WINE_WARN!(b"Bogus gfx sizes (LZ77): %u / %u\n\0".as_ptr() as *const ::core::ffi::c_char, sz77, sz);
            }
        }
        3 => {
            /* LZ77 then RLE */
            sz77 = HLPFILE_UncompressedLZ77_Size(src, src.offset(csz as isize)) as ::core::ffi::c_uint;
            tmp = HeapAlloc(GetProcessHeap(), 0, sz77 as SIZE_T) as *mut BYTE;
            if tmp.is_null() {
                return core::ptr::null();
            }
            HLPFILE_UncompressLZ77(src, src.offset(csz as isize), tmp);
            *alloc = HeapAlloc(GetProcessHeap(), 0, sz as SIZE_T) as *mut BYTE;
            dst = *alloc;
            if dst.is_null() {
                HeapFree(GetProcessHeap(), 0, tmp as PVOID);
                return core::ptr::null();
            }
            HLPFILE_UncompressRLE(tmp, tmp.offset(sz77 as isize), *alloc, sz);
            HeapFree(GetProcessHeap(), 0, tmp as PVOID);
        }
        _ => {
            WINE_FIXME!(b"Unsupported packing %u\n\0".as_ptr() as *const ::core::ffi::c_char, packing as ::core::ffi::c_int);
            return core::ptr::null();
        }
    }
    return dst;
}

unsafe extern "C" fn HLPFILE_RtfAddRawString(rd: *mut RtfData, str: *const core::ffi::c_char, sz: usize) -> BOOL {
    if (*rd).ptr.offset(sz as isize) >= (*rd).data.offset((*rd).allocated as isize) {
        (*rd).allocated = (*rd).allocated.wrapping_mul(2);
        let new: *mut core::ffi::c_char = HeapReAlloc(GetProcessHeap(), 0, (*rd).data as *mut core::ffi::c_void, ((*rd).allocated) as SIZE_T) as *mut core::ffi::c_char;
        if new.is_null() {
            return (FALSE) as BOOL;
        }
        (*rd).ptr = new.offset((*rd).ptr.offset_from((*rd).data) as core::ffi::c_longlong as isize);
        (*rd).data = new;
    }
    memcpy((*rd).ptr as *mut core::ffi::c_void, str as *const core::ffi::c_void, sz);
    (*rd).ptr = (*rd).ptr.offset(sz as isize);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_RtfAddControl(rd: *mut RtfData, str: *const i8) -> BOOL {
    WINE_TRACE!("%s\n", debugstr_a(str));
    if *str as i32 == '\\' as i32 || *str as i32 == '{' as i32 {
        (*rd).in_text = (FALSE as i32);
    } else if *str as i32 == '}' as i32 {
        (*rd).in_text = (TRUE as i32);
    }
    return HLPFILE_RtfAddRawString(rd, str, strlen(str));
}

unsafe extern "C" fn HLPFILE_RtfAddText(mut rd: *mut RtfData, mut str: *const c_char) -> BOOL {
    let mut p: *const c_char;
    let mut last: *const c_char;
    let mut replace: *const c_char = core::ptr::null();
    let mut rlen: c_uint = 0;

    if (*rd).in_text == 0 {
        if HLPFILE_RtfAddRawString(rd, b" \0".as_ptr() as *const c_char, 1 as size_t) == 0 {
            return (FALSE) as BOOL;
        }
        (*rd).in_text = (TRUE as i32);
    }

    p = str;
    last = p;
    while *p != 0 {
        let mut handled: bool = true;
        if ((*p as c_int) & 0x80) != 0 {
            static mut xx: [c_char; 8] = [0; 8];
            rlen = sprintf(
                &raw mut xx as *mut c_char,
                b"\\'%x\0".as_ptr() as *const c_char,
                *(p as *const BYTE) as c_int,
            ) as c_uint;
            replace = &raw mut xx as *mut c_char;
        } else {
            match *p as c_int {
                123 => {
                    rlen = 2;
                    replace = b"\\{\0".as_ptr() as *const c_char;
                }
                125 => {
                    rlen = 2;
                    replace = b"\\}\0".as_ptr() as *const c_char;
                }
                92 => {
                    rlen = 2;
                    replace = b"\\\\\0".as_ptr() as *const c_char;
                }
                _ => {
                    handled = false;
                }
            }
        }

        if handled {
            if (p != last && HLPFILE_RtfAddRawString(rd, last, p.offset_from(last) as ::core::ffi::c_longlong as size_t) == 0)
                || HLPFILE_RtfAddRawString(rd, replace, rlen as size_t) == 0
            {
                return (FALSE) as BOOL;
            }
            last = p.offset(1 as isize);
        }
        p = p.offset(1);
    }
    return HLPFILE_RtfAddRawString(rd, last, p.offset_from(last) as ::core::ffi::c_longlong as size_t);
}

unsafe extern "C" fn HLPFILE_RtfAddHexBytes(mut rd: *mut RtfData, mut _ptr: *const core::ffi::c_void, mut sz: core::ffi::c_uint) -> BOOL {
    let mut tmp: [core::ffi::c_char; 512] = [0; 512];
    let mut i: core::ffi::c_uint;
    let mut step: core::ffi::c_uint = 0;
    let mut ptr: *const BYTE = _ptr as *const BYTE;
    let _2hex: *const core::ffi::c_char = b"0123456789abcdef\0".as_ptr() as *const core::ffi::c_char;

    if (*rd).in_text == 0 {
        if HLPFILE_RtfAddRawString(rd, b" \0".as_ptr() as *const core::ffi::c_char, 1) == 0 {
            return (FALSE) as BOOL;
        }
        (*rd).in_text = (TRUE as i32);
    }
    while sz != 0 {
        step = min!(256, sz);
        i = 0;
        while i < step {
            tmp[(2 * i + 0) as usize] =
                *_2hex.offset((*ptr >> 4) as isize);
            let fresh = ptr;
            ptr = ptr.offset(1);
            tmp[(2 * i + 1) as usize] =
                *_2hex.offset((*fresh & 0xF) as isize);
            i = i.wrapping_add(1);
        }
        if HLPFILE_RtfAddRawString(rd, tmp.as_ptr(), (2 * step) as size_t) == 0 {
            return (FALSE) as BOOL;
        }
        sz = sz.wrapping_sub(step);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_AddHotSpotLinks(
    rd: *mut RtfData,
    file: *mut HLPFILE,
    mut start: *const BYTE,
    hs_size: ULONG,
    hs_offset: ULONG,
) {
    let mut i: ::core::ffi::c_uint;
    let mut hs_num: ::core::ffi::c_uint;
    let mut hs_macro: ULONG;
    let mut str: *const ::core::ffi::c_char;

    if hs_size == 0 || hs_offset == 0 {
        return;
    }

    start = start.offset(hs_offset as isize);
    /* always 1 ?? */
    hs_num = GET_USHORT(start, 1 as ::core::ffi::c_uint) as ::core::ffi::c_uint;
    hs_macro = GET_UINT(start, 3 as ::core::ffi::c_uint) as ULONG;

    str = (start as *const ::core::ffi::c_char)
        .offset(7 as isize)
        .offset((15 as ::core::ffi::c_uint).wrapping_mul(hs_num) as isize)
        .offset(hs_macro as isize);
    /* FIXME: should use hs_size to prevent out of bounds reads */
    i = 0 as ::core::ffi::c_uint;
    while i < hs_num {
        let mut hslink: *mut HLPFILE_HOTSPOTLINK = ::core::ptr::null_mut();

        WINE_TRACE!(
            b"%02x-%02x%02x {%s,%s}\n\0".as_ptr() as *const ::core::ffi::c_char,
            *start.offset(
                ((7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(0 as ::core::ffi::c_uint)) as isize,
            ) as ::core::ffi::c_int,
            *start.offset(
                ((7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(1 as ::core::ffi::c_uint)) as isize,
            ) as ::core::ffi::c_int,
            *start.offset(
                ((7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(2 as ::core::ffi::c_uint)) as isize,
            ) as ::core::ffi::c_int,
            debugstr_a(str),
            debugstr_a(str.add(strlen(str) + 1 as usize)),
        );
        /* str points to two null terminated strings:
         * hotspot name, then link name
         */
        str = str.add(strlen(str) + 1 as usize); /* skip hotspot name */

        hslink = ::core::ptr::null_mut();
        /* The next two chars always look like 0x04 0x00 ???
         * What are they for ?
         */
        match *start.offset(
            ((7 as ::core::ffi::c_uint)
                .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                .wrapping_add(0 as ::core::ffi::c_uint)) as isize,
        ) as ::core::ffi::c_int
        {
            0xC8 => {
                hslink = HLPFILE_AllocLink(
                    rd,
                    hlp_link_macro as ::core::ffi::c_int,
                    str,
                    (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint,
                    0 as LONG,
                    (FALSE) as BOOL,
                    (TRUE) as BOOL,
                    (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint,
                ) as *mut HLPFILE_HOTSPOTLINK;
            }
            0xE6 | 0xE7 => {
                hslink = HLPFILE_AllocLink(
                    rd,
                    if (*start.offset(
                        ((7 as ::core::ffi::c_uint)
                            .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                            .wrapping_add(0 as ::core::ffi::c_uint)) as isize,
                    ) as ::core::ffi::c_int & (1 as ::core::ffi::c_int)) != 0
                    {
                        hlp_link_link as ::core::ffi::c_int
                    } else {
                        hlp_link_popup as ::core::ffi::c_int
                    },
                    (*file).lpszPath as *const ::core::ffi::c_char,
                    (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint,
                    HLPFILE_Hash(str as LPCSTR),
                    (FALSE) as BOOL,
                    (TRUE) as BOOL,
                    (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint,
                ) as *mut HLPFILE_HOTSPOTLINK;
            }
            0xEE | 0xEF => {
                let mut win: *const ::core::ffi::c_char =
                    strchr(str, '>' as i32);
                let mut wnd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
                let mut tgt: *mut ::core::ffi::c_char = ::core::ptr::null_mut();

                if !win.is_null() {
                    wnd = (*file)
                        .numWindows
                        .wrapping_sub(1 as ::core::ffi::c_uint)
                        as ::core::ffi::c_int;
                    while wnd >= 0 as ::core::ffi::c_int {
                        if strcmp(
                            win.offset(1 as isize),
                            &raw mut (*(*file).windows.offset(wnd as isize)).name
                                as *mut ::core::ffi::c_char,
                        ) == 0
                        {
                            break;
                        }
                        wnd -= 1;
                    }
                    if wnd == -(1 as ::core::ffi::c_int) {
                        WINE_WARN!(
                            b"Couldn't find window info for %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                            debugstr_a(win),
                        );
                    }
                    tgt = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        (win.offset_from(str) as SIZE_T)
                            + (1 as SIZE_T),
                    ) as *mut ::core::ffi::c_char;
                    if !tgt.is_null() {
                        memcpy(
                            tgt as *mut ::core::ffi::c_void,
                            str as *const ::core::ffi::c_void,
                            win.offset_from(str) as usize,
                        );
                        *tgt.offset(win.offset_from(str) as isize) = '\0' as i32 as ::core::ffi::c_char;
                    }
                }
                hslink = HLPFILE_AllocLink(
                    rd,
                    if (*start.offset(
                        ((7 as ::core::ffi::c_uint)
                            .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                            .wrapping_add(0 as ::core::ffi::c_uint)) as isize,
                    ) as ::core::ffi::c_int & (1 as ::core::ffi::c_int)) != 0
                    {
                        hlp_link_link as ::core::ffi::c_int
                    } else {
                        hlp_link_popup as ::core::ffi::c_int
                    },
                    (*file).lpszPath as *const ::core::ffi::c_char,
                    (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint,
                    HLPFILE_Hash(if !tgt.is_null() {
                        tgt as LPCSTR
                    } else {
                        str as LPCSTR
                    }),
                    (FALSE) as BOOL,
                    (TRUE) as BOOL,
                    wnd as ::core::ffi::c_uint,
                ) as *mut HLPFILE_HOTSPOTLINK;
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    tgt as PVOID,
                );
            }
            _ => {
                WINE_FIXME!(
                    b"unknown hotsport target 0x%x\n\0".as_ptr() as *const ::core::ffi::c_char,
                    *start.offset(
                        ((7 as ::core::ffi::c_uint)
                            .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                            .wrapping_add(0 as ::core::ffi::c_uint)) as isize,
                    ) as ::core::ffi::c_int,
                );
            }
        }
        if !hslink.is_null() {
            (*hslink).x = GET_USHORT(
                start,
                (7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(3 as ::core::ffi::c_uint),
            ) as ::core::ffi::c_uint;
            (*hslink).y = GET_USHORT(
                start,
                (7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(5 as ::core::ffi::c_uint),
            ) as ::core::ffi::c_uint;
            (*hslink).width = GET_USHORT(
                start,
                (7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(7 as ::core::ffi::c_uint),
            ) as ::core::ffi::c_uint;
            (*hslink).height = GET_USHORT(
                start,
                (7 as ::core::ffi::c_uint)
                    .wrapping_add((15 as ::core::ffi::c_uint).wrapping_mul(i))
                    .wrapping_add(9 as ::core::ffi::c_uint),
            ) as ::core::ffi::c_uint;
            /* target = GET_UINT(start, 7 + 15 * i + 11); */
        }
        str = str.add(strlen(str) + 1 as usize);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn HLPFILE_RtfAddTransparentBitmap(rd: *mut RtfData, bi: *const BITMAPINFO, pict: *const ::core::ffi::c_void, nc: ::core::ffi::c_uint) -> BOOL {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hdcMask: HDC = core::ptr::null_mut();
    let mut hdcMem: HDC = core::ptr::null_mut();
    let mut hdcEMF: HDC = core::ptr::null_mut();
    let mut hbm: HBITMAP = core::ptr::null_mut();
    let mut hbmMask: HBITMAP = core::ptr::null_mut();
    let mut hbmOldMask: HBITMAP = core::ptr::null_mut();
    let mut hbmOldMem: HBITMAP = core::ptr::null_mut();
    let mut hEMF: HENHMETAFILE = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;
    let mut data: *mut ::core::ffi::c_void = core::ptr::null_mut();
    let mut sz: UINT = 0;

    hdc = GetDC(0 as HWND);
    hbm = CreateDIBitmap(
        hdc,
        &raw const (*bi).bmiHeader,
        CBM_INIT as DWORD,
        pict,
        bi,
        DIB_RGB_COLORS as UINT,
    );

    hdcMem = CreateCompatibleDC(hdc);
    hbmOldMem = SelectObject(hdcMem, hbm as HGDIOBJ) as HBITMAP;

    /* create the mask bitmap from the main bitmap */
    hdcMask = CreateCompatibleDC(hdc);
    hbmMask = CreateBitmap(
        (*bi).bmiHeader.biWidth as INT,
        (*bi).bmiHeader.biHeight as INT,
        1 as UINT,
        1 as UINT,
        core::ptr::null::<::core::ffi::c_void>(),
    );
    hbmOldMask = SelectObject(hdcMask, hbmMask as HGDIOBJ) as HBITMAP;
    SetBkColor(
        hdcMem,
        RGB!(
            (*(&raw const (*bi).bmiColors as *const RGBQUAD)
                .offset(nc.wrapping_sub(1 as ::core::ffi::c_uint) as isize))
            .rgbRed,
            (*(&raw const (*bi).bmiColors as *const RGBQUAD)
                .offset(nc.wrapping_sub(1 as ::core::ffi::c_uint) as isize))
            .rgbGreen,
            (*(&raw const (*bi).bmiColors as *const RGBQUAD)
                .offset(nc.wrapping_sub(1 as ::core::ffi::c_uint) as isize))
            .rgbBlue,
        ),
    );
    BitBlt(
        hdcMask,
        0 as INT,
        0 as INT,
        (*bi).bmiHeader.biWidth as INT,
        (*bi).bmiHeader.biHeight as INT,
        hdcMem,
        0 as INT,
        0 as INT,
        SRCCOPY as DWORD,
    );

    /* sets to RGB(0,0,0) the transparent bits in main bitmap */
    SetBkColor(
        hdcMem,
        RGB!(0 as BYTE, 0 as BYTE, 0 as BYTE),
    );
    SetTextColor(
        hdcMem,
        RGB!(255 as BYTE, 255 as BYTE, 255 as BYTE),
    );
    BitBlt(
        hdcMem,
        0 as INT,
        0 as INT,
        (*bi).bmiHeader.biWidth as INT,
        (*bi).bmiHeader.biHeight as INT,
        hdcMask,
        0 as INT,
        0 as INT,
        SRCAND as DWORD,
    );

    SelectObject(hdcMask, hbmOldMask as HGDIOBJ);
    DeleteDC(hdcMask);

    SelectObject(hdcMem, hbmOldMem as HGDIOBJ);
    DeleteDC(hdcMem);

    /* we create the bitmap on the fly */
    hdcEMF = CreateEnhMetaFileW(
        core::ptr::null_mut::<HDC__>(),
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<RECT>(),
        core::ptr::null::<WCHAR>(),
    );
    hdcMem = CreateCompatibleDC(hdcEMF);

    /* sets to RGB(0,0,0) the transparent bits in final bitmap */
    hbmOldMem = SelectObject(hdcMem, hbmMask as HGDIOBJ) as HBITMAP;
    SetBkColor(
        hdcEMF,
        RGB!(255 as BYTE, 255 as BYTE, 255 as BYTE),
    );
    SetTextColor(
        hdcEMF,
        RGB!(0 as BYTE, 0 as BYTE, 0 as BYTE),
    );
    BitBlt(
        hdcEMF,
        0 as INT,
        0 as INT,
        (*bi).bmiHeader.biWidth as INT,
        (*bi).bmiHeader.biHeight as INT,
        hdcMem,
        0 as INT,
        0 as INT,
        SRCAND as DWORD,
    );

    /* and copy the remaining bits of main bitmap */
    SelectObject(hdcMem, hbm as HGDIOBJ);
    BitBlt(
        hdcEMF,
        0 as INT,
        0 as INT,
        (*bi).bmiHeader.biWidth as INT,
        (*bi).bmiHeader.biHeight as INT,
        hdcMem,
        0 as INT,
        0 as INT,
        SRCPAINT as DWORD,
    );
    SelectObject(hdcMem, hbmOldMem as HGDIOBJ);
    DeleteDC(hdcMem);

    /* do the cleanup */
    ReleaseDC(0 as HWND, hdc);
    DeleteObject(hbmMask as HGDIOBJ);
    DeleteObject(hbm as HGDIOBJ);

    hEMF = CloseEnhMetaFile(hdcEMF);

    /* generate rtf stream */
    sz = GetEnhMetaFileBits(
        hEMF,
        0 as UINT,
        core::ptr::null_mut::<BYTE>(),
    );
    if sz != 0 && {
        data = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            sz as SIZE_T,
        ) as *mut ::core::ffi::c_void;
        !data.is_null()
    } {
        if sz == GetEnhMetaFileBits(hEMF, sz, data as LPBYTE) {
            ret = (HLPFILE_RtfAddControl(
                rd,
                b"{\\pict\\emfblip\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0
                && HLPFILE_RtfAddHexBytes(rd, data, sz as ::core::ffi::c_uint) != 0
                && HLPFILE_RtfAddControl(rd, b"}\0".as_ptr() as *const ::core::ffi::c_char) != 0)
                as ::core::ffi::c_int as BOOL;
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            data as PVOID,
        );
    }
    DeleteEnhMetaFile(hEMF);

    return ret;
}

unsafe extern "C" fn HLPFILE_RtfAddBitmap(
    rd: *mut RtfData,
    file: *mut HLPFILE,
    beg: *const BYTE,
    type_: BYTE,
    pack: BYTE,
) -> BOOL {
    let mut ptr: *const BYTE = core::ptr::null();
    let mut pict_beg: *const BYTE = core::ptr::null();
    let mut alloc: *mut BYTE = core::ptr::null_mut();
    let mut bi: *mut BITMAPINFO = core::ptr::null_mut();
    let mut off: ULONG = 0;
    let mut csz: ULONG = 0;
    let mut nc: core::ffi::c_uint = 0;
    let mut clrImportant: BOOL = (FALSE as i32);
    let mut ret: BOOL = (FALSE as i32);
    let mut tmp: [core::ffi::c_char; 256] = [0; 256];
    let mut hs_size: core::ffi::c_uint = 0;
    let mut hs_offset: core::ffi::c_uint = 0;

    bi = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<BITMAPINFO>() as SIZE_T,
    ) as *mut BITMAPINFO;
    if bi.is_null() {
        return (FALSE) as BOOL;
    }

    ptr = beg.offset(2 as isize); /* for type and pack */

    (*bi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    (*bi).bmiHeader.biXPelsPerMeter = fetch_ulong(&raw mut ptr) as LONG;
    (*bi).bmiHeader.biYPelsPerMeter = fetch_ulong(&raw mut ptr) as LONG;
    (*bi).bmiHeader.biPlanes = fetch_ushort(&raw mut ptr) as WORD;
    (*bi).bmiHeader.biBitCount = fetch_ushort(&raw mut ptr) as WORD;
    (*bi).bmiHeader.biWidth = fetch_ulong(&raw mut ptr) as LONG;
    (*bi).bmiHeader.biHeight = fetch_ulong(&raw mut ptr) as LONG;
    (*bi).bmiHeader.biClrUsed = fetch_ulong(&raw mut ptr) as DWORD;
    clrImportant = fetch_ulong(&raw mut ptr) as BOOL;
    (*bi).bmiHeader.biClrImportant = (if clrImportant > 1 {
        clrImportant
    } else {
        0
    }) as DWORD;
    (*bi).bmiHeader.biCompression = BI_RGB as DWORD;
    if ((*bi).bmiHeader.biBitCount as core::ffi::c_int) > 32 {
        WINE_FIXME!(
            b"Unknown bit count %u\n\0".as_ptr() as *const core::ffi::c_char,
            (*bi).bmiHeader.biBitCount as core::ffi::c_int,
        );
    }
    if ((*bi).bmiHeader.biPlanes as core::ffi::c_int) != 1 {
        WINE_FIXME!(
            b"Unsupported planes %u\n\0".as_ptr() as *const core::ffi::c_char,
            (*bi).bmiHeader.biPlanes as core::ffi::c_int,
        );
    }
    (*bi).bmiHeader.biSizeImage = ((((*bi).bmiHeader.biWidth
        * (*bi).bmiHeader.biBitCount as LONG
        + 31)
        & !(31 as LONG))
        / 8 as LONG
        * (*bi).bmiHeader.biHeight) as DWORD;
    WINE_TRACE!(
        b"planes=%d bc=%d size=(%d,%d)\n\0".as_ptr() as *const core::ffi::c_char,
        (*bi).bmiHeader.biPlanes as core::ffi::c_int,
        (*bi).bmiHeader.biBitCount as core::ffi::c_int,
        (*bi).bmiHeader.biWidth,
        (*bi).bmiHeader.biHeight,
    );

    csz = fetch_ulong(&raw mut ptr);
    hs_size = fetch_ulong(&raw mut ptr) as core::ffi::c_uint;

    off = GET_UINT(ptr, 0) as ULONG;
    ptr = ptr.offset(4 as isize);
    hs_offset = GET_UINT(ptr, 0) as core::ffi::c_uint;
    ptr = ptr.offset(4 as isize);
    HLPFILE_AddHotSpotLinks(rd, file, beg, hs_size as ULONG, hs_offset as ULONG);

    /* now read palette info */
    if type_ as core::ffi::c_int == 0x06 {
        let mut i: core::ffi::c_uint;

        nc = (*bi).bmiHeader.biClrUsed as core::ffi::c_uint;
        /* not quite right, especially for bitfields type of compression */
        if nc == 0 && ((*bi).bmiHeader.biBitCount as core::ffi::c_int) <= 8 {
            nc = (1 << (*bi).bmiHeader.biBitCount as core::ffi::c_int)
                as core::ffi::c_uint;
        }

        bi = HeapReAlloc(
            GetProcessHeap(),
            0,
            bi as PVOID,
            core::mem::size_of::<BITMAPINFO>() as SIZE_T
                + (nc as SIZE_T) * (core::mem::size_of::<RGBQUAD>() as SIZE_T),
        ) as *mut BITMAPINFO;
        if bi.is_null() {
            return (FALSE) as BOOL;
        }
        i = 0;
        while i < nc {
            (*(&raw mut (*bi).bmiColors as *mut RGBQUAD).offset(i as isize)).rgbBlue =
                *ptr.offset(0 as isize);
            (*(&raw mut (*bi).bmiColors as *mut RGBQUAD).offset(i as isize)).rgbGreen =
                *ptr.offset(1 as isize);
            (*(&raw mut (*bi).bmiColors as *mut RGBQUAD).offset(i as isize)).rgbRed =
                *ptr.offset(2 as isize);
            (*(&raw mut (*bi).bmiColors as *mut RGBQUAD).offset(i as isize)).rgbReserved =
                0 as BYTE;
            ptr = ptr.offset(4 as isize);
            i = i + 1;
        }
    }
    pict_beg = HLPFILE_DecompressGfx(
        beg.offset(off as isize),
        csz as core::ffi::c_uint,
        (*bi).bmiHeader.biSizeImage as core::ffi::c_uint,
        pack,
        &raw mut alloc,
    );

    'done: {
        if clrImportant == 1 && nc > 0 {
            ret = HLPFILE_RtfAddTransparentBitmap(rd, bi, pict_beg as *const core::ffi::c_void, nc);
            break 'done;
        }
        if HLPFILE_RtfAddControl(rd, b"{\\pict\0".as_ptr() as *const core::ffi::c_char) == 0 {
            break 'done;
        }
        if type_ as core::ffi::c_int == 0x06 {
            sprintf(
                &raw mut tmp as *mut core::ffi::c_char,
                b"\\dibitmap0\\picw%d\\pich%d\0".as_ptr() as *const core::ffi::c_char,
                (*bi).bmiHeader.biWidth,
                (*bi).bmiHeader.biHeight,
            );
            if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut core::ffi::c_char) == 0 {
                break 'done;
            }
            if HLPFILE_RtfAddHexBytes(
                rd,
                bi as *const core::ffi::c_void,
                (core::mem::size_of::<BITMAPINFO>() as usize
                    + (nc as usize) * (core::mem::size_of::<RGBQUAD>() as usize))
                    as core::ffi::c_uint,
            ) == 0
            {
                break 'done;
            }
        } else {
            sprintf(
                &raw mut tmp as *mut core::ffi::c_char,
                b"\\wbitmap0\\wbmbitspixel%d\\wbmplanes%d\\picw%d\\pich%d\0".as_ptr()
                    as *const core::ffi::c_char,
                (*bi).bmiHeader.biBitCount as core::ffi::c_int,
                (*bi).bmiHeader.biPlanes as core::ffi::c_int,
                (*bi).bmiHeader.biWidth,
                (*bi).bmiHeader.biHeight,
            );
            if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut core::ffi::c_char) == 0 {
                break 'done;
            }
        }
        if HLPFILE_RtfAddHexBytes(
            rd,
            pict_beg as *const core::ffi::c_void,
            (*bi).bmiHeader.biSizeImage as core::ffi::c_uint,
        ) == 0
        {
            break 'done;
        }
        if HLPFILE_RtfAddControl(rd, b"}\0".as_ptr() as *const core::ffi::c_char) == 0 {
            break 'done;
        }

        ret = (TRUE as i32);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        bi as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        alloc as PVOID,
    );

    return ret;
}
unsafe extern "C" fn HLPFILE_RtfAddMetaFile(rd: *mut RtfData, file: *mut HLPFILE, beg: *const BYTE, pack: BYTE) -> BOOL {
    let mut size: ULONG;
    let mut csize: ULONG;
    let mut off: ULONG;
    let mut hs_offset: ULONG;
    let mut hs_size: ULONG;
    let mut ptr: *const BYTE;
    let mut bits: *const BYTE;
    let mut alloc: *mut BYTE = core::ptr::null_mut();
    let mut tmp: [core::ffi::c_char; 256] = [0; 256];
    let mut mm: core::ffi::c_uint;
    let mut ret: BOOL;

    WINE_TRACE!(b"Loading metafile\n\0".as_ptr() as *const core::ffi::c_char);

    ptr = beg.offset(2 as isize); /* for type and pack */

    mm = fetch_ushort(core::ptr::addr_of_mut!(ptr)) as core::ffi::c_uint; /* mapping mode */
    sprintf(
        tmp.as_mut_ptr(),
        b"{\\pict\\wmetafile%u\\picw%u\\pich%u\0".as_ptr() as *const core::ffi::c_char,
        mm,
        GET_USHORT(ptr, 0) as core::ffi::c_uint,
        GET_USHORT(ptr, 2) as core::ffi::c_uint,
    );
    if HLPFILE_RtfAddControl(rd, tmp.as_mut_ptr()) == 0 {
        return (FALSE) as BOOL;
    }
    ptr = ptr.offset(4 as isize);

    size = fetch_ulong(core::ptr::addr_of_mut!(ptr)); /* decompressed size */
    csize = fetch_ulong(core::ptr::addr_of_mut!(ptr)); /* compressed size */
    hs_size = fetch_ulong(core::ptr::addr_of_mut!(ptr)); /* hotspot size */
    off = GET_UINT(ptr, 0) as ULONG;
    hs_offset = GET_UINT(ptr, 4) as ULONG;
    ptr = ptr.offset(8 as isize);

    HLPFILE_AddHotSpotLinks(rd, file, beg, hs_size, hs_offset);

    WINE_TRACE!(
        b"sz=%u csz=%u offs=%u/%u,%u/%u\n\0".as_ptr() as *const core::ffi::c_char,
        size,
        csize,
        off,
        ptr.offset_from(beg) as ULONG,
        hs_size,
        hs_offset,
    );

    bits = HLPFILE_DecompressGfx(beg.offset(off as isize), csize as core::ffi::c_uint, size as core::ffi::c_uint, pack, core::ptr::addr_of_mut!(alloc));
    if bits.is_null() {
        return (FALSE) as BOOL;
    }

    ret = (HLPFILE_RtfAddHexBytes(rd, bits as *const core::ffi::c_void, size as core::ffi::c_uint) != 0
        && HLPFILE_RtfAddControl(rd, b"}\0".as_ptr() as *const core::ffi::c_char) != 0) as core::ffi::c_int as BOOL;

    HeapFree(GetProcessHeap(), 0 as DWORD, alloc as PVOID);

    return ret;
}

unsafe extern "C" fn HLPFILE_RtfAddGfxByAddr(
    rd: *mut RtfData,
    hlpfile: *mut HLPFILE,
    ref_: *const BYTE,
    size: ULONG,
) -> BOOL {
    let mut i: ::core::ffi::c_uint;
    let mut numpict: ::core::ffi::c_uint;

    numpict = GET_USHORT(ref_, 2) as ::core::ffi::c_uint;
    WINE_TRACE!(
        b"Got picture magic=%04x #=%d\n\0".as_ptr() as *const ::core::ffi::c_char,
        GET_USHORT(ref_, 0) as ::core::ffi::c_int,
        numpict as ::core::ffi::c_int,
    );

    i = 0;
    while i < numpict {
        let mut beg: *const BYTE;
        let mut ptr: *const BYTE;
        let mut type_: BYTE;
        let mut pack: BYTE;

        WINE_TRACE!(
            b"Offset[%d] = %x\n\0".as_ptr() as *const ::core::ffi::c_char,
            i,
            GET_UINT(
                ref_,
                (1 + i) * 4,
            ),
        );
        ptr = ref_.offset(
            GET_UINT(
                ref_,
                (1 + i) * 4,
            ) as isize,
        );
        beg = ptr;

        type_ = *ptr;
        ptr = ptr.offset(1);
        pack = *ptr;
        ptr = ptr.offset(1);

        match type_ as ::core::ffi::c_int {
            5 | 6 => {
                HLPFILE_RtfAddBitmap(rd, hlpfile, beg, type_, pack);
            }
            8 => {
                HLPFILE_RtfAddMetaFile(rd, hlpfile, beg, pack);
            }
            _ => {
                WINE_FIXME!(
                    b"Unknown type %u\n\0".as_ptr() as *const ::core::ffi::c_char,
                    type_ as ::core::ffi::c_uint,
                );
                return (FALSE) as BOOL;
            }
        }

        /* FIXME: hotspots */

        /* FIXME: implement support for multiple picture format */
        if numpict != 1 {
            WINE_FIXME!(
                b"Supporting only one bitmap format per logical bitmap (for now). Using first format\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
        break;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_RtfAddGfxByIndex(
    mut rd: *mut RtfData,
    mut hlpfile: *mut HLPFILE,
    mut index: ::core::ffi::c_uint,
) -> BOOL {
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    let mut ref_0: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut end: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    WINE_TRACE!(
        b"Loading picture #%d\n\0".as_ptr() as *const ::core::ffi::c_char,
        index,
    );
    sprintf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"|bm%u\0".as_ptr() as *const ::core::ffi::c_char,
        index,
    );
    if HLPFILE_FindSubFile(
        hlpfile,
        &raw mut tmp as *mut ::core::ffi::c_char as LPCSTR,
        &raw mut ref_0,
        &raw mut end,
    ) == 0
    {
        WINE_WARN!(b"no sub file\n\0".as_ptr() as *const ::core::ffi::c_char);
        return (FALSE) as BOOL;
    }
    ref_0 = ref_0.offset(9 as isize);
    return HLPFILE_RtfAddGfxByAddr(
        rd,
        hlpfile,
        ref_0,
        end.offset_from(ref_0) as ::core::ffi::c_longlong as ULONG,
    );
}

unsafe extern "C" fn HLPFILE_AllocLink(
    rd: *mut RtfData,
    cookie: ::core::ffi::c_int,
    str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_uint,
    hash: LONG,
    clrChange: BOOL,
    bHotSpot: BOOL,
    wnd: ::core::ffi::c_uint,
) -> *mut HLPFILE_LINK {
    let mut link: *mut HLPFILE_LINK = ::core::ptr::null_mut();
    let mut link_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let asz: ::core::ffi::c_uint = (if bHotSpot != 0 {
        ::core::mem::size_of::<HLPFILE_HOTSPOTLINK>()
    } else {
        ::core::mem::size_of::<HLPFILE_LINK>()
    }) as ::core::ffi::c_uint;

    if len == -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint {
        len = strlen(str) as ::core::ffi::c_uint;
    }
    link = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        asz.wrapping_add(len).wrapping_add(1 as ::core::ffi::c_uint) as SIZE_T,
    ) as *mut HLPFILE_LINK;
    if link.is_null() {
        return ::core::ptr::null_mut();
    }

    (*link).cookie = cookie as _;
    link_str = (link as *mut ::core::ffi::c_char).offset(asz as isize);
    (*link).string = link_str as LPCSTR;
    memcpy(
        link_str as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len as usize,
    );
    *link_str.offset(len as isize) = 0;
    (*link).hash = hash;
    (*link).set_bClrChange(clrChange as ::core::ffi::c_uint);
    (*link).set_bHotSpot(bHotSpot as ::core::ffi::c_uint);
    (*link).window = wnd;
    (*link).next = (*rd).first_link as *mut tagHlpFileLink;
    (*rd).first_link = link;
    (*link).cpMin = (*rd).char_pos as DWORD;
    (*rd).force_color = clrChange;
    if !(*rd).current_link.is_null() {
        WINE_FIXME!("Pending link\n");
    }
    if bHotSpot != 0 {
        (*link).cpMax = (*rd).char_pos as DWORD;
    } else {
        (*rd).current_link = link;
    }

    WINE_TRACE!(
        "Link[%d] to %s@%08x:%d\n",
        (*link).cookie as ::core::ffi::c_uint,
        debugstr_a((*link).string as *const ::core::ffi::c_char),
        (*link).hash,
        (*link).window,
    );
    return link;
}

unsafe extern "C" fn HLPFILE_HalfPointsScale(page: *mut HLPFILE_PAGE, pts: core::ffi::c_uint) -> core::ffi::c_uint {
    return pts
        .wrapping_mul((*(*page).file).scale as core::ffi::c_uint)
        .wrapping_sub((*(*page).file).rounderr as core::ffi::c_uint);
}

#[no_mangle]
pub unsafe extern "C" fn HLPFILE_BrowsePage(
    page: *mut HLPFILE_PAGE,
    rd: *mut RtfData,
    font_scale: ::core::ffi::c_uint,
    relative: ::core::ffi::c_uint,
) -> BOOL {
    let hlpfile: *mut HLPFILE = (*page).file as *mut HLPFILE;
    let mut buf: *mut BYTE = core::ptr::null_mut();
    let mut end: *mut BYTE = core::ptr::null_mut();
    let mut ref_0: DWORD = (*page).reference;
    let mut index: ::core::ffi::c_uint = 0;
    let mut old_index: ::core::ffi::c_uint = -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint;
    let mut offset: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut offs: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut cpg: ::core::ffi::c_uint = 0;
    let mut parlen: ::core::ffi::c_uint = 0;
    let mut tmp: [::core::ffi::c_char; 1024 as usize] = [0; 1024 as usize];
    let mut ck: *const ::core::ffi::c_char = core::ptr::null();

    (*rd).in_text = TRUE as BOOL;
    (*rd).allocated = 32768 as ::core::ffi::c_uint;
    (*rd).ptr = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (*rd).allocated as SIZE_T,
    ) as *mut ::core::ffi::c_char;
    (*rd).data = (*rd).ptr;
    (*rd).char_pos = 0 as ::core::ffi::c_uint;
    (*rd).current_link = core::ptr::null_mut();
    (*rd).first_link = (*rd).current_link;
    (*rd).force_color = FALSE as BOOL;
    (*rd).font_scale = font_scale;
    (*rd).relative = relative;
    (*rd).char_pos_rel = 0 as ::core::ffi::c_uint;

    match ((*hlpfile).charset as ::core::ffi::c_int) as u32 {
        DEFAULT_CHARSET | ANSI_CHARSET => {
            cpg = 1252 as ::core::ffi::c_uint;
        }
        SHIFTJIS_CHARSET => {
            cpg = 932 as ::core::ffi::c_uint;
        }
        HANGEUL_CHARSET => {
            cpg = 949 as ::core::ffi::c_uint;
        }
        GB2312_CHARSET => {
            cpg = 936 as ::core::ffi::c_uint;
        }
        CHINESEBIG5_CHARSET => {
            cpg = 950 as ::core::ffi::c_uint;
        }
        GREEK_CHARSET => {
            cpg = 1253 as ::core::ffi::c_uint;
        }
        TURKISH_CHARSET => {
            cpg = 1254 as ::core::ffi::c_uint;
        }
        HEBREW_CHARSET => {
            cpg = 1255 as ::core::ffi::c_uint;
        }
        ARABIC_CHARSET => {
            cpg = 1256 as ::core::ffi::c_uint;
        }
        BALTIC_CHARSET => {
            cpg = 1257 as ::core::ffi::c_uint;
        }
        VIETNAMESE_CHARSET => {
            cpg = 1258 as ::core::ffi::c_uint;
        }
        RUSSIAN_CHARSET => {
            cpg = 1251 as ::core::ffi::c_uint;
        }
        EE_CHARSET => {
            cpg = 1250 as ::core::ffi::c_uint;
        }
        THAI_CHARSET => {
            cpg = 874 as ::core::ffi::c_uint;
        }
        JOHAB_CHARSET => {
            cpg = 1361 as ::core::ffi::c_uint;
        }
        MAC_CHARSET => {
            ck = b"mac\0".as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!(
                b"Unsupported charset %u\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*hlpfile).charset as ::core::ffi::c_uint,
            );
            cpg = 1252 as ::core::ffi::c_uint;
        }
    }
    if !ck.is_null() {
        sprintf(
            core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char,
            b"{\\rtf1\\%s\\deff0\0".as_ptr() as *const ::core::ffi::c_char,
            ck,
        );
        if HLPFILE_RtfAddControl(rd, core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char) == 0 {
            return FALSE as BOOL;
        }
    } else {
        sprintf(
            core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char,
            b"{\\rtf1\\ansi\\ansicpg%d\\deff0\0".as_ptr() as *const ::core::ffi::c_char,
            cpg,
        );
        if HLPFILE_RtfAddControl(rd, core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char) == 0 {
            return FALSE as BOOL;
        }
    }

    /* generate font table */
    if HLPFILE_RtfAddControl(rd, b"{\\fonttbl\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return FALSE as BOOL;
    }
    index = 0 as ::core::ffi::c_uint;
    while index < (*hlpfile).numFonts {
        let mut family: *const ::core::ffi::c_char = core::ptr::null();
        match (((*(*hlpfile).fonts.offset(index as isize)).LogFont.lfPitchAndFamily as ::core::ffi::c_int
            & 0xF0 as ::core::ffi::c_int)) as u32
        {
            FF_MODERN => {
                family = b"modern\0".as_ptr() as *const ::core::ffi::c_char;
            }
            FF_ROMAN => {
                family = b"roman\0".as_ptr() as *const ::core::ffi::c_char;
            }
            FF_SWISS => {
                family = b"swiss\0".as_ptr() as *const ::core::ffi::c_char;
            }
            FF_SCRIPT => {
                family = b"script\0".as_ptr() as *const ::core::ffi::c_char;
            }
            FF_DECORATIVE => {
                family = b"decor\0".as_ptr() as *const ::core::ffi::c_char;
            }
            _ => {
                family = b"nil\0".as_ptr() as *const ::core::ffi::c_char;
            }
        }
        sprintf(
            core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char,
            b"{\\f%d\\f%s\\fprq%d\\fcharset%d %s;}\0".as_ptr() as *const ::core::ffi::c_char,
            index,
            family,
            ((*(*hlpfile).fonts.offset(index as isize)).LogFont.lfPitchAndFamily as ::core::ffi::c_int
                & 0x0F as ::core::ffi::c_int),
            (*(*hlpfile).fonts.offset(index as isize)).LogFont.lfCharSet as ::core::ffi::c_int,
            core::ptr::addr_of_mut!((*(*hlpfile).fonts.offset(index as isize)).LogFont.lfFaceName) as *mut CHAR,
        );
        if HLPFILE_RtfAddControl(rd, core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char) == 0 {
            return FALSE as BOOL;
        }
        index = index.wrapping_add(1);
    }
    if HLPFILE_RtfAddControl(rd, b"}\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return FALSE as BOOL;
    }

    /* generate color table */
    if HLPFILE_RtfAddControl(
        rd,
        b"{\\colortbl ;\\red0\\green128\\blue0;\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0
    {
        return FALSE as BOOL;
    }
    index = 0 as ::core::ffi::c_uint;
    while index < (*hlpfile).numFonts {
        sprintf(
            core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char,
            b"\\red%d\\green%d\\blue%d;\0".as_ptr() as *const ::core::ffi::c_char,
            GetRValue!((*(*hlpfile).fonts.offset(index as isize)).color) as ::core::ffi::c_int,
            GetGValue!((*(*hlpfile).fonts.offset(index as isize)).color) as ::core::ffi::c_int,
            GetBValue!((*(*hlpfile).fonts.offset(index as isize)).color) as ::core::ffi::c_int,
        );
        if HLPFILE_RtfAddControl(rd, core::ptr::addr_of_mut!(tmp) as *mut ::core::ffi::c_char) == 0 {
            return FALSE as BOOL;
        }
        index = index.wrapping_add(1);
    }
    if HLPFILE_RtfAddControl(rd, b"}\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return FALSE as BOOL;
    }

    'done: {
        loop {
            if ((*hlpfile).version as ::core::ffi::c_int) <= (16 as ::core::ffi::c_int) {
                index = ref_0.wrapping_sub(0x0C as DWORD).wrapping_div((*hlpfile).dsize as DWORD)
                    as ::core::ffi::c_uint;
                offset = ref_0.wrapping_sub(0x0C as DWORD).wrapping_rem((*hlpfile).dsize as DWORD)
                    as ::core::ffi::c_uint;
            } else {
                index = (ref_0.wrapping_sub(0x0C as DWORD) >> (14 as ::core::ffi::c_int))
                    as ::core::ffi::c_uint;
                offset = (ref_0.wrapping_sub(0x0C as DWORD) & (0x3FFF as DWORD))
                    as ::core::ffi::c_uint;
            }

            if ((*hlpfile).version as ::core::ffi::c_int) <= (16 as ::core::ffi::c_int)
                && index != old_index
                && old_index != -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint
            {
                /* we jumped to the next block, adjust pointers */
                ref_0 = ref_0.wrapping_sub(12 as DWORD);
                offset = offset.wrapping_sub(12 as ::core::ffi::c_uint);
            }

            if index >= (*hlpfile).topic_maplen {
                WINE_WARN!(b"maplen\n\0".as_ptr() as *const ::core::ffi::c_char);
                break;
            }
            buf = (*(*hlpfile).topic_map.offset(index as isize)).offset(offset as isize);
            if buf.offset(0x15 as isize) >= (*hlpfile).topic_end {
                WINE_WARN!(b"extra\n\0".as_ptr() as *const ::core::ffi::c_char);
                break;
            }
            end = min!(
                buf.offset(GET_UINT(buf, 0 as ::core::ffi::c_uint) as isize),
                (*hlpfile).topic_end,
            );
            if index != old_index {
                offs = 0 as ::core::ffi::c_uint;
                old_index = index;
            }

            match (*buf.offset(0x14 as isize) as ::core::ffi::c_int) as u32 {
                HLP_TOPICHDR => {
                    let fresh32 = count;
                    count = count.wrapping_add(1);
                    if fresh32 != 0 {
                        break 'done;
                    }
                }
                HLP_DISPLAY30 | HLP_DISPLAY | HLP_TABLE => {
                    if HLPFILE_BrowseParagraph(page, rd, buf, end, core::ptr::addr_of_mut!(parlen)) == 0 {
                        return FALSE as BOOL;
                    }
                    if relative
                        > index
                            .wrapping_mul(0x8000 as ::core::ffi::c_uint)
                            .wrapping_add(offs)
                    {
                        (*rd).char_pos_rel = (*rd).char_pos;
                    }
                    offs = offs.wrapping_add(parlen);
                }
                _ => {
                    WINE_ERR!(
                        b"buf[0x14] = %x\n\0".as_ptr() as *const ::core::ffi::c_char,
                        *buf.offset(0x14 as isize) as ::core::ffi::c_int,
                    );
                }
            }

            if ((*hlpfile).version as ::core::ffi::c_int) <= (16 as ::core::ffi::c_int) {
                ref_0 = ref_0.wrapping_add(
                    GET_UINT(buf, 0xc as ::core::ffi::c_uint) as DWORD,
                );
                if GET_UINT(buf, 0xc as ::core::ffi::c_uint)
                    == 0 as ::core::ffi::c_uint
                {
                    break;
                }
            } else {
                ref_0 = GET_UINT(buf, 0xc as ::core::ffi::c_uint) as DWORD;
            }
            if !(ref_0 != 0xffffffff as DWORD) {
                break;
            }
        }
    }
    (*page).first_link = (*rd).first_link;
    return HLPFILE_RtfAddControl(rd, b"}\0".as_ptr() as *const ::core::ffi::c_char);
}

unsafe extern "C" fn HLPFILE_ReadFont(mut hlpfile: *mut HLPFILE) -> BOOL {
    let mut ref_0: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut end: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut i: core::ffi::c_uint = 0;
    let mut len: core::ffi::c_uint = 0;
    let mut idx: core::ffi::c_uint = 0;
    let mut face_num: core::ffi::c_uint = 0;
    let mut dscr_num: core::ffi::c_uint = 0;
    let mut face_offset: core::ffi::c_uint = 0;
    let mut dscr_offset: core::ffi::c_uint = 0;
    let mut flag: BYTE = 0;
    let mut family: BYTE = 0;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|FONT\0".as_ptr() as LPCSTR,
        &mut ref_0,
        &mut end,
    ) == 0
    {
        WINE_WARN!(b"no subfile FONT\n\0".as_ptr() as *const core::ffi::c_char);
        (*hlpfile).numFonts = 0 as core::ffi::c_uint;
        (*hlpfile).fonts = core::ptr::null_mut::<HLPFILE_FONT>();
        return FALSE as BOOL;
    }

    ref_0 = ref_0.offset(9 as isize);

    face_num = GET_USHORT(ref_0, 0 as core::ffi::c_uint) as core::ffi::c_uint;
    dscr_num = GET_USHORT(ref_0, 2 as core::ffi::c_uint) as core::ffi::c_uint;
    face_offset = GET_USHORT(ref_0, 4 as core::ffi::c_uint) as core::ffi::c_uint;
    dscr_offset = GET_USHORT(ref_0, 6 as core::ffi::c_uint) as core::ffi::c_uint;

    WINE_TRACE!(
        b"Got NumFacenames=%u@%u NumDesc=%u@%u\n\0".as_ptr() as *const core::ffi::c_char,
        face_num,
        face_offset,
        dscr_num,
        dscr_offset,
    );

    (*hlpfile).numFonts = dscr_num;
    (*hlpfile).fonts = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<HLPFILE_FONT>() as SIZE_T).wrapping_mul(dscr_num as SIZE_T),
    ) as *mut HLPFILE_FONT;

    len = dscr_offset.wrapping_sub(face_offset).wrapping_div(face_num);

    /* mvb font */
    if face_offset >= 16 as core::ffi::c_uint {
        (*hlpfile).scale = 1 as core::ffi::c_int;
        (*hlpfile).rounderr = 0 as core::ffi::c_int;
        WINE_FIXME!(b"mvb font: not implemented\n\0".as_ptr() as *const core::ffi::c_char);
        return FALSE as BOOL;
    }
    /* new font */
    if face_offset >= 12 as core::ffi::c_uint {
        (*hlpfile).scale = 1 as core::ffi::c_int;
        (*hlpfile).rounderr = 0 as core::ffi::c_int;
        WINE_FIXME!(b"new font: not implemented\n\0".as_ptr() as *const core::ffi::c_char);
        return FALSE as BOOL;
    }
    /* old font */
    (*hlpfile).scale = 10 as core::ffi::c_int;
    (*hlpfile).rounderr = 5 as core::ffi::c_int;
    /* EPP     for (i = face_offset; i < dscr_offset; i += len) */
    /* EPP         WINE_FIXME("[%d]: %*s\n", i / len, len, ref + i); */
    i = 0 as core::ffi::c_uint;
    while i < dscr_num {
        flag = *ref_0.offset(
            dscr_offset
                .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                .wrapping_add(0 as core::ffi::c_uint) as isize,
        );
        family = *ref_0.offset(
            dscr_offset
                .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                .wrapping_add(2 as core::ffi::c_uint) as isize,
        );

        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfHeight = *ref_0.offset(
            dscr_offset
                .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                .wrapping_add(1 as core::ffi::c_uint) as isize,
        ) as LONG;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfWidth = 0 as LONG;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfEscapement = 0 as LONG;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfOrientation = 0 as LONG;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfWeight =
            (if (flag as core::ffi::c_int & 1 as core::ffi::c_int) != 0 {
                700
            } else {
                400
            }) as LONG;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfItalic =
            ((flag as core::ffi::c_int & 2 as core::ffi::c_int)
                != 0 as core::ffi::c_int) as core::ffi::c_int as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfUnderline =
            ((flag as core::ffi::c_int & 4 as core::ffi::c_int)
                != 0 as core::ffi::c_int) as core::ffi::c_int as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfStrikeOut =
            ((flag as core::ffi::c_int & 8 as core::ffi::c_int)
                != 0 as core::ffi::c_int) as core::ffi::c_int as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfCharSet = (*hlpfile).charset as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfOutPrecision = OUT_DEFAULT_PRECIS as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfClipPrecision = CLIP_DEFAULT_PRECIS as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfQuality = DEFAULT_QUALITY as BYTE;
        (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfPitchAndFamily = DEFAULT_PITCH as BYTE;

        match family as core::ffi::c_int {
            0x01 => {
                let fresh19 = &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfPitchAndFamily;
                *fresh19 = (*fresh19 as core::ffi::c_int | FF_MODERN as core::ffi::c_int) as BYTE;
            }
            0x02 => {
                let fresh20 = &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfPitchAndFamily;
                *fresh20 = (*fresh20 as core::ffi::c_int | FF_ROMAN as core::ffi::c_int) as BYTE;
            }
            0x03 => {
                let fresh21 = &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfPitchAndFamily;
                *fresh21 = (*fresh21 as core::ffi::c_int | FF_SWISS as core::ffi::c_int) as BYTE;
            }
            0x04 => {
                let fresh22 = &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfPitchAndFamily;
                *fresh22 = (*fresh22 as core::ffi::c_int | FF_SCRIPT as core::ffi::c_int) as BYTE;
            }
            0x05 => {
                let fresh23 = &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfPitchAndFamily;
                *fresh23 = (*fresh23 as core::ffi::c_int | FF_DECORATIVE as core::ffi::c_int) as BYTE;
            }
            _ => {
                WINE_FIXME!(
                    b"Unknown family %u\n\0".as_ptr() as *const core::ffi::c_char,
                    family as core::ffi::c_int,
                );
            }
        }

        idx = GET_USHORT(
            ref_0,
            dscr_offset
                .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                .wrapping_add(3 as core::ffi::c_uint),
        ) as core::ffi::c_uint;

        if idx < face_num {
            memcpy(
                &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfFaceName as *mut CHAR
                    as *mut core::ffi::c_void,
                ref_0
                    .offset(face_offset as isize)
                    .offset(idx.wrapping_mul(len) as isize) as *const core::ffi::c_void,
                min!(len, (LF_FACESIZE - 1) as core::ffi::c_uint) as size_t,
            );
            (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfFaceName
                [min!(len, (LF_FACESIZE - 1) as core::ffi::c_uint) as usize] = b'\0' as CHAR;
        } else {
            WINE_FIXME!(
                b"Too high face ref (%u/%u)\n\0".as_ptr() as *const core::ffi::c_char,
                idx,
                face_num,
            );
            strcpy(
                &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfFaceName as *mut CHAR
                    as *mut core::ffi::c_char,
                b"Helv\0".as_ptr() as *const core::ffi::c_char,
            );
        }

        (*(*hlpfile).fonts.offset(i as isize)).hFont = core::ptr::null_mut();
        (*(*hlpfile).fonts.offset(i as isize)).color = RGB!(
            *ref_0.offset(
                dscr_offset
                    .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                    .wrapping_add(5 as core::ffi::c_uint) as isize,
            ),
            *ref_0.offset(
                dscr_offset
                    .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                    .wrapping_add(6 as core::ffi::c_uint) as isize,
            ),
            *ref_0.offset(
                dscr_offset
                    .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                    .wrapping_add(7 as core::ffi::c_uint) as isize,
            ),
        );

        /* X(b,s) ((flag & (1 << b)) ? "-"s: "") expanded inline */
        WINE_TRACE!(
            b"Font[%d]: flags=%02x%s%s%s%s%s%s pSize=%u family=%u face=%s[%u] color=%08x\n\0"
                .as_ptr() as *const core::ffi::c_char,
            i,
            flag as core::ffi::c_int,
            if (flag as core::ffi::c_int & (1 << 0)) != 0 {
                b"-bold\0".as_ptr() as *const core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const core::ffi::c_char
            },
            if (flag as core::ffi::c_int & (1 << 1)) != 0 {
                b"-italic\0".as_ptr() as *const core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const core::ffi::c_char
            },
            if (flag as core::ffi::c_int & (1 << 2)) != 0 {
                b"-underline\0".as_ptr() as *const core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const core::ffi::c_char
            },
            if (flag as core::ffi::c_int & (1 << 3)) != 0 {
                b"-strikeOut\0".as_ptr() as *const core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const core::ffi::c_char
            },
            if (flag as core::ffi::c_int & (1 << 4)) != 0 {
                b"-dblUnderline\0".as_ptr() as *const core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const core::ffi::c_char
            },
            if (flag as core::ffi::c_int & (1 << 5)) != 0 {
                b"-smallCaps\0".as_ptr() as *const core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const core::ffi::c_char
            },
            *ref_0.offset(
                dscr_offset
                    .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                    .wrapping_add(1 as core::ffi::c_uint) as isize,
            ) as core::ffi::c_int,
            family as core::ffi::c_int,
            debugstr_a(
                &mut (*(*hlpfile).fonts.offset(i as isize)).LogFont.lfFaceName as *mut CHAR,
            ),
            idx,
            GET_UINT(
                ref_0,
                dscr_offset
                    .wrapping_add(i.wrapping_mul(11 as core::ffi::c_uint))
                    .wrapping_add(5 as core::ffi::c_uint),
            ) & 0x00FFFFFF as core::ffi::c_uint,
        );

        i = i.wrapping_add(1);
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn HLPFILE_ReadFileToBuffer(hlpfile: *mut HLPFILE, hFile: HFILE) -> BOOL {
    let mut header: [BYTE; 16] = [0; 16];
    let mut dummy: [BYTE; 1] = [0; 1];

    if _hread(hFile, core::ptr::addr_of_mut!(header) as *mut BYTE as LPVOID, 16 as LONG) != 16 as LONG {
        WINE_WARN!(b"header\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    };

    /* sanity checks */
    if GET_UINT(core::ptr::addr_of_mut!(header) as *mut BYTE as *const BYTE, 0 as u32) != 0x00035F3F as u32
    {
        WINE_WARN!(b"wrong header\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    };

    (*hlpfile).file_buffer_size = GET_UINT(core::ptr::addr_of_mut!(header) as *mut BYTE as *const BYTE, 12 as u32) as UINT;
    (*hlpfile).file_buffer = HeapAlloc(GetProcessHeap(), 0 as DWORD, (*hlpfile).file_buffer_size.wrapping_add(1 as UINT) as SIZE_T) as *mut BYTE;
    if (*hlpfile).file_buffer.is_null() {
        return (FALSE) as BOOL;
    }

    memcpy((*hlpfile).file_buffer as *mut core::ffi::c_void, core::ptr::addr_of_mut!(header) as *mut BYTE as *const core::ffi::c_void, 16 as usize);
    if (_hread(hFile, (*hlpfile).file_buffer.offset(16 as isize) as LPVOID, ((*hlpfile).file_buffer_size.wrapping_sub(16 as UINT)) as LONG) as u32) != (*hlpfile).file_buffer_size.wrapping_sub(16 as UINT)
    {
        WINE_WARN!(b"filesize1\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    };

    if _hread(hFile, core::ptr::addr_of_mut!(dummy) as *mut BYTE as LPVOID, 1 as LONG) != 0 as LONG {
        WINE_WARN!(b"filesize2\n\0".as_ptr() as *const i8);
    }

    *(*hlpfile).file_buffer.offset((*hlpfile).file_buffer_size as isize) = '\0' as BYTE; /* FIXME: was '0', sounds backwards to me */

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_SystemCommands(mut hlpfile: *mut HLPFILE) -> BOOL {
    let mut buf: *mut BYTE = core::ptr::null_mut();
    let mut ptr: *mut BYTE = core::ptr::null_mut();
    let mut end: *mut BYTE = core::ptr::null_mut();
    let mut macro_0: *mut HLPFILE_MACRO = core::ptr::null_mut();
    let mut m: *mut *mut HLPFILE_MACRO = core::ptr::null_mut();
    let mut p: LPSTR = core::ptr::null_mut();
    let mut magic: ::core::ffi::c_ushort = 0;
    let mut minor: ::core::ffi::c_ushort = 0;
    let mut major: ::core::ffi::c_ushort = 0;
    let mut flags: ::core::ffi::c_ushort = 0;

    (*hlpfile).lpszTitle = core::ptr::null_mut();

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|SYSTEM\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(buf),
        core::ptr::addr_of_mut!(end),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    magic = GET_USHORT(
        buf.offset(9 as isize),
        0 as ::core::ffi::c_uint,
    );
    minor = GET_USHORT(
        buf.offset(9 as isize),
        2 as ::core::ffi::c_uint,
    );
    major = GET_USHORT(
        buf.offset(9 as isize),
        4 as ::core::ffi::c_uint,
    );
    flags = GET_USHORT(
        buf.offset(9 as isize),
        10 as ::core::ffi::c_uint,
    );
    WINE_TRACE!(
        b"Got system header: magic=%04x version=%d.%d flags=%04x\n\0".as_ptr()
            as *const ::core::ffi::c_char,
        magic as ::core::ffi::c_int,
        major as ::core::ffi::c_int,
        minor as ::core::ffi::c_int,
        flags as ::core::ffi::c_int,
    );
    if magic as ::core::ffi::c_int != 0x036C as ::core::ffi::c_int
        || major as ::core::ffi::c_int != 1 as ::core::ffi::c_int
    {
        WINE_WARN!(b"Wrong system header\n\0".as_ptr() as *const ::core::ffi::c_char);
        return (FALSE) as BOOL;
    }
    if minor as ::core::ffi::c_int <= 16 as ::core::ffi::c_int {
        (*hlpfile).tbsize = 0x800 as ::core::ffi::c_ushort;
        (*hlpfile).compressed = FALSE as BOOL;
    } else if flags as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*hlpfile).tbsize = 0x1000 as ::core::ffi::c_ushort;
        (*hlpfile).compressed = FALSE as BOOL;
    } else if flags as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
        (*hlpfile).tbsize = 0x1000 as ::core::ffi::c_ushort;
        (*hlpfile).compressed = TRUE as BOOL;
    } else {
        (*hlpfile).tbsize = 0x800 as ::core::ffi::c_ushort;
        (*hlpfile).compressed = TRUE as BOOL;
    }

    if (*hlpfile).compressed != 0 {
        (*hlpfile).dsize = 0x4000 as ::core::ffi::c_ushort;
    } else {
        (*hlpfile).dsize = ((*hlpfile).tbsize as ::core::ffi::c_int
            - 0x0C as ::core::ffi::c_int) as ::core::ffi::c_ushort;
    }

    (*hlpfile).version = minor;
    (*hlpfile).flags = flags;
    (*hlpfile).charset = DEFAULT_CHARSET as ::core::ffi::c_ushort;

    if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int {
        let mut str: *mut ::core::ffi::c_char =
            (buf as *mut ::core::ffi::c_char).offset(0x15 as isize);

        (*hlpfile).lpszTitle = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (strlen(str) as SIZE_T)
                .wrapping_add(1 as SIZE_T),
        ) as LPSTR;
        if (*hlpfile).lpszTitle.is_null() {
            return (FALSE) as BOOL;
        }
        strcpy((*hlpfile).lpszTitle as *mut ::core::ffi::c_char, str);
        WINE_TRACE!(
            b"Title: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_a((*hlpfile).lpszTitle as *const ::core::ffi::c_char),
        );
        return (TRUE) as BOOL;
    }

    ptr = buf.offset(0x15 as isize);
    while ptr.offset(4 as isize) <= end {
        let mut str: *mut ::core::ffi::c_char =
            (ptr as *mut ::core::ffi::c_char).offset(4 as isize);

        'sw: {
            match GET_USHORT(ptr, 0 as ::core::ffi::c_uint)
                as ::core::ffi::c_int
            {
                1 => {
                    if !(*hlpfile).lpszTitle.is_null() {
                        WINE_WARN!(
                            b"title\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break 'sw;
                    }
                    (*hlpfile).lpszTitle = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        (strlen(str) as SIZE_T)
                            .wrapping_add(1 as SIZE_T),
                    ) as LPSTR;
                    if (*hlpfile).lpszTitle.is_null() {
                        return (FALSE) as BOOL;
                    }
                    strcpy((*hlpfile).lpszTitle as *mut ::core::ffi::c_char, str);
                    WINE_TRACE!(
                        b"Title: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                        debugstr_a((*hlpfile).lpszTitle as *const ::core::ffi::c_char),
                    );
                    break 'sw;
                }

                2 => {
                    if !(*hlpfile).lpszCopyright.is_null() {
                        WINE_WARN!(
                            b"copyright\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break 'sw;
                    }
                    (*hlpfile).lpszCopyright = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        (strlen(str) as SIZE_T)
                            .wrapping_add(1 as SIZE_T),
                    ) as LPSTR;
                    if (*hlpfile).lpszCopyright.is_null() {
                        return (FALSE) as BOOL;
                    }
                    strcpy((*hlpfile).lpszCopyright as *mut ::core::ffi::c_char, str);
                    WINE_TRACE!(
                        b"Copyright: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                        debugstr_a(
                            (*hlpfile).lpszCopyright as *const ::core::ffi::c_char,
                        ),
                    );
                    break 'sw;
                }

                3 => {
                    if GET_USHORT(ptr, 2 as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                        != 4 as ::core::ffi::c_int
                    {
                        WINE_WARN!(
                            b"system3\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break 'sw;
                    }
                    (*hlpfile).contents_start = GET_UINT(
                        ptr,
                        4 as ::core::ffi::c_uint,
                    ) as ::core::ffi::c_ulong;
                    WINE_TRACE!(
                        b"Setting contents start at %08lx\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        (*hlpfile).contents_start,
                    );
                    break 'sw;
                }

                4 => {
                    macro_0 = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        (::core::mem::size_of::<HLPFILE_MACRO>() as SIZE_T)
                            .wrapping_add(strlen(str) as SIZE_T)
                            .wrapping_add(1 as SIZE_T),
                    ) as *mut HLPFILE_MACRO;
                    if macro_0.is_null() {
                        break 'sw;
                    }
                    p = (macro_0 as *mut ::core::ffi::c_char).offset(
                        ::core::mem::size_of::<HLPFILE_MACRO>() as isize,
                    ) as LPSTR;
                    strcpy(p as *mut ::core::ffi::c_char, str);
                    (*macro_0).lpszMacro = p as LPCSTR;
                    (*macro_0).next = core::ptr::null_mut();
                    m = core::ptr::addr_of_mut!((*hlpfile).first_macro);
                    while !(*m).is_null() {
                        m = core::ptr::addr_of_mut!((**m).next);
                    }
                    *m = macro_0;
                    break 'sw;
                }

                5 => {
                    if GET_USHORT(
                        ptr,
                        (4 + 4) as ::core::ffi::c_uint,
                    ) as ::core::ffi::c_int
                        != 1 as ::core::ffi::c_int
                    {
                        WINE_FIXME!(
                            b"More than one icon, picking up first\n\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                    /* 0x16 is sizeof(CURSORICONDIR), see user32/user_private.h */
                    (*hlpfile).hIcon = CreateIconFromResourceEx(
                        ptr.offset(4 as isize)
                            .offset(0x16 as isize),
                        ((GET_USHORT(ptr, 2 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                            - 0x16 as ::core::ffi::c_int)
                            as DWORD),
                        TRUE as BOOL,
                        0x30000 as DWORD,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as UINT,
                    );
                    break 'sw;
                }

                6 => {
                    if GET_USHORT(ptr, 2 as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                        != 90 as ::core::ffi::c_int
                    {
                        WINE_WARN!(
                            b"system6\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break 'sw;
                    }

                    if !(*hlpfile).windows.is_null() {
                        (*hlpfile).numWindows = (*hlpfile).numWindows.wrapping_add(1);
                        (*hlpfile).windows = HeapReAlloc(
                            GetProcessHeap(),
                            0 as DWORD,
                            (*hlpfile).windows as PVOID,
                            (::core::mem::size_of::<HLPFILE_WINDOWINFO>() as SIZE_T)
                                .wrapping_mul((*hlpfile).numWindows as SIZE_T),
                        ) as *mut HLPFILE_WINDOWINFO;
                    } else {
                        (*hlpfile).numWindows = (*hlpfile).numWindows.wrapping_add(1);
                        (*hlpfile).windows = HeapAlloc(
                            GetProcessHeap(),
                            0 as DWORD,
                            (::core::mem::size_of::<HLPFILE_WINDOWINFO>() as SIZE_T)
                                .wrapping_mul((*hlpfile).numWindows as SIZE_T),
                        ) as *mut HLPFILE_WINDOWINFO;
                    }

                    if !(*hlpfile).windows.is_null() {
                        let mut wi: *mut HLPFILE_WINDOWINFO = (*hlpfile)
                            .windows
                            .offset(
                                (*hlpfile)
                                    .numWindows
                                    .wrapping_sub(1 as ::core::ffi::c_uint)
                                    as isize,
                            ) as *mut HLPFILE_WINDOWINFO;

                        flags = GET_USHORT(ptr, 4 as ::core::ffi::c_uint);
                        if flags as ::core::ffi::c_int & 0x0001 as ::core::ffi::c_int != 0 {
                            strcpy(
                                core::ptr::addr_of_mut!((*wi).type_) as *mut ::core::ffi::c_char,
                                str.offset(2 as isize),
                            );
                        } else {
                            (*wi).type_[0 as usize] = '\0' as ::core::ffi::c_char;
                        }
                        if flags as ::core::ffi::c_int & 0x0002 as ::core::ffi::c_int != 0 {
                            strcpy(
                                core::ptr::addr_of_mut!((*wi).name) as *mut ::core::ffi::c_char,
                                str.offset(12 as isize),
                            );
                        } else {
                            (*wi).name[0 as usize] = '\0' as ::core::ffi::c_char;
                        }
                        if flags as ::core::ffi::c_int & 0x0004 as ::core::ffi::c_int != 0 {
                            strcpy(
                                core::ptr::addr_of_mut!((*wi).caption) as *mut ::core::ffi::c_char,
                                str.offset(21 as isize),
                            );
                        } else {
                            lstrcpynA(
                                core::ptr::addr_of_mut!((*wi).caption) as LPSTR,
                                (*hlpfile).lpszTitle as LPCSTR,
                                ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    as ::core::ffi::c_int,
                            );
                        }
                        (*wi).origin.x = (if flags as ::core::ffi::c_int & 0x0008 as ::core::ffi::c_int != 0 {
                            GET_USHORT(ptr, 76 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        } else {
                            CW_USEDEFAULT as ::core::ffi::c_int
                        }) as LONG;
                        (*wi).origin.y = (if flags as ::core::ffi::c_int & 0x0010 as ::core::ffi::c_int != 0 {
                            GET_USHORT(ptr, 78 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        } else {
                            CW_USEDEFAULT as ::core::ffi::c_int
                        }) as LONG;
                        (*wi).size.cx = (if flags as ::core::ffi::c_int & 0x0020 as ::core::ffi::c_int != 0 {
                            GET_USHORT(ptr, 80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        } else {
                            CW_USEDEFAULT as ::core::ffi::c_int
                        }) as LONG;
                        (*wi).size.cy = (if flags as ::core::ffi::c_int & 0x0040 as ::core::ffi::c_int != 0 {
                            GET_USHORT(ptr, 82 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        } else {
                            CW_USEDEFAULT as ::core::ffi::c_int
                        }) as LONG;
                        (*wi).style = if flags as ::core::ffi::c_int & 0x0080 as ::core::ffi::c_int != 0 {
                            GET_USHORT(ptr, 84 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        } else {
                            SW_SHOW as ::core::ffi::c_int
                        };
                        (*wi).win_style = WS_OVERLAPPEDWINDOW as DWORD;
                        (*wi).sr_color = (if flags as ::core::ffi::c_int & 0x0100 as ::core::ffi::c_int != 0 {
                            GET_UINT(ptr, 86 as ::core::ffi::c_uint)
                        } else {
                            0xFFFFFF as ::core::ffi::c_uint
                        }) as COLORREF;
                        (*wi).nsr_color = (if flags as ::core::ffi::c_int & 0x0200 as ::core::ffi::c_int != 0 {
                            GET_UINT(ptr, 90 as ::core::ffi::c_uint)
                        } else {
                            0xFFFFFF as ::core::ffi::c_uint
                        }) as COLORREF;
                        WINE_TRACE!(
                            b"System-Window: flags=%c%c%c%c%c%c%c%c type=%s name=%s caption=%s (%d,%d)x(%d,%d)\n\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            if flags as ::core::ffi::c_int & 0x0001 as ::core::ffi::c_int != 0 { 'T' as i32 } else { 't' as i32 },
                            if flags as ::core::ffi::c_int & 0x0002 as ::core::ffi::c_int != 0 { 'N' as i32 } else { 'n' as i32 },
                            if flags as ::core::ffi::c_int & 0x0004 as ::core::ffi::c_int != 0 { 'C' as i32 } else { 'c' as i32 },
                            if flags as ::core::ffi::c_int & 0x0008 as ::core::ffi::c_int != 0 { 'X' as i32 } else { 'x' as i32 },
                            if flags as ::core::ffi::c_int & 0x0010 as ::core::ffi::c_int != 0 { 'Y' as i32 } else { 'y' as i32 },
                            if flags as ::core::ffi::c_int & 0x0020 as ::core::ffi::c_int != 0 { 'W' as i32 } else { 'w' as i32 },
                            if flags as ::core::ffi::c_int & 0x0040 as ::core::ffi::c_int != 0 { 'H' as i32 } else { 'h' as i32 },
                            if flags as ::core::ffi::c_int & 0x0080 as ::core::ffi::c_int != 0 { 'S' as i32 } else { 's' as i32 },
                            debugstr_a(
                                core::ptr::addr_of_mut!((*wi).type_) as *mut ::core::ffi::c_char,
                            ),
                            debugstr_a(
                                core::ptr::addr_of_mut!((*wi).name) as *mut ::core::ffi::c_char,
                            ),
                            debugstr_a(
                                core::ptr::addr_of_mut!((*wi).caption) as *mut ::core::ffi::c_char,
                            ),
                            (*wi).origin.x,
                            (*wi).origin.y,
                            (*wi).size.cx,
                            (*wi).size.cy,
                        );
                    }
                    break 'sw;
                }

                8 => {
                    WINE_WARN!(
                        b"Citation: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                        debugstr_a(
                            (ptr as *mut ::core::ffi::c_char).offset(4 as isize),
                        ),
                    );
                    break 'sw;
                }

                11 => {
                    (*hlpfile).charset = *ptr.offset(4 as isize) as ::core::ffi::c_ushort;
                    WINE_TRACE!(
                        b"Charset: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                        (*hlpfile).charset as ::core::ffi::c_int,
                    );
                    break 'sw;
                }

                _ => {
                    WINE_WARN!(
                        b"Unsupported SystemRecord[%d]\n\0".as_ptr() as *const ::core::ffi::c_char,
                        GET_USHORT(ptr, 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int,
                    );
                    break 'sw;
                }
            }
        }
        ptr = ptr.offset(
            (GET_USHORT(ptr, 2 as ::core::ffi::c_uint) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int) as isize,
        );
    }

    if (*hlpfile).lpszTitle.is_null() {
        (*hlpfile).lpszTitle = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            1 as SIZE_T,
        ) as LPSTR;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_GetContext(hlpfile: *mut HLPFILE) -> BOOL {
    let mut cbuf: *mut BYTE = core::ptr::null_mut();
    let mut cend: *mut BYTE = core::ptr::null_mut();
    let mut clen: u32;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|CONTEXT\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(cbuf),
        core::ptr::addr_of_mut!(cend),
    ) == 0
    {
        WINE_WARN!(b"context0\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    clen = cend.offset_from(cbuf) as core::ffi::c_longlong as u32;
    (*hlpfile).Context = HeapAlloc(
        GetProcessHeap(),
        0,
        (clen) as SIZE_T,
    ) as *mut BYTE;
    if (*hlpfile).Context.is_null() {
        return (FALSE) as BOOL;
    }
    memcpy(
        (*hlpfile).Context as *mut core::ffi::c_void,
        cbuf as *const core::ffi::c_void,
        clen as usize,
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_GetKeywords(hlpfile: *mut HLPFILE) -> BOOL {
    let mut cbuf: *mut BYTE = core::ptr::null_mut();
    let mut cend: *mut BYTE = core::ptr::null_mut();
    let mut clen: u32;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|KWBTREE\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(cbuf),
        core::ptr::addr_of_mut!(cend),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    clen = cend.offset_from(cbuf) as u32;
    (*hlpfile).kwbtree = HeapAlloc(
        GetProcessHeap(),
        0,
        clen as SIZE_T,
    ) as *mut BYTE;
    if (*hlpfile).kwbtree.is_null() {
        return (FALSE) as BOOL;
    }
    memcpy(
        (*hlpfile).kwbtree as *mut core::ffi::c_void,
        cbuf as *const core::ffi::c_void,
        clen as usize,
    );

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|KWDATA\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(cbuf),
        core::ptr::addr_of_mut!(cend),
    ) == 0
    {
        WINE_ERR!(
            b"corrupted help file: kwbtree present but kwdata absent\n\0".as_ptr() as *const i8,
        );
        HeapFree(
            GetProcessHeap(),
            0,
            (*hlpfile).kwbtree as PVOID,
        );
        return (FALSE) as BOOL;
    }
    clen = cend.offset_from(cbuf) as u32;
    (*hlpfile).kwdata = HeapAlloc(
        GetProcessHeap(),
        0,
        clen as SIZE_T,
    ) as *mut BYTE;
    if (*hlpfile).kwdata.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            (*hlpfile).kwdata as PVOID,
        );
        return (FALSE) as BOOL;
    }
    memcpy(
        (*hlpfile).kwdata as *mut core::ffi::c_void,
        cbuf as *const core::ffi::c_void,
        clen as usize,
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_GetMap(hlpfile: *mut HLPFILE) -> BOOL {
    let mut cbuf: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut cend: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut entries: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|CTXOMAP\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(cbuf),
        core::ptr::addr_of_mut!(cend),
    ) == 0
    {
        WINE_WARN!(b"no map section\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    entries = GET_USHORT(cbuf, 9) as ::core::ffi::c_uint;
    (*hlpfile).Map = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (entries as SIZE_T).wrapping_mul(core::mem::size_of::<HLPFILE_MAP>() as SIZE_T),
    ) as *mut HLPFILE_MAP;
    if (*hlpfile).Map.is_null() {
        return (FALSE) as BOOL;
    }
    (*hlpfile).wMapLen = entries;
    i = 0 as ::core::ffi::c_uint;
    while i < entries {
        (*(*hlpfile).Map.offset(i as isize)).lMap = GET_UINT(
            cbuf.offset(11 as isize),
            i.wrapping_mul(8 as ::core::ffi::c_uint),
        ) as LONG;
        (*(*hlpfile).Map.offset(i as isize)).offset = GET_UINT(
            cbuf.offset(11 as isize),
            i.wrapping_mul(8 as ::core::ffi::c_uint)
                .wrapping_add(4 as ::core::ffi::c_uint),
        ) as ::core::ffi::c_ulong;
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_GetTOMap(hlpfile: *mut HLPFILE) -> BOOL {
    let mut cbuf: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut cend: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut clen: ::core::ffi::c_uint = 0;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|TOMAP\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(cbuf),
        core::ptr::addr_of_mut!(cend),
    ) == 0
    {
        WINE_WARN!(b"no tomap section\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    clen = (cend.offset_from(cbuf) as ::core::ffi::c_uint).wrapping_sub(9);
    (*hlpfile).TOMap = HeapAlloc(
        GetProcessHeap(),
        0,
        (clen) as SIZE_T,
    ) as *mut ::core::ffi::c_uint;
    if (*hlpfile).TOMap.is_null() {
        return (FALSE) as BOOL;
    }
    memcpy(
        (*hlpfile).TOMap as *mut ::core::ffi::c_void,
        cbuf.offset(9 as isize) as *const ::core::ffi::c_void,
        clen as usize,
    );
    (*hlpfile).wTOMapLen = clen.wrapping_div(4);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_DeleteMacro(mut macro_0: *mut HLPFILE_MACRO) {
    let mut next: *mut HLPFILE_MACRO = ::core::ptr::null_mut::<HLPFILE_MACRO>();
    while !macro_0.is_null() {
        next = (*macro_0).next as *mut HLPFILE_MACRO;
        HeapFree(GetProcessHeap(), 0 as DWORD, macro_0 as PVOID);
        macro_0 = next as *mut HLPFILE_MACRO;
    }
}

unsafe extern "C" fn HLPFILE_DeletePage(mut page: *mut HLPFILE_PAGE) {
    let mut next: *mut HLPFILE_PAGE = core::ptr::null_mut();
    while !page.is_null() {
        next = (*page).next as *mut HLPFILE_PAGE;
        HLPFILE_DeleteMacro((*page).first_macro);
        HeapFree(GetProcessHeap(), 0 as DWORD, page as PVOID);
        page = next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HLPFILE_FreeHlpFile(hlpfile: *mut HLPFILE) {
    let mut i: ::core::ffi::c_uint;

    if hlpfile.is_null() {
        return;
    }
    (*hlpfile).wRefCount = (*hlpfile).wRefCount.wrapping_sub(1);
    if (*hlpfile).wRefCount > 0 {
        return;
    }

    if !(*hlpfile).next.is_null() {
        (*(*hlpfile).next).prev = (*hlpfile).prev;
    }
    if !(*hlpfile).prev.is_null() {
        (*(*hlpfile).prev).next = (*hlpfile).next;
    } else {
        first_hlpfile = (*hlpfile).next as *mut HLPFILE;
    }

    if (*hlpfile).numFonts != 0 {
        i = 0;
        while i < (*hlpfile).numFonts {
            DeleteObject(((*(*hlpfile).fonts.offset(i as isize)).hFont).cast());
            i = i.wrapping_add(1);
        }
        HeapFree(GetProcessHeap(), 0, (*hlpfile).fonts as PVOID);
    }

    if (*hlpfile).numBmps != 0 {
        i = 0;
        while i < (*hlpfile).numBmps {
            DeleteObject((*(*hlpfile).bmps.offset(i as isize)).cast());
            i = i.wrapping_add(1);
        }
        HeapFree(GetProcessHeap(), 0, (*hlpfile).bmps as PVOID);
    }

    HLPFILE_DeletePage((*hlpfile).first_page);
    HLPFILE_DeleteMacro((*hlpfile).first_macro);

    DestroyIcon((*hlpfile).hIcon);
    if (*hlpfile).numWindows != 0 {
        HeapFree(GetProcessHeap(), 0, (*hlpfile).windows as PVOID);
    }
    HeapFree(GetProcessHeap(), 0, (*hlpfile).Context as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).Map as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).lpszTitle as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).lpszCopyright as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).file_buffer as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).phrases_offsets as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).phrases_buffer as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).topic_map as PVOID);
    HeapFree(GetProcessHeap(), 0, (*hlpfile).help_on_file as PVOID);
    HeapFree(GetProcessHeap(), 0, hlpfile as PVOID);
}

unsafe extern "C" fn HLPFILE_UncompressLZ77_Phrases(hlpfile: *mut HLPFILE) -> BOOL {
    let mut i: UINT = 0;
    let mut num: UINT = 0;
    let mut dec_size: UINT = 0;
    let mut head_size: UINT = 0;
    let mut buf: *mut BYTE = core::ptr::null_mut();
    let mut end: *mut BYTE = core::ptr::null_mut();

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|Phrases\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(buf),
        core::ptr::addr_of_mut!(end),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if ((*hlpfile).version as ::core::ffi::c_int) <= 16 {
        head_size = 13 as UINT;
    } else {
        head_size = 17 as UINT;
    }

    num = GET_USHORT(buf, 9) as UINT;
    (*hlpfile).num_phrases = num;

    if buf
        .offset((2 as UINT).wrapping_mul(num) as isize)
        .offset(0x13 as isize)
        >= end
    {
        WINE_WARN!(b"1a\n\0".as_ptr() as *const ::core::ffi::c_char);
        return (FALSE) as BOOL;
    }

    if ((*hlpfile).version as ::core::ffi::c_int) <= 16 {
        dec_size = (end.offset_from(buf) as ::core::ffi::c_longlong
            - 15 as ::core::ffi::c_longlong
            - (2 as UINT).wrapping_mul(num) as ::core::ffi::c_longlong)
            as UINT;
    } else {
        dec_size = HLPFILE_UncompressedLZ77_Size(
            buf.offset(0x13 as isize)
                .offset((2 as UINT).wrapping_mul(num) as isize),
            end,
        ) as UINT;
    }

    (*hlpfile).phrases_offsets = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<::core::ffi::c_uint>() as SIZE_T)
            .wrapping_mul(num.wrapping_add(1 as UINT) as SIZE_T),
    ) as *mut ::core::ffi::c_uint;
    (*hlpfile).phrases_buffer = HeapAlloc(
        GetProcessHeap(),
        0,
        dec_size as SIZE_T,
    ) as *mut ::core::ffi::c_char;

    if (*hlpfile).phrases_offsets.is_null() || (*hlpfile).phrases_buffer.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            (*hlpfile).phrases_offsets as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0,
            (*hlpfile).phrases_buffer as PVOID,
        );
        return (FALSE) as BOOL;
    }

    i = 0 as UINT;
    while i <= num {
        *(*hlpfile).phrases_offsets.offset(i as isize) = (GET_USHORT(
            buf,
            head_size.wrapping_add((2 as UINT).wrapping_mul(i)),
        ) as UINT)
            .wrapping_sub((2 as UINT).wrapping_mul(num))
            .wrapping_sub(2 as UINT)
            as ::core::ffi::c_uint;
        i = i.wrapping_add(1);
    }

    if ((*hlpfile).version as ::core::ffi::c_int) <= 16 {
        memcpy(
            (*hlpfile).phrases_buffer as *mut ::core::ffi::c_void,
            buf.offset(15 as isize)
                .offset((2 as UINT).wrapping_mul(num) as isize)
                as *const ::core::ffi::c_void,
            dec_size as usize,
        );
    } else {
        HLPFILE_UncompressLZ77(
            buf.offset(0x13 as isize)
                .offset((2 as UINT).wrapping_mul(num) as isize),
            end,
            (*hlpfile).phrases_buffer as *mut BYTE,
        );
    }

    (*hlpfile).hasPhrases = (TRUE as i32);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_Uncompress_Phrases40(hlpfile: *mut HLPFILE) -> BOOL {
    let mut num: UINT = 0;
    let mut dec_size: INT = 0;
    let mut cpr_size: INT = 0;
    let mut buf_idx: *mut BYTE = core::ptr::null_mut();
    let mut end_idx: *mut BYTE = core::ptr::null_mut();
    let mut buf_phs: *mut BYTE = core::ptr::null_mut();
    let mut end_phs: *mut BYTE = core::ptr::null_mut();
    let mut ptr: *mut ULONG = core::ptr::null_mut();
    let mut mask: ULONG = 0 as ULONG;
    let mut i: ::core::ffi::c_uint = 0;
    let mut bc: ::core::ffi::c_ushort = 0;
    let mut n: ::core::ffi::c_ushort = 0;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|PhrIndex\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(buf_idx),
        core::ptr::addr_of_mut!(end_idx),
    ) == 0
        || HLPFILE_FindSubFile(
            hlpfile,
            b"|PhrImage\0".as_ptr() as LPCSTR,
            core::ptr::addr_of_mut!(buf_phs),
            core::ptr::addr_of_mut!(end_phs),
        ) == 0
    {
        return FALSE as BOOL;
    }

    ptr = buf_idx.offset(9 as isize + 28 as isize) as *mut ULONG;
    bc = (GET_USHORT(buf_idx, (9 + 24) as ::core::ffi::c_uint)
        as ::core::ffi::c_int
        & (0x0F as ::core::ffi::c_int)) as ::core::ffi::c_ushort;
    (*hlpfile).num_phrases = GET_USHORT(
        buf_idx,
        (9 + 4) as ::core::ffi::c_uint,
    ) as UINT;
    num = (*hlpfile).num_phrases;

    WINE_TRACE!(
        b"Index: Magic=%08x #entries=%u CpsdSize=%u PhrImgSize=%u\n\tPhrImgCprsdSize=%u 0=%u bc=%x ukn=%x\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        GET_UINT(buf_idx, (9 + 0) as ::core::ffi::c_uint),
        GET_UINT(buf_idx, (9 + 4) as ::core::ffi::c_uint),
        GET_UINT(buf_idx, (9 + 8) as ::core::ffi::c_uint),
        GET_UINT(buf_idx, (9 + 12) as ::core::ffi::c_uint),
        GET_UINT(buf_idx, (9 + 16) as ::core::ffi::c_uint),
        GET_UINT(buf_idx, (9 + 20) as ::core::ffi::c_uint),
        GET_USHORT(buf_idx, (9 + 24) as ::core::ffi::c_uint)
            as ::core::ffi::c_int,
        GET_USHORT(buf_idx, (9 + 26) as ::core::ffi::c_uint)
            as ::core::ffi::c_int,
    );

    dec_size = GET_UINT(buf_idx, (9 + 12) as ::core::ffi::c_uint) as INT;
    cpr_size = GET_UINT(buf_idx, (9 + 16) as ::core::ffi::c_uint) as INT;

    if dec_size != cpr_size
        && dec_size
            != HLPFILE_UncompressedLZ77_Size(buf_phs.offset(9 as isize), end_phs) as INT
    {
        WINE_WARN!(
            b"size mismatch %u %u\n\0".as_ptr() as *const ::core::ffi::c_char,
            dec_size,
            HLPFILE_UncompressedLZ77_Size(buf_phs.offset(9 as isize), end_phs) as INT,
        );
        let __max_b: INT = HLPFILE_UncompressedLZ77_Size(buf_phs.offset(9 as isize), end_phs) as INT;
        dec_size = if dec_size > __max_b { dec_size } else { __max_b };
    }

    (*hlpfile).phrases_offsets = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (::core::mem::size_of::<::core::ffi::c_uint>() as SIZE_T)
            .wrapping_mul(num.wrapping_add(1 as UINT) as SIZE_T),
    ) as *mut ::core::ffi::c_uint;
    (*hlpfile).phrases_buffer = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        dec_size as SIZE_T,
    ) as *mut ::core::ffi::c_char;
    if (*hlpfile).phrases_offsets.is_null() || (*hlpfile).phrases_buffer.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*hlpfile).phrases_offsets as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*hlpfile).phrases_buffer as PVOID,
        );
        return FALSE as BOOL;
    }

    *(*hlpfile).phrases_offsets.offset(0 as isize) = 0 as ::core::ffi::c_uint;
    ptr = ptr.offset(-1); /* as we'll first increment ptr because mask is 0 on first getbit() call */
    i = 0 as ::core::ffi::c_uint;
    while i < num {
        n = 1 as ::core::ffi::c_ushort;
        loop {
            mask <<= 1;
            if !(if mask != 0 {
                (*ptr & mask) != 0
            } else {
                ptr = ptr.offset(1);
                mask = 1;
                (*ptr & mask) != 0
            }) {
                break;
            }
            n = ((n as ::core::ffi::c_int)
                + ((1 as ::core::ffi::c_int) << (bc as ::core::ffi::c_int)))
                as ::core::ffi::c_ushort;
        }
        let __gb0: bool = {
            mask <<= 1;
            if mask != 0 {
                (*ptr & mask) != 0
            } else {
                ptr = ptr.offset(1);
                mask = 1;
                (*ptr & mask) != 0
            }
        };
        if __gb0 {
            n = n.wrapping_add(1);
        }
        let __gb1: bool = (bc as ::core::ffi::c_int) > (1 as ::core::ffi::c_int) && {
            mask <<= 1;
            if mask != 0 {
                (*ptr & mask) != 0
            } else {
                ptr = ptr.offset(1);
                mask = 1;
                (*ptr & mask) != 0
            }
        };
        if __gb1 {
            n = ((n as ::core::ffi::c_int) + (2 as ::core::ffi::c_int)) as ::core::ffi::c_ushort;
        }
        let __gb2: bool = (bc as ::core::ffi::c_int) > (2 as ::core::ffi::c_int) && {
            mask <<= 1;
            if mask != 0 {
                (*ptr & mask) != 0
            } else {
                ptr = ptr.offset(1);
                mask = 1;
                (*ptr & mask) != 0
            }
        };
        if __gb2 {
            n = ((n as ::core::ffi::c_int) + (4 as ::core::ffi::c_int)) as ::core::ffi::c_ushort;
        }
        let __gb3: bool = (bc as ::core::ffi::c_int) > (3 as ::core::ffi::c_int) && {
            mask <<= 1;
            if mask != 0 {
                (*ptr & mask) != 0
            } else {
                ptr = ptr.offset(1);
                mask = 1;
                (*ptr & mask) != 0
            }
        };
        if __gb3 {
            n = ((n as ::core::ffi::c_int) + (8 as ::core::ffi::c_int)) as ::core::ffi::c_ushort;
        }
        let __gb4: bool = (bc as ::core::ffi::c_int) > (4 as ::core::ffi::c_int) && {
            mask <<= 1;
            if mask != 0 {
                (*ptr & mask) != 0
            } else {
                ptr = ptr.offset(1);
                mask = 1;
                (*ptr & mask) != 0
            }
        };
        if __gb4 {
            n = ((n as ::core::ffi::c_int) + (16 as ::core::ffi::c_int)) as ::core::ffi::c_ushort;
        }
        *(*hlpfile)
            .phrases_offsets
            .offset(i.wrapping_add(1 as ::core::ffi::c_uint) as isize) =
            (*(*hlpfile).phrases_offsets.offset(i as isize)).wrapping_add(n as ::core::ffi::c_uint);
        i = i.wrapping_add(1);
    }

    if dec_size == cpr_size {
        memcpy(
            (*hlpfile).phrases_buffer as *mut ::core::ffi::c_void,
            buf_phs.offset(9 as isize) as *const ::core::ffi::c_void,
            (dec_size as SIZE_T) as _,
        );
    } else {
        HLPFILE_UncompressLZ77(
            buf_phs.offset(9 as isize),
            end_phs,
            (*hlpfile).phrases_buffer as *mut BYTE,
        );
    }

    (*hlpfile).hasPhrases40 = TRUE as BOOL;
    return TRUE as BOOL;
}

unsafe extern "C" fn HLPFILE_Uncompress_Topic(hlpfile: *mut HLPFILE) -> BOOL {
    let mut buf: *mut BYTE = core::ptr::null_mut();
    let mut ptr: *mut BYTE = core::ptr::null_mut();
    let mut end: *mut BYTE = core::ptr::null_mut();
    let mut newptr: *mut BYTE = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;
    let mut newsize: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut topic_size: ::core::ffi::c_uint = 0;

    if HLPFILE_FindSubFile(
        hlpfile,
        b"|TOPIC\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(buf),
        core::ptr::addr_of_mut!(end),
    ) == 0
    {
        WINE_WARN!(b"topic0\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    buf = buf.offset(9 as isize); /* Skip file header */
    topic_size = end.offset_from(buf) as ::core::ffi::c_longlong as ::core::ffi::c_uint;
    if (*hlpfile).compressed != 0 {
        (*hlpfile).topic_maplen = topic_size
            .wrapping_sub(1 as ::core::ffi::c_uint)
            .wrapping_div((*hlpfile).tbsize as ::core::ffi::c_uint)
            .wrapping_add(1 as ::core::ffi::c_uint) as UINT;

        i = 0 as ::core::ffi::c_uint;
        while i < (*hlpfile).topic_maplen {
            ptr = buf.offset(i.wrapping_mul((*hlpfile).tbsize as ::core::ffi::c_uint) as isize);

            /* I don't know why, it's necessary for printman.hlp */
            if ptr.offset(0x44 as isize) > end {
                ptr = end.offset(-(0x44 as isize));
            }

            newsize = newsize.wrapping_add(HLPFILE_UncompressedLZ77_Size(
                ptr.offset(0xc as isize),
                min!(end, ptr.offset((*hlpfile).tbsize as isize)),
            ) as ::core::ffi::c_uint);
            i = i.wrapping_add(1);
        }

        (*hlpfile).topic_map = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((*hlpfile).topic_maplen as SIZE_T).wrapping_mul(
                (0 as SIZE_T * 0
                    + ::core::mem::size_of::<BYTE>() as SIZE_T),
            )
            .wrapping_add(newsize as SIZE_T),
        ) as *mut *mut BYTE;
        if (*hlpfile).topic_map.is_null() {
            return (FALSE) as BOOL;
        }
        newptr = (*hlpfile).topic_map.offset((*hlpfile).topic_maplen as isize) as *mut BYTE;
        (*hlpfile).topic_end = newptr.offset(newsize as isize);

        i = 0 as ::core::ffi::c_uint;
        while i < (*hlpfile).topic_maplen {
            ptr = buf.offset(i.wrapping_mul((*hlpfile).tbsize as ::core::ffi::c_uint) as isize);
            if ptr.offset(0x44 as isize) > end {
                ptr = end.offset(-(0x44 as isize));
            }

            *(*hlpfile).topic_map.offset(i as isize) = newptr;
            newptr = HLPFILE_UncompressLZ77(
                ptr.offset(0xc as isize),
                min!(end, ptr.offset((*hlpfile).tbsize as isize)),
                newptr,
            );
            i = i.wrapping_add(1);
        }
    } else {
        /* basically, we need to copy the TopicBlockSize byte pages
         * (removing the first 0x0C) in one single area in memory
         */
        (*hlpfile).topic_maplen = topic_size
            .wrapping_sub(1 as ::core::ffi::c_uint)
            .wrapping_div((*hlpfile).tbsize as ::core::ffi::c_uint)
            .wrapping_add(1 as ::core::ffi::c_uint) as UINT;
        (*hlpfile).topic_map = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((*hlpfile).topic_maplen as SIZE_T).wrapping_mul(
                (0 as SIZE_T * 0
                    + ::core::mem::size_of::<BYTE>() as SIZE_T)
                    .wrapping_add((*hlpfile).dsize as SIZE_T),
            ),
        ) as *mut *mut BYTE;
        if (*hlpfile).topic_map.is_null() {
            return (FALSE) as BOOL;
        }
        newptr = (*hlpfile).topic_map.offset((*hlpfile).topic_maplen as isize) as *mut BYTE;
        (*hlpfile).topic_end = newptr.offset(topic_size as isize);

        i = 0 as ::core::ffi::c_uint;
        while i < (*hlpfile).topic_maplen {
            *(*hlpfile).topic_map.offset(i as isize) =
                newptr.offset(i.wrapping_mul((*hlpfile).dsize as ::core::ffi::c_uint) as isize);
            memcpy(
                *(*hlpfile).topic_map.offset(i as isize) as *mut ::core::ffi::c_void,
                buf.offset(i.wrapping_mul((*hlpfile).tbsize as ::core::ffi::c_uint) as isize)
                    .offset(0x0C as isize)
                    as *const ::core::ffi::c_void,
                (*hlpfile).dsize as usize,
            );
            i = i.wrapping_add(1);
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_AddPage(hlpfile: *mut HLPFILE, buf: *const BYTE, end: *const BYTE, ref_0: ::core::ffi::c_uint, offset: ::core::ffi::c_uint) -> BOOL {
    let mut page: *mut HLPFILE_PAGE = ::core::ptr::null_mut::<HLPFILE_PAGE>();
    let mut title: *const BYTE = ::core::ptr::null::<BYTE>();
    let mut titlesize: UINT = 0;
    let mut blocksize: UINT = 0;
    let mut datalen: UINT = 0;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut macro_0: *mut HLPFILE_MACRO = ::core::ptr::null_mut::<HLPFILE_MACRO>();
    blocksize = GET_UINT(buf, 0) as UINT;
    datalen = GET_UINT(buf, 0x10) as UINT;
    title = buf.offset(datalen as isize);
    if title > end {
        WINE_WARN!(b"page2\n\0".as_ptr() as *const ::core::ffi::c_char);
        return (FALSE) as BOOL;
    }
    titlesize = GET_UINT(buf, 4) as UINT;
    page = HeapAlloc(
        GetProcessHeap(),
        0,
        (::core::mem::size_of::<HLPFILE_PAGE>() as SIZE_T)
            .wrapping_add(titlesize as SIZE_T)
            .wrapping_add(1 as SIZE_T),
    ) as *mut HLPFILE_PAGE;
    if page.is_null() {
        return (FALSE) as BOOL;
    }
    (*page).lpszTitle = (page as *mut ::core::ffi::c_char)
        .offset(::core::mem::size_of::<HLPFILE_PAGE>() as usize as isize)
        as LPSTR;

    if titlesize > blocksize.wrapping_sub(datalen) {
        /* need to decompress */
        if (*hlpfile).hasPhrases != 0 {
            HLPFILE_Uncompress2(
                hlpfile,
                title,
                end,
                (*page).lpszTitle as *mut BYTE,
                ((*page).lpszTitle as *mut BYTE).offset(titlesize as isize),
            );
        } else if (*hlpfile).hasPhrases40 != 0 {
            HLPFILE_Uncompress3(
                hlpfile,
                (*page).lpszTitle as *mut ::core::ffi::c_char,
                (*page).lpszTitle.offset(titlesize as isize) as *const ::core::ffi::c_char,
                title,
                end,
            );
        } else {
            WINE_FIXME!(b"Text size is too long, splitting\n\0".as_ptr() as *const ::core::ffi::c_char);
            titlesize = blocksize.wrapping_sub(datalen);
            memcpy(
                (*page).lpszTitle as *mut ::core::ffi::c_void,
                title as *const ::core::ffi::c_void,
                titlesize as size_t,
            );
        }
    } else {
        memcpy(
            (*page).lpszTitle as *mut ::core::ffi::c_void,
            title as *const ::core::ffi::c_void,
            titlesize as size_t,
        );
    }

    *(*page).lpszTitle.offset(titlesize as isize) = '\0' as i32 as CHAR;

    if !(*hlpfile).first_page.is_null() {
        (*(*hlpfile).last_page).next = page as *mut tagHlpFilePage;
        (*page).prev = (*hlpfile).last_page as *mut tagHlpFilePage;
        (*hlpfile).last_page = page;
    } else {
        (*hlpfile).first_page = page;
        (*hlpfile).last_page = page;
        (*page).prev = ::core::ptr::null_mut::<tagHlpFilePage>();
    }

    (*page).file = hlpfile as *mut tagHlpFileFile;
    (*page).next = ::core::ptr::null_mut::<tagHlpFilePage>();
    (*page).first_macro = ::core::ptr::null_mut::<HLPFILE_MACRO>();
    (*page).first_link = ::core::ptr::null_mut::<HLPFILE_LINK>();
    (*page).wNumber = GET_UINT(buf, 0x21);
    (*page).offset = offset;
    (*page).reference = ref_0 as DWORD;

    (*page).browse_bwd = GET_UINT(buf, 0x19) as DWORD;
    (*page).browse_fwd = GET_UINT(buf, 0x1D) as DWORD;

    if (*hlpfile).version as ::core::ffi::c_int <= 16 {
        if (*page).browse_bwd == 0xFFFF || (*page).browse_bwd == 0xFFFFFFFF {
            (*page).browse_bwd = 0xFFFFFFFF;
        } else {
            (*page).browse_bwd = *(*hlpfile).TOMap.offset((*page).browse_bwd as isize) as DWORD;
        }

        if (*page).browse_fwd == 0xFFFF || (*page).browse_fwd == 0xFFFFFFFF {
            (*page).browse_fwd = 0xFFFFFFFF;
        } else {
            (*page).browse_fwd = *(*hlpfile).TOMap.offset((*page).browse_fwd as isize) as DWORD;
        }
    }

    WINE_TRACE!(
        b"Added page[%d]: title=%s %08x << %08x >> %08x\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*page).wNumber,
        debugstr_a((*page).lpszTitle as *const ::core::ffi::c_char),
        (*page).browse_bwd,
        (*page).offset,
        (*page).browse_fwd,
    );

    /* now load macros */
    ptr = (*page)
        .lpszTitle
        .offset(strlen((*page).lpszTitle as *const ::core::ffi::c_char) as isize)
        .offset(1 as isize) as *mut ::core::ffi::c_char;
    while ptr < (*page).lpszTitle.offset(titlesize as isize) {
        let mut len: ::core::ffi::c_uint = strlen(ptr) as ::core::ffi::c_uint;
        let mut macro_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();

        WINE_TRACE!(
            b"macro: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_a(ptr),
        );
        macro_0 = HeapAlloc(
            GetProcessHeap(),
            0,
            (::core::mem::size_of::<HLPFILE_MACRO>() as SIZE_T)
                .wrapping_add(len as SIZE_T)
                .wrapping_add(1 as SIZE_T),
        ) as *mut HLPFILE_MACRO;
        macro_str = macro_0.offset(1 as isize) as *mut ::core::ffi::c_char;
        (*macro_0).lpszMacro = macro_str as LPCSTR;
        memcpy(
            macro_str as *mut ::core::ffi::c_void,
            ptr as *const ::core::ffi::c_void,
            len.wrapping_add(1 as ::core::ffi::c_uint) as size_t,
        );
        /* FIXME: shall we really link macro in reverse order ??
         * may produce strange results when played at page opening
         */
        (*macro_0).next = (*page).first_macro as *mut tagHlpFileMacro;
        (*page).first_macro = macro_0;
        ptr = ptr.offset(len.wrapping_add(1 as ::core::ffi::c_uint) as isize);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_SkipParagraph(hlpfile: *mut HLPFILE, buf: *const BYTE, end: *const BYTE, len: *mut ::core::ffi::c_uint) -> BOOL {
    let mut tmp: *const BYTE = core::ptr::null();

    if (*hlpfile).first_page.is_null() {
        WINE_WARN!("no page\n");
        return (FALSE) as BOOL;
    };
    if buf.offset(0x19 as isize) > end {
        WINE_WARN!("header too small\n");
        return (FALSE) as BOOL;
    };

    tmp = buf.offset(0x15 as isize);
    if *buf.offset(0x14 as isize) as ::core::ffi::c_int == HLP_DISPLAY as ::core::ffi::c_int
        || *buf.offset(0x14 as isize) as ::core::ffi::c_int == HLP_TABLE as ::core::ffi::c_int
    {
        fetch_long(core::ptr::addr_of_mut!(tmp));
        *len = fetch_ushort(core::ptr::addr_of_mut!(tmp)) as ::core::ffi::c_uint;
    } else {
        *len = (end.offset_from(buf) as ::core::ffi::c_longlong - 15 as ::core::ffi::c_longlong) as ::core::ffi::c_uint;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HLPFILE_DoReadHlpFile(
    mut hlpfile: *mut HLPFILE,
    mut lpszPath: LPCSTR,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut hFile: HFILE = 0;
    let mut ofs: OFSTRUCT = core::mem::zeroed();
    let mut buf: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut ref_0: DWORD = 0x0C as DWORD;
    let mut index: ::core::ffi::c_uint = 0;
    let mut old_index: ::core::ffi::c_uint = 0;
    let mut offset: ::core::ffi::c_uint = 0;
    let mut len: ::core::ffi::c_uint = 0;
    let mut offs: ::core::ffi::c_uint = 0;
    let mut topicoffset: ::core::ffi::c_uint = 0;
    hFile = OpenFile(
        lpszPath,
        core::ptr::addr_of_mut!(ofs),
        OF_READ as UINT,
    );
    if hFile == HFILE_ERROR as HFILE {
        return FALSE as BOOL;
    }
    ret = HLPFILE_ReadFileToBuffer(hlpfile, hFile);
    _lclose(hFile);
    if ret == 0 {
        return FALSE as BOOL;
    }
    if HLPFILE_SystemCommands(hlpfile) == 0 {
        return FALSE as BOOL;
    }
    if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int
        && HLPFILE_GetTOMap(hlpfile) == 0
    {
        return FALSE as BOOL;
    }
    /* load phrases support */
    if HLPFILE_UncompressLZ77_Phrases(hlpfile) == 0 {
        HLPFILE_Uncompress_Phrases40(hlpfile);
    }
    if HLPFILE_Uncompress_Topic(hlpfile) == 0 {
        return FALSE as BOOL;
    }
    if HLPFILE_ReadFont(hlpfile) == 0 {
        return FALSE as BOOL;
    }
    old_index = (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint;
    offs = 0 as ::core::ffi::c_uint;
    loop {
        let mut end: *mut BYTE = core::ptr::null_mut::<BYTE>();
        if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int {
            index = ref_0.wrapping_sub(0x0C as DWORD)
                .wrapping_div((*hlpfile).dsize as DWORD) as ::core::ffi::c_uint;
            offset = ref_0.wrapping_sub(0x0C as DWORD)
                .wrapping_rem((*hlpfile).dsize as DWORD) as ::core::ffi::c_uint;
        } else {
            index = (ref_0.wrapping_sub(0x0C as DWORD)
                >> 14 as ::core::ffi::c_int) as ::core::ffi::c_uint;
            offset = (ref_0.wrapping_sub(0x0C as DWORD)
                & 0x3FFF as DWORD) as ::core::ffi::c_uint;
        }
        if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int
            && index != old_index
            && old_index != (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint
        {
            /* we jumped to the next block, adjust pointers */
            ref_0 = ref_0.wrapping_sub(12 as DWORD);
            offset = offset.wrapping_sub(12 as ::core::ffi::c_uint);
        }
        WINE_TRACE!(
            b"ref=%08x => [%u/%u]\n\0".as_ptr() as *const ::core::ffi::c_char,
            ref_0,
            index,
            offset,
        );
        if index >= (*hlpfile).topic_maplen {
            WINE_WARN!(b"maplen\n\0".as_ptr() as *const ::core::ffi::c_char);
            break;
        }
        buf = (*(*hlpfile).topic_map.offset(index as isize)).offset(offset as isize);
        if buf.offset(0x15 as isize) >= (*hlpfile).topic_end {
            WINE_WARN!(b"extra\n\0".as_ptr() as *const ::core::ffi::c_char);
            break;
        }
        end = min!(
            buf.offset(GET_UINT(buf, 0) as isize),
            (*hlpfile).topic_end,
        );
        if index != old_index {
            offs = 0 as ::core::ffi::c_uint;
            old_index = index;
        }
        match (*buf.offset(0x14 as isize) as ::core::ffi::c_int) as u32 {
            HLP_TOPICHDR => {
                /* Topic Header */
                if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int {
                    topicoffset = ref_0.wrapping_add(
                        index.wrapping_mul(12 as ::core::ffi::c_uint) as DWORD,
                    ) as ::core::ffi::c_uint;
                } else {
                    topicoffset = index
                        .wrapping_mul(0x8000 as ::core::ffi::c_uint)
                        .wrapping_add(offs);
                }
                if HLPFILE_AddPage(hlpfile, buf, end, ref_0 as ::core::ffi::c_uint, topicoffset) == 0 {
                    return FALSE as BOOL;
                }
            }
            HLP_DISPLAY30 | HLP_DISPLAY | HLP_TABLE => {
                if HLPFILE_SkipParagraph(hlpfile, buf, end, core::ptr::addr_of_mut!(len)) == 0 {
                    return FALSE as BOOL;
                }
                offs = offs.wrapping_add(len);
            }
            _ => {
                WINE_ERR!(
                    b"buf[0x14] = %x\n\0".as_ptr() as *const ::core::ffi::c_char,
                    *buf.offset(0x14 as isize) as ::core::ffi::c_int,
                );
            }
        }
        if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int {
            ref_0 = ref_0.wrapping_add(
                GET_UINT(buf, 0xc) as DWORD,
            );
            if GET_UINT(buf, 0xc) == 0 as ::core::ffi::c_uint {
                break;
            }
        } else {
            ref_0 = GET_UINT(buf, 0xc) as DWORD;
        }
        if !(ref_0 != 0xffffffff as DWORD) {
            break;
        }
    }
    HLPFILE_GetKeywords(hlpfile);
    HLPFILE_GetMap(hlpfile);
    if (*hlpfile).version as ::core::ffi::c_int <= 16 as ::core::ffi::c_int {
        return TRUE as BOOL;
    }
    return HLPFILE_GetContext(hlpfile);
}
#[no_mangle]
pub unsafe extern "C" fn HLPFILE_ReadHlpFile(mut lpszPath: LPCSTR) -> *mut HLPFILE {
    let mut hlpfile: *mut HLPFILE = ::core::ptr::null_mut::<HLPFILE>();
    hlpfile = first_hlpfile as *mut HLPFILE;
    while !hlpfile.is_null() {
        if strcmp(
            lpszPath as *const ::core::ffi::c_char,
            (*hlpfile).lpszPath as *const ::core::ffi::c_char,
        ) == 0
        {
            (*hlpfile).wRefCount = (*hlpfile).wRefCount.wrapping_add(1);
            return hlpfile as *mut HLPFILE;
        }
        hlpfile = (*hlpfile).next as *mut HLPFILE;
    }
    hlpfile = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (::core::mem::size_of::<HLPFILE>() as SIZE_T)
            .wrapping_add(strlen(lpszPath as *const ::core::ffi::c_char) as SIZE_T)
            .wrapping_add(1 as SIZE_T),
    ) as *mut HLPFILE;
    if hlpfile.is_null() {
        return core::ptr::null_mut();
    }
    (*hlpfile).lpszPath = (hlpfile as *mut ::core::ffi::c_char)
        .offset(::core::mem::size_of::<HLPFILE>() as usize as isize)
        as LPSTR;
    (*hlpfile).contents_start = 0xFFFFFFFF as ::core::ffi::c_ulong;
    (*hlpfile).next = first_hlpfile as *mut tagHlpFileFile;
    (*hlpfile).wRefCount = 1 as ::core::ffi::c_uint;
    strcpy(
        (*hlpfile).lpszPath as *mut ::core::ffi::c_char,
        lpszPath as *const ::core::ffi::c_char,
    );
    first_hlpfile = hlpfile as *mut HLPFILE;
    if !(*hlpfile).next.is_null() {
        (*(*hlpfile).next).prev = hlpfile as *mut tagHlpFileFile;
    }
    if HLPFILE_DoReadHlpFile(hlpfile as *mut HLPFILE, lpszPath) == 0 {
        HLPFILE_FreeHlpFile(hlpfile as *mut HLPFILE);
        hlpfile = core::ptr::null_mut();
    }
    return hlpfile as *mut HLPFILE;
}

// HLPFILE_BrowseParagraph: c2rust's translation, banked by hand. Its 1,569-line c2rust anchor
// (the paragraph/table/format-code parser) exceeded the model's output budget on every
// sample; the expanded wine debug blocks are folded back to WINE_*! by c2rust_debug_fold.
unsafe extern "C" fn HLPFILE_BrowseParagraph(
    mut page: *mut HLPFILE_PAGE,
    mut rd: *mut RtfData,
    mut buf: *mut BYTE,
    mut end: *mut BYTE,
    mut parlen: *mut ::core::ffi::c_uint,
) -> BOOL {
    let mut current_block: u64;
    let mut textsize: UINT = 0;
    let mut format: *const BYTE = ::core::ptr::null::<BYTE>();
    let mut format_end: *const BYTE = ::core::ptr::null::<BYTE>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text_base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: LONG = 0;
    let mut blocksize: LONG = 0;
    let mut datalen: LONG = 0;
    let mut bits: ::core::ffi::c_ushort = 0;
    let mut ncol: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
    let mut nc: ::core::ffi::c_short = 0;
    let mut lastcol: ::core::ffi::c_short = 0;
    let mut table_width: ::core::ffi::c_short = 0;
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut ret: BOOL = (FALSE as i32);
    if buf.offset(0x19 as ::core::ffi::c_int as isize) > end {
        WINE_WARN!("header too small\n");
        return (FALSE) as BOOL;
    }
    *parlen = 0 as ::core::ffi::c_uint;
    blocksize = GET_UINT(buf, 0 as ::core::ffi::c_uint) as LONG;
    size = GET_UINT(buf, 0x4 as ::core::ffi::c_uint) as LONG;
    datalen = GET_UINT(buf, 0x10 as ::core::ffi::c_uint) as LONG;
    text_base = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T) as *mut ::core::ffi::c_char;
    text = text_base;
    if text.is_null() {
        return (FALSE) as BOOL;
    }
    if size > blocksize - datalen {
        if (*(*page).file).hasPhrases != 0 {
            HLPFILE_Uncompress2(
                (*page).file as *mut HLPFILE,
                buf.offset(datalen as isize),
                end,
                text as *mut BYTE,
                (text as *mut BYTE).offset(size as isize),
            );
        } else if (*(*page).file).hasPhrases40 != 0 {
            HLPFILE_Uncompress3(
                (*page).file as *mut HLPFILE,
                text,
                text.offset(size as isize),
                buf.offset(datalen as isize),
                end,
            );
        } else {
            WINE_FIXME!("Text size is too long, splitting\n");
            size = blocksize - datalen;
            memcpy(
                text as *mut ::core::ffi::c_void,
                buf.offset(datalen as isize) as *const ::core::ffi::c_void,
                size as size_t,
            );
        }
    } else {
        memcpy(
            text as *mut ::core::ffi::c_void,
            buf.offset(datalen as isize) as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    text_end = text.offset(size as isize);
    format = buf.offset(0x15 as ::core::ffi::c_int as isize);
    format_end = buf.offset(GET_UINT(buf, 0x10 as ::core::ffi::c_uint) as isize);
    WINE_TRACE!("Record type (buf[0x14]) = 0x%x\n", *buf.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
    if *buf.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == (HLP_DISPLAY as i32)
        || *buf.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == (HLP_TABLE as i32)
    {
        fetch_long(&raw mut format);
        *parlen = fetch_ushort(&raw mut format) as ::core::ffi::c_uint;
    }
    if *buf.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == (HLP_TABLE as i32) {
        let mut type_0: ::core::ffi::c_uchar = 0;
        let fresh65 = format;
        format = format.offset(1);
        ncol = *fresh65 as ::core::ffi::c_uint;
        if HLPFILE_RtfAddControl(rd, b"\\trowd\0" as *const u8 as *const ::core::ffi::c_char) == 0 {
            current_block = 6020540383401037962;
        } else {
            let fresh66 = format;
            format = format.offset(1);
            type_0 = *fresh66 as ::core::ffi::c_uchar;
            if type_0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || type_0 as ::core::ffi::c_int == 2 as ::core::ffi::c_int
            {
                table_width = GET_SHORT(format, 0 as ::core::ffi::c_uint);
                format = format.offset(2 as ::core::ffi::c_int as isize);
                if HLPFILE_RtfAddControl(rd, b"\\trqc\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0
                {
                    current_block = 6020540383401037962;
                } else {
                    current_block = 1345366029464561491;
                }
            } else {
                table_width = 32767 as ::core::ffi::c_short;
                current_block = 1345366029464561491;
            }
            match current_block {
                6020540383401037962 => {}
                _ => {
                    WINE_TRACE!("New table: cols=%d type=%x width=%d\n", ncol, type_0 as ::core::ffi::c_int, table_width as ::core::ffi::c_int);
                    if ncol > 1 as ::core::ffi::c_uint {
                        let mut pos: ::core::ffi::c_int = 0;
                        sprintf(
                            &raw mut tmp as *mut ::core::ffi::c_char,
                            b"\\trgaph%d\\trleft%d\0" as *const u8 as *const ::core::ffi::c_char,
                            MulDiv(
                                HLPFILE_HalfPointsScale(
                                    page,
                                    GET_SHORT(format, 6 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_uint,
                                ) as ::core::ffi::c_int,
                                table_width as ::core::ffi::c_int,
                                32767 as ::core::ffi::c_int,
                            ),
                            MulDiv(
                                HLPFILE_HalfPointsScale(
                                    page,
                                    (GET_SHORT(format, 2 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                        - GET_SHORT(format, 6 as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint,
                                ) as ::core::ffi::c_int,
                                table_width as ::core::ffi::c_int,
                                32767 as ::core::ffi::c_int,
                            ) - 1 as ::core::ffi::c_int,
                        );
                        if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0
                        {
                            current_block = 6020540383401037962;
                        } else {
                            pos = GET_SHORT(format, 6 as ::core::ffi::c_uint) as ::core::ffi::c_int
                                / 2 as ::core::ffi::c_int;
                            nc = 0 as ::core::ffi::c_short;
                            loop {
                                if !((nc as ::core::ffi::c_uint) < ncol) {
                                    current_block = 2723324002591448311;
                                    break;
                                }
                                WINE_TRACE!("column(%d/%d) gap=%d width=%d\n", nc as ::core::ffi::c_int, ncol, GET_SHORT( format, (nc as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as ::core::ffi::c_uint, ) as ::core::ffi::c_int, GET_SHORT( format, (nc as ::core::ffi::c_int * 4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as ::core::ffi::c_uint, ) as ::core::ffi::c_int);
                                pos += GET_SHORT(
                                    format,
                                    (nc as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint,
                                ) as ::core::ffi::c_int
                                    + GET_SHORT(
                                        format,
                                        (nc as ::core::ffi::c_int * 4 as ::core::ffi::c_int
                                            + 2 as ::core::ffi::c_int)
                                            as ::core::ffi::c_uint,
                                    ) as ::core::ffi::c_int;
                                sprintf(
                                    &raw mut tmp as *mut ::core::ffi::c_char,
                                    b"\\cellx%d\0" as *const u8 as *const ::core::ffi::c_char,
                                    MulDiv(
                                        HLPFILE_HalfPointsScale(page, pos as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int,
                                        table_width as ::core::ffi::c_int,
                                        32767 as ::core::ffi::c_int,
                                    ),
                                );
                                if HLPFILE_RtfAddControl(
                                    rd,
                                    &raw mut tmp as *mut ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break;
                                }
                                nc += 1;
                            }
                        }
                    } else {
                        WINE_TRACE!("column(0/%d) gap=%d width=%d\n", ncol, GET_SHORT(format, 0 as ::core::ffi::c_uint) as ::core::ffi::c_int, GET_SHORT(format, 2 as ::core::ffi::c_uint) as ::core::ffi::c_int);
                        sprintf(
                            &raw mut tmp as *mut ::core::ffi::c_char,
                            b"\\trleft%d\\cellx%d \0" as *const u8 as *const ::core::ffi::c_char,
                            MulDiv(
                                HLPFILE_HalfPointsScale(
                                    page,
                                    GET_SHORT(format, 2 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_uint,
                                ) as ::core::ffi::c_int,
                                table_width as ::core::ffi::c_int,
                                32767 as ::core::ffi::c_int,
                            ) - 1 as ::core::ffi::c_int,
                            MulDiv(
                                HLPFILE_HalfPointsScale(
                                    page,
                                    GET_SHORT(format, 0 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_uint,
                                ) as ::core::ffi::c_int,
                                table_width as ::core::ffi::c_int,
                                32767 as ::core::ffi::c_int,
                            ),
                        );
                        if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0
                        {
                            current_block = 6020540383401037962;
                        } else {
                            current_block = 2723324002591448311;
                        }
                    }
                    match current_block {
                        6020540383401037962 => {}
                        _ => {
                            format =
                                format.offset(ncol.wrapping_mul(4 as ::core::ffi::c_uint) as isize);
                            current_block = 8834769789432328951;
                        }
                    }
                }
            }
        }
    } else {
        current_block = 8834769789432328951;
    }
    match current_block {
        8834769789432328951 => {
            lastcol = -(1 as ::core::ffi::c_int) as ::core::ffi::c_short;
            nc = 0 as ::core::ffi::c_short;
            's_380: loop {
                if !((nc as ::core::ffi::c_uint) < ncol) {
                    current_block = 17581461676304123093;
                    break;
                }
                WINE_TRACE!("looking for format at offset %lu in column %d\n", format .offset_from(buf.offset(0x15 as ::core::ffi::c_int as isize)) as ::core::ffi::c_longlong as SIZE_T, nc as ::core::ffi::c_int);
                if HLPFILE_RtfAddControl(rd, b"\\pard\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0
                {
                    current_block = 6020540383401037962;
                    break;
                }
                if *buf.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == (HLP_TABLE as i32)
                {
                    lastcol = GET_SHORT(format, 0 as ::core::ffi::c_uint);
                    nc = lastcol;
                    if nc as ::core::ffi::c_int == -(1 as ::core::ffi::c_int) {
                        if HLPFILE_RtfAddControl(
                            rd,
                            b"\\row\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                        {
                            current_block = 6020540383401037962;
                            break;
                        }
                        (*rd).char_pos = (*rd).char_pos.wrapping_add(2 as ::core::ffi::c_uint);
                        current_block = 17581461676304123093;
                        break;
                    } else {
                        format = format.offset(5 as ::core::ffi::c_int as isize);
                        if HLPFILE_RtfAddControl(
                            rd,
                            b"\\intbl\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                        {
                            current_block = 6020540383401037962;
                            break;
                        }
                    }
                } else {
                    nc += 1;
                }
                if *buf.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == (HLP_DISPLAY30 as i32)
                {
                    format = format.offset(6 as ::core::ffi::c_int as isize);
                } else {
                    format = format.offset(4 as ::core::ffi::c_int as isize);
                }
                bits = GET_USHORT(format, 0 as ::core::ffi::c_uint);
                format = format.offset(2 as ::core::ffi::c_int as isize);
                if bits as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0 {
                    fetch_long(&raw mut format);
                }
                if bits as ::core::ffi::c_int & 0x2 as ::core::ffi::c_int != 0 {
                    sprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        b"\\sb%d\0" as *const u8 as *const ::core::ffi::c_char,
                        HLPFILE_HalfPointsScale(
                            page,
                            fetch_short(&raw mut format) as ::core::ffi::c_uint,
                        ),
                    );
                    if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0 {
                        current_block = 6020540383401037962;
                        break;
                    }
                }
                if bits as ::core::ffi::c_int & 0x4 as ::core::ffi::c_int != 0 {
                    sprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        b"\\sa%d\0" as *const u8 as *const ::core::ffi::c_char,
                        HLPFILE_HalfPointsScale(
                            page,
                            fetch_short(&raw mut format) as ::core::ffi::c_uint,
                        ),
                    );
                    if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0 {
                        current_block = 6020540383401037962;
                        break;
                    }
                }
                if bits as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int != 0 {
                    sprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        b"\\sl%d\0" as *const u8 as *const ::core::ffi::c_char,
                        HLPFILE_HalfPointsScale(
                            page,
                            fetch_short(&raw mut format) as ::core::ffi::c_uint,
                        ),
                    );
                    if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0 {
                        current_block = 6020540383401037962;
                        break;
                    }
                }
                if bits as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0 {
                    sprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        b"\\li%d\0" as *const u8 as *const ::core::ffi::c_char,
                        HLPFILE_HalfPointsScale(
                            page,
                            fetch_short(&raw mut format) as ::core::ffi::c_uint,
                        ),
                    );
                    if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0 {
                        current_block = 6020540383401037962;
                        break;
                    }
                }
                if bits as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 {
                    sprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        b"\\ri%d\0" as *const u8 as *const ::core::ffi::c_char,
                        HLPFILE_HalfPointsScale(
                            page,
                            fetch_short(&raw mut format) as ::core::ffi::c_uint,
                        ),
                    );
                    if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0 {
                        current_block = 6020540383401037962;
                        break;
                    }
                }
                if bits as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 {
                    sprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        b"\\fi%d\0" as *const u8 as *const ::core::ffi::c_char,
                        HLPFILE_HalfPointsScale(
                            page,
                            fetch_short(&raw mut format) as ::core::ffi::c_uint,
                        ),
                    );
                    if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0 {
                        current_block = 6020540383401037962;
                        break;
                    }
                }
                if bits as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int != 0 {
                    let fresh67 = format;
                    format = format.offset(1);
                    let mut brdr: BYTE = *fresh67;
                    let mut w: ::core::ffi::c_short = 0;
                    if brdr as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\box\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x2 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrt\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x4 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrl\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrb\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrr\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrth\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int == 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrs\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    if brdr as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0
                        && HLPFILE_RtfAddControl(
                            rd,
                            b"\\brdrdb\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                    {
                        current_block = 6020540383401037962;
                        break;
                    }
                    w = GET_SHORT(format, 0 as ::core::ffi::c_uint);
                    format = format.offset(2 as ::core::ffi::c_int as isize);
                    if w != 0 {
                        sprintf(
                            &raw mut tmp as *mut ::core::ffi::c_char,
                            b"\\brdrw%d\0" as *const u8 as *const ::core::ffi::c_char,
                            HLPFILE_HalfPointsScale(page, w as ::core::ffi::c_uint),
                        );
                        if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0
                        {
                            current_block = 6020540383401037962;
                            break;
                        }
                    }
                }
                if bits as ::core::ffi::c_int & 0x200 as ::core::ffi::c_int != 0 {
                    let mut i: ::core::ffi::c_int = 0;
                    let mut ntab: ::core::ffi::c_int =
                        fetch_short(&raw mut format) as ::core::ffi::c_int;
                    let mut tab: ::core::ffi::c_uint = 0;
                    let mut ts: ::core::ffi::c_uint = 0;
                    let mut kind: *const ::core::ffi::c_char =
                        ::core::ptr::null::<::core::ffi::c_char>();
                    i = 0 as ::core::ffi::c_int;
                    while i < ntab {
                        tab = fetch_ushort(&raw mut format) as ::core::ffi::c_uint;
                        ts = (if tab & 0x4000 as ::core::ffi::c_uint != 0 {
                            fetch_ushort(&raw mut format) as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as ::core::ffi::c_uint;
                        let mut current_block_107: u64;
                        match ts {
                            0 => {
                                current_block_107 = 6105156978888530013;
                            }
                            1 => {
                                kind = b"\\tqr\0" as *const u8 as *const ::core::ffi::c_char;
                                current_block_107 = 11227437541145425351;
                            }
                            2 => {
                                kind = b"\\tqc\0" as *const u8 as *const ::core::ffi::c_char;
                                current_block_107 = 11227437541145425351;
                            }
                            _ => {
                                WINE_FIXME!("Unknown tab style %x\n", ts);
                                current_block_107 = 6105156978888530013;
                            }
                        }
                        match current_block_107 {
                            6105156978888530013 => {
                                kind = b"\0" as *const u8 as *const ::core::ffi::c_char;
                            }
                            _ => {}
                        }
                        sprintf(
                            &raw mut tmp as *mut ::core::ffi::c_char,
                            b"%s\\tx%d\0" as *const u8 as *const ::core::ffi::c_char,
                            kind,
                            HLPFILE_HalfPointsScale(page, tab & 0x3fff as ::core::ffi::c_uint),
                        );
                        if HLPFILE_RtfAddControl(rd, &raw mut tmp as *mut ::core::ffi::c_char) == 0
                        {
                            current_block = 6020540383401037962;
                            break 's_380;
                        }
                        i += 1;
                    }
                }
                match bits as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int {
                    0 => {
                        if HLPFILE_RtfAddControl(
                            rd,
                            b"\\ql\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                        {
                            current_block = 6020540383401037962;
                            break;
                        }
                    }
                    1024 => {
                        if HLPFILE_RtfAddControl(
                            rd,
                            b"\\qr\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                        {
                            current_block = 6020540383401037962;
                            break;
                        }
                    }
                    2048 => {
                        if HLPFILE_RtfAddControl(
                            rd,
                            b"\\qc\0" as *const u8 as *const ::core::ffi::c_char,
                        ) == 0
                        {
                            current_block = 6020540383401037962;
                            break;
                        }
                    }
                    _ => {
                        WINE_FIXME!("Unsupported alignment 0xC00\n");
                    }
                }
                if bits as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int != 0
                    && HLPFILE_RtfAddControl(
                        rd,
                        b"\\keep\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0
                {
                    current_block = 6020540383401037962;
                    break;
                }
                if bits as ::core::ffi::c_int & 0xe080 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    WINE_FIXME!("Unsupported bits %04x, potential trouble ahead\n", bits as ::core::ffi::c_int);
                }
                while text < text_end && format < format_end {
                    WINE_TRACE!("Got text: %s (%p/%p - %p/%p)\n", debugstr_a(text), text, text_end, format, format_end);
                    textsize = strlen(text) as UINT;
                    if textsize != 0 {
                        if (*rd).force_color != 0 {
                            if if (*(*rd).current_link).cookie as ::core::ffi::c_uint
                                == hlp_link_popup as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                (HLPFILE_RtfAddControl(
                                    rd,
                                    b"{\\uld\\cf1\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0) as ::core::ffi::c_int
                            } else {
                                (HLPFILE_RtfAddControl(
                                    rd,
                                    b"{\\ul\\cf1\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0) as ::core::ffi::c_int
                            } != 0
                            {
                                current_block = 6020540383401037962;
                                break 's_380;
                            }
                        }
                        if HLPFILE_RtfAddText(rd, text) == 0 {
                            current_block = 6020540383401037962;
                            break 's_380;
                        }
                        if (*rd).force_color != 0
                            && HLPFILE_RtfAddControl(
                                rd,
                                b"}\0" as *const u8 as *const ::core::ffi::c_char,
                            ) == 0
                        {
                            current_block = 6020540383401037962;
                            break 's_380;
                        }
                        (*rd).char_pos =
                            (*rd).char_pos.wrapping_add(textsize as ::core::ffi::c_uint);
                    }
                    text = text.offset(textsize.wrapping_add(1 as UINT) as isize);
                    WINE_TRACE!("format=0x%02x\n", *format as ::core::ffi::c_int);
                    if *format as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
                        format = format.offset(1);
                        break;
                    } else {
                        match *format as ::core::ffi::c_int {
                            32 => {
                                WINE_FIXME!("NIY20\n");
                                format = format.offset(5 as ::core::ffi::c_int as isize);
                            }
                            33 => {
                                WINE_FIXME!("NIY21\n");
                                format = format.offset(3 as ::core::ffi::c_int as isize);
                            }
                            128 => {
                                let mut font: ::core::ffi::c_uint =
                                    GET_USHORT(format, 1 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_uint;
                                let mut fs: ::core::ffi::c_uint = 0;
                                WINE_TRACE!("Changing font to %d\n", font);
                                format = format.offset(3 as ::core::ffi::c_int as isize);
                                match (*rd).font_scale {
                                    0 => {
                                        fs = ((*(*(*page).file).fonts.offset(font as isize))
                                            .LogFont
                                            .lfHeight
                                            - 4 as LONG)
                                            as ::core::ffi::c_uint;
                                    }
                                    2 => {
                                        fs = ((*(*(*page).file).fonts.offset(font as isize))
                                            .LogFont
                                            .lfHeight
                                            + 4 as LONG)
                                            as ::core::ffi::c_uint;
                                    }
                                    1 | _ => {
                                        fs = (*(*(*page).file).fonts.offset(font as isize))
                                            .LogFont
                                            .lfHeight
                                            as ::core::ffi::c_uint;
                                    }
                                }
                                sprintf(
                                    &raw mut tmp as *mut ::core::ffi::c_char,
                                    b"\\f%u\\cf%u\\fs%u%s%s%s%s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    font,
                                    font.wrapping_add(2 as ::core::ffi::c_uint),
                                    fs,
                                    if (*(*(*page).file).fonts.offset(font as isize))
                                        .LogFont
                                        .lfWeight
                                        > 400 as LONG
                                    {
                                        b"\\b\0" as *const u8 as *const ::core::ffi::c_char
                                    } else {
                                        b"\\b0\0" as *const u8 as *const ::core::ffi::c_char
                                    },
                                    if (*(*(*page).file).fonts.offset(font as isize))
                                        .LogFont
                                        .lfItalic
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        b"\\i\0" as *const u8 as *const ::core::ffi::c_char
                                    } else {
                                        b"\\i0\0" as *const u8 as *const ::core::ffi::c_char
                                    },
                                    if (*(*(*page).file).fonts.offset(font as isize))
                                        .LogFont
                                        .lfUnderline
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        b"\\ul\0" as *const u8 as *const ::core::ffi::c_char
                                    } else {
                                        b"\\ul0\0" as *const u8 as *const ::core::ffi::c_char
                                    },
                                    if (*(*(*page).file).fonts.offset(font as isize))
                                        .LogFont
                                        .lfStrikeOut
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        b"\\strike\0" as *const u8 as *const ::core::ffi::c_char
                                    } else {
                                        b"\\strike0\0" as *const u8 as *const ::core::ffi::c_char
                                    },
                                );
                                if HLPFILE_RtfAddControl(
                                    rd,
                                    &raw mut tmp as *mut ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break 's_380;
                                }
                            }
                            129 => {
                                if HLPFILE_RtfAddControl(
                                    rd,
                                    b"\\line\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break 's_380;
                                }
                                format = format.offset(1 as ::core::ffi::c_int as isize);
                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                            }
                            130 => {
                                if *buf.offset(0x14 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == (HLP_TABLE as i32)
                                {
                                    if *format.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        != 0xff as ::core::ffi::c_int
                                    {
                                        if HLPFILE_RtfAddControl(
                                            rd,
                                            b"\\par\\intbl\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        ) == 0
                                        {
                                            current_block = 6020540383401037962;
                                            break 's_380;
                                        }
                                    } else if GET_SHORT(format, 2 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                        == -(1 as ::core::ffi::c_int)
                                    {
                                        if HLPFILE_RtfAddControl(
                                            rd,
                                            b"\\cell\\intbl\\row\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        ) == 0
                                        {
                                            current_block = 6020540383401037962;
                                            break 's_380;
                                        }
                                        (*rd).char_pos =
                                            (*rd).char_pos.wrapping_add(2 as ::core::ffi::c_uint);
                                    } else if GET_SHORT(format, 2 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                        == lastcol as ::core::ffi::c_int
                                    {
                                        if HLPFILE_RtfAddControl(
                                            rd,
                                            b"\\par\\pard\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        ) == 0
                                        {
                                            current_block = 6020540383401037962;
                                            break 's_380;
                                        }
                                    } else if HLPFILE_RtfAddControl(
                                        rd,
                                        b"\\cell\\pard\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    ) == 0
                                    {
                                        current_block = 6020540383401037962;
                                        break 's_380;
                                    }
                                } else if HLPFILE_RtfAddControl(
                                    rd,
                                    b"\\par\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break 's_380;
                                }
                                format = format.offset(1 as ::core::ffi::c_int as isize);
                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                            }
                            131 => {
                                if HLPFILE_RtfAddControl(
                                    rd,
                                    b"\\tab\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break 's_380;
                                }
                                format = format.offset(1 as ::core::ffi::c_int as isize);
                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                            }
                            134 | 135 | 136 => {
                                let mut type_1: BYTE =
                                    *format.offset(1 as ::core::ffi::c_int as isize);
                                format = format.offset(2 as ::core::ffi::c_int as isize);
                                size = fetch_long(&raw mut format);
                                let mut current_block_192: u64;
                                match type_1 as ::core::ffi::c_int {
                                    34 => {
                                        fetch_ushort(&raw mut format);
                                        current_block_192 = 9745474638934172015;
                                    }
                                    3 => {
                                        current_block_192 = 9745474638934172015;
                                    }
                                    5 => {
                                        WINE_FIXME!("Got an embedded element %s\n", debugstr_a( (format as *mut ::core::ffi::c_char) .offset(6 as ::core::ffi::c_int as isize), ));
                                        current_block_192 = 219164688580890164;
                                    }
                                    _ => {
                                        WINE_FIXME!("Got a type %d picture\n", type_1 as ::core::ffi::c_int);
                                        current_block_192 = 219164688580890164;
                                    }
                                }
                                match current_block_192 {
                                    9745474638934172015 => {
                                        match GET_SHORT(format, 0 as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int
                                        {
                                            0 => {
                                                HLPFILE_RtfAddGfxByIndex(
                                                    rd,
                                                    (*page).file as *mut HLPFILE,
                                                    GET_SHORT(format, 2 as ::core::ffi::c_uint)
                                                        as ::core::ffi::c_uint,
                                                );
                                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                                            }
                                            1 => {
                                                WINE_FIXME!("does it work ??? %x<%u>#%u\n", GET_SHORT(format, 0 as ::core::ffi::c_uint) as ::core::ffi::c_int, size, GET_SHORT(format, 2 as ::core::ffi::c_uint) as ::core::ffi::c_int);
                                                HLPFILE_RtfAddGfxByAddr(
                                                    rd,
                                                    (*page).file as *mut HLPFILE,
                                                    format.offset(2 as ::core::ffi::c_int as isize),
                                                    (size - 4 as LONG) as ULONG,
                                                );
                                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                                            }
                                            _ => {
                                                WINE_FIXME!("??? %u\n", GET_SHORT(format, 0 as ::core::ffi::c_uint) as ::core::ffi::c_int);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                format = format.offset(size as isize);
                            }
                            137 => {
                                format = format.offset(1 as ::core::ffi::c_int as isize);
                                if (*rd).current_link.is_null() {
                                    WINE_FIXME!("No existing link\n");
                                }
                                (*(*rd).current_link).cpMax = (*rd).char_pos as DWORD;
                                (*rd).current_link = ::core::ptr::null_mut::<HLPFILE_LINK>();
                                (*rd).force_color = FALSE as BOOL;
                            }
                            139 => {
                                if HLPFILE_RtfAddControl(
                                    rd,
                                    b"\\~\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break 's_380;
                                }
                                format = format.offset(1 as ::core::ffi::c_int as isize);
                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                            }
                            140 => {
                                if HLPFILE_RtfAddControl(
                                    rd,
                                    b"\\_\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 6020540383401037962;
                                    break 's_380;
                                }
                                format = format.offset(1 as ::core::ffi::c_int as isize);
                                (*rd).char_pos = (*rd).char_pos.wrapping_add(1);
                            }
                            200 | 204 => {
                                WINE_TRACE!("macro => %s\n", debugstr_a( (format as *mut ::core::ffi::c_char) .offset(3 as ::core::ffi::c_int as isize), ));
                                HLPFILE_AllocLink(
                                    rd,
                                    hlp_link_macro as ::core::ffi::c_int,
                                    (format as *const ::core::ffi::c_char)
                                        .offset(3 as ::core::ffi::c_int as isize),
                                    GET_USHORT(format, 1 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_uint,
                                    0 as LONG,
                                    (*format as ::core::ffi::c_int & 4 as ::core::ffi::c_int == 0)
                                        as ::core::ffi::c_int,
                                    (FALSE) as BOOL,
                                    -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
                                );
                                format = format.offset(
                                    (3 as ::core::ffi::c_int
                                        + GET_USHORT(format, 1 as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int)
                                        as isize,
                                );
                            }
                            224 | 225 => {
                                WINE_WARN!("jump topic 1 => %u\n", GET_UINT(format, 1 as ::core::ffi::c_uint));
                                HLPFILE_AllocLink(
                                    rd,
                                    if *format as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
                                    {
                                        hlp_link_link as ::core::ffi::c_int
                                    } else {
                                        hlp_link_popup as ::core::ffi::c_int
                                    },
                                    (*(*page).file).lpszPath as *const ::core::ffi::c_char,
                                    -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
                                    GET_UINT(format, 1 as ::core::ffi::c_uint) as LONG,
                                    (TRUE) as BOOL,
                                    (FALSE) as BOOL,
                                    -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
                                );
                                format = format.offset(5 as ::core::ffi::c_int as isize);
                            }
                            226 | 227 | 230 | 231 => {
                                WINE_WARN!("jump topic 1 => %u\n", GET_UINT(format, 1 as ::core::ffi::c_uint));
                                HLPFILE_AllocLink(
                                    rd,
                                    if *format as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
                                    {
                                        hlp_link_link as ::core::ffi::c_int
                                    } else {
                                        hlp_link_popup as ::core::ffi::c_int
                                    },
                                    (*(*page).file).lpszPath as *const ::core::ffi::c_char,
                                    -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
                                    GET_UINT(format, 1 as ::core::ffi::c_uint) as LONG,
                                    (*format as ::core::ffi::c_int & 4 as ::core::ffi::c_int == 0)
                                        as ::core::ffi::c_int,
                                    (FALSE) as BOOL,
                                    -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
                                );
                                format = format.offset(5 as ::core::ffi::c_int as isize);
                            }
                            234 | 235 | 238 | 239 => {
                                let mut ptr: *const ::core::ffi::c_char = (format
                                    as *const ::core::ffi::c_char)
                                    .offset(8 as ::core::ffi::c_int as isize);
                                let mut type_2: BYTE =
                                    *format.offset(3 as ::core::ffi::c_int as isize);
                                let mut wnd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
                                let mut current_block_246: u64;
                                match type_2 as ::core::ffi::c_int {
                                    1 => {
                                        wnd = *ptr as ::core::ffi::c_int;
                                        current_block_246 = 3863783756693701976;
                                    }
                                    0 => {
                                        current_block_246 = 3863783756693701976;
                                    }
                                    6 => {
                                        wnd = (*(*page).file)
                                            .numWindows
                                            .wrapping_sub(1 as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int;
                                        while wnd >= 0 as ::core::ffi::c_int {
                                            if strcmp(
                                                ptr,
                                                &raw mut (*(*(*page).file)
                                                    .windows
                                                    .offset(wnd as isize))
                                                .name
                                                    as *mut ::core::ffi::c_char,
                                            ) == 0
                                            {
                                                break;
                                            }
                                            wnd -= 1;
                                        }
                                        if wnd == -(1 as ::core::ffi::c_int) {
                                            WINE_WARN!("Couldn't find window info for %s\n", debugstr_a(ptr));
                                        }
                                        ptr = ptr
                                            .offset(strlen(ptr).wrapping_add(1 as size_t) as isize);
                                        current_block_246 = 11579725430234891816;
                                    }
                                    4 => {
                                        current_block_246 = 11579725430234891816;
                                    }
                                    _ => {
                                        WINE_WARN!("Unknown link type %d\n", type_2 as ::core::ffi::c_int);
                                        current_block_246 = 11579725430234891816;
                                    }
                                }
                                match current_block_246 {
                                    3863783756693701976 => {
                                        ptr =
                                            (*(*page).file).lpszPath as *const ::core::ffi::c_char;
                                    }
                                    _ => {}
                                }
                                HLPFILE_AllocLink(
                                    rd,
                                    if *format as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
                                    {
                                        hlp_link_link as ::core::ffi::c_int
                                    } else {
                                        hlp_link_popup as ::core::ffi::c_int
                                    },
                                    ptr,
                                    -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
                                    GET_UINT(format, 4 as ::core::ffi::c_uint) as LONG,
                                    (*format as ::core::ffi::c_int & 4 as ::core::ffi::c_int == 0)
                                        as ::core::ffi::c_int,
                                    (FALSE) as BOOL,
                                    wnd as ::core::ffi::c_uint,
                                );
                                format = format.offset(
                                    (3 as ::core::ffi::c_int
                                        + GET_USHORT(format, 1 as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int)
                                        as isize,
                                );
                            }
                            _ => {
                                WINE_WARN!("format %02x\n", *format as ::core::ffi::c_int);
                                format = format.offset(1);
                            }
                        }
                    }
                }
            }
            match current_block {
                6020540383401037962 => {}
                _ => {
                    ret = TRUE as BOOL;
                }
            }
        }
        _ => {}
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, text_base as PVOID);
    return ret;
}
