use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn RtlUlongByteSwap(ul: u32) -> u32;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
unsafe fn adjust_endianness32(ptr: *mut u32) {
    #[cfg(not(target_endian = "big"))]
    {
        *ptr = RtlUlongByteSwap(*ptr);
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_profile_header(profile: *const profile, header: *mut PROFILEHEADER) {
    core::ptr::copy_nonoverlapping(
        (*profile).data as *const u8,
        header as *mut u8,
        core::mem::size_of::<PROFILEHEADER>(),
    );

    let count = core::mem::size_of::<PROFILEHEADER>() / core::mem::size_of::<u32>();
    for i in 0..count {
        adjust_endianness32((header as *mut u32).add(i));
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_profile_header(profile: *const profile, header: *const PROFILEHEADER) {
    core::ptr::copy_nonoverlapping(
        header as *const u8,
        (*profile).data as *mut u8,
        core::mem::size_of::<PROFILEHEADER>(),
    );

    let count = core::mem::size_of::<PROFILEHEADER>() / core::mem::size_of::<u32>();
    for i in 0..count {
        adjust_endianness32(((*profile).data as *mut u32).add(i));
    }
}
unsafe fn get_adjusted_tag(profile: *const profile, type_: TAGTYPE, tag: *mut cmsTagEntry) -> i32 {
    let num_tags_ptr = (*profile).data.add(core::mem::size_of::<cmsICCHeader>()) as *const u32;
    let mut num_tags = *num_tags_ptr;
    adjust_endianness32(&mut num_tags);

    for i in 0..num_tags {
        let entry = (*profile).data
            .add(core::mem::size_of::<cmsICCHeader>() + core::mem::size_of::<u32>()
                + (i as usize) * core::mem::size_of::<cmsTagEntry>())
            as *mut cmsTagEntry;

        let mut sig = (*entry).sig;
        adjust_endianness32(&mut sig);
        if sig == type_ {
            (*tag).sig = sig;
            (*tag).offset = (*entry).offset;
            (*tag).size = (*entry).size;
            adjust_endianness32(&mut (*tag).offset);
            adjust_endianness32(&mut (*tag).size);
            return 1;
        }
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn get_tag_data(
    profile: *const profile,
    type_: TAGTYPE,
    offset: u32,
    buffer: *mut core::ffi::c_void,
    len: *mut u32,
) -> i32 {
    let mut tag: cmsTagEntry = core::mem::zeroed();
    if get_adjusted_tag(profile, type_, &mut tag) == 0 {
        return 0;
    }

    let mut offset = offset;
    if buffer.is_null() {
        offset = 0;
    }
    if offset > tag.size {
        return 0;
    }
    if *len < tag.size - offset || buffer.is_null() {
        *len = tag.size - offset;
        return 0;
    }
    core::ptr::copy_nonoverlapping(
        (*profile).data.add((tag.offset + offset) as usize) as *const u8,
        buffer as *mut u8,
        (tag.size - offset) as usize,
    );
    *len = tag.size - offset;
    1
}
#[no_mangle]
pub unsafe extern "C" fn set_tag_data(
    profile: *const profile,
    type_: TAGTYPE,
    offset: u32,
    buffer: *const core::ffi::c_void,
    len: *mut u32,
) -> i32 {
    let mut tag: cmsTagEntry = core::mem::zeroed();
    if get_adjusted_tag(profile, type_, &mut tag) == 0 {
        return 0;
    }

    if offset > tag.size {
        return 0;
    }
    *len = core::cmp::min(tag.size - offset, *len);
    core::ptr::copy_nonoverlapping(
        buffer as *const u8,
        (*profile).data.add((tag.offset + offset) as usize) as *mut u8,
        *len as usize,
    );
    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * MSCMS - Color Management System for Wine
//  *
//  * Copyright 2004, 2005 Hans Leidekker
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "config.h"
//
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "wine/winternl.h"
// #include "icm.h"
//
// #include "mscms_priv.h"
//
// #ifdef HAVE_LCMS2
//
// static inline void adjust_endianness32( ULONG *ptr )
// {
// #ifndef WORDS_BIGENDIAN
//     *ptr = RtlUlongByteSwap(*ptr);
// #endif
// }
//
// void get_profile_header( const struct profile *profile, PROFILEHEADER *header )
// {
//     unsigned int i;
//
//     memcpy( header, profile->data, sizeof(PROFILEHEADER) );
//
//     /* ICC format is big-endian, swap bytes if necessary */
//     for (i = 0; i < sizeof(PROFILEHEADER) / sizeof(ULONG); i++)
//         adjust_endianness32( (ULONG *)header + i );
// }
//
// void set_profile_header( const struct profile *profile, const PROFILEHEADER *header )
// {
//     unsigned int i;
//
//     memcpy( profile->data, header, sizeof(PROFILEHEADER) );
//
//     /* ICC format is big-endian, swap bytes if necessary */
//     for (i = 0; i < sizeof(PROFILEHEADER) / sizeof(ULONG); i++)
//         adjust_endianness32( (ULONG *)profile->data + i );
// }
//
// static BOOL get_adjusted_tag( const struct profile *profile, TAGTYPE type, cmsTagEntry *tag )
// {
//     DWORD i, num_tags = *(DWORD *)(profile->data + sizeof(cmsICCHeader));
//     cmsTagEntry *entry;
//     ULONG sig;
//
//     adjust_endianness32( &num_tags );
//     for (i = 0; i < num_tags; i++)
//     {
//         entry = (cmsTagEntry *)(profile->data + sizeof(cmsICCHeader) + sizeof(DWORD) + i * sizeof(*tag));
//         sig = entry->sig;
//         adjust_endianness32( &sig );
//         if (sig == type)
//         {
//             tag->sig    = sig;
//             tag->offset = entry->offset;
//             tag->size   = entry->size;
//             adjust_endianness32( &tag->offset );
//             adjust_endianness32( &tag->size );
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// BOOL get_tag_data( const struct profile *profile, TAGTYPE type, DWORD offset, void *buffer, DWORD *len )
// {
//     cmsTagEntry tag;
//
//     if (!get_adjusted_tag( profile, type, &tag )) return FALSE;
//
//     if (!buffer) offset = 0;
//     if (offset > tag.size) return FALSE;
//     if (*len < tag.size - offset || !buffer)
//     {
//         *len = tag.size - offset;
//         return FALSE;
//     }
//     memcpy( buffer, profile->data + tag.offset + offset, tag.size - offset );
//     *len = tag.size - offset;
//     return TRUE;
// }
//
// BOOL set_tag_data( const struct profile *profile, TAGTYPE type, DWORD offset, const void *buffer, DWORD *len )
// {
//     cmsTagEntry tag;
//
//     if (!get_adjusted_tag( profile, type, &tag )) return FALSE;
//
//     if (offset > tag.size) return FALSE;
//     *len = min( tag.size - offset, *len );
//     memcpy( profile->data + tag.offset + offset, buffer, *len );
//     return TRUE;
// }
//
// #endif /* HAVE_LCMS2 */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
