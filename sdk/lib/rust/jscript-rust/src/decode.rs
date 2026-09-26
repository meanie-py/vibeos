/*
 * Copyright 2012 Jacek Caban for CodeWeavers
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
 */
// Compiler translation; only stable syntax and compiler-derived scalar ABI normalization.
#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
extern "C" {
    fn wcsncmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> i32;
    fn wine_dbgstr_wn(s: *const WCHAR, n: i32) -> *const i8;
    fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const i8,
        func: *const i8,
        line: i32,
        format: *const i8,
        ...
    ) -> i32;
}
pub type size_t = usize;
pub type wchar_t = u16;
pub type BOOL = i32;
pub type DWORD = u32;
pub type LONG = i32;
pub type HRESULT = LONG;
pub type WCHAR = wchar_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: u8,
    pub name: [i8; 15],
}
pub type __wine_debug_class = u32;
pub const __WINE_DBCL_INIT: __wine_debug_class = 7;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const FALSE: i32 = 0 as i32;
pub const TRUE: i32 = 1 as i32;
pub const S_OK: HRESULT = 0 as i32 as HRESULT;
pub const E_FAIL: HRESULT = 0x80004005 as u32 as HRESULT;
pub const JS_E_INVALID_CHAR: HRESULT =
    ((1 as i32 as u32) << 31 as i32
        | (10 as i32 as u32) << 16 as i32
        | 0x3f6 as i32 as u32) as HRESULT;
#[inline]
unsafe extern "C" fn debugstr_w(mut s: *const WCHAR) -> *const i8 {
    return wine_dbgstr_wn(s, -(1 as i32));
}
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_jscript as *mut __wine_debug_channel };
static mut __wine_dbch_jscript: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: !(0 as i32) as u8,
        name: ::core::mem::transmute::<[u8; 15], [i8; 15]>(
            *b"jscript\0\0\0\0\0\0\0\0",
        ),
    }
};
static mut pick_encoding: [u8; 64] = [
    1 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    1 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    1 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    1 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    1 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    1 as i32 as u8,
    2 as i32 as u8,
    0 as i32 as u8,
    1 as i32 as u8,
    0 as i32 as u8,
    2 as i32 as u8,
];
static mut dictionary: [[u8; 3]; 128] = [
    [
        0 as i32 as u8,
        0 as i32 as u8,
        0 as i32 as u8,
    ],
    [
        0x1 as i32 as u8,
        0x1 as i32 as u8,
        0x1 as i32 as u8,
    ],
    [
        0x2 as i32 as u8,
        0x2 as i32 as u8,
        0x2 as i32 as u8,
    ],
    [
        0x3 as i32 as u8,
        0x3 as i32 as u8,
        0x3 as i32 as u8,
    ],
    [
        0x4 as i32 as u8,
        0x4 as i32 as u8,
        0x4 as i32 as u8,
    ],
    [
        0x5 as i32 as u8,
        0x5 as i32 as u8,
        0x5 as i32 as u8,
    ],
    [
        0x6 as i32 as u8,
        0x6 as i32 as u8,
        0x6 as i32 as u8,
    ],
    [
        0x7 as i32 as u8,
        0x7 as i32 as u8,
        0x7 as i32 as u8,
    ],
    [
        0x8 as i32 as u8,
        0x8 as i32 as u8,
        0x8 as i32 as u8,
    ],
    [
        0x7b as i32 as u8,
        0x57 as i32 as u8,
        0x6e as i32 as u8,
    ],
    [
        0xa as i32 as u8,
        0xa as i32 as u8,
        0xa as i32 as u8,
    ],
    [
        0xb as i32 as u8,
        0xb as i32 as u8,
        0xb as i32 as u8,
    ],
    [
        0xc as i32 as u8,
        0xc as i32 as u8,
        0xc as i32 as u8,
    ],
    [
        0xd as i32 as u8,
        0xd as i32 as u8,
        0xd as i32 as u8,
    ],
    [
        0xe as i32 as u8,
        0xe as i32 as u8,
        0xe as i32 as u8,
    ],
    [
        0xf as i32 as u8,
        0xf as i32 as u8,
        0xf as i32 as u8,
    ],
    [
        0x10 as i32 as u8,
        0x10 as i32 as u8,
        0x10 as i32 as u8,
    ],
    [
        0x11 as i32 as u8,
        0x11 as i32 as u8,
        0x11 as i32 as u8,
    ],
    [
        0x12 as i32 as u8,
        0x12 as i32 as u8,
        0x12 as i32 as u8,
    ],
    [
        0x13 as i32 as u8,
        0x13 as i32 as u8,
        0x13 as i32 as u8,
    ],
    [
        0x14 as i32 as u8,
        0x14 as i32 as u8,
        0x14 as i32 as u8,
    ],
    [
        0x15 as i32 as u8,
        0x15 as i32 as u8,
        0x15 as i32 as u8,
    ],
    [
        0x16 as i32 as u8,
        0x16 as i32 as u8,
        0x16 as i32 as u8,
    ],
    [
        0x17 as i32 as u8,
        0x17 as i32 as u8,
        0x17 as i32 as u8,
    ],
    [
        0x18 as i32 as u8,
        0x18 as i32 as u8,
        0x18 as i32 as u8,
    ],
    [
        0x19 as i32 as u8,
        0x19 as i32 as u8,
        0x19 as i32 as u8,
    ],
    [
        0x1a as i32 as u8,
        0x1a as i32 as u8,
        0x1a as i32 as u8,
    ],
    [
        0x1b as i32 as u8,
        0x1b as i32 as u8,
        0x1b as i32 as u8,
    ],
    [
        0x1c as i32 as u8,
        0x1c as i32 as u8,
        0x1c as i32 as u8,
    ],
    [
        0x1d as i32 as u8,
        0x1d as i32 as u8,
        0x1d as i32 as u8,
    ],
    [
        0x1e as i32 as u8,
        0x1e as i32 as u8,
        0x1e as i32 as u8,
    ],
    [
        0x1f as i32 as u8,
        0x1f as i32 as u8,
        0x1f as i32 as u8,
    ],
    [
        0x32 as i32 as u8,
        0x2e as i32 as u8,
        0x2d as i32 as u8,
    ],
    [
        0x30 as i32 as u8,
        0x47 as i32 as u8,
        0x75 as i32 as u8,
    ],
    [
        0x21 as i32 as u8,
        0x7a as i32 as u8,
        0x52 as i32 as u8,
    ],
    [
        0x29 as i32 as u8,
        0x56 as i32 as u8,
        0x60 as i32 as u8,
    ],
    [
        0x5b as i32 as u8,
        0x42 as i32 as u8,
        0x71 as i32 as u8,
    ],
    [
        0x38 as i32 as u8,
        0x6a as i32 as u8,
        0x5e as i32 as u8,
    ],
    [
        0x33 as i32 as u8,
        0x2f as i32 as u8,
        0x49 as i32 as u8,
    ],
    [
        0x3d as i32 as u8,
        0x26 as i32 as u8,
        0x5c as i32 as u8,
    ],
    [
        0x58 as i32 as u8,
        0x49 as i32 as u8,
        0x62 as i32 as u8,
    ],
    [
        0x3a as i32 as u8,
        0x41 as i32 as u8,
        0x7d as i32 as u8,
    ],
    [
        0x35 as i32 as u8,
        0x34 as i32 as u8,
        0x29 as i32 as u8,
    ],
    [
        0x65 as i32 as u8,
        0x32 as i32 as u8,
        0x36 as i32 as u8,
    ],
    [
        0x39 as i32 as u8,
        0x5b as i32 as u8,
        0x20 as i32 as u8,
    ],
    [
        0x5c as i32 as u8,
        0x76 as i32 as u8,
        0x7c as i32 as u8,
    ],
    [
        0x56 as i32 as u8,
        0x72 as i32 as u8,
        0x7a as i32 as u8,
    ],
    [
        0x73 as i32 as u8,
        0x43 as i32 as u8,
        0x7f as i32 as u8,
    ],
    [
        0x66 as i32 as u8,
        0x38 as i32 as u8,
        0x6b as i32 as u8,
    ],
    [
        0x4e as i32 as u8,
        0x39 as i32 as u8,
        0x63 as i32 as u8,
    ],
    [
        0x45 as i32 as u8,
        0x70 as i32 as u8,
        0x33 as i32 as u8,
    ],
    [
        0x6b as i32 as u8,
        0x45 as i32 as u8,
        0x2b as i32 as u8,
    ],
    [
        0x62 as i32 as u8,
        0x68 as i32 as u8,
        0x68 as i32 as u8,
    ],
    [
        0x59 as i32 as u8,
        0x71 as i32 as u8,
        0x51 as i32 as u8,
    ],
    [
        0x78 as i32 as u8,
        0x4f as i32 as u8,
        0x66 as i32 as u8,
    ],
    [
        0x5e as i32 as u8,
        0x9 as i32 as u8,
        0x76 as i32 as u8,
    ],
    [
        0x7d as i32 as u8,
        0x62 as i32 as u8,
        0x31 as i32 as u8,
    ],
    [
        0x4a as i32 as u8,
        0x44 as i32 as u8,
        0x64 as i32 as u8,
    ],
    [
        0x6d as i32 as u8,
        0x23 as i32 as u8,
        0x54 as i32 as u8,
    ],
    [
        0x71 as i32 as u8,
        0x75 as i32 as u8,
        0x43 as i32 as u8,
    ],
    [
        0 as i32 as u8,
        0 as i32 as u8,
        0 as i32 as u8,
    ],
    [
        0x60 as i32 as u8,
        0x7e as i32 as u8,
        0x3a as i32 as u8,
    ],
    [
        0 as i32 as u8,
        0 as i32 as u8,
        0 as i32 as u8,
    ],
    [
        0x53 as i32 as u8,
        0x5e as i32 as u8,
        0x7e as i32 as u8,
    ],
    [
        0 as i32 as u8,
        0 as i32 as u8,
        0 as i32 as u8,
    ],
    [
        0x42 as i32 as u8,
        0x77 as i32 as u8,
        0x45 as i32 as u8,
    ],
    [
        0x27 as i32 as u8,
        0x4a as i32 as u8,
        0x2c as i32 as u8,
    ],
    [
        0x48 as i32 as u8,
        0x61 as i32 as u8,
        0x2a as i32 as u8,
    ],
    [
        0x72 as i32 as u8,
        0x5d as i32 as u8,
        0x74 as i32 as u8,
    ],
    [
        0x75 as i32 as u8,
        0x22 as i32 as u8,
        0x27 as i32 as u8,
    ],
    [
        0x31 as i32 as u8,
        0x4b as i32 as u8,
        0x37 as i32 as u8,
    ],
    [
        0x37 as i32 as u8,
        0x6f as i32 as u8,
        0x44 as i32 as u8,
    ],
    [
        0x4d as i32 as u8,
        0x4e as i32 as u8,
        0x79 as i32 as u8,
    ],
    [
        0x52 as i32 as u8,
        0x3b as i32 as u8,
        0x59 as i32 as u8,
    ],
    [
        0x22 as i32 as u8,
        0x4c as i32 as u8,
        0x2f as i32 as u8,
    ],
    [
        0x54 as i32 as u8,
        0x50 as i32 as u8,
        0x6f as i32 as u8,
    ],
    [
        0x6a as i32 as u8,
        0x67 as i32 as u8,
        0x26 as i32 as u8,
    ],
    [
        0x47 as i32 as u8,
        0x2a as i32 as u8,
        0x72 as i32 as u8,
    ],
    [
        0x64 as i32 as u8,
        0x7d as i32 as u8,
        0x6a as i32 as u8,
    ],
    [
        0x2d as i32 as u8,
        0x74 as i32 as u8,
        0x39 as i32 as u8,
    ],
    [
        0x20 as i32 as u8,
        0x54 as i32 as u8,
        0x7b as i32 as u8,
    ],
    [
        0x7f as i32 as u8,
        0x2b as i32 as u8,
        0x3f as i32 as u8,
    ],
    [
        0x2e as i32 as u8,
        0x2d as i32 as u8,
        0x38 as i32 as u8,
    ],
    [
        0x4c as i32 as u8,
        0x2c as i32 as u8,
        0x77 as i32 as u8,
    ],
    [
        0x5d as i32 as u8,
        0x30 as i32 as u8,
        0x67 as i32 as u8,
    ],
    [
        0x7e as i32 as u8,
        0x6e as i32 as u8,
        0x53 as i32 as u8,
    ],
    [
        0x6c as i32 as u8,
        0x6b as i32 as u8,
        0x47 as i32 as u8,
    ],
    [
        0x6f as i32 as u8,
        0x66 as i32 as u8,
        0x34 as i32 as u8,
    ],
    [
        0x79 as i32 as u8,
        0x35 as i32 as u8,
        0x78 as i32 as u8,
    ],
    [
        0x74 as i32 as u8,
        0x25 as i32 as u8,
        0x5d as i32 as u8,
    ],
    [
        0x43 as i32 as u8,
        0x21 as i32 as u8,
        0x30 as i32 as u8,
    ],
    [
        0x26 as i32 as u8,
        0x64 as i32 as u8,
        0x23 as i32 as u8,
    ],
    [
        0x76 as i32 as u8,
        0x4d as i32 as u8,
        0x5a as i32 as u8,
    ],
    [
        0x25 as i32 as u8,
        0x52 as i32 as u8,
        0x5b as i32 as u8,
    ],
    [
        0x24 as i32 as u8,
        0x63 as i32 as u8,
        0x6c as i32 as u8,
    ],
    [
        0x2b as i32 as u8,
        0x3f as i32 as u8,
        0x48 as i32 as u8,
    ],
    [
        0x28 as i32 as u8,
        0x7b as i32 as u8,
        0x55 as i32 as u8,
    ],
    [
        0x23 as i32 as u8,
        0x78 as i32 as u8,
        0x70 as i32 as u8,
    ],
    [
        0x41 as i32 as u8,
        0x29 as i32 as u8,
        0x69 as i32 as u8,
    ],
    [
        0x34 as i32 as u8,
        0x28 as i32 as u8,
        0x2e as i32 as u8,
    ],
    [
        0x9 as i32 as u8,
        0x73 as i32 as u8,
        0x4c as i32 as u8,
    ],
    [
        0x2a as i32 as u8,
        0x59 as i32 as u8,
        0x21 as i32 as u8,
    ],
    [
        0x44 as i32 as u8,
        0x33 as i32 as u8,
        0x24 as i32 as u8,
    ],
    [
        0x3f as i32 as u8,
        0x7f as i32 as u8,
        0x4e as i32 as u8,
    ],
    [
        0x77 as i32 as u8,
        0x6d as i32 as u8,
        0x50 as i32 as u8,
    ],
    [
        0x3b as i32 as u8,
        0x55 as i32 as u8,
        0x9 as i32 as u8,
    ],
    [
        0x55 as i32 as u8,
        0x53 as i32 as u8,
        0x56 as i32 as u8,
    ],
    [
        0x69 as i32 as u8,
        0x7c as i32 as u8,
        0x73 as i32 as u8,
    ],
    [
        0x61 as i32 as u8,
        0x3a as i32 as u8,
        0x35 as i32 as u8,
    ],
    [
        0x63 as i32 as u8,
        0x5f as i32 as u8,
        0x61 as i32 as u8,
    ],
    [
        0x50 as i32 as u8,
        0x65 as i32 as u8,
        0x4b as i32 as u8,
    ],
    [
        0x67 as i32 as u8,
        0x46 as i32 as u8,
        0x58 as i32 as u8,
    ],
    [
        0x51 as i32 as u8,
        0x58 as i32 as u8,
        0x3b as i32 as u8,
    ],
    [
        0x49 as i32 as u8,
        0x31 as i32 as u8,
        0x57 as i32 as u8,
    ],
    [
        0x4f as i32 as u8,
        0x69 as i32 as u8,
        0x22 as i32 as u8,
    ],
    [
        0x46 as i32 as u8,
        0x6c as i32 as u8,
        0x6d as i32 as u8,
    ],
    [
        0x68 as i32 as u8,
        0x5a as i32 as u8,
        0x4d as i32 as u8,
    ],
    [
        0x7c as i32 as u8,
        0x48 as i32 as u8,
        0x25 as i32 as u8,
    ],
    [
        0x36 as i32 as u8,
        0x27 as i32 as u8,
        0x28 as i32 as u8,
    ],
    [
        0x70 as i32 as u8,
        0x5c as i32 as u8,
        0x46 as i32 as u8,
    ],
    [
        0x6e as i32 as u8,
        0x3d as i32 as u8,
        0x4a as i32 as u8,
    ],
    [
        0x7a as i32 as u8,
        0x24 as i32 as u8,
        0x32 as i32 as u8,
    ],
    [
        0x2f as i32 as u8,
        0x79 as i32 as u8,
        0x41 as i32 as u8,
    ],
    [
        0x5f as i32 as u8,
        0x37 as i32 as u8,
        0x3d as i32 as u8,
    ],
    [
        0x4b as i32 as u8,
        0x60 as i32 as u8,
        0x5f as i32 as u8,
    ],
    [
        0x5a as i32 as u8,
        0x51 as i32 as u8,
        0x4f as i32 as u8,
    ],
    [
        0x2c as i32 as u8,
        0x20 as i32 as u8,
        0x42 as i32 as u8,
    ],
    [
        0x57 as i32 as u8,
        0x36 as i32 as u8,
        0x65 as i32 as u8,
    ],
];
static mut digits: [i32; 128] = [
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0x3e as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0x3f as i32,
    0x34 as i32,
    0x35 as i32,
    0x36 as i32,
    0x37 as i32,
    0x38 as i32,
    0x39 as i32,
    0x3a as i32,
    0x3b as i32,
    0x3c as i32,
    0x3d as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0 as i32,
    0x1 as i32,
    0x2 as i32,
    0x3 as i32,
    0x4 as i32,
    0x5 as i32,
    0x6 as i32,
    0x7 as i32,
    0x8 as i32,
    0x9 as i32,
    0xa as i32,
    0xb as i32,
    0xc as i32,
    0xd as i32,
    0xe as i32,
    0xf as i32,
    0x10 as i32,
    0x11 as i32,
    0x12 as i32,
    0x13 as i32,
    0x14 as i32,
    0x15 as i32,
    0x16 as i32,
    0x17 as i32,
    0x18 as i32,
    0x19 as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0x1a as i32,
    0x1b as i32,
    0x1c as i32,
    0x1d as i32,
    0x1e as i32,
    0x1f as i32,
    0x20 as i32,
    0x21 as i32,
    0x22 as i32,
    0x23 as i32,
    0x24 as i32,
    0x25 as i32,
    0x26 as i32,
    0x27 as i32,
    0x28 as i32,
    0x29 as i32,
    0x2a as i32,
    0x2b as i32,
    0x2c as i32,
    0x2d as i32,
    0x2e as i32,
    0x2f as i32,
    0x30 as i32,
    0x31 as i32,
    0x32 as i32,
    0x33 as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
    0xff as i32,
];
unsafe extern "C" fn decode_dword(mut p: *const WCHAR, mut ret: *mut DWORD) -> BOOL {
    let mut i: DWORD = 0;
    i = 0 as DWORD;
    while i < 6 as DWORD {
        if *p.offset(i as isize) as usize
            >= (::core::mem::size_of::<[i32; 128]>() as usize)
                .wrapping_div(::core::mem::size_of::<i32>() as usize)
            || digits[*p.offset(i as isize) as usize] == 0xff as i32
        {
            return FALSE;
        }
        i = i.wrapping_add(1);
    }
    if *p.offset(6 as i32 as isize) as i32 != '=' as i32
        || *p.offset(7 as i32 as isize) as i32 != '=' as i32
    {
        return FALSE;
    }
    *ret = ((digits[*p.offset(0 as i32 as isize) as usize]
        << 2 as i32)
        + (digits[*p.offset(1 as i32 as isize) as usize] >> 4 as i32)
        + ((digits[*p.offset(1 as i32 as isize) as usize]
            & 0xf as i32)
            << 12 as i32)
        + ((digits[*p.offset(2 as i32 as isize) as usize]
            >> 2 as i32)
            << 8 as i32)
        + ((digits[*p.offset(2 as i32 as isize) as usize]
            & 0x3 as i32)
            << 22 as i32)
        + (digits[*p.offset(3 as i32 as isize) as usize]
            << 16 as i32)
        + ((digits[*p.offset(4 as i32 as isize) as usize]
            << 2 as i32)
            << 24 as i32)
        + ((digits[*p.offset(5 as i32 as isize) as usize]
            >> 4 as i32)
            << 24 as i32)) as DWORD;
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn decode_source(mut code: *mut WCHAR) -> HRESULT {
    let mut src: *const WCHAR = code;
    let mut dst: *mut WCHAR = code;
    static mut decode_beginW: [WCHAR; 5] =
        unsafe { ::core::mem::transmute::<[u8; 10], [WCHAR; 5]>(*b"#\0@\0~\0^\0\0\0") };
    static mut decode_endW: [WCHAR; 5] =
        unsafe { ::core::mem::transmute::<[u8; 10], [WCHAR; 5]>(*b"^\0#\0~\0@\0\0\0") };
    while *src != 0 {
        if wcsncmp(
            src as *const wchar_t,
            &raw const decode_beginW as *const wchar_t,
            (::core::mem::size_of::<[WCHAR; 5]>() as size_t)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as size_t)
                .wrapping_sub(1 as size_t),
        ) == 0
        {
            let mut len: DWORD = 0;
            let mut i: DWORD = 0;
            let mut j: DWORD = 0 as DWORD;
            let mut csum: DWORD = 0;
            let mut s: DWORD = 0 as DWORD;
            src = src.offset(
                (::core::mem::size_of::<[WCHAR; 5]>() as usize)
                    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
            if decode_dword(src, &raw mut len) == 0 {
                return JS_E_INVALID_CHAR;
            }
            src = src.offset(8 as i32 as isize);
            i = 0 as DWORD;
            while i < len {
                if *src.offset(i as isize) as i32 == '@' as i32 {
                    i = i.wrapping_add(1);
                    match *src.offset(i as isize) as i32 {
                        35 => {
                            let fresh0 = j;
                            j = j.wrapping_add(1);
                            let ref mut fresh1 = *dst.offset(fresh0 as isize);
                            *fresh1 = '\r' as i32 as WCHAR;
                            s = s.wrapping_add(*fresh1 as DWORD);
                        }
                        38 => {
                            let fresh2 = j;
                            j = j.wrapping_add(1);
                            let ref mut fresh3 = *dst.offset(fresh2 as isize);
                            *fresh3 = '\n' as i32 as WCHAR;
                            s = s.wrapping_add(*fresh3 as DWORD);
                        }
                        33 => {
                            let fresh4 = j;
                            j = j.wrapping_add(1);
                            let ref mut fresh5 = *dst.offset(fresh4 as isize);
                            *fresh5 = '<' as i32 as WCHAR;
                            s = s.wrapping_add(*fresh5 as DWORD);
                        }
                        42 => {
                            let fresh6 = j;
                            j = j.wrapping_add(1);
                            let ref mut fresh7 = *dst.offset(fresh6 as isize);
                            *fresh7 = '>' as i32 as WCHAR;
                            s = s.wrapping_add(*fresh7 as DWORD);
                        }
                        36 => {
                            let fresh8 = j;
                            j = j.wrapping_add(1);
                            let ref mut fresh9 = *dst.offset(fresh8 as isize);
                            *fresh9 = '@' as i32 as WCHAR;
                            s = s.wrapping_add(*fresh9 as DWORD);
                        }
                        _ => {
                            if (*__wine_dbch___default).flags as i32
                                & (1 as i32)
                                    << __WINE_DBCL_FIXME as i32
                                != 0
                            {
                                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                                ros_dbg_log(
                                    __dbcl,
                                    __dbch,
                                    b"/home/curi/Projects/os-playground/vibeos-rustconv-speedrun/dll/win32/jscript/decode.c\0"
                                        as *const u8 as *const i8,
                                    b"decode_source\0" as *const u8
                                        as *const i8,
                                    149 as i32,
                                    b"unescape %c\n\0" as *const u8
                                        as *const i8,
                                    *src.offset(i as isize) as i32,
                                );
                            }
                            return E_FAIL;
                        }
                    }
                } else if (*src.offset(i as isize) as i32)
                    < 128 as i32
                {
                    let ref mut fresh10 = *dst.offset(j as isize);
                    *fresh10 = dictionary[*src.offset(i as isize) as usize]
                        [pick_encoding[j.wrapping_rem(64 as DWORD) as usize] as usize]
                        as WCHAR;
                    s = s.wrapping_add(*fresh10 as DWORD);
                    j = j.wrapping_add(1);
                } else {
                    if (*__wine_dbch___default).flags as i32
                        & (1 as i32) << __WINE_DBCL_FIXME as i32
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                        ros_dbg_log(
                            __dbcl_0,
                            __dbch_0,
                            b"/home/curi/Projects/os-playground/vibeos-rustconv-speedrun/dll/win32/jscript/decode.c\0"
                                as *const u8 as *const i8,
                            b"decode_source\0" as *const u8
                                as *const i8,
                            156 as i32,
                            b"Unsupported char %c\n\0" as *const u8
                                as *const i8,
                            *src.offset(i as isize) as i32,
                        );
                    }
                    return E_FAIL;
                }
                i = i.wrapping_add(1);
            }
            src = src.offset(len as isize);
            dst = dst.offset(j as isize);
            if decode_dword(src, &raw mut csum) == 0 || s != csum {
                return JS_E_INVALID_CHAR;
            }
            src = src.offset(8 as i32 as isize);
            if wcsncmp(
                src as *const wchar_t,
                &raw const decode_endW as *const wchar_t,
                (::core::mem::size_of::<[WCHAR; 5]>() as size_t)
                    .wrapping_div(::core::mem::size_of::<WCHAR>() as size_t)
                    .wrapping_sub(1 as size_t),
            ) != 0
            {
                return JS_E_INVALID_CHAR;
            }
            src = src.offset(
                (::core::mem::size_of::<[WCHAR; 5]>() as usize)
                    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
        } else {
            let fresh11 = src;
            src = src.offset(1);
            let fresh12 = dst;
            dst = dst.offset(1);
            *fresh12 = *fresh11;
        }
    }
    *dst = 0 as WCHAR;
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"/home/curi/Projects/os-playground/vibeos-rustconv-speedrun/dll/win32/jscript/decode.c\0"
                as *const u8 as *const i8,
            b"decode_source\0" as *const u8 as *const i8,
            178 as i32,
            b"decoded %s\n\0" as *const u8 as *const i8,
            debugstr_w(code),
        );
    }
    return S_OK;
}
