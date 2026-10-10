use crate::progman::dialog::DIALOG_Delete;
use crate::progman::dialog::DIALOG_Execute;
use crate::progman::dialog::DIALOG_New;
use crate::progman::group::GROUP_ActiveGroup;
use crate::progman::group::GROUP_DeleteGroup;
use crate::progman::group::GROUP_ModifyGroup;
use crate::progman::group::GROUP_NewGroup;
use crate::progman::group::GROUP_RegisterGroupWinClass;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_01E67B48F30C: [u16; 12] = [
    0x0070u16, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x002E, 0x0069, 0x006E,
    0x0069, 0x0000,
];
pub static RUSTCONV_WSTR_19E698501EA1: [u16; 15] = [
    0x0050u16, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x0047, 0x0072,
    0x006F, 0x0075, 0x0070, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_28E3E2787D78: [u16; 12] = [
    0x0049u16, 0x006E, 0x0069, 0x004D, 0x0069, 0x0067, 0x0072, 0x0061, 0x0074, 0x0065,
    0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_2E5F286A7F7B: [u16; 15] = [
    0x0025u16, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025,
    0x0064, 0x0020, 0x0025, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_4B164B324B2E: [u16; 7] = [0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0000];
pub static RUSTCONV_WSTR_50A482475871: [u16; 9] = [0x004Du16, 0x0069, 0x006E, 0x004F, 0x006E, 0x0052, 0x0075, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_5BD156509380: [u16; 8] = [0x0025u16, 0x0073, 0x003A, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_68E65666D358: [u16; 23] = [
    0x0055u16, 0x004E, 0x0049, 0x0043, 0x004F, 0x0044, 0x0045, 0x0020, 0x0050, 0x0072,
    0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x0047, 0x0072, 0x006F, 0x0075,
    0x0070, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_6C49BEACD7CB: [u16; 6] = [0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_775E717CBCE2: [u16; 11] = [
    0x0025u16, 0x0073, 0x0020, 0x002D, 0x0020, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_7D4E750BDB23: [u16; 24] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0050,
    0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x0047, 0x0072, 0x006F,
    0x0075, 0x0070, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_8081465998AC: [u16; 8] = [0x0053u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0075, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_B47EBFED6E0E: [u16; 12] = [
    0x0070u16, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x002E, 0x0068, 0x006C,
    0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_BA35364C3243: [u16; 806] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061,
    0x006D, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020,
    0x0068, 0x0061, 0x0073, 0x0020, 0x0064, 0x0065, 0x0074, 0x0065, 0x0063, 0x0074,
    0x0065, 0x0064, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0070, 0x0072, 0x0065,
    0x0073, 0x0065, 0x006E, 0x0063, 0x0065, 0x0020, 0x006F, 0x0066, 0x0020, 0x0061,
    0x0020, 0x006C, 0x0065, 0x0067, 0x0061, 0x0063, 0x0079, 0x0020, 0x0073, 0x0065,
    0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0020, 0x0066, 0x0069, 0x006C,
    0x0065, 0x0020, 0x0050, 0x0052, 0x004F, 0x0047, 0x004D, 0x0041, 0x004E, 0x002E,
    0x0049, 0x004E, 0x0049, 0x0020, 0x0069, 0x006E, 0x0020, 0x0074, 0x0068, 0x0065,
    0x0020, 0x0064, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079,
    0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x0020, 0x0061, 0x006E, 0x0064, 0x0020,
    0x0069, 0x0073, 0x0020, 0x0067, 0x006F, 0x0069, 0x006E, 0x0067, 0x0020, 0x0074,
    0x006F, 0x0020, 0x006D, 0x0069, 0x0067, 0x0072, 0x0061, 0x0074, 0x0065, 0x0020,
    0x0069, 0x0074, 0x0073, 0x0020, 0x0063, 0x006F, 0x006E, 0x0074, 0x0065, 0x006E,
    0x0074, 0x0073, 0x0020, 0x0069, 0x006E, 0x0074, 0x006F, 0x0020, 0x0074, 0x0068,
    0x0065, 0x0020, 0x0063, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x002D,
    0x0075, 0x0073, 0x0065, 0x0072, 0x0020, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072,
    0x0061, 0x006D, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072,
    0x0020, 0x0073, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0020,
    0x0072, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x006B,
    0x0065, 0x0079, 0x003A, 0x000A, 0x0048, 0x004B, 0x0043, 0x0055, 0x005C, 0x0053,
    0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069,
    0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069,
    0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C, 0x0043,
    0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073,
    0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061,
    0x006D, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x000A,
    0x000A, 0x2022, 0x0020, 0x0054, 0x0068, 0x0065, 0x0020, 0x006D, 0x0069, 0x0067,
    0x0072, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x006F, 0x0070, 0x0065,
    0x0072, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0077, 0x0069, 0x006C,
    0x006C, 0x0020, 0x0070, 0x006F, 0x0074, 0x0065, 0x006E, 0x0074, 0x0069, 0x0061,
    0x006C, 0x006C, 0x0079, 0x0020, 0x006F, 0x0076, 0x0065, 0x0072, 0x0077, 0x0072,
    0x0069, 0x0074, 0x0065, 0x0020, 0x0061, 0x006C, 0x006C, 0x0020, 0x0074, 0x0068,
    0x0065, 0x0020, 0x0065, 0x0078, 0x0069, 0x0073, 0x0074, 0x0069, 0x006E, 0x0067,
    0x0020, 0x0063, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x002D, 0x0075,
    0x0073, 0x0065, 0x0072, 0x0020, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061,
    0x006D, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020,
    0x0073, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0020, 0x0069,
    0x006E, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0072, 0x0065, 0x0067, 0x0069,
    0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x0062, 0x0079, 0x0020, 0x0074, 0x0068,
    0x006F, 0x0073, 0x0065, 0x0020, 0x0073, 0x0074, 0x006F, 0x0072, 0x0065, 0x0064,
    0x0020, 0x0069, 0x006E, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0050, 0x0052,
    0x004F, 0x0047, 0x004D, 0x0041, 0x004E, 0x002E, 0x0049, 0x004E, 0x0049, 0x0020,
    0x0066, 0x0069, 0x006C, 0x0065, 0x002E, 0x000A, 0x000A, 0x2022, 0x0020, 0x0054,
    0x0068, 0x0065, 0x0020, 0x006D, 0x0069, 0x0067, 0x0072, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020, 0x0064, 0x006F, 0x006E, 0x0065,
    0x0020, 0x006F, 0x006E, 0x0063, 0x0065, 0x002C, 0x0020, 0x0073, 0x006F, 0x0020,
    0x0074, 0x0068, 0x0061, 0x0074, 0x002C, 0x0020, 0x0061, 0x0074, 0x0020, 0x0074,
    0x0068, 0x0065, 0x0020, 0x006E, 0x0065, 0x0078, 0x0074, 0x0020, 0x006C, 0x0061,
    0x0075, 0x006E, 0x0063, 0x0068, 0x0020, 0x006F, 0x0066, 0x0020, 0x0074, 0x0068,
    0x0065, 0x0020, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020,
    0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x002C, 0x0020, 0x0074,
    0x0068, 0x0065, 0x0020, 0x006E, 0x0065, 0x0077, 0x0020, 0x006D, 0x0069, 0x0067,
    0x0072, 0x0061, 0x0074, 0x0065, 0x0064, 0x0020, 0x0073, 0x0065, 0x0074, 0x0074,
    0x0069, 0x006E, 0x0067, 0x0073, 0x0020, 0x0061, 0x0072, 0x0065, 0x0020, 0x0064,
    0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006C, 0x0079, 0x0020, 0x0075, 0x0073,
    0x0065, 0x0064, 0x002E, 0x000A, 0x000A, 0x2022, 0x0020, 0x0049, 0x0074, 0x0020,
    0x0069, 0x0073, 0x0020, 0x0070, 0x006F, 0x0073, 0x0073, 0x0069, 0x0062, 0x006C,
    0x0065, 0x0020, 0x0074, 0x006F, 0x0020, 0x0074, 0x0072, 0x0069, 0x0067, 0x0067,
    0x0065, 0x0072, 0x0020, 0x006C, 0x0061, 0x0074, 0x0065, 0x0072, 0x0020, 0x0074,
    0x0068, 0x0065, 0x0020, 0x006D, 0x0069, 0x0067, 0x0072, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0020, 0x0062, 0x0079, 0x0020, 0x006D, 0x0061, 0x006E, 0x0075,
    0x0061, 0x006C, 0x006C, 0x0079, 0x0020, 0x0064, 0x0065, 0x006C, 0x0065, 0x0074,
    0x0069, 0x006E, 0x0067, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0072, 0x0065,
    0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x0076, 0x0061, 0x006C,
    0x0075, 0x0065, 0x0020, 0x0022, 0x0049, 0x006E, 0x0069, 0x004D, 0x0069, 0x0067,
    0x0072, 0x0061, 0x0074, 0x0065, 0x0064, 0x0022, 0x0020, 0x0075, 0x006E, 0x0064,
    0x0065, 0x0072, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0063, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x002D, 0x0075, 0x0073, 0x0065, 0x0072, 0x0020,
    0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x004D, 0x0061,
    0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020, 0x0073, 0x0065, 0x0074, 0x0074,
    0x0069, 0x006E, 0x0067, 0x0073, 0x0020, 0x0072, 0x0065, 0x0067, 0x0069, 0x0073,
    0x0074, 0x0072, 0x0079, 0x0020, 0x006B, 0x0065, 0x0079, 0x0020, 0x0028, 0x0073,
    0x0070, 0x0065, 0x0063, 0x0069, 0x0066, 0x0069, 0x0065, 0x0064, 0x0020, 0x0061,
    0x0062, 0x006F, 0x0076, 0x0065, 0x0029, 0x002E, 0x000A, 0x000A, 0x0057, 0x006F,
    0x0075, 0x006C, 0x0064, 0x0020, 0x0079, 0x006F, 0x0075, 0x0020, 0x006C, 0x0069,
    0x006B, 0x0065, 0x0020, 0x0074, 0x006F, 0x0020, 0x006D, 0x0069, 0x0067, 0x0072,
    0x0061, 0x0074, 0x0065, 0x0020, 0x0069, 0x0074, 0x0073, 0x0020, 0x0063, 0x006F,
    0x006E, 0x0074, 0x0065, 0x006E, 0x0074, 0x0073, 0x0020, 0x0069, 0x006E, 0x0074,
    0x006F, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0072, 0x0065, 0x0067, 0x0069,
    0x0073, 0x0074, 0x0072, 0x0079, 0x003F, 0x0000,
];
pub static RUSTCONV_WSTR_CB30580B1EBE: [u16; 12] = [
    0x0049u16, 0x006E, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_D08E1D572161: [u16; 14] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0025, 0x0064, 0x003A, 0x0020,
    0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D1046FF7D861: [u16; 13] = [
    0x0053u16, 0x0061, 0x0076, 0x0065, 0x0053, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E,
    0x0067, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_DFB671E68C51: [u16; 15] = [
    0x0055u16, 0x004E, 0x0049, 0x0043, 0x004F, 0x0044, 0x0045, 0x0020, 0x0047, 0x0072,
    0x006F, 0x0075, 0x0070, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_E961535A4B2A: [u16; 61] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072,
    0x0061, 0x006D, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072,
    0x0000,
];
pub static RUSTCONV_WSTR_F54ABF80FBC0: [u16; 7] = [0x0047u16, 0x0072, 0x006F, 0x0075, 0x0070, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_F87194ABC5EC: [u16; 14] = [
    0x0043u16, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0047, 0x0072, 0x006F,
    0x0075, 0x0070, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_FE1F48BE2065: [u16; 12] = [
    0x0041u16, 0x0075, 0x0074, 0x006F, 0x0041, 0x0072, 0x0072, 0x0061, 0x006E, 0x0067,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_FE9467C4B48F: [u16; 9] = [0x0053u16, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const STRING_MAIN_WIN_CLASS_NAME: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"P\0M\0M\0a\0i\0n\0\0\0")
};

#[no_mangle]
pub static mut Globals: GLOBALS = _GLOBALS {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    hAccel: ::core::ptr::null::<HACCEL__>() as *mut HACCEL__,
    hMainWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hMDIWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hDefaultIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    hMainIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    hPersonalGroupIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    hCommonGroupIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    hMainMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    hFileMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    hOptionMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    hWindowsMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    hLanguageMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    hKeyProgMan: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyPMSettings: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyPMCommonGroups: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyPMAnsiGroups: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyPMUnicodeGroups: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyAnsiGroups: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyUnicodeGroups: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    hKeyCommonGroups: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
    bAutoArrange: 0,
    bSaveSettings: 0,
    bMinOnRun: 0,
    hGroups: ::core::ptr::null::<PROGGROUP>() as *mut PROGGROUP,
    hActiveGroup: ::core::ptr::null::<PROGGROUP>() as *mut PROGGROUP,
};

pub const WC_MDICLIENTA: *const i8 = b"MDICLIENT\0".as_ptr() as *const i8;

pub const WC_MDICLIENTW: *const u16 = {
    const BUF: &[u16; 11] = &[77, 68, 73, 67, 76, 73, 69, 78, 84, 0, 0];
    BUF.as_ptr()
};

pub const WC_MDICLIENT: *const u16 = WC_MDICLIENTW;

pub const BUFFER_SIZE: i32 = 1024; // nolint: const-contradiction (progman main.c's own #define)
#[no_mangle]
pub unsafe extern "C" fn Alloc(dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID {
    return HeapAlloc(GetProcessHeap(), dwFlags, dwBytes);
}
#[no_mangle]
pub unsafe extern "C" fn Free(lpMem: PVOID) -> BOOL {
    return HeapFree(GetProcessHeap(), 0, lpMem);
}
#[no_mangle]
pub unsafe extern "C" fn ReAlloc(
    mut dwFlags: DWORD,
    mut lpMem: PVOID,
    mut dwBytes: SIZE_T,
) -> PVOID {
    return HeapReAlloc(GetProcessHeap(), dwFlags, lpMem, dwBytes);
}

#[no_mangle]
pub unsafe extern "C" fn AppendToBuffer(
    pBuffer: PVOID,
    pdwBufferSize: PSIZE_T,
    pData: PVOID,
    dwDataSize: SIZE_T,
) -> PVOID {
    let mut pTmp: PVOID = core::ptr::null_mut();
    let mut dwBufferSize: SIZE_T;

    dwBufferSize = (dwDataSize as ULONG_PTR).wrapping_add(*pdwBufferSize as ULONG_PTR) as SIZE_T;

    if !pBuffer.is_null() {
        pTmp = ReAlloc(0, pBuffer, dwBufferSize);
    } else {
        pTmp = Alloc(0, dwBufferSize);
    }

    if pTmp.is_null() {
        return core::ptr::null_mut();
    }

    memcpy(
        (pTmp as ULONG_PTR).wrapping_add(*pdwBufferSize as ULONG_PTR) as PVOID,
        pData as *const core::ffi::c_void,
        dwDataSize as usize,
    );
    *pdwBufferSize = dwBufferSize;

    return pTmp;
}
#[no_mangle]
pub unsafe extern "C" fn PrintStringV(szStr: LPCWSTR, args: va_list) {
    let mut Buffer: [WCHAR; 4096] = [0; 4096];

    _vsnwprintf(addr_of_mut!(Buffer) as *mut WCHAR, ARRAYSIZE!(Buffer) as usize, szStr, args);
    MessageBoxW(Globals.hMainWnd, addr_of_mut!(Buffer) as LPCWSTR, RUSTCONV_WSTR_CB30580B1EBE.as_ptr(), MB_OK as UINT);
}
#[no_mangle]
pub unsafe extern "C" fn PrintString_v(szStr: LPCWSTR, ap_list: *mut ::core::ffi::c_void) {
    let args_va: va_list = *(ap_list as *mut va_list);
    PrintStringV(szStr, args_va);
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `PrintString_v` above.
    pub fn PrintString(szStr: LPCWSTR, ...);
}

#[no_mangle]
pub unsafe extern "C" fn PrintResourceString_v(mut uID: UINT, ap_list: *mut ::core::ffi::c_void) {
    let mut Buffer: [WCHAR; 4096] = [0; 4096];
    let args_0: va_list = *(ap_list as *mut va_list);
    LoadStringW(
        Globals.hInstance,
        uID,
        core::ptr::addr_of_mut!(Buffer) as LPWSTR,
        ARRAYSIZE!(Buffer) as i32,
    );
    PrintStringV(core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPCWSTR, args_0);
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `PrintResourceString_v` above.
    pub fn PrintResourceString(uID: UINT, ...);
}

#[no_mangle]
pub unsafe extern "C" fn PrintWin32Error(mut Message: LPWSTR, mut ErrorCode: DWORD) {
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();

    FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD,
        core::ptr::null(),
        ErrorCode,
        MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
        core::ptr::addr_of_mut!(lpMsgBuf) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    );

    PrintString(RUSTCONV_WSTR_5BD156509380.as_ptr() as LPCWSTR, Message, lpMsgBuf);
    LocalFree(lpMsgBuf as HLOCAL);
}
#[no_mangle]
pub unsafe extern "C" fn ShowLastWin32Error() -> ::core::ffi::c_int {
    let mut dwError: DWORD = 0;
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();
    let mut Buffer: [WCHAR; 4096] = [0; 4096];

    dwError = GetLastError();

    FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD,
        core::ptr::null(),
        dwError,
        MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
        core::ptr::addr_of_mut!(lpMsgBuf) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    );
    _snwprintf(
        core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
        ARRAYSIZE!(Buffer) as usize,
        RUSTCONV_WSTR_D08E1D572161.as_ptr(),
        dwError,
        lpMsgBuf,
    );
    LocalFree(lpMsgBuf as HLOCAL);
    return MessageBoxW(
        Globals.hMainWnd,
        core::ptr::addr_of!(Buffer) as LPCWSTR,
        RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(),
        MB_OK as UINT,
    );
}

unsafe extern "C" fn GetUserAndDomainName(
    UserName: *mut LPWSTR,
    DomainName: *mut LPWSTR,
) -> BOOL {
    let mut bRet: BOOL = TRUE as BOOL;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut cbTokenBuffer: DWORD = 0;
    let mut pUserToken: PTOKEN_USER = core::ptr::null_mut();

    let mut lpUserName: LPWSTR = core::ptr::null_mut();
    let mut lpDomainName: LPWSTR = core::ptr::null_mut();
    let mut cbUserName: DWORD = 0;
    let mut cbDomainName: DWORD = 0;

    let mut SidNameUse: SID_NAME_USE = core::mem::zeroed();

    /* Get the process token */
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, core::ptr::addr_of_mut!(hToken)) == 0 {
        return FALSE as BOOL;
    }

    /* Retrieve token's information */
    if GetTokenInformation(hToken, TokenUser, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(cbTokenBuffer)) == 0 {
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            CloseHandle(hToken);
            return FALSE as BOOL;
        }
    }

    pUserToken = Alloc(HEAP_ZERO_MEMORY, cbTokenBuffer as SIZE_T) as PTOKEN_USER;
    if pUserToken.is_null() {
        CloseHandle(hToken);
        return FALSE as BOOL;
    }

    if GetTokenInformation(hToken, TokenUser, pUserToken as PVOID, cbTokenBuffer, core::ptr::addr_of_mut!(cbTokenBuffer)) == 0 {
        Free(pUserToken as PVOID);
        CloseHandle(hToken);
        return FALSE as BOOL;
    }

    CloseHandle(hToken);

    'done: {
        /* Retrieve the domain and user name */
        if LookupAccountSidW(
            core::ptr::null::<WCHAR>(),
            (*pUserToken).User.Sid,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::addr_of_mut!(cbUserName),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::addr_of_mut!(cbDomainName),
            core::ptr::addr_of_mut!(SidNameUse),
        ) == 0
        {
            if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                bRet = FALSE as BOOL;
                break 'done;
            }
        }

        lpUserName = Alloc(
            HEAP_ZERO_MEMORY,
            (cbUserName as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if lpUserName.is_null() {
            bRet = FALSE as BOOL;
            break 'done;
        }

        lpDomainName = Alloc(
            HEAP_ZERO_MEMORY,
            (cbDomainName as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if lpDomainName.is_null() {
            bRet = FALSE as BOOL;
            break 'done;
        }

        if LookupAccountSidW(
            core::ptr::null::<WCHAR>(),
            (*pUserToken).User.Sid,
            lpUserName,
            core::ptr::addr_of_mut!(cbUserName),
            lpDomainName,
            core::ptr::addr_of_mut!(cbDomainName),
            core::ptr::addr_of_mut!(SidNameUse),
        ) == 0
        {
            bRet = FALSE as BOOL;
            break 'done;
        }

        *UserName = lpUserName;
        *DomainName = lpDomainName;
    }

    /* done: */
    if bRet == FALSE as BOOL {
        if lpUserName != core::ptr::null_mut() {
            Free(lpUserName as PVOID);
        }

        if lpDomainName != core::ptr::null_mut() {
            Free(lpDomainName as PVOID);
        }
    }

    Free(pUserToken as PVOID);

    return bRet;
}

unsafe extern "C" fn MAIN_SetMainWindowTitle() {
    let mut caption: LPWSTR = core::ptr::null_mut();
    let mut size: SIZE_T;

    let mut lpDomainName: LPWSTR = core::ptr::null_mut();
    let mut lpUserName: LPWSTR = core::ptr::null_mut();

    if GetUserAndDomainName(core::ptr::addr_of_mut!(lpUserName), core::ptr::addr_of_mut!(lpDomainName)) != 0
        && !lpUserName.is_null()
        && !lpDomainName.is_null()
    {
        size = ((256 as SIZE_T)
            + (3 as SIZE_T)
            + (wcslen(lpDomainName as *const u16) as SIZE_T)
            + (wcslen(lpUserName as *const u16) as SIZE_T)
            + (1 as SIZE_T))
            * (core::mem::size_of::<WCHAR>() as SIZE_T);
        caption = Alloc(HEAP_ZERO_MEMORY as DWORD, size) as LPWSTR;
        if !caption.is_null() {
            StringCbPrintfW!(
                caption,
                size as usize,
                RUSTCONV_WSTR_775E717CBCE2.as_ptr(),
                core::ptr::addr_of_mut!(szTitle) as *mut WCHAR,
                lpDomainName,
                lpUserName,
            );
            SetWindowTextW(Globals.hMainWnd, caption as LPCWSTR);
            Free(caption as PVOID);
        } else {
            SetWindowTextW(Globals.hMainWnd, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR);
        }
    } else {
        SetWindowTextW(Globals.hMainWnd, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR);
    }

    if !lpUserName.is_null() {
        Free(lpUserName as PVOID);
    }
    if !lpDomainName.is_null() {
        Free(lpDomainName as PVOID);
    }
}

unsafe extern "C" fn MAIN_LoadSettings() -> BOOL {
    let mut lpszTmp: LPWSTR = core::ptr::null_mut();
    let mut lpszSection: LPWSTR = core::ptr::null_mut();
    let mut lRet: LONG = 0;
    let mut dummy: [WCHAR; 2] = [0; 2];
    let mut lpszKeyValue: LPWSTR = core::ptr::null_mut();
    let lpszIniFile: LPCWSTR = RUSTCONV_WSTR_01E67B48F30C.as_ptr();
    let mut szWinDir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut lpszKey: LPWSTR = core::ptr::null_mut();
    let mut Value: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut bIsIniMigrated: BOOL = 0;
    let mut dwSize: DWORD = 0;
    let mut lpszSections: LPWSTR = core::ptr::null_mut();
    let mut lpszData: LPWSTR = core::ptr::null_mut();
    let mut dwRet: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut lpszValue: LPWSTR = core::ptr::null_mut();

    bIsIniMigrated = FALSE as BOOL;
    lpszSections = core::ptr::null_mut();
    lpszData = core::ptr::null_mut();

    /* Try to create/open the Program Manager user key */
    if RegCreateKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_E961535A4B2A.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyProgMan),
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS as LONG
    {
        return FALSE as BOOL;
    }

    /*
     * TODO: Add the explanation for the migration...
     */
    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lRet = RegQueryValueExW(
        Globals.hKeyProgMan,
        RUSTCONV_WSTR_28E3E2787D78.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(Value) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    if lRet != ERROR_SUCCESS as LONG || dwType != REG_DWORD {
        Value = 0;
    }
    bIsIniMigrated = (Value != 0) as BOOL;

    if bIsIniMigrated == 0 {
        /* Perform the migration */
        'migdone: {
            bIsIniMigrated = TRUE as BOOL;
            dwSize = ARRAYSIZE!(dummy) as DWORD;
            SetLastError(0);
            GetPrivateProfileSectionW(
                RUSTCONV_WSTR_FE9467C4B48F.as_ptr(),
                core::ptr::addr_of_mut!(dummy) as LPWSTR,
                dwSize,
                lpszIniFile,
            );
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                break 'migdone;
            }

            SetLastError(0);
            GetPrivateProfileSectionW(
                RUSTCONV_WSTR_F54ABF80FBC0.as_ptr(),
                core::ptr::addr_of_mut!(dummy) as LPWSTR,
                dwSize,
                lpszIniFile,
            );
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                break 'migdone;
            }

            GetWindowsDirectoryW(
                core::ptr::addr_of_mut!(szWinDir) as LPWSTR,
                ARRAYSIZE!(szWinDir) as UINT,
            );
            // NOTE: GCC complains we cannot use the "\u2022" (UNICODE Code Point) notation for specifying the bullet character,
            // because it's only available in C++ or in C99. On the contrary MSVC is fine with it.
            // Instead we use a hex specification for the character: "\x2022".
            // Note also that the character "\x07" gives also a bullet, but a larger one.
            PrintString(
                RUSTCONV_WSTR_BA35364C3243.as_ptr(),
                core::ptr::addr_of_mut!(szWinDir) as LPWSTR,
            );

            dwSize = BUFFER_SIZE as DWORD;
            loop {
                lpszSections = Alloc(
                    0,
                    (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as LPWSTR;
                dwRet = GetPrivateProfileSectionNamesW(lpszSections, dwSize, lpszIniFile);
                if dwRet < dwSize.wrapping_sub(2) {
                    break;
                }
                Free(lpszSections as PVOID);
                dwSize = dwSize.wrapping_add(BUFFER_SIZE as DWORD);
            }

            lpszSection = lpszSections;
            while *lpszSection != 0 {
                lRet = RegCreateKeyExW(
                    Globals.hKeyProgMan,
                    lpszSection as LPCWSTR,
                    0,
                    core::ptr::null_mut(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_WRITE,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(hKey),
                    core::ptr::null_mut(),
                );
                if lRet == ERROR_SUCCESS as LONG {
                    dwSize = BUFFER_SIZE as DWORD;
                    loop {
                        lpszData = Alloc(
                            0,
                            (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                        ) as LPWSTR;
                        dwRet = GetPrivateProfileSectionW(
                            lpszSection as LPCWSTR,
                            lpszData,
                            dwSize,
                            lpszIniFile,
                        );
                        if dwRet < dwSize.wrapping_sub(2) {
                            break;
                        }
                        Free(lpszData as PVOID);
                        dwSize = dwSize.wrapping_add(BUFFER_SIZE as DWORD);
                    }

                    lpszKeyValue = lpszData;
                    while *lpszKeyValue != 0 {
                        lpszKey = lpszKeyValue;
                        lpszValue = wcschr(
                            lpszKeyValue as LPCWSTR,
                            0x003Du16,
                        ) as LPWSTR;
                        lpszKeyValue = lpszKeyValue.add(
                            (wcslen(lpszKeyValue as LPCWSTR)
                                + 1 as usize),
                        );
                        if !lpszValue.is_null() {
                            *lpszValue = 0;
                            lpszValue = lpszValue.add(1);
                            Value = wcstoul(
                                lpszValue as LPCWSTR,
                                core::ptr::addr_of_mut!(lpszTmp),
                                0,
                            ) as DWORD;
                            if (lpszTmp.offset_from(lpszValue) as usize)
                                >= wcslen(lpszValue as LPCWSTR)
                            {
                                lpszValue = core::ptr::addr_of_mut!(Value) as LPWSTR;
                                dwSize = core::mem::size_of::<DWORD>() as DWORD;
                                dwType = REG_DWORD;
                            } else {
                                dwSize = (wcslen(lpszValue as LPCWSTR)
                                    * core::mem::size_of::<WCHAR>()) as DWORD;
                                dwType = REG_SZ;
                            }
                        } else {
                            dwSize = 0;
                            dwType = REG_DWORD;
                        }
                        lRet = RegSetValueExW(
                            hKey,
                            lpszKey as LPCWSTR,
                            0,
                            dwType,
                            lpszValue as LPBYTE,
                            dwSize,
                        );
                    }
                    Free(lpszData as PVOID);
                    RegCloseKey(hKey);
                    lpszSection = lpszSection.add(
                        (wcslen(lpszSection as LPCWSTR)
                            + 1 as usize),
                    );
                }
            }
            Free(lpszSections as PVOID);
        }

        /* MigrationDone: */
        Value = TRUE;
        RegSetValueExW(
            Globals.hKeyProgMan,
            RUSTCONV_WSTR_28E3E2787D78.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Value) as LPBYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );
    }

    /* LoadSettings: */
    /* Create the necessary registry keys for the Program Manager and load its settings from the registry */

    lRet = RegCreateKeyExW(
        Globals.hKeyProgMan,
        RUSTCONV_WSTR_FE9467C4B48F.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyPMSettings),
        core::ptr::null_mut(),
    );

    lRet = RegCreateKeyExW(
        Globals.hKeyProgMan,
        RUSTCONV_WSTR_F87194ABC5EC.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyPMCommonGroups),
        core::ptr::null_mut(),
    );

    lRet = RegCreateKeyExW(
        Globals.hKeyProgMan,
        RUSTCONV_WSTR_F54ABF80FBC0.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyPMAnsiGroups),
        core::ptr::null_mut(),
    );

    lRet = RegCreateKeyExW(
        Globals.hKeyProgMan,
        RUSTCONV_WSTR_DFB671E68C51.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyPMUnicodeGroups),
        core::ptr::null_mut(),
    );

    lRet = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_19E698501EA1.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyAnsiGroups),
        core::ptr::null_mut(),
    );

    lRet = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_68E65666D358.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyUnicodeGroups),
        core::ptr::null_mut(),
    );

    lRet = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_7D4E750BDB23.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Globals.hKeyCommonGroups),
        core::ptr::null_mut(),
    );

    dwSize = core::mem::size_of::<BOOL>() as DWORD;
    RegQueryValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_FE1F48BE2065.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(Globals.bAutoArrange) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );

    dwSize = core::mem::size_of::<BOOL>() as DWORD;
    RegQueryValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_50A482475871.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(Globals.bMinOnRun) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );

    dwSize = core::mem::size_of::<BOOL>() as DWORD;
    RegQueryValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_D1046FF7D861.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(Globals.bSaveSettings) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );

    return TRUE as BOOL;
}

unsafe extern "C" fn MAIN_SaveSettings() -> BOOL {
    let mut WndPl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut dwSize: DWORD = 0;
    let mut buffer: [WCHAR; 100] = [0; 100];

    WndPl.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    GetWindowPlacement(Globals.hMainWnd, &raw mut WndPl);
    StringCbPrintfW!(
        &raw mut buffer as *mut WCHAR,
        core::mem::size_of::<[WCHAR; 100]>() as usize,
        RUSTCONV_WSTR_2E5F286A7F7B.as_ptr(),
        WndPl.rcNormalPosition.left,
        WndPl.rcNormalPosition.top,
        WndPl.rcNormalPosition.right,
        WndPl.rcNormalPosition.bottom,
        WndPl.showCmd,
    );

    dwSize = (wcslen(&raw mut buffer as *mut WCHAR) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    RegSetValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_4B164B324B2E.as_ptr(),
        0 as DWORD,
        REG_SZ as DWORD,
        &raw mut buffer as *mut WCHAR as LPBYTE,
        dwSize,
    );

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInstance: HINSTANCE, lpCmdLine: LPWSTR, nCmdShow: INT) -> INT {
    let mut msg: MSG = core::mem::zeroed();
    let mut icex: INITCOMMONCONTROLSEX = core::mem::zeroed();

    /*
     * Set our shutdown parameters: we want to shutdown the very last,
     * but before any TaskMgr instance (which has a shutdown level of 1).
     */
    SetProcessShutdownParameters(2, 0);

    Globals.hInstance = hInstance;
    Globals.hGroups = core::ptr::null_mut();
    Globals.hActiveGroup = core::ptr::null_mut();

    /* Load Program Manager's settings */
    MAIN_LoadSettings();

    /* Load the default icons */
    Globals.hDefaultIcon = LoadIconW(core::ptr::null_mut(), MAKEINTRESOURCEW!(IDI_WINLOGO));
    Globals.hMainIcon = LoadIconW(Globals.hInstance, MAKEINTRESOURCEW!(IDI_APPICON));
    Globals.hPersonalGroupIcon = LoadIconW(Globals.hInstance, MAKEINTRESOURCEW!(IDI_GROUP_PERSONAL_ICON));
    Globals.hCommonGroupIcon = LoadIconW(Globals.hInstance, MAKEINTRESOURCEW!(IDI_GROUP_COMMON_ICON));

    /* Initialize the common controls */
    icex.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    icex.dwICC = (ICC_HOTKEY_CLASS | ICC_LISTVIEW_CLASSES) as DWORD; // | ICC_STANDARD_CLASSES;
    InitCommonControlsEx(core::ptr::addr_of_mut!(icex));

    'quit: {
        /* Register the window classes */
        if hPrevInstance.is_null() // FIXME: Unused on Win32!
        {
            if MAIN_RegisterMainWinClass() == 0 {
                break 'quit;
            }
            if GROUP_RegisterGroupWinClass() == 0 {
                break 'quit;
            }
        }

        /* Set up the strings, the main window, the accelerators, the menu, and the MDI child window */
        STRING_LoadStrings();
        MAIN_CreateMainWindow();
        Globals.hAccel = LoadAcceleratorsW(Globals.hInstance, MAKEINTRESOURCEW!(IDA_ACCEL));
        STRING_LoadMenus();
        MAIN_CreateMDIWindow();

        /* Load all the groups */
        // MAIN_CreateGroups();
        MAIN_LoadGroups();

        /* Load the Startup group: start the initial applications */
        MAIN_AutoStart();

        /* Message loop */
        while GetMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0
        {
            if TranslateMDISysAccel(Globals.hMDIWnd, core::ptr::addr_of_mut!(msg)) == 0
                && TranslateAcceleratorW(Globals.hMainWnd, Globals.hAccel, core::ptr::addr_of_mut!(msg)) == 0
            {
                TranslateMessage(core::ptr::addr_of_mut!(msg));
                DispatchMessageW(core::ptr::addr_of_mut!(msg));
            }
        }
    }

    /* Save the settings, close the registry keys and quit */

    // MAIN_SaveSettings();
    RegCloseKey(Globals.hKeyCommonGroups);
    RegCloseKey(Globals.hKeyUnicodeGroups);
    RegCloseKey(Globals.hKeyAnsiGroups);
    RegCloseKey(Globals.hKeyPMUnicodeGroups);
    RegCloseKey(Globals.hKeyPMAnsiGroups);
    RegCloseKey(Globals.hKeyPMCommonGroups);
    RegCloseKey(Globals.hKeyPMSettings);
    RegCloseKey(Globals.hKeyProgMan);

    return 0;
}

unsafe extern "C" fn MAIN_LoadGroups() {}

unsafe extern "C" fn MAIN_AutoStart() {
    let mut lRet: LONG;
    let mut dwSize: DWORD;
    let mut dwType: DWORD = core::mem::zeroed();

    let mut hGroup: *mut PROGGROUP;
    let mut hProgram: *mut PROGRAM;

    let mut buffer: [WCHAR; BUFFER_SIZE as usize] = [0; BUFFER_SIZE as usize];

    dwSize = core::mem::size_of::<[WCHAR; BUFFER_SIZE as usize]>() as DWORD;
    lRet = RegQueryValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_8081465998AC.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) as LONG;
    if lRet != ERROR_SUCCESS as LONG || dwType != REG_SZ as DWORD {
        return;
    }

    hGroup = Globals.hGroups;
    while !hGroup.is_null() {
        if _wcsicmp(
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
            (*hGroup).hName,
        ) == 0 as ::core::ffi::c_int
        {
            hProgram = (*hGroup).hPrograms;
            while !hProgram.is_null() {
                PROGRAM_ExecuteProgram(hProgram);
                hProgram = (*hProgram).hNext;
            }
        }
        hGroup = (*hGroup).hNext;
    }
}

unsafe extern "system" fn MAIN_MainWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_INITMENU => {
            let mut hActiveGroup: *mut PROGGROUP = GROUP_ActiveGroup();
            if !hActiveGroup.is_null() {
                if !PROGRAM_ActiveProgram(hActiveGroup).is_null() {
                    EnableMenuItem(Globals.hFileMenu, PM_OPEN as UINT, MF_ENABLED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_MOVE as UINT, MF_ENABLED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_COPY as UINT, MF_ENABLED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_DELETE as UINT, MF_ENABLED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_ATTRIBUTES as UINT, MF_ENABLED as UINT);
                } else {
                    if (*hActiveGroup).hWnd.is_null() || IsIconic((*hActiveGroup).hWnd) != 0 {
                        EnableMenuItem(Globals.hFileMenu, PM_OPEN as UINT, MF_ENABLED as UINT);
                    } else {
                        EnableMenuItem(Globals.hFileMenu, PM_OPEN as UINT, MF_GRAYED as UINT);
                    }
                    EnableMenuItem(Globals.hFileMenu, PM_MOVE as UINT, MF_GRAYED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_COPY as UINT, MF_GRAYED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_DELETE as UINT, MF_ENABLED as UINT);
                    EnableMenuItem(Globals.hFileMenu, PM_ATTRIBUTES as UINT, MF_ENABLED as UINT);
                }
            } else {
                EnableMenuItem(Globals.hFileMenu, PM_OPEN as UINT, MF_GRAYED as UINT);
                EnableMenuItem(Globals.hFileMenu, PM_MOVE as UINT, MF_GRAYED as UINT);
                EnableMenuItem(Globals.hFileMenu, PM_COPY as UINT, MF_GRAYED as UINT);
                EnableMenuItem(Globals.hFileMenu, PM_DELETE as UINT, MF_GRAYED as UINT);
                EnableMenuItem(Globals.hFileMenu, PM_ATTRIBUTES as UINT, MF_GRAYED as UINT);
            }

            CheckMenuItem(
                Globals.hOptionMenu,
                PM_AUTO_ARRANGE as UINT,
                (MF_BYCOMMAND | (if Globals.bAutoArrange != 0 { MF_CHECKED } else { MF_UNCHECKED })) as UINT,
            );
            CheckMenuItem(
                Globals.hOptionMenu,
                PM_MIN_ON_RUN as UINT,
                (MF_BYCOMMAND | (if Globals.bMinOnRun != 0 { MF_CHECKED } else { MF_UNCHECKED })) as UINT,
            );
            CheckMenuItem(
                Globals.hOptionMenu,
                PM_SAVE_SETTINGS as UINT,
                (MF_BYCOMMAND | (if Globals.bSaveSettings != 0 { MF_CHECKED } else { MF_UNCHECKED })) as UINT,
            );
            // break
        }

        WM_DESTROY => {
            if Globals.bSaveSettings != 0 {
                MAIN_SaveSettings();
            }
            PostQuitMessage(0 as ::core::ffi::c_int);
            // break
        }

        WM_COMMAND => {
            if (LOWORD!(wParam) as UINT) < PM_FIRST_CHILD {
                MAIN_MenuCommand(hWnd, LOWORD!(wParam) as WPARAM, lParam);
            }
            // break
        }

        _ => {}
    }

    return DefFrameProcW(hWnd, Globals.hMDIWnd, uMsg, wParam, lParam);
}

unsafe extern "C" fn MAIN_MenuCommand(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    let mut Value: DWORD = 0;

    let mut hActiveGroup: *mut PROGGROUP = core::ptr::null_mut();
    let mut hActiveProgram: *mut PROGRAM = core::ptr::null_mut();
    let mut hActiveGroupWnd: HWND = core::ptr::null_mut();

    hActiveGroup = GROUP_ActiveGroup();
    hActiveProgram = PROGRAM_ActiveProgram(hActiveGroup);
    hActiveGroupWnd = if !hActiveGroup.is_null() {
        (*hActiveGroup).hWnd
    } else {
        core::ptr::null_mut()
    };

    match wParam as UINT {
        PM_NEW => {
            let mut Success: BOOL = 0;
            let mut nResult: INT = 0;

            if hActiveGroupWnd.is_null() || IsIconic(hActiveGroupWnd) != 0 {
                Success = DIALOG_New(PM_NEW_GROUP as INT, &raw mut nResult);
            } else {
                Success = DIALOG_New(PM_NEW_PROGRAM as INT, &raw mut nResult);
            }
            if Success != 0 {
                if (nResult & 1) != 0 {
                    let mut format: GROUPFORMAT;
                    let mut bIsCommonGroup: BOOL = 0;

                    format = ((nResult & 0xC) >> 2) as GROUPFORMAT;
                    bIsCommonGroup = ((nResult & 2) != 0) as BOOL;
                    GROUP_NewGroup(format, bIsCommonGroup);
                } else if !hActiveGroup.is_null() {
                    PROGRAM_NewProgram(hActiveGroup);
                }
            }
        }

        PM_OPEN => {
            if !hActiveProgram.is_null() {
                PROGRAM_ExecuteProgram(hActiveProgram);
            } else if !hActiveGroupWnd.is_null() {
                OpenIcon(hActiveGroupWnd);
            }
        }

        PM_MOVE | PM_COPY => {
            if !hActiveProgram.is_null() {
                PROGRAM_CopyMoveProgram(hActiveProgram, (wParam == PM_MOVE as WPARAM) as BOOL);
            }
        }

        PM_DELETE => {
            if !hActiveProgram.is_null() {
                if DIALOG_Delete(IDS_DELETE_PROGRAM_s as UINT, (*hActiveProgram).hName as LPCWSTR) != 0 {
                    PROGRAM_DeleteProgram(hActiveProgram, TRUE as BOOL);
                }
            } else if !hActiveGroup.is_null()
                && DIALOG_Delete(IDS_DELETE_GROUP_s as UINT, (*hActiveGroup).hName as LPCWSTR) != 0
            {
                GROUP_DeleteGroup(hActiveGroup);
            }
        }

        PM_ATTRIBUTES => {
            if !hActiveProgram.is_null() {
                PROGRAM_ModifyProgram(hActiveProgram);
            } else if !hActiveGroup.is_null() {
                GROUP_ModifyGroup(hActiveGroup);
            }
        }

        PM_EXECUTE => {
            DIALOG_Execute();
        }

        PM_EXIT => {
            // MAIN_SaveSettings();
            PostQuitMessage(0);
        }

        /* Menu Options */

        PM_AUTO_ARRANGE => {
            Globals.bAutoArrange = (Globals.bAutoArrange == 0) as BOOL;
            CheckMenuItem(
                Globals.hOptionMenu,
                PM_AUTO_ARRANGE,
                MF_BYCOMMAND | (if Globals.bAutoArrange != 0 { MF_CHECKED } else { MF_UNCHECKED }),
            );
            Value = Globals.bAutoArrange as DWORD;
            RegSetValueExW(
                Globals.hKeyPMSettings,
                RUSTCONV_WSTR_FE1F48BE2065.as_ptr(),
                0,
                REG_DWORD,
                &raw mut Value as LPBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        PM_MIN_ON_RUN => {
            Globals.bMinOnRun = (Globals.bMinOnRun == 0) as BOOL;
            CheckMenuItem(
                Globals.hOptionMenu,
                PM_MIN_ON_RUN,
                MF_BYCOMMAND | (if Globals.bMinOnRun != 0 { MF_CHECKED } else { MF_UNCHECKED }),
            );
            Value = Globals.bMinOnRun as DWORD;
            RegSetValueExW(
                Globals.hKeyPMSettings,
                RUSTCONV_WSTR_50A482475871.as_ptr(),
                0,
                REG_DWORD,
                &raw mut Value as LPBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        PM_SAVE_SETTINGS => {
            Globals.bSaveSettings = (Globals.bSaveSettings == 0) as BOOL;
            CheckMenuItem(
                Globals.hOptionMenu,
                PM_SAVE_SETTINGS,
                MF_BYCOMMAND | (if Globals.bSaveSettings != 0 { MF_CHECKED } else { MF_UNCHECKED }),
            );
            Value = Globals.bSaveSettings as DWORD;
            RegSetValueExW(
                Globals.hKeyPMSettings,
                RUSTCONV_WSTR_D1046FF7D861.as_ptr(),
                0,
                REG_DWORD,
                &raw mut Value as LPBYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }

        PM_SAVE_SETTINGS_NOW => {
            MAIN_SaveSettings();
        }

        /* Menu Windows */

        PM_OVERLAP => {
            SendMessageW(Globals.hMDIWnd, WM_MDICASCADE, 0, 0);
        }

        PM_SIDE_BY_SIDE => {
            SendMessageW(Globals.hMDIWnd, WM_MDITILE, MDITILE_VERTICAL as WPARAM, 0);
        }

        PM_ARRANGE => {
            if hActiveGroupWnd.is_null() || IsIconic(hActiveGroupWnd) != 0 {
                SendMessageW(Globals.hMDIWnd, WM_MDIICONARRANGE, 0, 0);
            } else {
                SendMessageA((*hActiveGroup).hListView, LVM_ARRANGE, 0, 0);
            }
        }

        /* Menu Help */

        PM_CONTENTS => {
            if WinHelpW(Globals.hMainWnd, RUSTCONV_WSTR_B47EBFED6E0E.as_ptr(), HELP_CONTENTS, 0) == 0 {
                MAIN_MessageBoxIDS(IDS_WINHELP_ERROR, IDS_ERROR, MB_OK as WORD);
            }
        }

        PM_ABOUT => {
            ShellAboutW(hWnd, &raw mut szTitle as *mut WCHAR as LPCWSTR, core::ptr::null_mut(), Globals.hMainIcon);
        }

        _ => {
            MAIN_MessageBoxIDS(IDS_NOT_IMPLEMENTED, IDS_ERROR, MB_OK as WORD);
        }
    }
}

unsafe extern "C" fn MAIN_RegisterMainWinClass() -> ATOM {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    wndClass.style = CS_HREDRAW | CS_VREDRAW;
    wndClass.lpfnWndProc =
        Some(MAIN_MainWndProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT);
    wndClass.cbClsExtra = 0;
    wndClass.cbWndExtra = 0;
    wndClass.hInstance = Globals.hInstance;
    wndClass.hIcon = Globals.hMainIcon;
    wndClass.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW as usize as LPCWSTR);
    wndClass.hbrBackground = GetStockObject(NULL_BRUSH as i32) as HBRUSH;
    wndClass.lpszMenuName = core::ptr::null();
    wndClass.lpszClassName = STRING_MAIN_WIN_CLASS_NAME.as_ptr();

    return RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}

unsafe extern "C" fn MAIN_CreateMainWindow() {
    let mut left: INT = 0;
    let mut top: INT = 0;
    let mut right: INT = 0;
    let mut bottom: INT = 0;
    let mut width: INT = 0;
    let mut height: INT = 0;
    let mut nCmdShow: INT = 0;
    let mut buffer: [WCHAR; 100] = [0; 100];

    let mut lRet: LONG = 0;
    let mut dwSize: DWORD = 0;
    let mut dwType: DWORD = 0;

    Globals.hMDIWnd = core::ptr::null_mut();
    Globals.hMainMenu = core::ptr::null_mut();

    /* Get the geometry of the main window */
    dwSize = core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    lRet = RegQueryValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_4B164B324B2E.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) as LONG;
    if lRet != ERROR_SUCCESS as LONG || dwType != REG_SZ as DWORD {
        buffer[0 as usize] = '\0' as WCHAR;
    }

    if swscanf(
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
        RUSTCONV_WSTR_2E5F286A7F7B.as_ptr() as *const WCHAR,
        core::ptr::addr_of_mut!(left),
        core::ptr::addr_of_mut!(top),
        core::ptr::addr_of_mut!(right),
        core::ptr::addr_of_mut!(bottom),
        core::ptr::addr_of_mut!(nCmdShow),
    ) == 5
    {
        width = right - left;
        height = bottom - top;
    } else {
        height = CW_USEDEFAULT as INT;
        width = height;
        top = width;
        left = top;
        nCmdShow = SW_SHOWNORMAL as INT;
    }

    /* Create the main window */
    Globals.hMainWnd = CreateWindowW!(
        STRING_MAIN_WIN_CLASS_NAME.as_ptr() as LPCWSTR,
        core::ptr::addr_of!(szTitle) as LPCWSTR,
        WS_OVERLAPPEDWINDOW as DWORD, // /* | WS_CLIPSIBLINGS | WS_CLIPCHILDREN */
        left,
        top,
        width,
        height,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Globals.hInstance,
        core::ptr::null_mut(),
    );

    MAIN_SetMainWindowTitle();
    ShowWindow(Globals.hMainWnd, nCmdShow);
    UpdateWindow(Globals.hMainWnd);
}

unsafe extern "C" fn MAIN_CreateMDIWindow() {
    let mut ccs: CLIENTCREATESTRUCT = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();

    /* Get the geometry of the MDI window */
    GetClientRect(Globals.hMainWnd, core::ptr::addr_of_mut!(rect));

    ccs.hWindowMenu = Globals.hWindowsMenu as *mut core::ffi::c_void;
    ccs.idFirstChild = PM_FIRST_CHILD;

    /* Create MDI Window */
    Globals.hMDIWnd = CreateWindowW!(
        WC_MDICLIENT,
        core::ptr::null_mut(),
        WS_CHILD | WS_CLIPCHILDREN | WS_VSCROLL | WS_HSCROLL,
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
        Globals.hMainWnd,
        0,
        Globals.hInstance,
        core::ptr::addr_of_mut!(ccs) as LPVOID,
    );

    /* Reset the background of the MDI client window (default: COLOR_APPWORKSPACE + 1) */
    SetClassLongPtrW(Globals.hMDIWnd, GCLP_HBRBACKGROUND, (COLOR_WINDOW + 1) as isize);

    ShowWindow(Globals.hMDIWnd, SW_SHOW as core::ffi::c_int);
    UpdateWindow(Globals.hMDIWnd);
}
#[no_mangle]
pub unsafe extern "C" fn MAIN_MessageBoxIDS(ids_text: UINT, ids_title: UINT, type_: WORD) -> INT {
    let mut text: [WCHAR; MAX_STRING_LEN as usize] = [0; MAX_STRING_LEN as usize];
    let mut title: [WCHAR; MAX_STRING_LEN as usize] = [0; MAX_STRING_LEN as usize];

    LoadStringW(Globals.hInstance, ids_text, text.as_mut_ptr(), ARRAYSIZE!(text) as core::ffi::c_int);
    LoadStringW(Globals.hInstance, ids_title, title.as_mut_ptr(), ARRAYSIZE!(title) as core::ffi::c_int);

    return MessageBoxW(Globals.hMainWnd, text.as_ptr() as LPCWSTR, title.as_ptr() as LPCWSTR, type_ as UINT) as INT;
}
#[no_mangle]
pub unsafe extern "C" fn MAIN_MessageBoxIDS_s(ids_text: UINT, str: LPCWSTR, ids_title: UINT, type_: WORD) -> INT {
    let mut text: [WCHAR; MAX_STRING_LEN as usize] = [0; MAX_STRING_LEN as usize];
    let mut title: [WCHAR; MAX_STRING_LEN as usize] = [0; MAX_STRING_LEN as usize];
    let mut newtext: [WCHAR; (MAX_STRING_LEN + MAX_PATHNAME_LEN) as usize] = [0; (MAX_STRING_LEN + MAX_PATHNAME_LEN) as usize];

    LoadStringW(Globals.hInstance, ids_text, text.as_mut_ptr(), ARRAYSIZE!(text) as INT);
    LoadStringW(Globals.hInstance, ids_title, title.as_mut_ptr(), ARRAYSIZE!(title) as INT);
    wsprintfW(newtext.as_mut_ptr(), text.as_ptr(), str);

    return MessageBoxW(Globals.hMainWnd, newtext.as_ptr(), title.as_ptr(), type_ as UINT) as INT;
}
#[no_mangle]
pub unsafe extern "C" fn MAIN_ReplaceString(string: *mut LPWSTR, replace: LPWSTR) {
    let mut newstring: LPWSTR = core::ptr::null_mut::<WCHAR>();
    newstring = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        (wcslen(replace as *const wchar_t) as SIZE_T)
            .wrapping_add(1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if !newstring.is_null() {
        wcscpy(newstring as *mut wchar_t, replace as *const wchar_t);
        *string = newstring;
    } else {
        MAIN_MessageBoxIDS(
            IDS_OUT_OF_MEMORY as UINT,
            IDS_ERROR as UINT,
            MB_OK as WORD,
        );
    }
}
