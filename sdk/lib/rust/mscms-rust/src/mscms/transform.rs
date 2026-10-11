use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- BMFORMAT (icm.h) — values COMPUTED by tools/gen_enum_consts.py. The tail members are implicit increments off BM_XYZTRIPLETS=0x201 / BM_xXYZQUADS=0x301, which is exactly the kind of value nobody should hand-copy ----
pub const BM_RGBTRIPLETS: u32 = 2;
pub const BM_BGRTRIPLETS: u32 = 4;
pub const BM_xRGBQUADS: u32 = 8;
pub const BM_xBGRQUADS: u32 = 16;
pub const BM_GRAY: u32 = 521;
pub const BM_KYMCQUADS: u32 = 773;

extern "C" {
    fn FIXME(format: *const i8, ...);
    fn cmsCreate_sRGBProfile() -> cmsHPROFILE;
    fn cmsCreateProofingTransform( Input: cmsHPROFILE, InputFormat: DWORD, Output: cmsHPROFILE, OutputFormat: DWORD, Proofing: cmsHPROFILE, Intent: i32, ProofingIntent: i32, dwFlags: DWORD, ) -> cmsHPROFILE;
    fn create_transform(transform: *mut transform) -> HTRANSFORM;
    fn dbgstr_tag(tag: DWORD) -> *const i8;
    fn debugstr_w(wstr: *const u16) -> *const i8;
    fn grab_profile(profile: HPROFILE) -> *mut profile;
    fn release_profile(profile: *mut profile);
}
extern "C" {
    fn cmsCreateMultiprofileTransform(profiles: *const *mut core::ffi::c_void, nprofiles: u32, arg1: i32, arg2: i32, intent: u32, arg3: i32) -> *mut core::ffi::c_void;
    fn close_transform(handle: HTRANSFORM) -> BOOL;
}
unsafe extern "system" {
    fn GetProcessHeap() -> isize;
    fn HeapAlloc(heap: isize, flags: u32, size: usize) -> *mut core::ffi::c_void;
    fn HeapFree(heap: isize, flags: u32, mem: *mut core::ffi::c_void) -> i32;
    fn MultiByteToWideChar( CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32, ) -> i32;
}

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(mscms) – corresponds to file‑scope static debug channel
static __wine_dbch_mscms: __wine_debug_channel = __wine_debug_channel {
    flags: !0,
    name: c"mscms".as_ptr() as *const i8,
};

const COLOR_GRAY: u32 = 1; // nolint: const-contradiction (unrelated namespaces: an ICM colour type vs a console colour)
                           // icm.h:124 (the COLOR enum starts at 1, not 0)
const COLOR_RGB: u32 = 0;
const COLOR_XYZ: u32 = 0;
const COLOR_Yxy: u32 = 0;
const COLOR_Lab: u32 = 0;
const COLOR_CMYK: u32 = 0;
const TYPE_GRAY_16: u32 = 0;
const TYPE_RGB_16: u32 = 0;
const TYPE_XYZ_16: u32 = 0;
const TYPE_Yxy_16: u32 = 0;
const TYPE_Lab_16: u32 = 0;
const TYPE_CMYK_16: u32 = 0;

unsafe extern "C" {
    fn FIXME(format: *const i8, ...);
}

const CP_ACP: u32 = 0;

unsafe fn from_bmformat(format: BMFORMAT) -> DWORD {
    let ret: DWORD;
    match format {
        BM_RGBTRIPLETS => { ret = TYPE_RGB_8; }
        BM_BGRTRIPLETS => { ret = TYPE_BGR_8; }
        BM_GRAY => { ret = TYPE_GRAY_8; }
        BM_xRGBQUADS => { ret = TYPE_ARGB_8; }
        BM_xBGRQUADS => { ret = TYPE_ABGR_8; }
        BM_KYMCQUADS => { ret = TYPE_KYMC_8; }
        _ => {
            if QUIETFIXME == 0 {
                FIXME(b"unhandled bitmap format %08x\n\0".as_ptr() as *const i8, format);
                QUIETFIXME = 1;
            }
            ret = TYPE_RGB_8;
        }
    }
    TRACE!(b"color space: %08x -> %08x\n\0".as_ptr() as *const i8, format, ret);
    ret
}

unsafe fn from_type(type_: COLORTYPE) -> u32 {
    let mut ret: u32;

    match type_ {
        COLOR_GRAY => ret = TYPE_GRAY_16,
        COLOR_RGB => ret = TYPE_RGB_16,
        COLOR_XYZ => ret = TYPE_XYZ_16,
        COLOR_Yxy => ret = TYPE_Yxy_16,
        COLOR_Lab => ret = TYPE_Lab_16,
        COLOR_CMYK => ret = TYPE_CMYK_16,
        _ => {
            FIXME(b"unhandled color type %08x\n\0".as_ptr() as *const i8, type_);
            ret = TYPE_RGB_16;
        }
    }

    TRACE!(b"color type: %08x -> %08x\n\0".as_ptr() as *const i8, type_, ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreateColorTransformA(
    space: *const LOGCOLORSPACEA,
    dest: HPROFILE,
    target: HPROFILE,
    flags: DWORD,
) -> HTRANSFORM {
    // TRACE( "( %p, %p, %p, 0x%08x )\n", space, dest, target, flags );

    if space.is_null() || dest == 0 {
        return core::ptr::null_mut();
    }

    let mut spaceW: LOGCOLORSPACEW = core::mem::zeroed();
    core::ptr::copy_nonoverlapping(
        space as *const u8,
        &mut spaceW as *mut LOGCOLORSPACEW as *mut u8,
        core::mem::offset_of!(LOGCOLORSPACEA, lcsFilename),
    );
    spaceW.lcsSize = core::mem::size_of::<LOGCOLORSPACEW>() as u32;

    let len = MultiByteToWideChar(
        CP_ACP,
        0,
        (*space).lcsFilename.as_ptr(),
        -1,
        core::ptr::null_mut(),
        0,
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        (*space).lcsFilename.as_ptr(),
        -1,
        spaceW.lcsFilename.as_mut_ptr(),
        len,
    );

    CreateColorTransformW(&raw const spaceW, dest, target, flags)
}

#[no_mangle]
pub unsafe extern "system" fn CreateColorTransformW(
    space: *const LOGCOLORSPACEW,
    dest: HPROFILE,
    target: HPROFILE,
    flags: DWORD,
) -> HTRANSFORM {
    // forward extern block for unknown functions

    // constants (values assumed from LCMS)
    const INTENT_PERCEPTUAL: i32 = 0;
    const INTENT_ABSOLUTE_COLORIMETRIC: i32 = 3;
    const cmsFLAGS_SOFTPROOFING: DWORD = 0x0001; // cmsFLAGS_SOFTPROOFING

    unsafe {
        let mut ret: HTRANSFORM = core::ptr::null_mut();
        let mut transform: transform;
        let mut dst: *mut profile;
        let mut tgt: *mut profile = core::ptr::null_mut();
        let mut cmsinput: cmsHPROFILE;
        let mut cmsoutput: cmsHPROFILE;
        let mut cmstarget: cmsHPROFILE = core::ptr::null_mut();
        let mut proofing: DWORD = 0;
        let mut intent: i32;

        TRACE!(b"( %p, %p, %p, 0x%08x )\n\0".as_ptr() as *const i8, space, dest, target, flags);

        if space.is_null() {
            return core::ptr::null_mut();
        }
        dst = grab_profile(dest);
        if dst.is_null() {
            return core::ptr::null_mut();
        }

        if !target.is_null() {
            tgt = grab_profile(target);
            if tgt.is_null() {
                release_profile(dst);
                return core::ptr::null_mut();
            }
        }

        intent = if (*space).lcsIntent > 3 {
            INTENT_PERCEPTUAL
        } else {
            (*space).lcsIntent as i32
        };

        TRACE!(b"lcsIntent:   %x\n\0".as_ptr() as *const i8, (*space).lcsIntent);
        TRACE!(b"lcsCSType:   %s\n\0".as_ptr() as *const i8, dbgstr_tag((*space).lcsCSType));
        TRACE!(b"lcsFilename: %s\n\0".as_ptr() as *const i8, debugstr_w((*space).lcsFilename.as_ptr()));

        cmsinput = cmsCreate_sRGBProfile(); /* FIXME: create from supplied color space */
        if !target.is_null() {
            proofing = cmsFLAGS_SOFTPROOFING;
            cmstarget = (*tgt).cmsprofile;
        }
        cmsoutput = (*dst).cmsprofile;
        transform.cmstransform = cmsCreateProofingTransform(
            cmsinput,
            0,
            cmsoutput,
            0,
            cmstarget,
            intent,
            INTENT_ABSOLUTE_COLORIMETRIC,
            proofing,
        );
        if transform.cmstransform.is_null() {
            if !tgt.is_null() {
                release_profile(tgt);
            }
            release_profile(dst);
            return core::ptr::null_mut();
        }

        ret = create_transform(&mut transform);

        if !tgt.is_null() {
            release_profile(tgt);
        }
        release_profile(dst);

        ret
    }
}

#[no_mangle]
pub unsafe extern "system" fn CreateMultiProfileTransform(
    profiles: *mut isize,
    nprofiles: u32,
    intents: *mut u32,
    nintents: u32,
    flags: u32,
    cmm: u32,
) -> isize {
    let mut ret: isize = 0;
    // #ifdef HAVE_LCMS2 (dropped)
    let cmsprofiles: *mut *mut core::ffi::c_void;
    let mut transform: transform;
    let profile0: *mut profile;
    let profile1: *mut profile;

    TRACE!(b"( %p, 0x%08x, %p, 0x%08x, 0x%08x, 0x%08x )\n".as_ptr() as *const i8, profiles as *mut core::ffi::c_void, nprofiles, intents as *mut core::ffi::c_void, nintents, flags, cmm);

    if profiles.is_null() || nprofiles == 0 || intents.is_null() {
        return 0;
    }

    if nprofiles > 2 {
        FIXME(b"more than 2 profiles not supported\n".as_ptr() as *const i8);
        return 0;
    }

    profile0 = grab_profile(*profiles);
    if profile0.is_null() {
        return 0;
    }
    profile1 = grab_profile(*profiles.offset(1));
    if profile1.is_null() {
        release_profile(profile0);
        return 0;
    }

    cmsprofiles = HeapAlloc(
        GetProcessHeap(),
        0,
        (nprofiles as usize + 1) * core::mem::size_of::<*mut core::ffi::c_void>(),
    ) as *mut *mut core::ffi::c_void;

    if !cmsprofiles.is_null() {
        *cmsprofiles = (*profile0).cmsprofile;
        *cmsprofiles.offset(1) = (*profile1).cmsprofile;

#[no_mangle]
pub unsafe extern "system" fn DeleteColorTransform(handle: HTRANSFORM) -> BOOL {
    let mut ret: BOOL = 0;
    #[cfg(feature = "lcms2")]
    {
        TRACE!(b"( %p )\n\0".as_ptr() as *const i8, handle);
        ret = close_transform(handle);
    }
    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * MSCMS - Color Management System for Wine
//  *
//  * Copyright 2005, 2006, 2008 Hans Leidekker
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
// #include "winnls.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "icm.h"
//
// #include "mscms_priv.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(mscms);
//
// #ifdef HAVE_LCMS2
//
// static DWORD from_bmformat( BMFORMAT format )
// {
//     static BOOL quietfixme = FALSE;
//     DWORD ret;
//
//     switch (format)
//     {
//     case BM_RGBTRIPLETS: ret = TYPE_RGB_8; break;
//     case BM_BGRTRIPLETS: ret = TYPE_BGR_8; break;
//     case BM_GRAY:        ret = TYPE_GRAY_8; break;
//     case BM_xRGBQUADS:   ret = TYPE_ARGB_8; break;
//     case BM_xBGRQUADS:   ret = TYPE_ABGR_8; break;
//     case BM_KYMCQUADS:   ret = TYPE_KYMC_8; break;
//     default:
//         if (!quietfixme)
//         {
//             FIXME( "unhandled bitmap format %08x\n", format );
//             quietfixme = TRUE;
//         }
//         ret = TYPE_RGB_8;
//         break;
//     }
//     TRACE( "color space: %08x -> %08x\n", format, ret );
//     return ret;
// }
//
// static DWORD from_type( COLORTYPE type )
// {
//     DWORD ret;
//
//     switch (type)
//     {
//     case COLOR_GRAY: ret = TYPE_GRAY_16; break;
//     case COLOR_RGB:  ret = TYPE_RGB_16; break;
//     case COLOR_XYZ:  ret = TYPE_XYZ_16; break;
//     case COLOR_Yxy:  ret = TYPE_Yxy_16; break;
//     case COLOR_Lab:  ret = TYPE_Lab_16; break;
//     case COLOR_CMYK: ret = TYPE_CMYK_16; break;
//     default:
//         FIXME( "unhandled color type %08x\n", type );
//         ret = TYPE_RGB_16;
//         break;
//     }
//
//     TRACE( "color type: %08x -> %08x\n", type, ret );
//     return ret;
// }
//
// #endif /* HAVE_LCMS2 */
//
// /******************************************************************************
//  * CreateColorTransformA            [MSCMS.@]
//  *
//  * See CreateColorTransformW.
//  */
// HTRANSFORM WINAPI CreateColorTransformA( LPLOGCOLORSPACEA space, HPROFILE dest,
//     HPROFILE target, DWORD flags )
// {
//     LOGCOLORSPACEW spaceW;
//     DWORD len;
//
//     TRACE( "( %p, %p, %p, 0x%08x )\n", space, dest, target, flags );
//
//     if (!space || !dest) return FALSE;
//
//     memcpy( &spaceW, space, FIELD_OFFSET(LOGCOLORSPACEA, lcsFilename) );
//     spaceW.lcsSize = sizeof(LOGCOLORSPACEW);
//
//     len = MultiByteToWideChar( CP_ACP, 0, space->lcsFilename, -1, NULL, 0 );
//     MultiByteToWideChar( CP_ACP, 0, space->lcsFilename, -1, spaceW.lcsFilename, len );
//
//     return CreateColorTransformW( &spaceW, dest, target, flags );
// }
//
// /******************************************************************************
//  * CreateColorTransformW            [MSCMS.@]
//  *
//  * Create a color transform.
//  *
//  * PARAMS
//  *  space  [I] Input color space.
//  *  dest   [I] Color profile of destination device.
//  *  target [I] Color profile of target device.
//  *  flags  [I] Flags.
//  *
//  * RETURNS
//  *  Success: Handle to a transform.
//  *  Failure: NULL
//  */
// HTRANSFORM WINAPI CreateColorTransformW( LPLOGCOLORSPACEW space, HPROFILE dest,
//     HPROFILE target, DWORD flags )
// {
//     HTRANSFORM ret = NULL;
// #ifdef HAVE_LCMS2
//     struct transform transform;
//     struct profile *dst, *tgt = NULL;
//     cmsHPROFILE cmsinput, cmsoutput, cmstarget = NULL;
//     DWORD proofing = 0;
//     int intent;
//
//     TRACE( "( %p, %p, %p, 0x%08x )\n", space, dest, target, flags );
//
//     if (!space || !(dst = grab_profile( dest ))) return FALSE;
//
//     if (target && !(tgt = grab_profile( target )))
//     {
//         release_profile( dst );
//         return FALSE;
//     }
//     intent = space->lcsIntent > 3 ? INTENT_PERCEPTUAL : space->lcsIntent;
//
//     TRACE( "lcsIntent:   %x\n", space->lcsIntent );
//     TRACE( "lcsCSType:   %s\n", dbgstr_tag( space->lcsCSType ) );
//     TRACE( "lcsFilename: %s\n", debugstr_w( space->lcsFilename ) );
//
//     cmsinput = cmsCreate_sRGBProfile(); /* FIXME: create from supplied color space */
//     if (target)
//     {
//         proofing = cmsFLAGS_SOFTPROOFING;
//         cmstarget = tgt->cmsprofile;
//     }
//     cmsoutput = dst->cmsprofile;
//     transform.cmstransform = cmsCreateProofingTransform(cmsinput, 0, cmsoutput, 0, cmstarget,
//                                                         intent, INTENT_ABSOLUTE_COLORIMETRIC,
//                                                         proofing);
//     if (!transform.cmstransform)
//     {
//         if (tgt) release_profile( tgt );
//         release_profile( dst );
//         return FALSE;
//     }
//
//     ret = create_transform( &transform );
//
//     if (tgt) release_profile( tgt );
//     release_profile( dst );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * CreateMultiProfileTransform      [MSCMS.@]
//  *
//  * Create a color transform from an array of color profiles.
//  *
//  * PARAMS
//  *  profiles  [I] Array of color profiles.
//  *  nprofiles [I] Number of color profiles.
//  *  intents   [I] Array of rendering intents.
//  *  flags     [I] Flags.
//  *  cmm       [I] Profile to take the CMM from.
//  *
//  * RETURNS
//  *  Success: Handle to a transform.
//  *  Failure: NULL
//  */ 
// HTRANSFORM WINAPI CreateMultiProfileTransform( PHPROFILE profiles, DWORD nprofiles,
//     PDWORD intents, DWORD nintents, DWORD flags, DWORD cmm )
// {
//     HTRANSFORM ret = NULL;
// #ifdef HAVE_LCMS2
//     cmsHPROFILE *cmsprofiles;
//     struct transform transform;
//     struct profile *profile0, *profile1;
//
//     TRACE( "( %p, 0x%08x, %p, 0x%08x, 0x%08x, 0x%08x )\n",
//            profiles, nprofiles, intents, nintents, flags, cmm );
//
//     if (!profiles || !nprofiles || !intents) return NULL;
//
//     if (nprofiles > 2)
//     {
//         FIXME("more than 2 profiles not supported\n");
//         return NULL;
//     }
//
//     profile0 = grab_profile( profiles[0] );
//     if (!profile0) return NULL;
//     profile1 = grab_profile( profiles[1] );
//     if (!profile1)
//     {
//         release_profile( profile0 );
//         return NULL;
//     }
//
//     if ((cmsprofiles = HeapAlloc( GetProcessHeap(), 0, (nprofiles + 1) * sizeof(cmsHPROFILE) )))
//     {
//         cmsprofiles[0] = profile0->cmsprofile;
//         cmsprofiles[1] = profile1->cmsprofile;
//
//         transform.cmstransform = cmsCreateMultiprofileTransform( cmsprofiles, nprofiles, 0,
//                                                                  0, *intents, 0 );
//         HeapFree( GetProcessHeap(), 0, cmsprofiles );
//         if (!transform.cmstransform)
//         {
//             release_profile( profile0 );
//             release_profile( profile1 );
//             return FALSE;
//         }
//         ret = create_transform( &transform );
//     }
//
//     release_profile( profile0 );
//     release_profile( profile1 );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * DeleteColorTransform             [MSCMS.@]
//  *
//  * Delete a color transform.
//  *
//  * PARAMS
//  *  transform [I] Handle to a color transform.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */ 
// BOOL WINAPI DeleteColorTransform( HTRANSFORM handle )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//
//     TRACE( "( %p )\n", handle );
//
//     ret = close_transform( handle );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * TranslateBitmapBits              [MSCMS.@]
//  *
//  * Perform color translation.
//  *
//  * PARAMS
//  *  transform    [I] Handle to a color transform.
//  *  srcbits      [I] Source bitmap.
//  *  input        [I] Format of the source bitmap.
//  *  width        [I] Width of the source bitmap.
//  *  height       [I] Height of the source bitmap.
//  *  inputstride  [I] Number of bytes in one scanline.
//  *  destbits     [I] Destination bitmap.
//  *  output       [I] Format of the destination bitmap.
//  *  outputstride [I] Number of bytes in one scanline. 
//  *  callback     [I] Callback function.
//  *  data         [I] Callback data. 
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI TranslateBitmapBits( HTRANSFORM handle, PVOID srcbits, BMFORMAT input,
//     DWORD width, DWORD height, DWORD inputstride, PVOID destbits, BMFORMAT output,
//     DWORD outputstride, PBMCALLBACKFN callback, ULONG data )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct transform *transform = grab_transform( handle );
//
//     TRACE( "( %p, %p, 0x%08x, 0x%08x, 0x%08x, 0x%08x, %p, 0x%08x, 0x%08x, %p, 0x%08x )\n",
//            handle, srcbits, input, width, height, inputstride, destbits, output,
//            outputstride, callback, data );
//
//     if (!transform) return FALSE;
//     if (!cmsChangeBuffersFormat( transform->cmstransform, from_bmformat(input), from_bmformat(output) ))
//         return FALSE;
//
//     cmsDoTransform( transform->cmstransform, srcbits, destbits, width * height );
//     release_transform( transform );
//     ret = TRUE;
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * TranslateColors              [MSCMS.@]
//  *
//  * Perform color translation.
//  *
//  * PARAMS
//  *  transform    [I] Handle to a color transform.
//  *  input        [I] Array of input colors.
//  *  number       [I] Number of colors to translate.
//  *  input_type   [I] Input color format.
//  *  output       [O] Array of output colors.
//  *  output_type  [I] Output color format.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI TranslateColors( HTRANSFORM handle, PCOLOR in, DWORD count,
//                              COLORTYPE input_type, PCOLOR out, COLORTYPE output_type )
// {
// #ifdef HAVE_LCMS2
//     BOOL ret = TRUE;
//     struct transform *transform = grab_transform( handle );
//     cmsHTRANSFORM xfrm;
//     unsigned int i;
//
//     TRACE( "( %p, %p, %d, %d, %p, %d )\n", handle, in, count, input_type, out, output_type );
//
//     if (!transform) return FALSE;
//
//     xfrm = transform->cmstransform;
//     if (!cmsChangeBuffersFormat( xfrm, from_type(input_type), from_type(output_type) ))
//         return FALSE;
//
//     switch (input_type)
//     {
//     case COLOR_RGB:
//     {
//         switch (output_type)
//         {
//         case COLOR_RGB:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].rgb, &out[i].rgb, 1 ); goto done;
//         case COLOR_Lab:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].rgb, &out[i].Lab, 1 ); goto done;
//         case COLOR_GRAY: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].rgb, &out[i].gray, 1 ); goto done;
//         case COLOR_CMYK: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].rgb, &out[i].cmyk, 1 ); goto done;
//         case COLOR_XYZ:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].rgb, &out[i].XYZ, 1 ); goto done;
//         default:
//             FIXME("unhandled input/output pair: %d/%d\n", input_type, output_type);
//             ret = FALSE;
//             break;
//         }
//         break;
//     }
//     case COLOR_Lab:
//     {
//         switch (output_type)
//         {
//         case COLOR_RGB:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].Lab, &out[i].rgb, 1 ); goto done;
//         case COLOR_Lab:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].Lab, &out[i].Lab, 1 ); goto done;
//         case COLOR_GRAY: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].Lab, &out[i].gray, 1 ); goto done;
//         case COLOR_CMYK: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].Lab, &out[i].cmyk, 1 ); goto done;
//         case COLOR_XYZ:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].Lab, &out[i].XYZ, 1 ); goto done;
//         default:
//             FIXME("unhandled input/output pair: %d/%d\n", input_type, output_type);
//             ret = FALSE;
//             break;
//         }
//         break;
//     }
//     case COLOR_GRAY:
//     {
//         switch (output_type)
//         {
//         case COLOR_RGB:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].gray, &out[i].rgb, 1 ); goto done;
//         case COLOR_Lab:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].gray, &out[i].Lab, 1 ); goto done;
//         case COLOR_GRAY: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].gray, &out[i].gray, 1 ); goto done;
//         case COLOR_CMYK: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].gray, &out[i].cmyk, 1 ); goto done;
//         case COLOR_XYZ:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].gray, &out[i].XYZ, 1 ); goto done;
//         default:
//             FIXME("unhandled input/output pair: %d/%d\n", input_type, output_type);
//             ret = FALSE;
//             break;
//         }
//         break;
//     }
//     case COLOR_CMYK:
//     {
//         switch (output_type)
//         {
//         case COLOR_RGB:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].cmyk, &out[i].rgb, 1 ); goto done;
//         case COLOR_Lab:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].cmyk, &out[i].Lab, 1 ); goto done;
//         case COLOR_GRAY: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].cmyk, &out[i].gray, 1 ); goto done;
//         case COLOR_CMYK: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].cmyk, &out[i].cmyk, 1 ); goto done;
//         case COLOR_XYZ:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].cmyk, &out[i].XYZ, 1 ); goto done;
//         default:
//             FIXME("unhandled input/output pair: %d/%d\n", input_type, output_type);
//             ret = FALSE;
//             break;
//         }
//         break;
//     }
//     case COLOR_XYZ:
//     {
//         switch (output_type)
//         {
//         case COLOR_RGB:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].XYZ, &out[i].rgb, 1 ); goto done;
//         case COLOR_Lab:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].XYZ, &out[i].Lab, 1 ); goto done;
//         case COLOR_GRAY: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].XYZ, &out[i].gray, 1 ); goto done;
//         case COLOR_CMYK: for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].XYZ, &out[i].cmyk, 1 ); goto done;
//         case COLOR_XYZ:  for (i = 0; i < count; i++) cmsDoTransform( xfrm, &in[i].XYZ, &out[i].XYZ, 1 ); goto done;
//         default:
//             FIXME("unhandled input/output pair: %d/%d\n", input_type, output_type);
//             ret = FALSE;
//             break;
//         }
//         break;
//     }
//     default:
//         FIXME("unhandled input/output pair: %d/%d\n", input_type, output_type);
//         ret = FALSE;
//         break;
//     }
//
// done:
//     release_transform( transform );
//     return ret;
//
// #else  /* HAVE_LCMS2 */
//     return FALSE;
// #endif /* HAVE_LCMS2 */
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
