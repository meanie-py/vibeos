use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn dbgstr_tag(id: u32) -> *const i8;
    fn debugstr_w(s: PCWSTR) -> *const i8;
}

// Wine debug helpers are cdecl (FIXME is variadic, which stdcall cannot be).
unsafe extern "C" {
    fn FIXME(format: *const i8, ...);
    fn debugstr_w(s: *const u16) -> *const i8;
}

// variadic — cdecl on all arches (never stdcall)
extern "C" {
    fn FIXME(format: *const i8, ...);
}

#[no_mangle]
pub unsafe extern "system" fn CheckBitmapBits(
    transform: HTRANSFORM,
    srcbits: *mut core::ffi::c_void,
    format: BMFORMAT,
    width: u32,
    height: u32,
    stride: u32,
    result: *mut u8,
    callback: PBMCALLBACKFN,
    data: LPARAM,
) -> BOOL {
    FIXME(
        b"( %p, %p, 0x%08x, 0x%08x, 0x%08x, 0x%08x, %p, %p, 0x%08lx ) stub\n\0".as_ptr() as *const i8,
        transform as *mut core::ffi::c_void,
        srcbits,
        format as u32,
        width,
        height,
        stride,
        result as *mut core::ffi::c_void,
        callback as *mut core::ffi::c_void,
        data,
    );

    0
}

#[no_mangle]
pub unsafe extern "system" fn ConvertColorNameToIndex(
    profile: isize,
    name: *const COLOR_NAME,
    index: *mut u32,
    count: u32,
) -> i32 {
    eprintln!(
        "( {:p}, {:p}, {:p}, 0x{:08x} ) stub\n",
        profile as *const core::ffi::c_void,
        name as *const core::ffi::c_void,
        index as *const core::ffi::c_void,
        count
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ConvertIndexToColorName(
    profile: HPROFILE,
    index: *mut u32,
    name: *mut COLOR_NAME,
    count: u32,
) -> i32 {
    FIXME(
        b"( %p, %p, %p, 0x%08x ) stub\n\0".as_ptr() as *const i8,
        profile as *const core::ffi::c_void,
        index as *const core::ffi::c_void,
        name as *const core::ffi::c_void,
        count,
    );

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CreateDeviceLinkProfile(profiles: *mut ::core::ffi::c_void, nprofiles: u32, intents: *mut u32, nintents: u32, flags: u32, data: *mut *mut u8, index: u32) -> i32 {
    FIXME(b"( %p, 0x%08x, %p, 0x%08x, 0x%08x, %p, 0x%08x ) stub\n\0".as_ptr() as *const i8, profiles, nprofiles as u32, intents, nintents as u32, flags as u32, data, index as u32);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CreateProfileFromLogColorSpaceA(
    space: *mut LOGCOLORSPACEA,
    buffer: *mut *mut u8,
) -> i32 {
    unsafe {
        FIXME(
            b"( %p, %p ) stub\n\0".as_ptr() as *const i8,
            space,
            buffer,
        );
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CreateProfileFromLogColorSpaceW(space: LPLOGCOLORSPACEW, buffer: *mut *mut u8) -> i32 {
    unsafe {
        FIXME(b"( %p, %p ) stub\n\0".as_ptr().cast::<i8>(), space, buffer);
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn GenerateCopyFilePaths(
    printer: *const u16,
    directory: *const u16,
    clientinfo: *mut u8,
    level: u32,
    sourcedir: *mut u16,
    sourcedirsize: *mut u32,
    targetdir: *mut u16,
    targetdirsize: *mut u32,
    flags: u32,
) -> u32 {
    FIXME(
        b"( %s, %s, %p, 0x%08x, %p, %p, %p, %p, 0x%08x ) stub\n\0".as_ptr() as *const i8,
        debugstr_w(printer),
        debugstr_w(directory),
        clientinfo as *const core::ffi::c_void,
        level,
        sourcedir as *const core::ffi::c_void,
        sourcedirsize as *const core::ffi::c_void,
        targetdir as *const core::ffi::c_void,
        targetdirsize as *const core::ffi::c_void,
        flags,
    );
    ntdll_sys::nt::ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn GetCMMInfo(transform: HTRANSFORM, info: DWORD) -> DWORD {
    FIXME(b"( %p, 0x%08x ) stub\n\0".as_ptr() as *const i8, transform, info);
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedProfileInfo(
    profile: HPROFILE,
    info: PNAMED_PROFILE_INFO,
) -> BOOL {
    FIXME(
        b"(%p, %p) stub\n\0".as_ptr() as *const core::ffi::c_char,
        profile as *const core::ffi::c_void,
        info as *const core::ffi::c_void,
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetPS2ColorRenderingDictionary(
    profile: HPROFILE,
    intent: u32,
    buffer: *mut u8,
    size: *mut u32,
    binary: *mut i32,
) -> i32 {
    FIXME(
        b"( %p, 0x%08x, %p, %p, %p ) stub\n\0".as_ptr() as *const i8,
        profile,
        intent,
        buffer,
        size,
        binary,
    );
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetPS2ColorRenderingIntent(
    profile: HPROFILE,
    intent: u32,
    buffer: *mut u8,
    size: *mut u32,
) -> i32 {

    let fmt_ptr = b"(%p, 0x%08x, %p, %p) stub\n\0".as_ptr() as *const i8;
    // Call to FIXME with variadic arguments matching the format string
    FIXME(
        fmt_ptr,
        profile as *const core::ffi::c_void,
        intent,
        buffer as *const core::ffi::c_void,
        size as *const core::ffi::c_void,
    );

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetPS2ColorSpaceArray(
    profile: *mut core::ffi::c_void,
    intent: u32,
    r#type: u32,
    buffer: *mut u8,
    size: *mut u32,
    binary: *mut i32,
) -> i32 {
    FIXME(
        c" ( %p, 0x%08x, 0x%08x, %p, %p, %p ) stub\n".as_ptr(),
        profile,
        intent,
        r#type,
        buffer,
        size,
        binary,
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterCMMA(machine: PCSTR, id: u32, dll: PCSTR) -> i32 {
    FIXME(
        b"( %p, %s, %p ) stub\n\0".as_ptr() as *const i8,
        machine as *const core::ffi::c_void,
        dbgstr_tag(id),
        dll as *const core::ffi::c_void,
    );
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterCMMW(machine: PCWSTR, id: u32, dll: PCWSTR) -> i32 {
    FIXME(
        b"( %p, %s, %p ) stub\n\0".as_ptr() as *const i8,
        machine as *const core::ffi::c_void,
        dbgstr_tag(id),
        dll as *const core::ffi::c_void,
    );
    1
}

#[no_mangle]
pub unsafe extern "system" fn SelectCMM(id: u32) -> i32 {
    FIXME(
        b"(%s) stub\n\0".as_ptr() as *const i8,
        dbgstr_tag(id),
    );
    1
}

#[no_mangle]
pub unsafe extern "system" fn SetColorProfileElementReference(
    profile: *mut core::ffi::c_void,
    type_: u32,
    ref_: u32,
) -> i32 {
    FIXME(
        b"( %p, 0x%08x, 0x%08x ) stub\n".as_ptr() as *const i8,
        profile,
        type_,
        ref_,
    );

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetColorProfileElementSize(profile: HPROFILE, r#type: TAGTYPE, size: u32) -> i32 {
    FIXME(
        b"( %p, 0x%08x, 0x%08x ) stub\n\0".as_ptr() as *const i8,
        profile as *const core::ffi::c_void,
        r#type,
        size,
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetStandardColorSpaceProfileA(
    machine: *const i8,
    id: u32,
    profile: *mut i8,
) -> i32 {
    FIXME(b"( 0x%08x, %p ) stub\n\0".as_ptr() as *const i8, id, profile);
    1
}

#[no_mangle]
pub unsafe extern "system" fn SetStandardColorSpaceProfileW(
    machine: *const u16,
    id: u32,
    profile: *mut u16,
) -> i32 {
    FIXME!("( 0x{:08x}, {:p} ) stub\n", id, profile);
    1 // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SpoolerCopyFileEvent(
    printer: PWSTR,
    key: PWSTR,
    event: u32,
) -> i32 {
    let fmt = b"( %s, %s, 0x%08x ) stub\n\0".as_ptr() as *const i8;
    FIXME(fmt, debugstr_w(printer), debugstr_w(key), event);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterCMMA(machine: PCSTR, id: u32) -> i32 {
    FIXME(
        b"( %p, %s ) stub\n\0".as_ptr() as *const i8,
        machine,
        dbgstr_tag(id),
    );
    1
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterCMMW(machine: PCWSTR, id: u32) -> i32 {
    FIXME(b"( %p, %s ) stub\n\0".as_ptr() as *const i8, machine, dbgstr_tag(id));
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
// #include "wine/debug.h"
//
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "icm.h"
//
// #include "mscms_priv.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(mscms);
//
// BOOL WINAPI CheckBitmapBits( HTRANSFORM transform, PVOID srcbits, BMFORMAT format, DWORD width,
//                              DWORD height, DWORD stride, PBYTE result, PBMCALLBACKFN callback,
//                              LPARAM data )
// {
//     FIXME( "( %p, %p, 0x%08x, 0x%08x, 0x%08x, 0x%08x, %p, %p, 0x%08lx ) stub\n",
//            transform, srcbits, format, width, height, stride, result, callback, data );
//
//     return FALSE;
// }
//
// BOOL WINAPI CheckColors( HTRANSFORM transform, PCOLOR colors, DWORD number, COLORTYPE type,
//                          PBYTE result )
// {
//     FIXME( "( %p, %p, 0x%08x, 0x%08x, %p ) stub\n", transform, colors, number, type, result );
//
//     return FALSE;
// }
//
// BOOL WINAPI ConvertColorNameToIndex( HPROFILE profile, PCOLOR_NAME name, PDWORD index, DWORD count )
// {
//     FIXME( "( %p, %p, %p, 0x%08x ) stub\n", profile, name, index, count );
//
//     return FALSE;
// }
//
// BOOL WINAPI ConvertIndexToColorName( HPROFILE profile, PDWORD index, PCOLOR_NAME name, DWORD count )
// {
//     FIXME( "( %p, %p, %p, 0x%08x ) stub\n", profile, index, name, count );
//
//     return FALSE;
// }
//
// BOOL WINAPI CreateDeviceLinkProfile( PHPROFILE profiles, DWORD nprofiles, PDWORD intents,
//                                      DWORD nintents, DWORD flags, PBYTE *data, DWORD index )
// {
//     FIXME( "( %p, 0x%08x, %p, 0x%08x, 0x%08x, %p, 0x%08x ) stub\n",
//            profiles, nprofiles, intents, nintents, flags, data, index );
//
//     return FALSE;
// }
//
// BOOL WINAPI CreateProfileFromLogColorSpaceA( LPLOGCOLORSPACEA space, PBYTE *buffer )
// {
//     FIXME( "( %p, %p ) stub\n", space, buffer );
//
//     return FALSE;
// }
//
// BOOL WINAPI CreateProfileFromLogColorSpaceW( LPLOGCOLORSPACEW space, PBYTE *buffer )
// {
//     FIXME( "( %p, %p ) stub\n", space, buffer );
//
//     return FALSE;
// }
//
// DWORD WINAPI GenerateCopyFilePaths( LPCWSTR printer, LPCWSTR directory, LPBYTE clientinfo,
//                                     DWORD level, LPWSTR sourcedir, LPDWORD sourcedirsize,
//                                     LPWSTR targetdir, LPDWORD targetdirsize, DWORD flags )
// {
//     FIXME( "( %s, %s, %p, 0x%08x, %p, %p, %p, %p, 0x%08x ) stub\n",
//            debugstr_w(printer), debugstr_w(directory), clientinfo, level, sourcedir,
//            sourcedirsize, targetdir, targetdirsize, flags );
//     return ERROR_SUCCESS;
// }
//
// DWORD WINAPI GetCMMInfo( HTRANSFORM transform, DWORD info )
// {
//     FIXME( "( %p, 0x%08x ) stub\n", transform, info );
//
//     return 0;
// }
//
// BOOL WINAPI GetNamedProfileInfo( HPROFILE profile, PNAMED_PROFILE_INFO info )
// {
//     FIXME( "( %p, %p ) stub\n", profile, info );
//
//     return FALSE;
// }
//
// BOOL WINAPI GetPS2ColorRenderingDictionary( HPROFILE profile, DWORD intent, PBYTE buffer,
//                                             PDWORD size, PBOOL binary )
// {
//     FIXME( "( %p, 0x%08x, %p, %p, %p ) stub\n", profile, intent, buffer, size, binary );
//
//     return FALSE;
// }
//
// BOOL WINAPI GetPS2ColorRenderingIntent( HPROFILE profile, DWORD intent, PBYTE buffer, PDWORD size )
// {
//     FIXME( "( %p, 0x%08x, %p, %p ) stub\n", profile, intent, buffer, size );
//
//     return FALSE;
// }
//
// BOOL WINAPI GetPS2ColorSpaceArray( HPROFILE profile, DWORD intent, DWORD type, PBYTE buffer,
//                                    PDWORD size, PBOOL binary )
// {
//     FIXME( "( %p, 0x%08x, 0x%08x, %p, %p, %p ) stub\n", profile, intent, type, buffer, size, binary );
//
//     return FALSE;
// }
//
// BOOL WINAPI RegisterCMMA( PCSTR machine, DWORD id, PCSTR dll )
// {
//     FIXME( "( %p, %s, %p ) stub\n", machine, dbgstr_tag(id), dll );
//
//     return TRUE;
// }
//
// BOOL WINAPI RegisterCMMW( PCWSTR machine, DWORD id, PCWSTR dll )
// {
//     FIXME( "( %p, %s, %p ) stub\n", machine, dbgstr_tag(id), dll );
//
//     return TRUE;
// }
//
// BOOL WINAPI SelectCMM( DWORD id )
// {
//     FIXME( "(%s) stub\n", dbgstr_tag(id) );
//
//     return TRUE;
// }
//
// BOOL WINAPI SetColorProfileElementReference( HPROFILE profile, TAGTYPE type, TAGTYPE ref )
// {
//     FIXME( "( %p, 0x%08x, 0x%08x ) stub\n", profile, type, ref );
//
//     return TRUE;
// }
//
// BOOL WINAPI SetColorProfileElementSize( HPROFILE profile, TAGTYPE type, DWORD size )
// {
//     FIXME( "( %p, 0x%08x, 0x%08x ) stub\n", profile, type, size );
//
//     return FALSE;
// }
//
// BOOL WINAPI SetStandardColorSpaceProfileA( PCSTR machine, DWORD id, PSTR profile )
// {
//     FIXME( "( 0x%08x, %p ) stub\n", id, profile );
//     return TRUE;
// }
//
// BOOL WINAPI SetStandardColorSpaceProfileW( PCWSTR machine, DWORD id, PWSTR profile )
// {
//     FIXME( "( 0x%08x, %p ) stub\n", id, profile );
//     return TRUE;
// }
//
// BOOL WINAPI SpoolerCopyFileEvent( LPWSTR printer, LPWSTR key, DWORD event )
// {
//     FIXME( "( %s, %s, 0x%08x ) stub\n", debugstr_w(printer), debugstr_w(key), event );
//     return TRUE;
// }
//
// BOOL WINAPI UnregisterCMMA( PCSTR machine, DWORD id )
// {
//     FIXME( "( %p, %s ) stub\n", machine, dbgstr_tag(id) );
//
//     return TRUE;
// }
//
// BOOL WINAPI UnregisterCMMW( PCWSTR machine, DWORD id )
// {
//     FIXME( "( %p, %s ) stub\n", machine, dbgstr_tag(id) );
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
