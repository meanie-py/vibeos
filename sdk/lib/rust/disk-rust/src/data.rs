//! data.rs — hand translation of drivers/storage/class/disk/data.c (static lookup
//! tables; data.c has no functions so the bulk translate skipped it). Faithful to
//! the C: DiskBadControllers (HackFlags), DiskMediaTypesExclude, DiskMediaTypes.
//! String fields are `char*` into 'static NUL-terminated byte-string literals;
//! `static mut` avoids the raw-pointer `Sync` requirement (matches classpnp data.rs).
#![allow(non_upper_case_globals)]
use crate::types::{CLASSPNP_SCAN_FOR_SPECIAL_INFO, DISK_MEDIA_TYPES_LIST,
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO, _DISK_MEDIA_TYPES_LIST};
use ntkrnl_sys::ddk::{PCHAR, PCCHAR, ULONG, ULONG_PTR, STORAGE_MEDIA_TYPE,
    _STORAGE_MEDIA_TYPE_PC_5_RW, _STORAGE_MEDIA_TYPE_MO_5_WO, _STORAGE_MEDIA_TYPE_MO_5_RW,
    _STORAGE_MEDIA_TYPE_IOMEGA_JAZ, _STORAGE_MEDIA_TYPE_IOMEGA_ZIP,
    _STORAGE_MEDIA_TYPE_PINNACLE_APEX_5_RW};

// char* literal → PCHAR/PCCHAR. const-eval: byte-string literal has 'static lifetime.
macro_rules! cstr { ($s:literal) => { $s.as_ptr() as *const i8 as PCHAR }; }
macro_rules! cstrc { ($s:literal) => { $s.as_ptr() as *const i8 as PCCHAR }; }
const NULLP: PCHAR = core::ptr::null_mut();
const NULLPC: PCCHAR = core::ptr::null_mut();

// STORAGE_MEDIA_TYPE element for the MediaTypes[4] arrays (0 = Unknown).
const PC_5_RW: STORAGE_MEDIA_TYPE = _STORAGE_MEDIA_TYPE_PC_5_RW;
const MO_5_WO: STORAGE_MEDIA_TYPE = _STORAGE_MEDIA_TYPE_MO_5_WO;
const MO_5_RW: STORAGE_MEDIA_TYPE = _STORAGE_MEDIA_TYPE_MO_5_RW;
const IOMEGA_JAZ: STORAGE_MEDIA_TYPE = _STORAGE_MEDIA_TYPE_IOMEGA_JAZ;
const IOMEGA_ZIP: STORAGE_MEDIA_TYPE = _STORAGE_MEDIA_TYPE_IOMEGA_ZIP;
const PINNACLE_APEX_5_RW: STORAGE_MEDIA_TYPE = _STORAGE_MEDIA_TYPE_PINNACLE_APEX_5_RW;

macro_rules! bad { ($v:expr, $p:expr, $r:expr, $h:expr) => {
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO { VendorId: $v, ProductId: $p, ProductRevision: $r, Data: $h as ULONG_PTR }
}; }

#[no_mangle]
pub static mut DiskBadControllers: [CLASSPNP_SCAN_FOR_SPECIAL_INFO; 21] = [
    bad!(cstr!(b"COMPAQ\0"),   cstr!(b"PD-1\0"),             NULLP,          0x02),
    bad!(cstr!(b"CONNER\0"),   cstr!(b"CP3500\0"),           NULLP,          0x02),
    bad!(cstr!(b"FUJITSU\0"),  cstr!(b"M2652S-512\0"),       NULLP,          0x01),
    bad!(cstr!(b"HP      \0"), cstr!(b"C1113F  \0"),         NULLP,          0x20),
    bad!(cstr!(b"iomega\0"),   cstr!(b"jaz\0"),              NULLP,          0x30),
    bad!(cstr!(b"iomega\0"),   NULLP,                        NULLP,          0x20),
    bad!(cstr!(b"IOMEGA\0"),   cstr!(b"ZIP\0"),              NULLP,          0x27),
    bad!(cstr!(b"IOMEGA\0"),   NULLP,                        NULLP,          0x20),
    bad!(cstr!(b"MAXTOR\0"),   cstr!(b"MXT-540SL\0"),        cstr!(b"I1.2\0"), 0x01),
    bad!(cstr!(b"MICROP\0"),   cstr!(b"1936-21MW1002002\0"), NULLP,          0x03),
    bad!(cstr!(b"OLIVETTI\0"), cstr!(b"CP3500\0"),           NULLP,          0x02),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST41601N\0"),         cstr!(b"0102\0"), 0x02),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST3655N\0"),          NULLP,          0x08),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST3390N\0"),          NULLP,          0x08),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST12550N\0"),         NULLP,          0x08),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST32430N\0"),         NULLP,          0x08),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST31230N\0"),         NULLP,          0x08),
    bad!(cstr!(b"SEAGATE\0"),  cstr!(b"ST15230N\0"),         NULLP,          0x08),
    bad!(cstr!(b"SyQuest\0"),  cstr!(b"SQ5110\0"),           cstr!(b"CHC\0"),  0x03),
    bad!(cstr!(b"TOSHIBA\0"),  cstr!(b"MK538FB\0"),          cstr!(b"60\0"),   0x01),
    bad!(NULLP,                NULLP,                        NULLP,          0x0),
];

macro_rules! mt { ($v:expr, $p:expr, $r:expr, $nt:expr, $ns:expr, $m:expr) => {
    _DISK_MEDIA_TYPES_LIST { VendorId: $v, ProductId: $p, Revision: $r,
        NumberOfTypes: $nt as ULONG, NumberOfSides: $ns as ULONG, MediaTypes: $m }
}; }

#[no_mangle]
pub static mut DiskMediaTypesExclude: [DISK_MEDIA_TYPES_LIST; 2] = [
    mt!(cstrc!(b"HP\0"), cstrc!(b"RDX\0"), NULLPC, 0, 0, [0, 0, 0, 0]),
    mt!(NULLPC,          NULLPC,          NULLPC, 0, 0, [0, 0, 0, 0]),
];

#[no_mangle]
pub static mut DiskMediaTypes: [DISK_MEDIA_TYPES_LIST; 11] = [
    mt!(cstrc!(b"COMPAQ\0"),   cstrc!(b"PD-1 LF-1094\0"), NULLPC, 1, 1, [PC_5_RW, 0, 0, 0]),
    mt!(cstrc!(b"HP\0"),       NULLPC,                    NULLPC, 2, 2, [MO_5_WO, MO_5_RW, 0, 0]),
    mt!(cstrc!(b"iomega\0"),   cstrc!(b"jaz\0"),          NULLPC, 1, 1, [IOMEGA_JAZ, 0, 0, 0]),
    mt!(cstrc!(b"IOMEGA\0"),   cstrc!(b"ZIP\0"),          NULLPC, 1, 1, [IOMEGA_ZIP, 0, 0, 0]),
    mt!(cstrc!(b"PINNACLE\0"), cstrc!(b"Apex 4.6GB\0"),   NULLPC, 3, 2, [PINNACLE_APEX_5_RW, MO_5_RW, MO_5_WO, 0]),
    mt!(cstrc!(b"SONY\0"),     cstrc!(b"SMO-F541\0"),     NULLPC, 2, 2, [MO_5_WO, MO_5_RW, 0, 0]),
    mt!(cstrc!(b"SONY\0"),     cstrc!(b"SMO-F551\0"),     NULLPC, 2, 2, [MO_5_WO, MO_5_RW, 0, 0]),
    mt!(cstrc!(b"SONY\0"),     cstrc!(b"SMO-F561\0"),     NULLPC, 2, 2, [MO_5_WO, MO_5_RW, 0, 0]),
    mt!(cstrc!(b"Maxoptix\0"), cstrc!(b"T5-2600\0"),      NULLPC, 2, 2, [MO_5_WO, MO_5_RW, 0, 0]),
    mt!(cstrc!(b"Maxoptix\0"), cstrc!(b"T6-5200\0"),      NULLPC, 2, 2, [MO_5_WO, MO_5_RW, 0, 0]),
    mt!(NULLPC,                NULLPC,                    NULLPC, 0, 0, [0, 0, 0, 0]),
];
