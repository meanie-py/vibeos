//! cabinet-rust shared types — one canonical definition per type.
//! Maps to dll/win32/cabinet/cabinet.h + sdk/include/psdk/fci.h + fdi.h.

use core::ffi::c_void;

// ---- basic typedefs ----
pub type cab_UBYTE = u8;
pub type cab_UWORD = u16;
pub type cab_ULONG = u32;
pub type cab_LONG = i32;
pub type cab_off_t = u32;

// ---- common FCI/FDI types (INCLUDED_TYPES_FCI_FDI) ----
#[repr(C)]
pub struct ERF {
    pub erfOper: i32,
    pub erfType: i32,
    pub fError: i32, // BOOL
}
pub type PERF = *mut ERF;

pub type TCOMP = u16;

pub type HFCI = *mut c_void;
pub type HFDI = *mut c_void;

// ---- CCAB ----
#[repr(C)]
pub struct CCAB {
    pub cb: u32,
    pub cbFolderThresh: u32,
    pub cbReserveCFHeader: u32,
    pub cbReserveCFFolder: u32,
    pub cbReserveCFData: u32,
    pub iCab: i32,
    pub iDisk: i32,
    pub setID: u16,
    pub szDisk: [i8; 256],
    pub szCab: [i8; 256],
    pub szCabPath: [i8; 256],
}
pub type PCCAB = *const CCAB;

// ---- FCI function pointer types ----
pub type PFNFCIALLOC = Option<unsafe extern "C" fn(cb: u32) -> *mut c_void>;
pub type PFNFCIFREE = Option<unsafe extern "C" fn(memory: *mut c_void)>;
pub type PFNFCIOPEN = Option<unsafe extern "C" fn(pszFile: *mut i8, oflag: i32, pmode: i32, err: *mut i32, pv: *mut c_void) -> isize>;
pub type PFNFCIREAD = Option<unsafe extern "C" fn(hf: isize, memory: *mut c_void, cb: u32, err: *mut i32, pv: *mut c_void) -> u32>;
pub type PFNFCIWRITE = Option<unsafe extern "C" fn(hf: isize, memory: *const c_void, cb: u32, err: *mut i32, pv: *mut c_void) -> u32>;
pub type PFNFCICLOSE = Option<unsafe extern "C" fn(hf: isize, err: *mut i32, pv: *mut c_void) -> i32>;
pub type PFNFCISEEK = Option<unsafe extern "C" fn(hf: isize, dist: i32, seektype: i32, err: *mut i32, pv: *mut c_void) -> i32>;
pub type PFNFCIDELETE = Option<unsafe extern "C" fn(pszFile: *mut i8, err: *mut i32, pv: *mut c_void) -> i32>;
pub type PFNFCIGETNEXTCABINET = Option<unsafe extern "C" fn(pccab: PCCAB, cbPrevCab: u32, pv: *mut c_void) -> i32>;
pub type PFNFCIFILEPLACED = Option<unsafe extern "C" fn(pccab: PCCAB, pszFile: *mut i8, cbFile: i32, fContinuation: i32, pv: *mut c_void) -> i32>;
pub type PFNFCIGETOPENINFO = Option<unsafe extern "C" fn(pszName: *mut i8, pdate: *mut u16, ptime: *mut u16, pattribs: *mut u16, err: *mut i32, pv: *mut c_void) -> isize>;
pub type PFNFCISTATUS = Option<unsafe extern "C" fn(typeStatus: u32, cb1: u32, cb2: u32, pv: *mut c_void) -> i32>;
pub type PFNFCIGETTEMPFILE = Option<unsafe extern "C" fn(pszTempName: *mut i8, cbTempName: i32, pv: *mut c_void) -> i32>;

// ---- FDI function pointer types ----
pub type PFNALLOC = Option<unsafe extern "C" fn(cb: u32) -> *mut c_void>;
pub type PFNFREE = Option<unsafe extern "C" fn(pv: *mut c_void)>;
pub type PFNOPEN = Option<unsafe extern "C" fn(pszFile: *mut i8, oflag: i32, pmode: i32) -> isize>;
pub type PFNREAD = Option<unsafe extern "C" fn(hf: isize, pv: *mut c_void, cb: u32) -> u32>;
pub type PFNWRITE = Option<unsafe extern "C" fn(hf: isize, pv: *const c_void, cb: u32) -> u32>;
pub type PFNCLOSE = Option<unsafe extern "C" fn(hf: isize) -> i32>;
pub type PFNSEEK = Option<unsafe extern "C" fn(hf: isize, dist: i32, seektype: i32) -> i32>;

// ---- FDICABINETINFO ----
#[repr(C)]
pub struct FDICABINETINFO {
    pub cbCabinet: i32,
    pub cFolders: u16,
    pub cFiles: u16,
    pub setID: u16,
    pub iCabinet: u16,
    pub fReserve: i32,
    pub hasprev: i32,
    pub hasnext: i32,
}
pub type PFDICABINETINFO = *mut FDICABINETINFO;

// ---- FDINOTIFICATION ----
pub type FDINOTIFICATIONTYPE = i32;

// fdi.h declares this under <pshpack4.h>: on 64-bit every pointer after `cb` sits at a
// 4-byte offset (pv at 0x1c, not 0x20). Natural alignment shifted every field a C client
// reads -- rapps' fnNotify took `pv` from 0x1c, got NULL, and crashed extracting its cab.
#[repr(C, packed(4))]
pub struct FDINOTIFICATION {
    pub cb: i32,
    pub psz1: *mut i8,
    pub psz2: *mut i8,
    pub psz3: *mut i8,
    pub pv: *mut c_void,
    pub hf: isize,
    pub date: u16,
    pub time: u16,
    pub attribs: u16,
    pub setID: u16,
    pub iCabinet: u16,
    pub iFolder: u16,
    pub fdie: i32,
}
pub type PFDINOTIFICATION = *mut FDINOTIFICATION;
// Offsets measured with clang against sdk/include/psdk/fdi.h (amd64 and arm64 agree).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::offset_of!(FDINOTIFICATION, psz1) == 4);
    assert!(core::mem::offset_of!(FDINOTIFICATION, pv) == 28);
    assert!(core::mem::offset_of!(FDINOTIFICATION, hf) == 36);
    assert!(core::mem::offset_of!(FDINOTIFICATION, date) == 44);
    assert!(core::mem::offset_of!(FDINOTIFICATION, fdie) == 56);
    assert!(core::mem::size_of::<FDINOTIFICATION>() == 60);
};

// ---- fdint constants ----
pub const fdintCABINET_INFO: FDINOTIFICATIONTYPE = 0;
pub const fdintPARTIAL_FILE: FDINOTIFICATIONTYPE = 1;
pub const fdintCOPY_FILE: FDINOTIFICATIONTYPE = 2;
pub const fdintCLOSE_FILE_INFO: FDINOTIFICATIONTYPE = 3;
pub const fdintNEXT_CABINET: FDINOTIFICATIONTYPE = 4;
pub const fdintENUMERATE: FDINOTIFICATIONTYPE = 5;

// ---- Common CAB constants ----
pub const CAB_BLOCKMAX: u32 = 32768;
pub const CAB_INPUTMAX: u32 = 32768 + 6144;

pub const CB_MAX_CABINET_NAME: usize = 256;
pub const CB_MAX_DISK_NAME: usize = 256;
pub const CB_MAX_FILENAME: i32 = 256;
pub const CB_MAX_CAB_PATH: usize = 256;

// ---- cfhead offsets ----
pub const cfhead_Signature: usize = 0x00;
pub const cfhead_CabinetSize: usize = 0x08;
pub const cfhead_MinorVersion: usize = 0x18;
pub const cfhead_MajorVersion: usize = 0x19;
pub const cfhead_NumFolders: usize = 0x1A;
pub const cfhead_NumFiles: usize = 0x1C;
pub const cfhead_Flags: usize = 0x1E;
pub const cfhead_SetID: usize = 0x20;
pub const cfhead_CabinetIndex: usize = 0x22;
pub const cfhead_SIZEOF: usize = 0x24;

pub const cfheadext_HeaderReserved: usize = 0x00;
pub const cfheadext_FolderReserved: usize = 0x02;
pub const cfheadext_DataReserved: usize = 0x03;
pub const cfheadext_SIZEOF: usize = 0x04;

// ---- cfhead flags ----
pub const cfheadPREV_CABINET: u32 = 0x0001;
pub const cfheadNEXT_CABINET: u32 = 0x0002;
pub const cfheadRESERVE_PRESENT: u32 = 0x0004;

// ---- tcomp constants ----
pub const tcompTYPE_NONE: u32 = 0x00000000;
pub const tcompTYPE_MSZIP: u32 = 0x00000001;

// ---- FCI error codes ----

// ---- FDI error codes ----

// (SEEK_SET / SEEK_CUR are in ntdll_sys::nt — keep locals to avoid ambiguity)

// ---- status values ----
pub const statusFile: i32 = 0;
pub const statusFolder: u32 = 1;
pub const statusCabinet: u32 = 0x0003;

// ---- ZIP/MSZIP constants ----
pub const ZIPWSIZE: u32 = 0x8000;
pub const ZIPLBITS: i32 = 9;
pub const ZIPDBITS: i32 = 6;

// ---- LZX constants ----
pub const PRETREE: u32 = 0;

// ---- ZIP tables ----
pub static Zipborder: [u8; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
pub static Zipcplens: [u16; 30] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31,
    35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258, 0,
];
pub static Zipcplext: [u16; 30] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
    3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 99,
];
pub static Zipcpdist: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193,
    257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
pub static Zipcpdext: [u16; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6,
    7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];
pub static Zipmask: [u16; 17] = [
    0x0000, 0x0001, 0x0003, 0x0007, 0x000f, 0x001f, 0x003f, 0x007f,
    0x00ff, 0x01ff, 0x03ff, 0x07ff, 0x0fff, 0x1fff, 0x3fff, 0x7fff, 0xffff,
];

// ---- Ziphuft (MSZIP huffman tree node) ----
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ziphuft {
    pub e: u8,
    pub b: u8,
    pub v: Ziphuft_u,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union Ziphuft_u {
    pub n: u16,
    pub t: *mut Ziphuft,
}

// ---- ZIPstate ---- (cabinet.h: ZIPBMAX=16, ZIPN_MAX=288)
pub const ZIPBMAX: u32 = 16;
pub const ZIPN_MAX: u32 = 288;
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ZIPstate {
    pub window_posn: u32,                 // cab_ULONG
    pub bb: u32,                          // cab_ULONG
    pub bk: u32,                          // cab_ULONG
    pub ll: [u32; 288 + 32],              // cab_ULONG ll[288+32]
    pub c: [u32; 16 + 1],                 // cab_ULONG c[ZIPBMAX+1]
    pub lx: [i32; 16 + 1],                // cab_LONG  lx[ZIPBMAX+1]
    pub u: [*mut Ziphuft; 16],            // struct Ziphuft *u[ZIPBMAX]
    pub v: [u32; 288],                    // cab_ULONG v[ZIPN_MAX]
    pub x: [u32; 16 + 1],                 // cab_ULONG x[ZIPBMAX+1]
    pub inpos: *mut u8,                   // cab_UBYTE *inpos
}

// ---- QTMmodelsym ----
#[repr(C)]
#[derive(Copy, Clone)]
pub struct QTMmodelsym {
    pub sym: u16,
    pub cumfreq: u16,
}

// ---- QTMmodel ----
#[derive(Clone, Copy)]
#[repr(C)]
pub struct QTMmodel {
    pub shiftsleft: i32,
    pub entries: i32,
    pub syms: *mut QTMmodelsym,
    pub tabloc: [u16; 256],
}

// ---- QTMstate ----
#[derive(Clone, Copy)]
#[repr(C)]
pub struct QTMstate {
    pub window: *mut u8,
    pub window_size: u32,
    pub actual_size: u32,
    pub window_posn: u32,
    pub model7: QTMmodel,
    pub m7sym: [QTMmodelsym; 8],
    pub model4: QTMmodel,
    pub model5: QTMmodel,
    pub model6pos: QTMmodel,
    pub model6len: QTMmodel,
    pub m4sym: [QTMmodelsym; 25],
    pub m5sym: [QTMmodelsym; 37],
    pub m6psym: [QTMmodelsym; 43],
    pub m6lsym: [QTMmodelsym; 28],
    pub model00: QTMmodel,
    pub model40: QTMmodel,
    pub model80: QTMmodel,
    pub modelC0: QTMmodel,
    pub m00sym: [QTMmodelsym; 65],
    pub m40sym: [QTMmodelsym; 65],
    pub m80sym: [QTMmodelsym; 65],
    pub mC0sym: [QTMmodelsym; 65],
}

// ---- LZXstate ----
#[derive(Clone, Copy)]
#[repr(C)]
pub struct LZXstate {
    pub window: *mut u8,
    pub window_size: u32,
    pub actual_size: u32,
    pub window_posn: u32,
    pub R0: u32,
    pub R1: u32,
    pub R2: u32,
    pub main_elements: u16,
    pub header_read: i32,
    pub block_type: u16,
    pub block_length: u32,
    pub block_remaining: u32,
    pub frames_read: u32,
    pub intel_filesize: i32,
    pub intel_curpos: i32,
    pub intel_started: i32,
    pub PRETREE_table: [u16; (1 << 6) + (20 << 1)],
    pub PRETREE_len: [u8; 20 + 64],
    pub MAINTREE_table: [u16; (1 << 12) + (256 + 50 * 8) * 2],
    pub MAINTREE_len: [u8; (256 + 50 * 8) + 64],
    pub LENGTH_table: [u16; (1 << 12) + (250) * 2],
    pub LENGTH_len: [u8; 250 + 64],
    pub ALIGNED_table: [u16; (1 << 7) + 8 * 2],
    pub ALIGNED_len: [u8; 8 + 64],
}

// ---- lzx_bits ----
#[repr(C)]
pub struct lzx_bits {
    pub bb: u32,
    pub bl: i32,
    pub ip: *mut u8,
}

// ---- z_stream (zlib, opaque) ----
#[repr(C)]
pub struct z_stream {
    pub next_in: *const u8,
    pub avail_in: u32,
    pub total_in: u32,
    pub next_out: *mut u8,
    pub avail_out: u32,
    pub total_out: u32,
    pub msg: *mut i8,
    pub state: *mut c_void,
    pub zalloc: Option<unsafe extern "C" fn(opaque: *mut c_void, items: u32, size: u32) -> *mut c_void>,
    pub zfree: Option<unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void)>,
    pub opaque: *mut c_void,
    pub data_type: i32,
    pub adler: u32,
    pub reserved: u32,
}

// ---- Reserve (anonymous struct in write_cabinet) ----
#[repr(C)]
pub struct Reserve {
    pub cbCFHeader: u16,
    pub cbCFFolder: u8,
    pub cbCFData: u8,
}

// ---- FILELIST ----
#[repr(C)]
pub struct FILELIST {
    pub FileName: *mut i8,
    pub next: *mut FILELIST,
    pub DoExtract: i32,
}

// ---- SESSION ----
pub const MAX_PATH: usize = 260;

#[repr(C)]
pub struct SESSION {
    pub FileSize: i32,
    pub Error: ERF,
    pub FileList: *mut FILELIST,
    pub FileCount: i32,
    pub Operation: i32,
    pub Destination: [i8; MAX_PATH],
    pub CurrentFile: [i8; MAX_PATH],
    pub Reserved: [i8; MAX_PATH],
    pub FilterList: *mut FILELIST,
}

// ---- DLLVERSIONINFO ----
#[repr(C)]
pub struct DLLVERSIONINFO {
    pub cbSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformID: u32,
}

// ---- (fdi_decomp_state type alias lives in fdi.rs, not here) ----

// fci.h / fdi.h enums, in header order. The hand-typed copies these replace had
// FCIERR_USER_ABORT = 5 (BAD_COMPR_TYPE) and FCIERR_CAB_FILE = 8 (MCI_FAIL), and fci.rs /
// fdi.rs shadowed them with further invented values (FDIERROR_NOT_A_CABINET = 5, ...).
pub const FCIERR_NONE: i32 = 0;
pub const FCIERR_OPEN_SRC: i32 = 1;
pub const FCIERR_READ_SRC: i32 = 2;
pub const FCIERR_ALLOC_FAIL: i32 = 3;
pub const FCIERR_TEMP_FILE: i32 = 4;
pub const FCIERR_BAD_COMPR_TYPE: i32 = 5;
pub const FCIERR_CAB_FILE: u32 = 6;
pub const FCIERR_USER_ABORT: i32 = 7;
pub const FCIERR_MCI_FAIL: i32 = 8;
pub const FDIERROR_NONE: i32 = 0;
pub const FDIERROR_CABINET_NOT_FOUND: i32 = 1;
pub const FDIERROR_NOT_A_CABINET: i32 = 2;
pub const FDIERROR_UNKNOWN_CABINET_VERSION: i32 = 3;
pub const FDIERROR_CORRUPT_CABINET: i32 = 4;
pub const FDIERROR_ALLOC_FAIL: i32 = 5;
pub const FDIERROR_BAD_COMPR_TYPE: i32 = 6;
pub const FDIERROR_MDI_FAIL: i32 = 7;
pub const FDIERROR_TARGET_FILE: i32 = 8;
pub const FDIERROR_RESERVE_MISMATCH: i32 = 9;
pub const FDIERROR_WRONG_CABINET: i32 = 10;
pub const FDIERROR_USER_ABORT: i32 = 11;
