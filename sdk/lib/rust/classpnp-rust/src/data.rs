// @generated: c2rust file-scope data (compiler-exact). Pure data file.
#![allow(non_upper_case_globals, non_camel_case_types, dead_code)]
use crate::types::*;
use ntkrnl_sys::driver_ddk::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{CHAR, GUID, LIST_ENTRY, PCHAR, ULONG_PTR, _LIST_ENTRY};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{_GUID};

pub static mut AllFdosList: LIST_ENTRY = unsafe {
    _LIST_ENTRY {
        Flink: &raw const AllFdosList as *mut _LIST_ENTRY,
        Blink: &raw const AllFdosList as *mut _LIST_ENTRY,
    }
};

pub static mut ClassBadItems: [CLASSPNP_SCAN_FOR_SPECIAL_INFO; 142] = [
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"MITSUMI CD-ROM FX240\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x2 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"MITSUMI CD-ROM FX320\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x2 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"MITSUMI CD-ROM FX322\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x2 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"TEAC DV-28E-A\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: b"2.0A\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        Data: 0x2 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HP CD-Writer cd16h\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: b"Q000\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        Data: 0x2 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"_NEC NR-7800A\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: b"1.33\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        Data: 0x2 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"COMPAQ CRD-8481B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x4 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA-4020B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GCA-4020B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4040B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA-4040B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4040B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4081B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4082B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4082B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4120B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD+RW GRA-4120B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4160B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4160B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4163B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4163B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4165B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM_GSA-4165B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4165B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4167B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-H10N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM_GSA-H10N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-H10L\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM_GSA-H10L\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-H10A\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM_GSA-H10A\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GSA-H11N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW_GSA-H11N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4080N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA-4080N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GCA-4080N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4080N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4082N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM_GSA-4082N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA-4082N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GRA-4082N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW GWA-4082N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA4082Nf\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA4082Nj\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4084N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GMA-4084N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HP\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD Writer 550s\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-T10N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM_GSA-T10N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD+-RW GSA-T11N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8160B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"COMPAQ\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8160B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8161B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8162B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8163B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8164B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDRH10N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8081N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8082N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8083N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-ROM GDR8085N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD GCC-4080N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD_GCC-4080N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD GCC-4160N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD GCC-4240N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD GCC-4241N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD_GCC-4241N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD GCC-4242N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"RW/DVD GCC-4246N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"BD-RE  GBW-H10N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVDRAM GSA-4083N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"HL-DT-ST\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD+-RW GWA4083N\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"PIONEER\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW  DVR-106D\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"ASUS\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"DVD-RW DRW-0402P\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA-4020B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GCA-4020B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4040B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA-4040B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4040B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4081B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4082B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4082B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4120B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD+RW GRA-4120B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4160B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4160B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4163B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4163B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4165B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM_GSA-4165B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4165B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4167B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-H10N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM_GSA-H10N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-H10L\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM_GSA-H10L\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-H10A\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM_GSA-H10A\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GSA-H11N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW_GSA-H11N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4080N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA-4080N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GCA-4080N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4080N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4082N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM_GSA-4082N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA-4082N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GRA-4082N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-RW GWA-4082N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA4082Nf\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA4082Nj\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4084N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GMA-4084N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HP DVD Writer 550s\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-T10N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM_GSA-T10N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD+-RW GSA-T11N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8160B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"COMPAQ DVD-ROM GDR8160B\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8161B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8162B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8163B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8164B\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDRH10N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8081N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8082N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8083N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD-ROM GDR8085N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD GCC-4080N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD_GCC-4080N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD GCC-4160N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD GCC-4240N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD GCC-4241N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD_GCC-4241N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD GCC-4242N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST RW/DVD GCC-4246N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST BD-RE  GBW-H10N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVDRAM GSA-4083N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"HL-DT-ST DVD+-RW GWA4083N\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"PIONEER DVD-RW  DVR-106D\0" as *const u8 as *const ::core::ffi::c_char
            as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductId: b"ASUS DVD-RW DRW-0402P\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0x10 as ULONG_PTR,
    },
    _CLASSPNP_SCAN_FOR_SPECIAL_INFO {
        VendorId: ::core::ptr::null::<CHAR>() as *mut CHAR,
        ProductId: ::core::ptr::null::<CHAR>() as *mut CHAR,
        ProductRevision: ::core::ptr::null::<CHAR>() as *mut CHAR,
        Data: 0 as ULONG_PTR,
    },
];

#[no_mangle]
pub static mut ClassGuidQueryRegInfoEx: GUID = _GUID {
    Data1: 0xe34b11 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x2444 as ::core::ffi::c_ushort,
    Data3: 0x4745 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut ClassGuidSenseInfo2: GUID = _GUID {
    Data1: 0x509a8c5f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x71d7 as ::core::ffi::c_ushort,
    Data3: 0x48f6 as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut ClassGuidWorkingSet: GUID = _GUID {
    Data1: 0x105701b0 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x9e9b as ::core::ffi::c_ushort,
    Data3: 0x47cb as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut ClassGuidSrbSupport: GUID = _GUID {
    Data1: 0xa483941 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xbdfd as ::core::ffi::c_ushort,
    Data3: 0x4f7b as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
