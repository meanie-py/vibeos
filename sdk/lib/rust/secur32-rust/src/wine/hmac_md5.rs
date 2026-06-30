use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn HMACMD5Init(ctx: *mut HMAC_MD5_CTX, mut key: *const core::ffi::c_uchar, mut key_len: core::ffi::c_uint) {
    let mut inner_padding: [core::ffi::c_uchar; 64] = [0; 64];
    let mut temp_key: [core::ffi::c_uchar; 16] = [0; 16];

    if key_len > 64 {
        let mut temp_ctx: MD5_CTX = core::mem::zeroed();

        MD5Init(&mut temp_ctx);
        MD5Update(&mut temp_ctx, key, key_len);
        MD5Final(&mut temp_ctx);
        memcpy(
            temp_key.as_mut_ptr() as *mut core::ffi::c_void,
            temp_ctx.digest.as_ptr() as *const core::ffi::c_void,
            16,
        );

        key = temp_key.as_ptr();
        key_len = 16;
    }

    memset(inner_padding.as_mut_ptr() as *mut core::ffi::c_void, 0, 64);
    memset((*ctx).outer_padding.as_mut_ptr() as *mut core::ffi::c_void, 0, 64);
    memcpy(
        inner_padding.as_mut_ptr() as *mut core::ffi::c_void,
        key as *const core::ffi::c_void,
        key_len as usize,
    );
    memcpy(
        (*ctx).outer_padding.as_mut_ptr() as *mut core::ffi::c_void,
        key as *const core::ffi::c_void,
        key_len as usize,
    );

    let mut i: core::ffi::c_int = 0;
    while i < 64 {
        inner_padding[i as usize] ^= 0x36;
        (*ctx).outer_padding[i as usize] ^= 0x5c;
        i += 1;
    }

    MD5Init(&mut (*ctx).ctx);
    MD5Update(&mut (*ctx).ctx, inner_padding.as_ptr(), 64);
}
#[no_mangle]
pub unsafe extern "C" fn HMACMD5Update(ctx: *mut HMAC_MD5_CTX, data: *const core::ffi::c_uchar, data_len: core::ffi::c_uint) {
    MD5Update(core::ptr::addr_of_mut!((*ctx).ctx), data, data_len);
}
#[no_mangle]
pub unsafe extern "C" fn HMACMD5Final(ctx: *mut HMAC_MD5_CTX, digest: *mut core::ffi::c_uchar) {
    let mut outer_ctx: MD5_CTX = core::mem::zeroed();
    let mut inner_digest: [core::ffi::c_uchar; 16] = [0; 16];

    MD5Final(core::ptr::addr_of_mut!((*ctx).ctx));
    memcpy(
        inner_digest.as_mut_ptr() as *mut core::ffi::c_void,
        (*ctx).ctx.digest.as_ptr() as *const core::ffi::c_void,
        16,
    );

    MD5Init(core::ptr::addr_of_mut!(outer_ctx));
    MD5Update(
        core::ptr::addr_of_mut!(outer_ctx),
        (*ctx).outer_padding.as_ptr(),
        64,
    );
    MD5Update(core::ptr::addr_of_mut!(outer_ctx), inner_digest.as_ptr(), 16);
    MD5Final(core::ptr::addr_of_mut!(outer_ctx));

    memcpy(
        digest as *mut core::ffi::c_void,
        outer_ctx.digest.as_ptr() as *const core::ffi::c_void,
        16,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2006 Kai Blin
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
//  *
//  * This file implements RFC 2104 (HMAC) for the MD5 provider.
//  * It is needed for NTLM2 signing and sealing.
//  */
//
// #include "hmac_md5.h"
//
// void HMACMD5Init(HMAC_MD5_CTX *ctx, const unsigned char *key, unsigned int key_len)
// {
//     int i;
//     unsigned char inner_padding[64];
//     unsigned char temp_key[16];
//
//     if(key_len > 64)
//     {
//         MD5_CTX temp_ctx;
//
//         MD5Init(&temp_ctx);
//         MD5Update(&temp_ctx, key, key_len);
//         MD5Final(&temp_ctx);
//         memcpy(temp_key, temp_ctx.digest, 16);
//
//         key = temp_key;
//         key_len = 16;
//     }
//
//     memset(inner_padding, 0, 64);
//     memset(ctx->outer_padding, 0, 64);
//     memcpy(inner_padding, key, key_len);
//     memcpy(ctx->outer_padding, key, key_len);
//
//     for(i = 0; i < 64; ++i)
//     {
//         inner_padding[i] ^= 0x36;
//         ctx->outer_padding[i] ^= 0x5c;
//     }
//
//     MD5Init(&(ctx->ctx));
//     MD5Update(&(ctx->ctx), inner_padding, 64);
// }
//
// void HMACMD5Update(HMAC_MD5_CTX *ctx, const unsigned char *data, unsigned int data_len)
// {
//     MD5Update(&(ctx->ctx), data, data_len);
// }
//
// void HMACMD5Final(HMAC_MD5_CTX *ctx, unsigned char *digest)
// {
//     MD5_CTX outer_ctx;
//     unsigned char inner_digest[16];
//
//     MD5Final(&(ctx->ctx));
//     memcpy(inner_digest, ctx->ctx.digest, 16);
//
//     MD5Init(&outer_ctx);
//     MD5Update(&outer_ctx, ctx->outer_padding, 64);
//     MD5Update(&outer_ctx, inner_digest, 16);
//     MD5Final(&outer_ctx);
//
//     memcpy(digest, outer_ctx.digest, 16);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
