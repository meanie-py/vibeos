//! typelib.rs — PARTIAL Rust translation of dll/win32/oleaut32/typelib.c
//! (11,651 LOC). See the STATUS section below and MANIFEST.md before touching.
//!
//! TRANSLATED (faithful 1:1):
//!   * the complete on-disk .tlb format structures from typelib.h (MSFT +
//!     SLTG, byte-exact, SLTG pack(1)) — `msft`/`sltg` sections below;
//!   * the registry / load-path section (typelib.c lines ~110–1055):
//!     FromLE* helpers, find_typelib_key, get_typelib_key, get_interface_key,
//!     get_lcid_subkey, query_typelib_path, QueryPathOfRegTypeLib,
//!     CreateTypeLib, LoadTypeLib, LoadTypeLibEx, LoadRegTypeLib,
//!     TLB_register_interface, RegisterTypeLib, TLB_unregister_interface,
//!     UnRegisterTypeLib, RegisterTypeLibForUser, UnRegisterTypeLibForUser.
//!
//! NOT YET TRANSLATED (typelib.c stays active C until they are — do NOT flip
//! this TU): TLB_ReadTypeLib + the MSFT/SLTG loaders, the TLB internal data
//! model, ITypeLib2/ITypeComp/ITypeInfo2 implementations, DispCallFunc (per-
//! arch asm), CreateDispTypeInfo, the ICreateTypeLib2/ICreateTypeInfo2 WMSFT
//! writer, ClearCustData. `TLB_ReadTypeLib` here is a LOUD fail stub
//! (ERR print + TYPE_E_CANTLOADLIBRARY) so the translated callers are
//! complete and honest about the gap.
//!
//! Copyright (original C): 1997 Marcus Meissner, 1999 Rein Klazes,
//! 2000 Francois Jacques, 2001 Huw D M Davies, 2004 Alastair Bridgewater,
//! 2005 Robert Shearman, 2013 Andrew Eikum (CodeWeavers). LGPL 2.1+.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_unsafe, dead_code, unused_variables, unused_mut, unused_assignments, unused_macros)]

use core::ffi::c_void;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::FindActCtxSectionGuid;
use core::mem::size_of;
use core::ptr::{null, null_mut};

use super::olesys::*;
use super::olesys::wide;

// ===========================================================================
// Lane A2 continuation — the remaining ~10.5K LOC of typelib.c, split into
// submodules (see MANIFEST.md §"Lane A2 continuation" for the exact map).
// NOTE: these `mod` declarations must precede the macro_rules! below, so the
// parent's textual macros don't leak into (and clash inside) the children.
// ===========================================================================

pub mod sys;
pub mod data;
pub mod msft;
pub mod sltg;
pub mod itypelib;
pub mod itypeinfo;
pub mod dispcall;
pub mod typecomp;
pub mod writer;

use msft::TLB_ReadTypeLib;
// CreateTypeLib2 is now translated (Lane A2) — lives in typelib/writer.rs.
use writer::CreateTypeLib2;

macro_rules! TRACE {
    ($($t:tt)*) => {{}};
}
macro_rules! WARN {
    ($($t:tt)*) => {{}};
}
macro_rules! ERR {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::olesys::dbg_print(format_args!(concat!("err:ole:typelib ", $fmt, "\n") $(, $e)*))
    };
}
macro_rules! FIXME {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::olesys::dbg_print(format_args!(concat!("fixme:ole:typelib ", $fmt, "\n") $(, $e)*))
    };
}

// ===========================================================================
// On-disk .tlb structures — byte-exact mirrors of dll/win32/oleaut32/typelib.h
// ===========================================================================

pub const HELPDLLFLAG: u16 = 0x0100;
pub const DO_NOT_SEEK: i32 = -1;

#[inline]
pub fn MSFT_HREFTYPE_INTHISFILE(href: i32) -> bool {
    (href & 3) == 0
}
#[inline]
pub fn MSFT_HREFTYPE_INDEX(href: i32) -> usize {
    href as usize / size_of::<MSFT_TypeInfoBase>()
}

pub const MSFT_SIGNATURE: u32 = 0x5446_534D; /* "MSFT" */
pub const SLTG_SIGNATURE: u32 = 0x4754_4C53; /* "SLTG" */

/// MSFT typelib header (start of file).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_Header {
    pub magic1: i32,            /* 0x5446534D "MSFT" */
    pub magic2: i32,            /* 0x00010002 version nr? */
    pub posguid: i32,           /* position of libid in guid table (or -1) */
    pub lcid: i32,              /* locale id */
    pub lcid2: i32,
    pub varflags: i32,          /* lower nibble is syskind; 0x41/0x51/0x151 */
    pub version: i32,           /* set with SetVersion() */
    pub flags: i32,             /* set with SetFlags() */
    pub nrtypeinfos: i32,       /* number of typeinfos */
    pub helpstring: i32,        /* position of help string in stringtable */
    pub helpstringcontext: i32,
    pub helpcontext: i32,
    pub nametablecount: i32,    /* number of names in name table */
    pub nametablechars: i32,    /* nr of characters in name table */
    pub NameOffset: i32,        /* offset of name in name table */
    pub helpfile: i32,          /* position of helpfile in stringtable */
    pub CustomDataOffset: i32,  /* -1 if none, else offset in custom data/guid offset table */
    pub res44: i32,             /* unknown always: 0x20 (guid hash size?) */
    pub res48: i32,             /* unknown always: 0x80 (name hash size?) */
    pub dispatchpos: i32,       /* HREFTYPE to IDispatch, or -1 if no IDispatch */
    pub nimpinfos: i32,         /* number of impinfos */
}

/// Segment descriptor.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_pSeg {
    pub offset: i32, /* absolute offset in file */
    pub length: i32, /* length of segment */
    pub res08: i32,  /* unknown always -1 */
    pub res0c: i32,  /* unknown always 0x0f in the header, 0x03 in typeinfo_data */
}

/// Main segment directory.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_SegDir {
    pub pTypeInfoTab: MSFT_pSeg,       /* 1: each typeinfo gets a 0x64-byte entry */
    pub pImpInfo: MSFT_pSeg,           /* 2: table with info for imported types */
    pub pImpFiles: MSFT_pSeg,          /* 3: import libraries */
    pub pRefTab: MSFT_pSeg,            /* 4: references table */
    pub pGuidHashTab: MSFT_pSeg,       /* 5: hash table with offsets to guid (0x80) */
    pub pGuidTab: MSFT_pSeg,           /* 6: all guids */
    pub pNameHashTab: MSFT_pSeg,       /* 7: hash table with offsets to names (0x200) */
    pub pNametab: MSFT_pSeg,           /* 8: name tables */
    pub pStringtab: MSFT_pSeg,         /* 9: string table */
    pub pTypdescTab: MSFT_pSeg,        /* A: type descriptors */
    pub pArrayDescriptions: MSFT_pSeg, /* B */
    pub pCustData: MSFT_pSeg,          /* C: custom data + default param values */
    pub pCDGuids: MSFT_pSeg,           /* D: offsets for guids into custom data table */
    pub res0e: MSFT_pSeg,              /* E: unknown */
    pub res0f: MSFT_pSeg,              /* F: unknown */
}

/// Base type info data (0x64 bytes per entry).
/// NOTE: the two INT16s are in little-endian field order (WORDS_BIGENDIAN
/// never applies on our targets).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_TypeInfoBase {
    pub typekind: i32,          /* TKIND_xxx + byte-alignment stuff */
    pub memoffset: i32,         /* points past the file if no elements */
    pub res2: i32,              /* zero if no element, N*0x40 */
    pub res3: i32,              /* -1 if no element, (N-1)*0x38 */
    pub res4: i32,              /* always? 3 */
    pub res5: i32,              /* always? zero */
    pub cElement: i32,          /* HI=cVars, LO=cFuncs */
    pub res7: i32,              /* always? zero */
    pub res8: i32,              /* always? zero */
    pub res9: i32,              /* always? zero */
    pub resA: i32,              /* always? zero */
    pub posguid: i32,           /* position in guid table */
    pub flags: i32,             /* Typeflags */
    pub NameOffset: i32,        /* offset in name table */
    pub version: i32,           /* element version */
    pub docstringoffs: i32,     /* offset of docstring in string tab */
    pub helpstringcontext: i32,
    pub helpcontext: i32,
    pub oCustData: i32,         /* offset in customer data table */
    pub cImplTypes: i16,        /* nr of implemented interfaces */
    pub cbSizeVft: i16,         /* virtual table size, including inherits */
    pub size: i32,              /* size in bytes, at least for structures */
    pub datatype1: i32,         /* position in typedesc table / base intf / dllname */
    pub datatype2: i32,         /* iface: HI=#inherited funcs, LO=#inherited ifaces */
    pub res18: i32,             /* always? 0 */
    pub res19: i32,             /* always? -1 */
}

const _: () = assert!(size_of::<MSFT_TypeInfoBase>() == 0x64);

/// Entry with information on imported types.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_ImpInfo {
    pub flags: i32,    /* bits 0-15 count; bit 16 oGuid-is-offset; bits 24-31 TKIND */
    pub oImpFile: i32, /* offset in the Import File table */
    pub oGuid: i32,    /* offset in Guid table or typeinfo index (see bit 16) */
}

pub const MSFT_IMPINFO_OFFSET_IS_GUID: i32 = 0x0001_0000;

/// Function description record (variable length; trailing arrays).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_FuncRecord {
    pub Info: i32,         /* record size including some extra stuff */
    pub DataType: i32,     /* data type of the member, eg return of function */
    pub Flags: i32,        /* attribute flags (LOWORD) */
    pub VtableOffset: i16, /* offset in vtable */
    pub funcdescsize: i16, /* size of reconstituted FUNCDESC and related */
    pub FKCCIC: i32,       /* bit 2: oEntry numeric; bit 3: has defaults;
                              bits 4-7 callconv; bit 8: custdata present;
                              bits 9-12 invocation kind; bits 13-15 funckind */
    pub nrargs: i16,       /* number of arguments (including optional?) */
    pub nroargs: i16,      /* nr of optional arguments */
    /* optional attribute fields, the number of them is variable */
    pub HelpContext: i32,
    pub oHelpString: i32,
    pub oEntry: i32,       /* either offset in string table or numeric */
    pub res9: i32,         /* unknown (-1) */
    pub resA: i32,         /* unknown (-1) */
    pub HelpStringContext: i32,
    /* controlled by a bit set in the FKCCIC field */
    pub oCustData: i32,          /* custom data for function */
    pub oArgCustData: [i32; 1],  /* custom data per argument */
}

/// Parameter info, one per argument.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_ParameterInfo {
    pub DataType: i32,
    pub oName: i32,
    pub Flags: i32,
}

/// Variable description record (variable length).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_VarRecord {
    pub Info: i32,         /* record size including some extra stuff */
    pub DataType: i32,     /* data type of the variable */
    pub Flags: i32,        /* VarFlags (LOWORD) */
    pub VarKind: i16,      /* VarKind */
    pub vardescsize: i16,  /* size of reconstituted VARDESC and related */
    pub OffsValue: i32,    /* value of the variable or the offset in the data structure */
    /* optional attribute fields, controlled by record length */
    pub HelpContext: i32,
    pub HelpString: i32,
    pub res9: i32,         /* unknown (-1) */
    pub oCustData: i32,    /* custom data for variable */
    pub HelpStringContext: i32,
}

/// Reference data record.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_RefRecord {
    pub reftype: i32,   /* offset in typeinfo table (multiple of 64) or
                           offset in the external reference table + 1 */
    pub flags: i32,
    pub oCustData: i32, /* custom data */
    pub onext: i32,     /* next offset, -1 if last */
}

/// GUID table entry.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_GuidEntry {
    pub guid: GUID,
    pub hreftype: i32,  /* -2 for the typelib guid; typeinfo offset for
                           typeinfo guid; low 2 bits 01 = imported typeinfo,
                           10 = imported typelib */
    pub next_hash: i32, /* offset to next guid in the hash bucket */
}

/// Data preceding entries in the name table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_NameIntro {
    pub hreftype: i32,  /* -1, or offset of the first typeinfo this name refers to */
    pub next_hash: i32, /* offset to next name in the hash bucket */
    pub namelen: i32,   /* only lower 8 bits are valid; 0x1000/0x2000/0x3800 flags */
}

/// Custom data table directory entry.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSFT_CDGuid {
    pub GuidOffset: i32,
    pub DataOffset: i32,
    pub next: i32, /* next offset in the table, -1 if last */
}

// ---------------------------------------------------------------------------
// SLTG typelibs (created with ICreateTypeLib) — all pack(1) in the C header.
// ---------------------------------------------------------------------------

pub const SLTG_COMPOBJ_MAGIC: &[u8; 8] = b"CompObj\0";
pub const SLTG_DIR_MAGIC: &[u8; 4] = b"dir\0";

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_Header {
    pub SLTG_magic: u32, /* 0x47544c53 == "SLTG" */
    pub nrOfFileBlks: u16, /* no of SLTG_BlkEntry's + 1 */
    pub res06: u16,      /* ?? always 9 */
    pub res08: u16,      /* some kind of len/offset ?? */
    pub first_blk: u16,  /* 1-based index into blk entries of first block in file */
    pub res0c: u32,      /* always 0x000204ff */
    pub res10: u32,      /* always 0x00000000 */
    pub res14: u32,      /* always 0x000000c0 */
    pub res18: u32,      /* always 0x46000000 */
    pub res1c: u32,      /* always 0x00000044 */
    pub res20: u32,      /* always 0xffff0000 */
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_BlkEntry {
    pub len: u32,
    pub index_string: u16, /* offs from start of SLTG_Magic to index string */
    pub next: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_Magic {
    pub res00: u8,             /* always 0x01 */
    pub CompObj_magic: [i8; 8], /* always "CompObj" */
    pub dir_magic: [i8; 4],     /* always "dir" */
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_Index {
    pub string: [i8; 11],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_Pad9 {
    pub pad: [i8; 9], /* 9 '\0's */
}

/// SLTG_NAME: like a BSTR except length (bytes) is a leading WORD and the
/// string is 8-bit chars.
pub type SLTG_Name = u16;

/// The main library block (seems to come last).
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_LibBlk {
    pub magic: u16,        /* 0x51cc */
    pub res02: u16,        /* 0x0003, 0x0004 */
    pub name: u16,         /* offset to name in name table */
    pub res06: SLTG_Name,  /* maybe this is just WORD == 0xffff */
    pub helpstring: SLTG_Name,
    pub helpfile: SLTG_Name,
    pub helpcontext: u32,
    pub syskind: u16,      /* == 1 for win32, 0 for win16 */
    pub lcid: u16,         /* == 0x409, 0x809 etc */
    pub res12: u32,        /* == 0 */
    pub libflags: u16,     /* LIBFLAG_* */
    pub maj_vers: u16,
    pub min_vers: u16,
    pub uuid: GUID,
}

pub const SLTG_LIBBLK_MAGIC: u16 = 0x51cc;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_OtherTypeInfo {
    pub index_name: SLTG_Name, /* refers to a name in the directory */
    pub other_name: SLTG_Name, /* another one of these weird names */
    pub res1a: u16,            /* 0xffff */
    pub name_offs: u16,        /* offset to name in name table */
    pub hlpstr_len: u16,       /* if non-zero, this many bytes follow (docstring?) */
    pub res20: u16,            /* 0xffff */
    pub helpcontext: u32,
    pub res26: u16,            /* 0xffff */
    pub uuid: GUID,
    pub typekind: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_TypeInfoHeader {
    pub magic: u16,          /* 0x0501 */
    pub href_table: u32,     /* if not 0xffffffff, byte offset from start of struct to href table */
    pub res06: u32,          /* 0xffffffff */
    pub elem_table: u32,     /* offset to members */
    pub res0e: u32,          /* 0xffffffff */
    pub major_version: u16,
    pub minor_version: u16,
    pub res16: u32,          /* 0xfffe0000 or 0xfffc0000 (on dual interfaces?) */
    pub typeflags1: u8,      /* 0x02 | top 5 bits hold l5sbs of TYPEFLAGS */
    pub typeflags2: u8,      /* TYPEFLAGS >> 5 */
    pub typeflags3: u8,      /* 0x02 */
    pub typekind: u8,        /* 0x03 == TKIND_INTERFACE etc. */
    pub res1e: u32,          /* 0x00000000 or 0xffffffff */
}

pub const SLTG_TIHEADER_MAGIC: u16 = 0x0501;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_TypeInfoTail {
    pub cFuncs: u16,
    pub cVars: u16,
    pub cImplTypes: u16,
    pub res06: u16,        /* always 0000 */
    pub funcs_off: u16,    /* offset to functions (from the member header) */
    pub vars_off: u16,     /* offset to vars (from the member header) */
    pub impls_off: u16,    /* offset to implemented types (from the member header) */
    pub funcs_bytes: u16,  /* bytes used by function data */
    pub vars_bytes: u16,   /* bytes used by var data */
    pub impls_bytes: u16,  /* bytes used by implemented type data */
    pub tdescalias_vt: u16, /* for TKIND_ALIAS */
    pub res16: u16,        /* always ffff */
    pub res18: u16,        /* always 0000 */
    pub res1a: u16,        /* always 0000 */
    pub simple_alias: u16, /* tdescalias_vt is a vt rather than an offset? */
    pub res1e: u16,        /* always 0000 */
    pub cbSizeInstance: u16,
    pub cbAlignment: u16,
    pub res24: u16,
    pub res26: u16,
    pub cbSizeVft: u16,
    pub res2a: u16,        /* always ffff */
    pub res2c: u16,        /* always ffff */
    pub res2e: u16,        /* always ffff */
    pub res30: u16,        /* always ffff */
    pub res32: u16,
    pub res34: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_MemberHeader {
    pub res00: u16,   /* 0x0001 sometimes 0x0003 ?? */
    pub res02: u16,   /* 0xffff */
    pub res04: u8,    /* 0x01 */
    pub cbExtra: u32, /* No of bytes that follow */
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_EnumItem {
    pub magic: u16,       /* 0x120a */
    pub next: u16,        /* offset in bytes to next block, 0xffff if last */
    pub name: u16,        /* offset to name within name table */
    pub value: u16,       /* offset to value from start of block group */
    pub res08: u16,       /* 0x56 */
    pub memid: u32,
    pub helpcontext: u16, /* 0xfffe == no context, 0x0001 == in EnumInfo, else offset */
    pub helpstring: u16,  /* offset from start of block group to string offset */
}

pub const SLTG_ENUMITEM_MAGIC: u16 = 0x120a;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_Function {
    pub magic: u8,       /* 0x4c, 0xcb or 0x8b with optional SLTG_FUNCTION_FLAGS_PRESENT flag */
    pub inv: u8,         /* high nibble is INVOKE_KIND, low nibble = 2 */
    pub next: u16,       /* byte offset from beginning of group to next fn */
    pub name: u16,       /* offset within name table to name */
    pub dispid: u32,     /* dispid */
    pub helpcontext: u16, /* helpcontext (again 1 is special) */
    pub helpstring: u16, /* helpstring offset to offset */
    pub arg_off: u16,    /* offset to args from start of block */
    pub nacc: u8,        /* lowest 3 bits are CALLCONV, rest are no of args */
    pub retnextopt: u8,  /* if 0x80 set, ret type follows; else next WORD is
                            offset to ret type. #optional args in middle 6 bits */
    pub rettype: u16,    /* return type VT_?? or offset to ret type */
    pub vtblpos: u16,    /* position in vtbl? */
    pub funcflags: u16,  /* present if magic & 0x20 */
}

pub const SLTG_FUNCTION_FLAGS_PRESENT: u8 = 0x20;
pub const SLTG_FUNCTION_MAGIC: u8 = 0x4c;
pub const SLTG_DISPATCH_FUNCTION_MAGIC: u8 = 0xcb;
pub const SLTG_STATIC_FUNCTION_MAGIC: u8 = 0x8b;

/// NOTE: mirrors the C declaration field-for-field; the header's inline
/// offset comments are descriptive of on-disk variable-length layout, not of
/// this struct (same as the C).
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_RefInfo {
    pub magic: u8,  /* 0xdf */
    pub res01: u8,  /* 0x00 */
    pub res02: u32, /* 0xffffffff */
    pub res06: u32, /* 0xffffffff */
    pub res0a: u32, /* 0xffffffff */
    pub res0e: u32, /* 0xffffffff */
    pub res12: u32, /* 0xffffffff */
    pub res16: u32, /* 0xffffffff */
    pub res1a: u32, /* 0xffffffff */
    pub res1e: u32, /* 0xffffffff */
    pub res22: u32, /* 0xffffffff */
    pub res26: u32, /* 0xffffffff */
    pub res2a: u32, /* 0xffffffff */
    pub res2e: u32, /* 0xffffffff */
    pub res32: u32, /* 0xffffffff */
    pub res36: u32, /* 0xffffffff */
    pub res3a: u32, /* 0xffffffff */
    pub res3e: u32, /* 0xffffffff */
    pub res42: u16, /* 0xffff */
    pub number: u32, /* 8 times the number of refs */
    pub res50: u16, /* 0xffff */
    pub res52: u8,  /* 0x01 */
    pub res53: u32, /* 0x00000000 */
    pub names: [SLTG_Name; 1],
    pub resxx: u8,  /* 0xdf */
}

pub const SLTG_REF_MAGIC: u8 = 0xdf;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_UnknownRefInfo {
    pub res00: u16, /* 0x0001 */
    pub res02: u8,  /* 0x02 */
    pub res03: u8,  /* 0x40 if internal ref, 0x00 if external ? */
    pub res04: u16, /* 0xffff */
    pub res06: u16, /* 0x0000, 0x0013 or 0xffff ?? */
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_ImplInfo {
    pub res00: u16, /* 0x004a */
    pub next: u16,  /* byte offs to next interface */
    pub res04: u16, /* 0xffff */
    pub impltypeflags: u8, /* IMPLTYPEFLAG_* */
    pub res07: u8,  /* 0x80 */
    pub res08: u16, /* 0x0012, 0x0028 ?? */
    pub ref_: u16,  /* number in ref table ? */
    pub res0c: u16, /* 0x4000 */
    pub res0e: u16, /* 0xfffe */
    pub res10: u16, /* 0xffff */
    pub res12: u16, /* 0x001d */
    pub pos_in_table: u16, /* 0x0, 0x4, ? */
}

pub const SLTG_IMPL_MAGIC: u16 = 0x004a;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SLTG_Variable {
    pub magic: u8,      /* 0x0a */
    pub flags: u8,
    pub next: u16,
    pub name: u16,
    pub byte_offs: u16, /* pos in struct, or offset to const type/data (if flags & 0x08) */
    pub type_: u16,     /* if flags & 0x02 this is the type, else offset to type */
    pub memid: u32,
    pub helpcontext: u16, /* ?? */
    pub helpstring: u16,  /* ?? */
    pub varflags: u16,    /* only present if magic & 0x02 */
}

pub const SLTG_VAR_MAGIC: u8 = 0x0a;
pub const SLTG_VAR_WITH_FLAGS_MAGIC: u8 = 0x2a;

// Typelib-marshaler proxy CLSIDs (typelib.h DEFINE_OLEGUIDs).
macro_rules! def_guid_tlb {
    ($name:ident, $d1:expr) => {
        pub static $name: GUID = GUID {
            Data1: $d1,
            Data2: 0,
            Data3: 0,
            Data4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
        };
    };
}
def_guid_tlb!(CLSID_PSDispatch,    0x00020420);
def_guid_tlb!(CLSID_PSEnumVariant, 0x00020421);
def_guid_tlb!(CLSID_PSTypeInfo,    0x00020422);
def_guid_tlb!(CLSID_PSTypeLib,     0x00020423);
def_guid_tlb!(CLSID_PSOAInterface, 0x00020424);
def_guid_tlb!(CLSID_PSTypeComp,    0x00020425);

// ===========================================================================
// typelib.c registry / load-path section (lines ~110–1055) — TRANSLATED
// ===========================================================================

pub const DISPATCH_HREF_OFFSET: u32 = 0x0100_0000;
pub const DISPATCH_HREF_MASK: u32 = 0xff00_0000;

// FromLEWord/FromLEDWord are identity on little-endian (all 3 targets).
#[inline]
pub fn FromLEWord(v: u16) -> u16 {
    v
}
#[inline]
pub fn FromLEDWord(v: u32) -> u32 {
    v
}
#[inline]
pub fn FromLEWords(_p: *mut c_void, _n: i32) {}
#[inline]
pub fn FromLEDWords(_p: *mut c_void, _n: i32) {}

// ---------------------------------------------------------------------------
// externs used by this section (full list in MANIFEST.md)
// ---------------------------------------------------------------------------

pub type REGSAM = u32;
pub type LSTATUS = i32;
pub type ACCESS_MASK = u32;

pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as usize as HKEY;
pub const KEY_READ: REGSAM = 0x2_0019;
pub const KEY_WRITE: REGSAM = 0x2_0006;
pub const KEY_WOW64_64KEY: REGSAM = 0x0100;
pub const KEY_WOW64_32KEY: REGSAM = 0x0200;
pub const ERROR_SUCCESS: LSTATUS = 0;
pub const ERROR_FILE_NOT_FOUND: LSTATUS = 2;
pub const REG_SZ: DWORD = 1;
pub const REG_CREATED_NEW_KEY: DWORD = 1;
pub const ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION: ULONG = 6;

#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA {
    pub lpInformation: PVOID,
    pub lpSectionBase: PVOID,
    pub ulSectionLength: ULONG,
    pub lpSectionGlobalDataBase: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
}
#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA {
    pub cbSize: ULONG,
    pub ulDataFormatVersion: ULONG,
    pub lpData: PVOID,
    pub ulLength: ULONG,
    pub lpSectionGlobalData: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
    pub lpSectionBase: PVOID,
    pub ulSectionTotalLength: ULONG,
    pub hActCtx: HANDLE,
    pub ulAssemblyRosterIndex: ULONG,
    pub ulFlags: ULONG,
    pub AssemblyMetadata: ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA,
}

extern "system" {
    // advapi32
    fn RegOpenKeyExW(
        hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: *mut HKEY,
    ) -> LSTATUS;
    fn RegCreateKeyExW(
        hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD,
        samDesired: REGSAM, lpSecurityAttributes: PVOID, phkResult: *mut HKEY,
        lpdwDisposition: *mut DWORD,
    ) -> LSTATUS;
    fn RegEnumKeyExA(
        hKey: HKEY, dwIndex: DWORD, lpName: LPSTR, lpcchName: *mut DWORD, lpReserved: *mut DWORD,
        lpClass: LPSTR, lpcchClass: *mut DWORD, lpftLastWriteTime: PVOID,
    ) -> LSTATUS;
    fn RegEnumKeyExW(
        hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: *mut DWORD, lpReserved: *mut DWORD,
        lpClass: LPWSTR, lpcchClass: *mut DWORD, lpftLastWriteTime: PVOID,
    ) -> LSTATUS;
    fn RegQueryValueW(hKey: HKEY, lpSubKey: LPCWSTR, lpData: LPWSTR, lpcbData: *mut LONG)
        -> LSTATUS;
    fn RegSetValueExW(
        hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const u8,
        cbData: DWORD,
    ) -> LSTATUS;
    fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS;
    fn RegDeleteKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, samDesired: REGSAM, Reserved: DWORD)
        -> LSTATUS;
    fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    // kernel32
    fn SearchPathW(
        lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD,
        lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    fn IsWow64Process(hProcess: HANDLE, Wow64Process: *mut BOOL) -> BOOL;
    fn GetCurrentProcess() -> HANDLE;
    fn lstrlenW(lpString: LPCWSTR) -> INT;
    fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    // ole32
    fn StringFromGUID2(id: REFGUID, str_: LPOLESTR, cmax: INT) -> INT;
    // oleaut32's own exports from OTHER TUs (oleaut.c — BSTR API)
    pub fn SysAllocString(str_: LPCOLESTR) -> BSTR;
    pub fn SysFreeString(bstr: BSTR);
}

extern "C" {
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn wcscmp(a: *const u16, b: *const u16) -> i32;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

def_guid_tlb!(IID_ICreateTypeLib, 0x00020406);

#[inline]
fn SUBLANGID(lcid: LCID) -> LCID {
    ((lcid as u16) >> 10) as LCID
}

// small swprintf/sscanf replacements (hand-rolled, byte-exact output — see
// MANIFEST "typelib deviations")
unsafe fn put_hex_lower(buf: *mut u16, mut pos: usize, val: u32) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if val == 0 {
        *buf.add(pos) = b'0' as u16;
        return pos + 1;
    }
    let mut digits = [0u16; 8];
    let mut n = 0usize;
    let mut v = val;
    while v != 0 {
        digits[n] = HEX[(v & 0xF) as usize] as u16;
        v >>= 4;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        *buf.add(pos) = digits[n];
        pos += 1;
    }
    pos
}

unsafe fn put_dec(buf: *mut u16, mut pos: usize, val: u32) -> usize {
    if val == 0 {
        *buf.add(pos) = b'0' as u16;
        return pos + 1;
    }
    let mut digits = [0u16; 10];
    let mut n = 0usize;
    let mut v = val;
    while v != 0 {
        digits[n] = b'0' as u16 + (v % 10) as u16;
        v /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        *buf.add(pos) = digits[n];
        pos += 1;
    }
    pos
}

/// sscanf(key_name, "%x.%x", &maj, &min) == 2 for ASCII registry version keys.
unsafe fn parse_hex_dot_hex(s: *const i8, v_maj: *mut i32, v_min: *mut i32) -> bool {
    unsafe fn hex_digit(c: i8) -> Option<i32> {
        match c as u8 {
            b'0'..=b'9' => Some((c as u8 - b'0') as i32),
            b'a'..=b'f' => Some((c as u8 - b'a') as i32 + 10),
            b'A'..=b'F' => Some((c as u8 - b'A') as i32 + 10),
            _ => None,
        }
    }
    let mut i = 0usize;
    let mut val: i32 = 0;
    let mut ndig = 0usize;
    loop {
        match hex_digit(*s.add(i)) {
            Some(d) => {
                val = (val << 4) | d;
                ndig += 1;
                i += 1;
            }
            None => break,
        }
    }
    if ndig == 0 {
        return false;
    }
    *v_maj = val;
    if *s.add(i) != b'.' as i8 {
        return false;
    }
    i += 1;
    val = 0;
    ndig = 0;
    loop {
        match hex_digit(*s.add(i)) {
            Some(d) => {
                val = (val << 4) | d;
                ndig += 1;
                i += 1;
            }
            None => break,
        }
    }
    if ndig == 0 {
        return false;
    }
    *v_min = val;
    true
}

/*
 * Find a typelib key which matches a requested maj.min version.
 */
unsafe fn find_typelib_key(guid: REFGUID, wMaj: *mut WORD, wMin: *mut WORD) -> BOOL {
    let typelibW = wide!("Typelib\\");
    let mut buffer: [WCHAR; 60] = [0; 60];
    let mut key_name: [i8; 16] = [0; 16];
    let mut best_maj: INT = -1;
    let mut best_min: INT = -1;
    let mut hkey: HKEY = null_mut();

    memcpy(
        buffer.as_mut_ptr() as *mut c_void,
        typelibW.as_ptr() as *const c_void,
        typelibW.len() * 2,
    );
    StringFromGUID2(guid, buffer.as_mut_ptr().add(lstrlenW(buffer.as_ptr()) as usize), 40);

    if RegOpenKeyExW(HKEY_CLASSES_ROOT, buffer.as_ptr(), 0, KEY_READ, &raw mut hkey)
        != ERROR_SUCCESS
    {
        return FALSE;
    }

    let mut len: DWORD = size_of::<[i8; 16]>() as DWORD;
    let mut i: DWORD = 0;
    loop {
        let res = RegEnumKeyExA(
            hkey,
            i,
            key_name.as_mut_ptr(),
            &raw mut len,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
        i += 1;
        if res != ERROR_SUCCESS {
            break;
        }
        let mut v_maj: INT = 0;
        let mut v_min: INT = 0;

        if parse_hex_dot_hex(key_name.as_ptr(), &raw mut v_maj, &raw mut v_min) {
            TRACE!();

            if *wMaj == 0xffff && *wMin == 0xffff {
                if v_maj > best_maj {
                    best_maj = v_maj;
                }
                if v_min > best_min {
                    best_min = v_min;
                }
            } else if *wMaj as INT == v_maj {
                best_maj = v_maj;

                if *wMin as INT == v_min {
                    best_min = v_min;
                    break; /* exact match */
                }
                // Wine 10.0: never settle for a minor version BELOW the one
                // asked for (a caller wanting 1.5 could get 1.2 before).
                if *wMin != 0xffff && v_min >= *wMin as INT && v_min > best_min {
                    best_min = v_min;
                }
            }
        }
        len = size_of::<[i8; 16]>() as DWORD;
    }
    RegCloseKey(hkey);

    TRACE!();

    if *wMaj == 0xffff && *wMin == 0xffff {
        if best_maj >= 0 && best_min >= 0 {
            *wMaj = best_maj as WORD;
            *wMin = best_min as WORD;
            return TRUE;
        }
    }

    if *wMaj as INT == best_maj && best_min >= 0 {
        *wMin = best_min as WORD;
        return TRUE;
    }
    FALSE
}

/// get the path of a typelib key, in the form "Typelib\\<guid>\\<maj>.<min>"
/// buffer must be at least 60 characters long
unsafe fn get_typelib_key(guid: REFGUID, wMaj: WORD, wMin: WORD, buffer: *mut WCHAR) -> *mut WCHAR {
    let TypelibW = wide!("Typelib\\");

    memcpy(
        buffer as *mut c_void,
        TypelibW.as_ptr() as *const c_void,
        TypelibW.len() * 2,
    );
    StringFromGUID2(guid, buffer.add(lstrlenW(buffer) as usize), 40);
    // swprintf(buffer + lstrlenW(buffer), "\\%x.%x", wMaj, wMin)
    let mut pos = lstrlenW(buffer) as usize;
    *buffer.add(pos) = b'\\' as u16;
    pos += 1;
    pos = put_hex_lower(buffer, pos, wMaj as u32);
    *buffer.add(pos) = b'.' as u16;
    pos += 1;
    pos = put_hex_lower(buffer, pos, wMin as u32);
    *buffer.add(pos) = 0;
    buffer
}

/// get the path of an interface key, in the form "Interface\\<guid>"
/// buffer must be at least 50 characters long
unsafe fn get_interface_key(guid: REFGUID, buffer: *mut WCHAR) -> *mut WCHAR {
    let InterfaceW = wide!("Interface\\");

    memcpy(
        buffer as *mut c_void,
        InterfaceW.as_ptr() as *const c_void,
        InterfaceW.len() * 2,
    );
    StringFromGUID2(guid, buffer.add(lstrlenW(buffer) as usize), 40);
    buffer
}

/// get the lcid subkey for a typelib, in the form "<lcid>\\<syskind>"
/// buffer must be at least 16 characters long
unsafe fn get_lcid_subkey(lcid: LCID, syskind: SYSKIND, buffer: *mut WCHAR) -> *mut WCHAR {
    let win16W = wide!("win16");
    let win32W = wide!("win32");
    let win64W = wide!("win64");

    // swprintf(buffer, "%lx\\", lcid)
    let mut pos = put_hex_lower(buffer, 0, lcid);
    *buffer.add(pos) = b'\\' as u16;
    pos += 1;
    *buffer.add(pos) = 0;
    match syskind {
        SYS_WIN16 => {
            lstrcatW(buffer, win16W.as_ptr());
        }
        SYS_WIN32 => {
            lstrcatW(buffer, win32W.as_ptr());
        }
        SYS_WIN64 => {
            lstrcatW(buffer, win64W.as_ptr());
        }
        _ => {
            TRACE!();
            return null_mut();
        }
    }
    buffer
}

/// struct tlibredirect_data (activation-context typelib section entry)
#[repr(C)]
pub struct tlibredirect_data {
    pub size: ULONG,
    pub res: DWORD,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub langid: LANGID,
    pub flags: WORD,
    pub help_len: ULONG,
    pub help_offset: ULONG,
    pub major_version: WORD,
    pub minor_version: WORD,
}

/// Get the path to a registered type library. Helper for QueryPathOfRegTypeLib.
/// (pub(crate): also used by typelib/itypeinfo.rs GetRefTypeInfo — same TU in C.)
pub(crate) unsafe fn query_typelib_path(
    guid: REFGUID,
    wMaj: WORD,
    wMin: WORD,
    syskind: SYSKIND,
    lcid: LCID,
    path: *mut BSTR,
    redir: BOOL,
) -> HRESULT {
    let mut hr: HRESULT = TYPE_E_LIBNOTREGISTERED;
    let mut myLCID: LCID = lcid;
    let mut hkey: HKEY = null_mut();
    let mut buffer: [WCHAR; 60] = [0; 60];
    let mut Path: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut wMaj = wMaj;
    let mut wMin = wMin;

    TRACE!();

    if redir != FALSE {
        let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();

        data.cbSize = size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
        if FindActCtxSectionGuid(
            0,
            null(),
            ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION,
            (guid).cast(),
            (&raw mut data).cast(),
        ) != FALSE
        {
            let tlib = data.lpData as *mut tlibredirect_data;

            if (wMaj != 0xffff || wMin != 0xffff)
                && ((*tlib).major_version != wMaj || (*tlib).minor_version < wMin)
            {
                return TYPE_E_LIBNOTREGISTERED;
            }

            let nameW = (data.lpSectionBase as *mut u8).add((*tlib).name_offset as usize)
                as *mut WCHAR;
            let len = SearchPathW(
                null(),
                nameW,
                null(),
                MAX_PATH as DWORD,
                Path.as_mut_ptr(),
                null_mut(),
            );
            if len == 0 {
                return TYPE_E_LIBNOTREGISTERED;
            }

            TRACE!();
            *path = SysAllocString(Path.as_ptr());
            return S_OK;
        }
    }

    if find_typelib_key(guid, &raw mut wMaj, &raw mut wMin) == FALSE {
        return TYPE_E_LIBNOTREGISTERED;
    }
    get_typelib_key(guid, wMaj, wMin, buffer.as_mut_ptr());

    let res = RegOpenKeyExW(HKEY_CLASSES_ROOT, buffer.as_ptr(), 0, KEY_READ, &raw mut hkey);
    if res == ERROR_FILE_NOT_FOUND {
        TRACE!();
        return TYPE_E_LIBNOTREGISTERED;
    } else if res != ERROR_SUCCESS {
        TRACE!();
        return TYPE_E_REGISTRYACCESS;
    }

    while hr != S_OK {
        let mut dwPathLen: LONG = size_of::<[WCHAR; MAX_PATH]>() as LONG;

        get_lcid_subkey(myLCID, syskind, buffer.as_mut_ptr());

        if RegQueryValueW(hkey, buffer.as_ptr(), Path.as_mut_ptr(), &raw mut dwPathLen) != 0 {
            if lcid == 0 {
                break;
            } else if myLCID == lcid {
                /* try with sub-langid */
                myLCID = SUBLANGID(lcid);
            } else if myLCID == SUBLANGID(lcid) && myLCID != 0 {
                /* try with system langid */
                myLCID = 0;
            } else {
                break;
            }
        } else {
            *path = SysAllocString(Path.as_ptr());
            hr = S_OK;
        }
    }
    RegCloseKey(hkey);
    TRACE!();
    hr
}

/****************************************************************************
 *		QueryPathOfRegTypeLib	[OLEAUT32.164]
 *
 * Gets the path to a registered type library.
 */
#[no_mangle]
pub unsafe extern "system" fn QueryPathOfRegTypeLib(
    guid: REFGUID,
    wMaj: WORD,
    wMin: WORD,
    lcid: LCID,
    path: *mut BSTR,
) -> HRESULT {
    // Wine 10.0: the native syskind first, then the OTHER one on either
    // bitness (32-bit used to stop after win32, so a typelib registered only
    // as win64 could not be found from a 32-bit process).
    let (native, other) = if cfg!(target_pointer_width = "64") {
        (SYS_WIN64, SYS_WIN32)
    } else {
        (SYS_WIN32, SYS_WIN64)
    };
    let hres = query_typelib_path(guid, wMaj, wMin, native, lcid, path, TRUE);
    if SUCCEEDED(hres) {
        return hres;
    }
    query_typelib_path(guid, wMaj, wMin, other, lcid, path, FALSE)
}

/******************************************************************************
 * CreateTypeLib [OLEAUT32.160]  creates a typelib
 */
#[no_mangle]
pub unsafe extern "system" fn CreateTypeLib(
    syskind: SYSKIND,
    file: LPCOLESTR,
    ctlib: *mut *mut c_void, // ICreateTypeLib**
) -> HRESULT {
    let mut typelib2: *mut ICreateTypeLib2 = null_mut();

    FIXME!("({}, {}, {:p}): forwarding to CreateTypeLib2", syskind, DbgW(file), ctlib);

    let hres = CreateTypeLib2(syskind, file, &raw mut typelib2);
    if SUCCEEDED(hres) {
        // NOTE: preserved C bug — the QI result is written to the address of
        // the *parameter* (&ctlib), so the caller's pointer is never set and
        // the reference leaks. Faithful reproduction.
        let mut ctlib_param_slot: *mut *mut c_void = ctlib;
        let unk = typelib2 as *mut IUnknown;
        let hres2 = IUnknown_QueryInterface(
            unk,
            &IID_ICreateTypeLib,
            &raw mut ctlib_param_slot as *mut *mut c_void,
        );
        IUnknown_Release(unk);
        return hres2;
    }

    hres
}

/******************************************************************************
 *		LoadTypeLib	[OLEAUT32.161]
 *
 * Loads a type library.
 */
#[no_mangle]
pub unsafe extern "system" fn LoadTypeLib(szFile: LPCOLESTR, pptLib: *mut *mut ITypeLib) -> HRESULT {
    TRACE!();
    LoadTypeLibEx(szFile, REGKIND_DEFAULT, pptLib)
}

/******************************************************************************
 *		LoadTypeLibEx	[OLEAUT32.183]
 *
 * Loads and optionally registers a type library.
 */
#[no_mangle]
pub unsafe extern "system" fn LoadTypeLibEx(
    szFile: LPCOLESTR,
    regkind: REGKIND,
    pptLib: *mut *mut ITypeLib,
) -> HRESULT {
    let mut szPath: [WCHAR; MAX_PATH + 1] = [0; MAX_PATH + 1];

    TRACE!();

    if szFile.is_null() || pptLib.is_null() {
        return E_INVALIDARG;
    }

    *pptLib = null_mut();

    let mut res = TLB_ReadTypeLib(
        szFile,
        szPath.as_mut_ptr(),
        (MAX_PATH + 1) as UINT,
        pptLib as *mut *mut ITypeLib2,
    );

    if SUCCEEDED(res) {
        match regkind {
            REGKIND_DEFAULT | REGKIND_REGISTER => {
                let mut do_register = true;
                if regkind == REGKIND_DEFAULT {
                    /* don't register typelibs supplied with full path.
                     * Experimentation confirms the following */
                    if (*szFile == '\\' as u16 && *szFile.add(1) == '\\' as u16)
                        || (*szFile != 0 && *szFile.add(1) == ':' as u16)
                    {
                        do_register = false;
                    }
                }
                if do_register {
                    res = RegisterTypeLib(*pptLib, szPath.as_ptr(), null());
                    if FAILED(res) {
                        let tl = *pptLib;
                        ((*(*tl).lpVtbl).Release)(tl);
                        *pptLib = null_mut();
                    }
                }
            }
            _ => {} // REGKIND_NONE
        }
    }

    TRACE!();
    res
}

/******************************************************************************
 *		LoadRegTypeLib	[OLEAUT32.162]
 *
 * Loads a registered type library.
 */
#[no_mangle]
pub unsafe extern "system" fn LoadRegTypeLib(
    rguid: REFGUID,
    wVerMajor: WORD,
    wVerMinor: WORD,
    lcid: LCID,
    ppTLib: *mut *mut ITypeLib,
) -> HRESULT {
    let mut bstr: BSTR = null_mut();

    *ppTLib = null_mut();

    let mut res = QueryPathOfRegTypeLib(rguid, wVerMajor, wVerMinor, lcid, &raw mut bstr);

    if SUCCEEDED(res) {
        res = LoadTypeLib(bstr, ppTLib);
        SysFreeString(bstr);

        if (wVerMajor != 0xffff || wVerMinor != 0xffff) && !(*ppTLib).is_null() {
            let mut attr: *mut TLIBATTR = null_mut();

            let tl = *ppTLib;
            res = ((*(*tl).lpVtbl).GetLibAttr)(tl, &raw mut attr);
            if res == S_OK {
                let mismatch = (*attr).wMajorVerNum != wVerMajor
                    || (*attr).wMinorVerNum < wVerMinor;
                ((*(*tl).lpVtbl).ReleaseTLibAttr)(tl, attr);

                if mismatch {
                    ((*(*tl).lpVtbl).Release)(tl);
                    *ppTLib = null_mut();
                    res = TYPE_E_LIBNOTREGISTERED;
                }
            }
        }
    }

    TRACE!();

    res
}

// string constants shared between RegisterTypeLib and UnRegisterTypeLib
static TypeLibW: &[u16] = wide!("TypeLib");
static FLAGSW: &[u16] = wide!("FLAGS");
static HELPDIRW: &[u16] = wide!("HELPDIR");
static ProxyStubClsidW: &[u16] = wide!("ProxyStubClsid");
static ProxyStubClsid32W: &[u16] = wide!("ProxyStubClsid32");

unsafe fn TLB_register_interface(
    libattr: *mut TLIBATTR,
    name: LPOLESTR,
    tattr: *mut TYPEATTR,
    flag: DWORD,
) {
    let mut keyName: [WCHAR; 60] = [0; 60];
    let mut key: HKEY = null_mut();
    let mut subKey: HKEY = null_mut();

    let typelib_proxy_clsid = wide!("{00020424-0000-0000-C000-000000000046}");
    let dispatch_proxy_clsid = wide!("{00020420-0000-0000-C000-000000000046}");

    get_interface_key(&(*tattr).guid, keyName.as_mut_ptr());
    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        keyName.as_ptr(),
        0,
        null_mut(),
        0,
        KEY_WRITE | flag,
        null_mut(),
        &raw mut key,
        null_mut(),
    ) == ERROR_SUCCESS
    {
        let proxy_clsid: &[u16] =
            if (*tattr).typekind == TKIND_INTERFACE || ((*tattr).wTypeFlags & TYPEFLAG_FDUAL) != 0
            {
                typelib_proxy_clsid
            } else {
                dispatch_proxy_clsid
            };

        if !name.is_null() {
            RegSetValueExW(
                key,
                null(),
                0,
                REG_SZ,
                name as *const u8,
                ((lstrlenW(name) + 1) as usize * size_of::<OLECHAR>()) as DWORD,
            );
        }

        if RegCreateKeyExW(
            key,
            ProxyStubClsidW.as_ptr(),
            0,
            null_mut(),
            0,
            KEY_WRITE | flag,
            null_mut(),
            &raw mut subKey,
            null_mut(),
        ) == ERROR_SUCCESS
        {
            RegSetValueExW(
                subKey,
                null(),
                0,
                REG_SZ,
                proxy_clsid.as_ptr() as *const u8,
                (typelib_proxy_clsid.len() * 2) as DWORD, // sizeof(typelib_proxy_clsid)
            );
            RegCloseKey(subKey);
        }

        if RegCreateKeyExW(
            key,
            ProxyStubClsid32W.as_ptr(),
            0,
            null_mut(),
            0,
            KEY_WRITE | flag,
            null_mut(),
            &raw mut subKey,
            null_mut(),
        ) == ERROR_SUCCESS
        {
            RegSetValueExW(
                subKey,
                null(),
                0,
                REG_SZ,
                proxy_clsid.as_ptr() as *const u8,
                (typelib_proxy_clsid.len() * 2) as DWORD,
            );
            RegCloseKey(subKey);
        }

        if RegCreateKeyExW(
            key,
            TypeLibW.as_ptr(),
            0,
            null_mut(),
            0,
            KEY_WRITE | flag,
            null_mut(),
            &raw mut subKey,
            null_mut(),
        ) == ERROR_SUCCESS
        {
            let mut buffer: [WCHAR; 40] = [0; 40];
            let VersionW = wide!("Version");

            StringFromGUID2(&(*libattr).guid, buffer.as_mut_ptr(), 40);
            RegSetValueExW(
                subKey,
                null(),
                0,
                REG_SZ,
                buffer.as_ptr() as *const u8,
                ((lstrlenW(buffer.as_ptr()) + 1) as usize * size_of::<WCHAR>()) as DWORD,
            );
            // swprintf(buffer, "%x.%x", wMajorVerNum, wMinorVerNum)
            let mut pos = put_hex_lower(buffer.as_mut_ptr(), 0, (*libattr).wMajorVerNum as u32);
            buffer[pos] = b'.' as u16;
            pos += 1;
            pos = put_hex_lower(buffer.as_mut_ptr(), pos, (*libattr).wMinorVerNum as u32);
            buffer[pos] = 0;
            RegSetValueExW(
                subKey,
                VersionW.as_ptr(),
                0,
                REG_SZ,
                buffer.as_ptr() as *const u8,
                ((lstrlenW(buffer.as_ptr()) + 1) as usize * size_of::<WCHAR>()) as DWORD,
            );
            RegCloseKey(subKey);
        }

        RegCloseKey(key);
    }
}

/******************************************************************************
 *		RegisterTypeLib	[OLEAUT32.163]
 * Adds information about a type library to the System Registry.
 */
#[no_mangle]
pub unsafe extern "system" fn RegisterTypeLib(
    ptlib: *mut ITypeLib,
    szFullPath: LPCWSTR,
    szHelpDir: LPCWSTR,
) -> HRESULT {
    let mut res: HRESULT;
    let mut attr: *mut TLIBATTR = null_mut();
    let mut keyName: [WCHAR; 60] = [0; 60];
    let mut tmp: [WCHAR; 16] = [0; 16];
    let mut key: HKEY = null_mut();
    let mut subKey: HKEY = null_mut();
    let mut kind: TYPEKIND = 0;
    let mut disposition: DWORD = 0;

    if ptlib.is_null() || szFullPath.is_null() {
        return E_INVALIDARG;
    }

    if FAILED(((*(*ptlib).lpVtbl).GetLibAttr)(ptlib, &raw mut attr)) {
        return E_FAIL;
    }

    // Wine 10.0 dropped the 32-bit refusal of SYS_WIN64 typelibs: their keys
    // are registered under win64, which QueryPathOfRegTypeLib now reaches.

    get_typelib_key(
        &(*attr).guid,
        (*attr).wMajorVerNum,
        (*attr).wMinorVerNum,
        keyName.as_mut_ptr(),
    );

    res = S_OK;
    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        keyName.as_ptr(),
        0,
        null_mut(),
        0,
        KEY_WRITE,
        null_mut(),
        &raw mut key,
        null_mut(),
    ) == ERROR_SUCCESS
    {
        let mut doc: LPOLESTR = null_mut();
        let mut libName: BSTR = null_mut();

        /* Set the human-readable name of the typelib to the typelib's doc,
         * if it exists, else to its name (Wine 10.0; a typelib without a doc
         * string used to get an empty default value). */
        if FAILED(((*(*ptlib).lpVtbl).GetDocumentation)(
            ptlib,
            -1,
            &raw mut libName,
            &raw mut doc,
            null_mut(),
            null_mut(),
        )) {
            res = E_FAIL;
        } else if !doc.is_null() || !libName.is_null() {
            let name = if !doc.is_null() { doc } else { libName };
            if RegSetValueExW(
                key,
                null(),
                0,
                REG_SZ,
                name as *const u8,
                ((lstrlenW(name) + 1) as usize * size_of::<OLECHAR>()) as DWORD,
            ) != ERROR_SUCCESS
            {
                res = E_FAIL;
            }

            SysFreeString(doc);
            SysFreeString(libName);
        }

        /* Make up the name of the typelib path subkey */
        if get_lcid_subkey((*attr).lcid, (*attr).syskind, tmp.as_mut_ptr()).is_null() {
            res = E_FAIL;
        }

        /* Create the typelib path subkey */
        if res == S_OK
            && RegCreateKeyExW(
                key,
                tmp.as_ptr(),
                0,
                null_mut(),
                0,
                KEY_WRITE,
                null_mut(),
                &raw mut subKey,
                null_mut(),
            ) == ERROR_SUCCESS
        {
            if RegSetValueExW(
                subKey,
                null(),
                0,
                REG_SZ,
                szFullPath as *const u8,
                ((lstrlenW(szFullPath) + 1) as usize * size_of::<OLECHAR>()) as DWORD,
            ) != ERROR_SUCCESS
            {
                res = E_FAIL;
            }

            RegCloseKey(subKey);
        } else {
            res = E_FAIL;
        }

        /* Create the flags subkey */
        if res == S_OK
            && RegCreateKeyExW(
                key,
                FLAGSW.as_ptr(),
                0,
                null_mut(),
                0,
                KEY_WRITE,
                null_mut(),
                &raw mut subKey,
                null_mut(),
            ) == ERROR_SUCCESS
        {
            /* FIXME: is %u correct? */
            let mut buf: [WCHAR; 20] = [0; 20];
            let pos = put_dec(buf.as_mut_ptr(), 0, (*attr).wLibFlags as u32);
            buf[pos] = 0;
            if RegSetValueExW(
                subKey,
                null(),
                0,
                REG_SZ,
                buf.as_ptr() as *const u8,
                ((lstrlenW(buf.as_ptr()) + 1) as usize * size_of::<WCHAR>()) as DWORD,
            ) != ERROR_SUCCESS
            {
                res = E_FAIL;
            }

            RegCloseKey(subKey);
        } else {
            res = E_FAIL;
        }

        /* create the helpdir subkey */
        if res == S_OK
            && RegCreateKeyExW(
                key,
                HELPDIRW.as_ptr(),
                0,
                null_mut(),
                0,
                KEY_WRITE,
                null_mut(),
                &raw mut subKey,
                &raw mut disposition,
            ) == ERROR_SUCCESS
        {
            let mut freeHelpDir: BSTR = null_mut();
            let mut szHelpDir = szHelpDir;

            /* if we created a new key, and helpDir was null, set the helpdir
               to the directory which contains the typelib. However,
               if we just opened an existing key, we leave the helpdir alone */
            if disposition == REG_CREATED_NEW_KEY && szHelpDir.is_null() {
                freeHelpDir = SysAllocString(szFullPath);
                szHelpDir = freeHelpDir;
                let mut file_name = wcsrchr(szHelpDir, '\\' as u16);
                // Wine 10.0: a path ending in a numeric "\N" is a resource
                // index (foo.dll\2), not a file; drop it first, or HELPDIR
                // became the DLL path itself instead of its directory.
                if !file_name.is_null() && *file_name.add(1) != 0 {
                    let mut end_ptr = file_name.add(1);
                    while (b'0' as u16..=b'9' as u16).contains(&*end_ptr) {
                        end_ptr = end_ptr.add(1);
                    }
                    if *end_ptr == 0 {
                        *file_name = 0;
                        file_name = wcsrchr(szHelpDir, '\\' as u16);
                    }
                }
                if !file_name.is_null() {
                    *file_name = 0;
                }
            }

            /* if we have an szHelpDir, set it! */
            if !szHelpDir.is_null() {
                if RegSetValueExW(
                    subKey,
                    null(),
                    0,
                    REG_SZ,
                    szHelpDir as *const u8,
                    ((lstrlenW(szHelpDir) + 1) as usize * size_of::<OLECHAR>()) as DWORD,
                ) != ERROR_SUCCESS
                {
                    res = E_FAIL;
                }
            }

            SysFreeString(freeHelpDir);
            RegCloseKey(subKey);
        } else {
            res = E_FAIL;
        }

        RegCloseKey(key);
    } else {
        res = E_FAIL;
    }

    /* register OLE Automation-compatible interfaces for this typelib */
    let types = ((*(*ptlib).lpVtbl).GetTypeInfoCount)(ptlib);
    let mut tidx: UINT = 0;
    while tidx < types {
        if SUCCEEDED(((*(*ptlib).lpVtbl).GetTypeInfoType)(ptlib, tidx, &raw mut kind)) {
            let mut name: LPOLESTR = null_mut();
            let mut tinfo: *mut ITypeInfo = null_mut();

            ((*(*ptlib).lpVtbl).GetDocumentation)(
                ptlib,
                tidx as INT,
                &raw mut name,
                null_mut(),
                null_mut(),
                null_mut(),
            );

            match kind {
                k if k == TKIND_INTERFACE => {
                    TRACE!();
                    ((*(*ptlib).lpVtbl).GetTypeInfo)(ptlib, tidx, &raw mut tinfo);
                }
                k if k == TKIND_DISPATCH => {
                    TRACE!();
                    ((*(*ptlib).lpVtbl).GetTypeInfo)(ptlib, tidx, &raw mut tinfo);
                }
                _ => {
                    TRACE!();
                }
            }

            if !tinfo.is_null() {
                let mut tattr: *mut TYPEATTR = null_mut();
                ((*(*tinfo).lpVtbl).GetTypeAttr)(tinfo, &raw mut tattr);

                if !tattr.is_null() {
                    TRACE!();

                    /* Register all dispinterfaces (which includes dual
                       interfaces) and oleautomation interfaces */
                    if (kind == TKIND_INTERFACE
                        && ((*tattr).wTypeFlags & TYPEFLAG_FOLEAUTOMATION) != 0)
                        || kind == TKIND_DISPATCH
                    {
                        let mut is_wow64: BOOL = FALSE;
                        let opposite: DWORD = if size_of::<*mut c_void>() == 8 {
                            KEY_WOW64_32KEY
                        } else {
                            KEY_WOW64_64KEY
                        };

                        /* register interface<->typelib coupling */
                        TLB_register_interface(attr, name, tattr, 0);

                        /* register TLBs into the opposite registry view, too */
                        if opposite == KEY_WOW64_32KEY
                            || (IsWow64Process(GetCurrentProcess(), &raw mut is_wow64) != FALSE
                                && is_wow64 != FALSE)
                        {
                            TLB_register_interface(attr, name, tattr, opposite);
                        }
                    }

                    ((*(*tinfo).lpVtbl).ReleaseTypeAttr)(tinfo, tattr);
                }

                ((*(*tinfo).lpVtbl).Release)(tinfo);
            }

            SysFreeString(name);
        }
        tidx += 1;
    }

    ((*(*ptlib).lpVtbl).ReleaseTLibAttr)(ptlib, attr);

    res
}

unsafe fn TLB_unregister_interface(guid: *mut GUID, flag: REGSAM) {
    let mut subKeyName: [WCHAR; 50] = [0; 50];
    let mut subKey: HKEY = null_mut();

    /* the path to the type */
    get_interface_key(guid, subKeyName.as_mut_ptr());

    /* Delete its bits */
    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        subKeyName.as_ptr(),
        0,
        KEY_WRITE | flag,
        &raw mut subKey,
    ) != ERROR_SUCCESS
    {
        return;
    }

    RegDeleteKeyW(subKey, ProxyStubClsidW.as_ptr());
    RegDeleteKeyW(subKey, ProxyStubClsid32W.as_ptr());
    RegDeleteKeyW(subKey, TypeLibW.as_ptr());
    RegCloseKey(subKey);
    RegDeleteKeyExW(HKEY_CLASSES_ROOT, subKeyName.as_ptr(), flag, 0);
}

/******************************************************************************
 *	UnRegisterTypeLib	[OLEAUT32.186]
 * Removes information about a type library from the System Registry.
 */
#[no_mangle]
pub unsafe extern "system" fn UnRegisterTypeLib(
    libid: REFGUID,
    wVerMajor: WORD,
    wVerMinor: WORD,
    lcid: LCID,
    syskind: SYSKIND,
) -> HRESULT {
    let mut tlibPath: BSTR = null_mut();
    let mut tmpLength: DWORD;
    let mut keyName: [WCHAR; 60] = [0; 60];
    let mut subKeyName: [WCHAR; 50] = [0; 50];
    let mut result: HRESULT = S_OK;
    let mut i: DWORD = 0;
    let mut deleteOtherStuff: bool;
    let mut key: HKEY = null_mut();
    let mut typeAttr: *mut TYPEATTR = null_mut();
    let mut kind: TYPEKIND = 0;
    let mut typeInfo: *mut ITypeInfo = null_mut();
    let mut typeLib: *mut ITypeLib = null_mut();
    let mut numTypes: INT;

    TRACE!();

    'end: {
        /* Create the path to the key */
        get_typelib_key(libid, wVerMajor, wVerMinor, keyName.as_mut_ptr());

        if syskind != SYS_WIN16 && syskind != SYS_WIN32 && syskind != SYS_WIN64 {
            TRACE!();
            result = E_INVALIDARG;
            break 'end;
        }

        /* get the path to the typelib on disk */
        if query_typelib_path(libid, wVerMajor, wVerMinor, syskind, lcid, &raw mut tlibPath, FALSE)
            != S_OK
        {
            result = E_INVALIDARG;
            break 'end;
        }

        /* Try and open the key to the type library. */
        if RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            keyName.as_ptr(),
            0,
            KEY_READ | KEY_WRITE,
            &raw mut key,
        ) != ERROR_SUCCESS
        {
            result = E_INVALIDARG;
            break 'end;
        }

        /* Try and load the type library */
        if LoadTypeLibEx(tlibPath, REGKIND_NONE, &raw mut typeLib) != S_OK {
            result = TYPE_E_INVALIDSTATE;
            break 'end;
        }

        /* remove any types registered with this typelib */
        numTypes = ((*(*typeLib).lpVtbl).GetTypeInfoCount)(typeLib) as INT;
        let mut i2: INT = 0;
        while i2 < numTypes {
            'enddeleteloop: {
                /* get the kind of type */
                if ((*(*typeLib).lpVtbl).GetTypeInfoType)(typeLib, i2 as UINT, &raw mut kind)
                    != S_OK
                {
                    break 'enddeleteloop;
                }

                /* skip non-interfaces, and get type info for the type */
                if kind != TKIND_INTERFACE && kind != TKIND_DISPATCH {
                    break 'enddeleteloop;
                }
                if ((*(*typeLib).lpVtbl).GetTypeInfo)(typeLib, i2 as UINT, &raw mut typeInfo)
                    != S_OK
                {
                    break 'enddeleteloop;
                }
                if ((*(*typeInfo).lpVtbl).GetTypeAttr)(typeInfo, &raw mut typeAttr) != S_OK {
                    break 'enddeleteloop;
                }

                if (kind == TKIND_INTERFACE
                    && ((*typeAttr).wTypeFlags & TYPEFLAG_FOLEAUTOMATION) != 0)
                    || kind == TKIND_DISPATCH
                {
                    let mut is_wow64: BOOL = FALSE;
                    let opposite: REGSAM = if size_of::<*mut c_void>() == 8 {
                        KEY_WOW64_32KEY
                    } else {
                        KEY_WOW64_64KEY
                    };

                    TLB_unregister_interface(&raw mut (*typeAttr).guid, 0);

                    /* unregister TLBs into the opposite registry view, too */
                    if opposite == KEY_WOW64_32KEY
                        || (IsWow64Process(GetCurrentProcess(), &raw mut is_wow64) != FALSE
                            && is_wow64 != FALSE)
                    {
                        TLB_unregister_interface(&raw mut (*typeAttr).guid, opposite);
                    }
                }
            }

            // enddeleteloop:
            if !typeAttr.is_null() {
                ((*(*typeInfo).lpVtbl).ReleaseTypeAttr)(typeInfo, typeAttr);
            }
            typeAttr = null_mut();
            if !typeInfo.is_null() {
                ((*(*typeInfo).lpVtbl).Release)(typeInfo);
            }
            typeInfo = null_mut();
            i2 += 1;
        }

        /* Now, delete the type library path subkey */
        get_lcid_subkey(lcid, syskind, subKeyName.as_mut_ptr());
        RegDeleteKeyW(key, subKeyName.as_ptr());
        *wcsrchr(subKeyName.as_ptr(), '\\' as u16) = 0; /* remove last path component */
        RegDeleteKeyW(key, subKeyName.as_ptr());

        /* check if there is anything besides the FLAGS/HELPDIR keys.
           If there is, we don't delete them */
        tmpLength = 50; // ARRAY_SIZE(subKeyName)
        deleteOtherStuff = true;
        i = 0;
        loop {
            let res = RegEnumKeyExW(
                key,
                i,
                subKeyName.as_mut_ptr(),
                &raw mut tmpLength,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            );
            i += 1;
            if res != ERROR_SUCCESS {
                break;
            }
            tmpLength = 50;

            /* if its not FLAGS or HELPDIR, then we must keep the rest of the key */
            if wcscmp(subKeyName.as_ptr(), FLAGSW.as_ptr()) == 0 {
                continue;
            }
            if wcscmp(subKeyName.as_ptr(), HELPDIRW.as_ptr()) == 0 {
                continue;
            }
            deleteOtherStuff = false;
            break;
        }

        /* only delete the other parts of the key if we're absolutely sure */
        if deleteOtherStuff {
            RegDeleteKeyW(key, FLAGSW.as_ptr());
            RegDeleteKeyW(key, HELPDIRW.as_ptr());
            RegCloseKey(key);
            key = null_mut();

            RegDeleteKeyW(HKEY_CLASSES_ROOT, keyName.as_ptr());
            *wcsrchr(keyName.as_ptr(), '\\' as u16) = 0; /* remove last path component */
            RegDeleteKeyW(HKEY_CLASSES_ROOT, keyName.as_ptr());
        }
    }

    // end:
    SysFreeString(tlibPath);
    if !typeLib.is_null() {
        ((*(*typeLib).lpVtbl).Release)(typeLib);
    }
    if !key.is_null() {
        RegCloseKey(key);
    }
    result
}

/******************************************************************************
 *		RegisterTypeLibForUser	[OLEAUT32.442]
 * Adds information about a type library to the user registry.
 */
#[no_mangle]
pub unsafe extern "system" fn RegisterTypeLibForUser(
    ptlib: *mut ITypeLib,
    szFullPath: *mut OLECHAR,
    szHelpDir: *mut OLECHAR,
) -> HRESULT {
    FIXME!(
        "({:p}, {}, {}) registering the typelib system-wide",
        ptlib, DbgW(szFullPath), DbgW(szHelpDir)
    );
    RegisterTypeLib(ptlib, szFullPath, szHelpDir)
}

/******************************************************************************
 *	UnRegisterTypeLibForUser	[OLEAUT32.443]
 * Removes information about a type library from the user registry.
 */
#[no_mangle]
pub unsafe extern "system" fn UnRegisterTypeLibForUser(
    libid: REFGUID,
    wVerMajor: WORD,
    wVerMinor: WORD,
    lcid: LCID,
    syskind: SYSKIND,
) -> HRESULT {
    FIXME!(
        "({}, {}, {}, {}, {}) unregistering the typelib system-wide",
        DbgGuid(libid), wVerMajor, wVerMinor, lcid, syskind
    );
    UnRegisterTypeLib(libid, wVerMajor, wVerMinor, lcid, syskind)
}

// (Lane A2 submodule declarations moved to the top of this file — see there.
//  Module map:
//   sys        extra Win32/CRT import surface + wine/list.h
//   data       TLB in-memory data model + TLB_* helpers   (C ~1057–2028)
//   msft       MSFT loader + tlb cache + TLB_ReadTypeLib  (C ~2030–3671)
//   sltg       SLTG loader                                 (C ~3674–4780)
//   itypelib   ITypeLib2 + ITypeLibComp implementations    (C ~4782–5620)
//   itypeinfo  ITypeInfo2 implementation + fnInvoke        (C ~5623–8790)
//   dispcall   DispCallFunc + per-arch call_method asm
//   typecomp   CreateDispTypeInfo + ITypeComp impl         (C ~8792–9040)
//   writer     ICreateTypeLib2/ICreateTypeInfo2 + WMSFT    (C ~9043–11651))
