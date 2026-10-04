//! sltg.rs — the SLTG typelib loader (typelib.c lines ~3674–4780):
//! TLB_GUIDFromString, the bitstream/lookup_code/decode_string help-string
//! decompressor, SLTG_ReadString/ReadStringA/ReadName/ReadLibBlk, the
//! SLTG_Do{Type,Elem,Refs,Impls,Vars,Funcs} readers, the per-TYPEKIND
//! SLTG_Process* dispatchers and ITypeLib2_Constructor_SLTG.
//! Lane A2 continuation, faithful 1:1.
//!
//! DEVIATION (documented in MANIFEST): the C's sscanf/strtol calls
//! ("%x-%hx-%hx-%hx", "*\\R%x*#%x", "}#%hd.%hd#%x#%s", strtol(..,16)) are
//! hand-rolled fixed-format parsers with the same accept/reject behavior for
//! the strings SLTG files actually contain (like the registry-section
//! swprintf/sscanf deviation Lane A already made). Partial-match sscanf
//! semantics (earlier fields assigned, later ones untouched) are preserved.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_assignments, unused_imports, unused_labels)]

use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{null, null_mut, read_unaligned};

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::msft::TypeLibImpl_Constructor;
use super::sys::*;
use super::{
    SLTG_BlkEntry, SLTG_Function, SLTG_Header, SLTG_ImplInfo, SLTG_Index, SLTG_LibBlk,
    SLTG_Magic, SLTG_MemberHeader, SLTG_Pad9, SLTG_RefInfo, SLTG_TypeInfoHeader,
    SLTG_TypeInfoTail, SLTG_Variable, SLTG_COMPOBJ_MAGIC, SLTG_DIR_MAGIC,
    SLTG_DISPATCH_FUNCTION_MAGIC, SLTG_ENUMITEM_MAGIC, SLTG_FUNCTION_FLAGS_PRESENT,
    SLTG_FUNCTION_MAGIC, SLTG_IMPL_MAGIC, SLTG_LIBBLK_MAGIC, SLTG_REF_MAGIC, SLTG_SIGNATURE,
    SLTG_STATIC_FUNCTION_MAGIC, SLTG_TIHEADER_MAGIC, SLTG_VAR_MAGIC, SLTG_VAR_WITH_FLAGS_MAGIC,
};

// oaidl SAFEARRAY (only cDims/rgsabound are read by the SLTG path)
#[repr(C)]
pub struct SAFEARRAY {
    pub cDims: USHORT,
    pub fFeatures: USHORT,
    pub cbElements: ULONG,
    pub cLocks: ULONG,
    pub pvData: PVOID,
    pub rgsabound: [SAFEARRAYBOUND; 1],
}

// IID_StdOle (stdole2 typelib guid, from psdk)
pub static IID_StdOle: GUID = GUID {
    Data1: 0x0002_0430,
    Data2: 0,
    Data3: 0,
    Data4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
};

// winnt.h language helpers
pub const SUBLANG_NEUTRAL: u16 = 0;
#[inline]
fn PRIMARYLANGID(l: u16) -> u16 {
    l & 0x3ff
}
#[inline]
fn SUBLANGID_W(l: u16) -> u16 {
    l >> 10
}
#[inline]
fn MAKELANGID(p: u16, s: u16) -> u16 {
    ((s as u16) << 10) | p
}
#[inline]
fn MAKELCID(lgid: u16, srtid: u16) -> LCID {
    ((srtid as u32) << 16) | lgid as u32
}

// ---------------------------------------------------------------------------
// fixed-format scanf helpers (see module DEVIATION note)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn hex_val(c: i8) -> Option<u32> {
    match c as u8 {
        b'0'..=b'9' => Some((c as u8 - b'0') as u32),
        b'a'..=b'f' => Some((c as u8 - b'a') as u32 + 10),
        b'A'..=b'F' => Some((c as u8 - b'A') as u32 + 10),
        _ => None,
    }
}

/// `%x` — parse hex digits at *p, advancing it. None if no digit.
unsafe fn scan_hex(p: *mut *const i8) -> Option<u32> {
    let mut val: u32 = 0;
    let mut n = 0;
    while let Some(d) = hex_val(**p) {
        val = (val << 4) | d;
        *p = (*p).add(1);
        n += 1;
    }
    if n == 0 {
        None
    } else {
        Some(val)
    }
}

/// `%hd` — parse (optionally signed) decimal at *p, advancing it.
unsafe fn scan_dec(p: *mut *const i8) -> Option<i32> {
    let mut neg = false;
    let mut q = *p;
    if *q == b'-' as i8 {
        neg = true;
        q = q.add(1);
    }
    let mut val: i32 = 0;
    let mut n = 0;
    while (*q as u8).is_ascii_digit() {
        val = val.wrapping_mul(10).wrapping_add((*q as u8 - b'0') as i32);
        q = q.add(1);
        n += 1;
    }
    if n == 0 {
        return None;
    }
    *p = q;
    Some(if neg { -val } else { val })
}

/// match a literal char, advancing on success.
unsafe fn scan_lit(p: *mut *const i8, c: u8) -> bool {
    if **p == c as i8 {
        *p = (*p).add(1);
        true
    } else {
        false
    }
}

/// TLB_GUIDFromString (typelib.c:3674) — sscanf(str, "%x-%hx-%hx-%hx", ...)
/// + 6 strtol'd byte pairs from str+24.
pub unsafe fn TLB_GUIDFromString(str_: *const i8, guid: *mut GUID) -> BOOL {
    let mut p = str_;

    let ok = 'scan: {
        let Some(d1) = scan_hex(&mut p) else { break 'scan false };
        (*guid).Data1 = d1;
        if !scan_lit(&mut p, b'-') {
            break 'scan false;
        }
        let Some(d2) = scan_hex(&mut p) else { break 'scan false };
        (*guid).Data2 = d2 as u16;
        if !scan_lit(&mut p, b'-') {
            break 'scan false;
        }
        let Some(d3) = scan_hex(&mut p) else { break 'scan false };
        (*guid).Data3 = d3 as u16;
        if !scan_lit(&mut p, b'-') {
            break 'scan false;
        }
        let Some(s) = scan_hex(&mut p) else { break 'scan false };
        let s = s as u16 as i16;
        (*guid).Data4[0] = ((s as i32) >> 8) as u8;
        (*guid).Data4[1] = (s & 0xff) as u8;
        true
    };

    if !ok {
        FIXME!("Can't parse guid {}", DbgGuid(guid));
        return FALSE;
    }

    let mut i = 0usize;
    while i < 6 {
        /* strtol(b, NULL, 16) on the 2-char pair at str+24+2*i */
        let q = str_.add(24 + 2 * i);
        let mut val: u32 = 0;
        if let Some(hi) = hex_val(*q) {
            val = hi;
            if let Some(lo) = hex_val(*q.add(1)) {
                val = (val << 4) | lo;
            }
        }
        (*guid).Data4[i + 2] = val as u8;
        i += 1;
    }
    TRUE
}

// ---------------------------------------------------------------------------
// help-string decompressor
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct bitstream {
    pub buffer: *const u8,
    pub length: DWORD,
    pub current: WORD,
}

unsafe fn lookup_code(table: *const u8, table_size: DWORD, bits: *mut bitstream) -> *const i8 {
    let mut p = table;
    let end = table.add(table_size as usize);

    while p < end && *p == 0x80 {
        if p.add(2) >= end {
            return null();
        }

        if (*bits).current & 0xff == 0 {
            if (*bits).length == 0 {
                return null();
            }
            (*bits).current = ((*(*bits).buffer as u16) << 8) | 1;
            (*bits).buffer = (*bits).buffer.add(1);
            (*bits).length -= 1;
        }

        if (*bits).current & 0x8000 != 0 {
            p = p.add(3);
        } else {
            p = table.add((*p.add(2) as usize) | ((*p.add(1) as usize) << 8));
        }

        (*bits).current <<= 1;
    }

    if p.add(1) < end && *p.add(1) != 0 {
        /* FIXME: Whats the meaning of *p? */
        let mut q = p.add(1);
        while q < end && *q != 0 {
            q = q.add(1);
        }
        return if q < end { p.add(1) as *const i8 } else { null() };
    }

    null()
}

pub unsafe fn decode_string(
    table: *const u8,
    stream: *const i8,
    stream_length: DWORD,
    lib: *mut ITypeLibImpl,
) -> *const TLBString {
    let mut table = table;

    if stream_length == 0 {
        return null();
    }

    let mut bits = bitstream {
        buffer: stream as *const u8,
        length: stream_length,
        current: 0,
    };

    let buf_size = read_unaligned(table as *const WORD) as DWORD;
    table = table.add(size_of::<WORD>());
    let table_size = read_unaligned(table as *const DWORD);
    table = table.add(size_of::<DWORD>());

    let buf: BSTR = SysAllocStringLen(null(), buf_size);
    *buf = 0;

    loop {
        let p = lookup_code(table, table_size, &raw mut bits);
        if p.is_null() {
            break;
        }
        let spaceW = wide!(" ");
        if *buf != 0 {
            lstrcatW(buf, spaceW.as_ptr());
        }
        MultiByteToWideChar(
            CP_ACP,
            0,
            p,
            -1,
            buf.add(lstrlenW(buf) as usize),
            buf_size as INT - lstrlenW(buf),
        );
    }

    let tlbstr = TLB_append_str(&raw mut (*lib).string_list, buf);
    SysFreeString(buf);

    tlbstr
}

pub unsafe fn SLTG_ReadString(
    ptr: *const i8,
    pStr: *mut *const TLBString,
    lib: *mut ITypeLibImpl,
) -> WORD {
    *pStr = null();
    let bytelen: WORD = read_unaligned(ptr as *const WORD);
    if bytelen == 0xffff {
        return 2;
    }

    let len = MultiByteToWideChar(CP_ACP, 0, ptr.add(2), bytelen as INT, null_mut(), 0);
    let tmp_str = SysAllocStringLen(null(), len as UINT);
    if !tmp_str.is_null() {
        MultiByteToWideChar(CP_ACP, 0, ptr.add(2), bytelen as INT, tmp_str, len);
        *pStr = TLB_append_str(&raw mut (*lib).string_list, tmp_str);
        SysFreeString(tmp_str);
    }
    bytelen + 2
}

pub unsafe fn SLTG_ReadStringA(ptr: *const i8, str_: *mut *mut i8) -> WORD {
    *str_ = null_mut();
    let bytelen: WORD = read_unaligned(ptr as *const WORD);
    if bytelen == 0xffff {
        return 2;
    }
    *str_ = heap_alloc(bytelen as usize + 1) as *mut i8;
    memcpy(*str_ as *mut c_void, ptr.add(2) as *const c_void, bytelen as usize);
    *(*str_).add(bytelen as usize) = 0;
    bytelen + 2
}

pub unsafe fn SLTG_ReadName(
    pNameTable: *const i8,
    offset: i32,
    lib: *mut ITypeLibImpl,
) -> *mut TLBString {
    let name_list = &raw mut (*lib).name_list;
    let mut cur = (*name_list).next;
    while cur != name_list {
        let tlbstr = list_entry!(cur, TLBString, entry);
        if (*tlbstr).offset == offset as UINT {
            return tlbstr;
        }
        cur = (*cur).next;
    }

    let tmp_str = TLB_MultiByteToBSTR(pNameTable.add(offset as usize));
    let tlbstr = TLB_append_str(&raw mut (*lib).name_list, tmp_str);
    SysFreeString(tmp_str);

    /* NOTE: like the C, the returned entry's `offset` is whatever
     * TLB_append_str left there (uninitialized for a fresh entry) — the C
     * never sets it either on this path. Faithful. */
    tlbstr
}

pub unsafe fn SLTG_ReadLibBlk(pLibBlk: LPVOID, pTypeLibImpl: *mut ITypeLibImpl) -> DWORD {
    let mut ptr = pLibBlk as *const i8;

    let w = read_unaligned(ptr as *const WORD);
    if w != SLTG_LIBBLK_MAGIC {
        FIXME!("libblk magic = {:04x}", w);
        return 0;
    }

    ptr = ptr.add(6);
    let w = read_unaligned(ptr as *const WORD);
    if w != 0xffff {
        FIXME!("LibBlk.res06 = {:04x}. Assumung string and skipping", w);
        ptr = ptr.add(w as usize);
    }
    ptr = ptr.add(2);

    ptr = ptr.add(SLTG_ReadString(ptr, &raw mut (*pTypeLibImpl).DocString, pTypeLibImpl) as usize);

    ptr = ptr.add(SLTG_ReadString(ptr, &raw mut (*pTypeLibImpl).HelpFile, pTypeLibImpl) as usize);

    (*pTypeLibImpl).dwHelpContext = read_unaligned(ptr as *const DWORD);
    ptr = ptr.add(4);

    (*pTypeLibImpl).syskind = read_unaligned(ptr as *const WORD) as SYSKIND;
    (*pTypeLibImpl).ptr_size = get_ptr_size((*pTypeLibImpl).syskind);
    ptr = ptr.add(2);

    let langid = read_unaligned(ptr as *const WORD);
    if SUBLANGID_W(langid) == SUBLANG_NEUTRAL {
        let lcid = MAKELCID(MAKELANGID(PRIMARYLANGID(langid), 0), 0);
        (*pTypeLibImpl).lcid = lcid;
        (*pTypeLibImpl).set_lcid = lcid;
    } else {
        (*pTypeLibImpl).lcid = 0;
        (*pTypeLibImpl).set_lcid = 0;
    }
    ptr = ptr.add(2);

    ptr = ptr.add(4); /* skip res12 */

    (*pTypeLibImpl).libflags = read_unaligned(ptr as *const WORD);
    ptr = ptr.add(2);

    (*pTypeLibImpl).ver_major = read_unaligned(ptr as *const WORD);
    ptr = ptr.add(2);

    (*pTypeLibImpl).ver_minor = read_unaligned(ptr as *const WORD);
    ptr = ptr.add(2);

    let mut g: GUID = core::mem::zeroed();
    memcpy(
        &raw mut g as *mut c_void,
        ptr as *const c_void,
        size_of::<GUID>(),
    );
    (*pTypeLibImpl).guid =
        TLB_append_guid(&raw mut (*pTypeLibImpl).guid_list, &raw const g, -2i32 as HREFTYPE);
    ptr = ptr.add(size_of::<GUID>());

    ptr.offset_from(pLibBlk as *const i8) as DWORD
}

/// stores a mapping between the sltg typeinfo's references and the typelib's
/// HREFTYPEs
#[repr(C)]
pub struct sltg_ref_lookup_t {
    pub num: u32,
    pub refs: [HREFTYPE; 1],
}

pub unsafe fn sltg_get_typelib_ref(
    table: *const sltg_ref_lookup_t,
    typeinfo_ref: DWORD,
    typelib_ref: *mut HREFTYPE,
) -> HRESULT {
    if !table.is_null() && typeinfo_ref < (*table).num {
        *typelib_ref = *(&raw const (*table).refs)
            .cast::<HREFTYPE>()
            .add(typeinfo_ref as usize);
        return S_OK;
    }

    ERR!("Unable to find reference");
    *typelib_ref = -1i32 as HREFTYPE;
    E_FAIL
}

pub unsafe fn SLTG_DoType(
    mut pType: *mut WORD,
    pBlk: *mut i8,
    mut pTD: *mut TYPEDESC,
    ref_lookup: *const sltg_ref_lookup_t,
) -> *mut WORD {
    let mut done = false;

    while !done {
        let t = read_unaligned(pType);
        if (t & 0xe00) == 0xe00 {
            (*pTD).vt = VT_PTR;
            (*pTD).u.lptdesc = heap_alloc_zero(size_of::<TYPEDESC>()) as *mut TYPEDESC;
            pTD = (*pTD).u.lptdesc;
        }
        match (t & 0x3f) as VARTYPE {
            VT_PTR => {
                (*pTD).vt = VT_PTR;
                (*pTD).u.lptdesc = heap_alloc_zero(size_of::<TYPEDESC>()) as *mut TYPEDESC;
                pTD = (*pTD).u.lptdesc;
            }
            VT_USERDEFINED => {
                (*pTD).vt = VT_USERDEFINED;
                pType = pType.add(1);
                let mut href: HREFTYPE = 0;
                sltg_get_typelib_ref(
                    ref_lookup,
                    read_unaligned(pType) as DWORD / 4,
                    &raw mut href,
                );
                (*pTD).u.hreftype = href;
                done = true;
            }
            VT_CARRAY => {
                /* *(pType+1) is offset to a SAFEARRAY, *(pType+2) is type of
                 * array */
                pType = pType.add(1);
                let pSA = pBlk.add(read_unaligned(pType) as usize) as *const SAFEARRAY;
                let cDims = read_unaligned(&raw const (*pSA).cDims);

                (*pTD).vt = VT_CARRAY;
                (*pTD).u.lpadesc = heap_alloc_zero(
                    size_of::<ARRAYDESC>()
                        + (cDims as usize).wrapping_sub(1) * size_of::<SAFEARRAYBOUND>(),
                ) as *mut ARRAYDESC;
                (*(*pTD).u.lpadesc).cDims = cDims;
                memcpy(
                    &raw mut (*(*pTD).u.lpadesc).rgbounds as *mut c_void,
                    &raw const (*pSA).rgsabound as *const c_void,
                    cDims as usize * size_of::<SAFEARRAYBOUND>(),
                );

                pTD = &raw mut (*(*pTD).u.lpadesc).tdescElem;
            }
            VT_SAFEARRAY => {
                /* FIXME: *(pType+1) gives an offset to SAFEARRAY, is this
                 * useful? */
                pType = pType.add(1);
                (*pTD).vt = VT_SAFEARRAY;
                (*pTD).u.lptdesc = heap_alloc_zero(size_of::<TYPEDESC>()) as *mut TYPEDESC;
                pTD = (*pTD).u.lptdesc;
            }
            _ => {
                (*pTD).vt = (t & 0x3f) as VARTYPE;
                done = true;
            }
        }
        pType = pType.add(1);
    }
    pType
}

pub unsafe fn SLTG_DoElem(
    pType: *mut WORD,
    pBlk: *mut i8,
    pElem: *mut ELEMDESC,
    ref_lookup: *const sltg_ref_lookup_t,
) -> *mut WORD {
    let t = read_unaligned(pType);
    /* Handle [in/out] first */
    if (t & 0xc000) == 0xc000 {
        (*pElem).u.paramdesc.wParamFlags = PARAMFLAG_NONE;
    } else if t & 0x8000 != 0 {
        (*pElem).u.paramdesc.wParamFlags = PARAMFLAG_FIN | PARAMFLAG_FOUT;
    } else if t & 0x4000 != 0 {
        (*pElem).u.paramdesc.wParamFlags = PARAMFLAG_FOUT;
    } else {
        (*pElem).u.paramdesc.wParamFlags = PARAMFLAG_FIN;
    }

    if t & 0x2000 != 0 {
        (*pElem).u.paramdesc.wParamFlags |= PARAMFLAG_FLCID;
    }

    if t & 0x80 != 0 {
        (*pElem).u.paramdesc.wParamFlags |= PARAMFLAG_FRETVAL;
    }

    SLTG_DoType(pType, pBlk, &raw mut (*pElem).tdesc, ref_lookup)
}

pub unsafe fn SLTG_DoRefs(
    pRef: *mut SLTG_RefInfo,
    pTL: *mut ITypeLibImpl,
    pNameTable: *mut i8,
) -> *mut sltg_ref_lookup_t {
    let magic = (*pRef).magic;
    if magic != SLTG_REF_MAGIC {
        FIXME!("Ref magic = {:x}", magic);
        return null_mut();
    }
    let number: u32 = {
        // packed field
        let n = &raw const (*pRef).number;
        read_unaligned(n)
    };
    let mut name = (pRef as *mut i8)
        .add(offset_of!(SLTG_RefInfo, names))
        .add(number as usize);

    let table = heap_alloc(
        size_of::<sltg_ref_lookup_t>()
            + ((number as usize >> 3).wrapping_sub(1)) * size_of::<HREFTYPE>(),
    ) as *mut sltg_ref_lookup_t;
    (*table).num = number >> 3;

    /* FIXME should scan the existing list and reuse matching refs added by
     * previous typeinfos */

    /* We don't want the first href to be 0 */
    let mut typelib_ref: HREFTYPE = (list_count(&raw const (*pTL).ref_list) + 1) << 2;

    let mut ref_: u32 = 0;
    while ref_ < number >> 3 {
        let mut refname: *mut i8 = null_mut();

        let ref_type = heap_alloc_zero(size_of::<TLBRefType>()) as *mut TLBRefType;

        name = name.add(SLTG_ReadStringA(name, &raw mut refname) as usize);

        /* sscanf(refname, "*\\R%x*#%x", &lib_offs, &type_num) */
        let mut lib_offs: u32 = 0;
        let mut type_num: u32 = 0;
        {
            let mut p = refname as *const i8;
            let matched = 'scan: {
                if !scan_lit(&mut p, b'*') || !scan_lit(&mut p, b'\\') || !scan_lit(&mut p, b'R') {
                    break 'scan 0;
                }
                let Some(lo) = scan_hex(&mut p) else { break 'scan 0 };
                lib_offs = lo;
                if !scan_lit(&mut p, b'*') || !scan_lit(&mut p, b'#') {
                    break 'scan 1;
                }
                let Some(tn) = scan_hex(&mut p) else { break 'scan 1 };
                type_num = tn;
                2
            };
            if matched != 2 {
                FIXME!("Can't sscanf ref");
            }
        }

        if lib_offs != 0xffff {
            /* LIST_FOR_EACH_ENTRY with completion sentinel, hand-rolled */
            let implib_list = &raw mut (*pTL).implib_list;
            let mut import: *mut TLBImpLib = null_mut();
            let mut cur = (*implib_list).next;
            while cur != implib_list {
                let candidate = list_entry!(cur, TLBImpLib, entry);
                if (*candidate).offset == lib_offs as i32 {
                    import = candidate;
                    break;
                }
                cur = (*cur).next;
            }

            if import.is_null() {
                let mut fname: [i8; MAX_PATH + 1] = [0; MAX_PATH + 1];
                let mut tmpguid: GUID = core::mem::zeroed();

                import = heap_alloc_zero(size_of::<TLBImpLib>()) as *mut TLBImpLib;
                (*import).offset = lib_offs as i32;
                TLB_GUIDFromString(pNameTable.add(lib_offs as usize + 4), &raw mut tmpguid);
                (*import).guid =
                    TLB_append_guid(&raw mut (*pTL).guid_list, &raw const tmpguid, 2);

                /* sscanf(pNameTable + lib_offs + 40, "}#%hd.%hd#%x#%s", ...) */
                {
                    let mut p = pNameTable.add(lib_offs as usize + 40) as *const i8; // nolint: magic-offset (SLTG on-disk name-table string offset, per wine typelib.c sscanf)
                    let matched = 'scan: {
                        if !scan_lit(&mut p, b'}') || !scan_lit(&mut p, b'#') {
                            break 'scan 0;
                        }
                        let Some(maj) = scan_dec(&mut p) else { break 'scan 0 };
                        (*import).wVersionMajor = maj as i16 as WORD;
                        if !scan_lit(&mut p, b'.') {
                            break 'scan 1;
                        }
                        let Some(min) = scan_dec(&mut p) else { break 'scan 1 };
                        (*import).wVersionMinor = min as i16 as WORD;
                        if !scan_lit(&mut p, b'#') {
                            break 'scan 2;
                        }
                        let Some(lcid) = scan_hex(&mut p) else { break 'scan 2 };
                        (*import).lcid = lcid as LCID;
                        if !scan_lit(&mut p, b'#') {
                            break 'scan 3;
                        }
                        /* %s — copy non-whitespace run */
                        let mut n = 0usize;
                        while *p != 0
                            && !(*p as u8).is_ascii_whitespace()
                            && n < MAX_PATH
                        {
                            fname[n] = *p;
                            p = p.add(1);
                            n += 1;
                        }
                        fname[n] = 0;
                        if n == 0 {
                            break 'scan 3;
                        }
                        4
                    };
                    if matched != 4 {
                        FIXME!("can't sscanf ref");
                    }
                }

                let len = strlen(fname.as_ptr());
                if fname[len - 1] != b'#' as i8 {
                    FIXME!("fname mismatch (no trailing #)");
                }
                fname[len - 1] = 0;
                (*import).name = TLB_MultiByteToBSTR(fname.as_ptr());
                list_add_tail(&raw mut (*pTL).implib_list, &raw mut (*import).entry);
            }
            (*ref_type).pImpTLInfo = import;

            /* Store a reference to IDispatch */
            if (*pTL).dispatch_href == -1i32 as HREFTYPE
                && guid_eq(&raw const (*(*import).guid).guid, &raw const IID_StdOle)
                && type_num == 4
            {
                (*pTL).dispatch_href = typelib_ref;
            }
        } else {
            /* internal ref */
            (*ref_type).pImpTLInfo = TLB_REF_INTERNAL;
        }
        (*ref_type).reference = typelib_ref;
        (*ref_type).index = type_num as INT;

        heap_free(refname as *mut c_void);
        list_add_tail(&raw mut (*pTL).ref_list, &raw mut (*ref_type).entry);

        *(&raw mut (*table).refs)
            .cast::<HREFTYPE>()
            .add(ref_ as usize) = typelib_ref;
        typelib_ref += 4;
        ref_ += 1;
    }
    if *name as u8 != SLTG_REF_MAGIC {
        FIXME!("End of ref block magic = {:x}", *name);
    }
    /* dump_TLBRefType — TRACE-only, compiled out */
    table
}

pub unsafe fn SLTG_DoImpls(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    OneOnly: BOOL,
    ref_lookup: *const sltg_ref_lookup_t,
) -> *mut i8 {
    /* I don't really get this structure, usually it's 0x16 bytes long, but
     * iuser.tlb contains some that are 0x18 bytes long. That's ok because we
     * can use the next ptr to jump to the next one. But how do we know the
     * length of the last one? The WORD at offs 0x8 might be the clue. For now
     * I'm just assuming that the last one is the regular 0x16 bytes. */

    let mut info = pBlk as *mut SLTG_ImplInfo;
    loop {
        (*pTI).typeattr.cImplTypes += 1;
        if (*info).next == 0xffff {
            break;
        }
        info = pBlk.add((*info).next as usize) as *mut SLTG_ImplInfo;
    }

    let mut info = pBlk as *mut SLTG_ImplInfo;
    (*pTI).impltypes = TLBImplType_Alloc((*pTI).typeattr.cImplTypes as UINT);
    let mut pImplType = (*pTI).impltypes;
    loop {
        let mut href: HREFTYPE = 0;
        sltg_get_typelib_ref(ref_lookup, (*info).ref_ as DWORD, &raw mut href);
        (*pImplType).hRef = href;
        (*pImplType).implflags = (*info).impltypeflags as i32;
        pImplType = pImplType.add(1);

        if (*info).next == 0xffff {
            break;
        }
        if OneOnly != FALSE {
            FIXME!("Interface inheriting more than one interface");
        }
        info = pBlk.add((*info).next as usize) as *mut SLTG_ImplInfo;
    }
    info = info.add(1); /* see comment at top of function */
    info as *mut i8
}

pub unsafe fn SLTG_DoVars(
    pBlk: *mut i8,
    pFirstItem: *mut i8,
    pTI: *mut ITypeInfoImpl,
    cVars: u16,
    pNameTable: *const i8,
    ref_lookup: *const sltg_ref_lookup_t,
    hlp_strings: *const u8,
) {
    let mut prevName: *const TLBString = null();

    (*pTI).vardescs = TLBVarDesc_Alloc(cVars as UINT);
    let mut pVarDesc = (*pTI).vardescs;

    let mut pItem = pFirstItem as *mut SLTG_Variable;
    let mut i: u16 = 0;
    while i < cVars {
        (*pVarDesc).vardesc.memid = (*pItem).memid as MEMBERID;

        let magic = (*pItem).magic;
        if magic != SLTG_VAR_MAGIC && magic != SLTG_VAR_WITH_FLAGS_MAGIC {
            FIXME!("var magic = {:02x}", magic);
            return;
        }

        if (*pItem).name == 0xfffe {
            (*pVarDesc).Name = prevName;
        } else {
            (*pVarDesc).Name = SLTG_ReadName(pNameTable, (*pItem).name as i32, (*pTI).pTypeLib);
        }

        TRACE!();
        TRACE!();
        TRACE!();

        if (*pItem).helpstring != 0xffff {
            (*pVarDesc).HelpString = decode_string(
                hlp_strings,
                pBlk.add((*pItem).helpstring as usize),
                pNameTable.offset_from(pBlk as *const i8) as DWORD,
                (*pTI).pTypeLib,
            );
            TRACE!();
        }

        let pType: *mut WORD = if (*pItem).flags & 0x02 != 0 {
            &raw mut (*pItem).type_
        } else {
            pBlk.add((*pItem).type_ as usize) as *mut WORD
        };

        if (*pItem).flags & !0xdau8 != 0 {
            FIXME!("unhandled flags = {:02x}", (*pItem).flags & !0xdau8);
        }

        SLTG_DoElem(pType, pBlk, &raw mut (*pVarDesc).vardesc.elemdescVar, ref_lookup);

        /* TRACE_ON dump_TypeDesc — compiled out */

        if (*pItem).flags & 0x40 != 0 {
            TRACE!();
            (*pVarDesc).vardesc.varkind = VAR_DISPATCH;
        } else if (*pItem).flags & 0x10 != 0 {
            TRACE!();
            (*pVarDesc).vardesc.varkind = VAR_CONST;
            (*pVarDesc).vardesc.u.lpvarValue = heap_alloc(size_of::<VARIANT>()) as *mut VARIANT;
            (*(*pVarDesc).vardesc.u.lpvarValue).vt = VT_INT;
            if (*pItem).flags & 0x08 != 0 {
                (*(*pVarDesc).vardesc.u.lpvarValue).n3.lVal = (*pItem).byte_offs as i32;
            } else {
                match (*pVarDesc).vardesc.elemdescVar.tdesc.vt {
                    VT_LPSTR | VT_LPWSTR | VT_BSTR => {
                        let len: WORD =
                            read_unaligned(pBlk.add((*pItem).byte_offs as usize) as *const WORD);
                        let str_: BSTR;
                        TRACE!();
                        if len == 0xffff {
                            str_ = null_mut();
                        } else {
                            let alloc_len = MultiByteToWideChar(
                                CP_ACP,
                                0,
                                pBlk.add((*pItem).byte_offs as usize + 2),
                                len as INT,
                                null_mut(),
                                0,
                            );
                            str_ = SysAllocStringLen(null(), alloc_len as UINT);
                            MultiByteToWideChar(
                                CP_ACP,
                                0,
                                pBlk.add((*pItem).byte_offs as usize + 2),
                                len as INT,
                                str_,
                                alloc_len,
                            );
                        }
                        (*(*pVarDesc).vardesc.u.lpvarValue).vt = VT_BSTR;
                        (*(*pVarDesc).vardesc.u.lpvarValue).n3.bstrVal = str_;
                    }
                    VT_I2 | VT_UI2 | VT_I4 | VT_UI4 | VT_INT | VT_UINT => {
                        (*(*pVarDesc).vardesc.u.lpvarValue).n3.lVal =
                            read_unaligned(pBlk.add((*pItem).byte_offs as usize) as *const INT);
                    }
                    _ => {
                        FIXME!(
                            "VAR_CONST unimplemented for type {}",
                            (*pVarDesc).vardesc.elemdescVar.tdesc.vt
                        );
                    }
                }
            }
        } else {
            TRACE!();
            (*pVarDesc).vardesc.u.oInst = (*pItem).byte_offs as ULONG;
            (*pVarDesc).vardesc.varkind = VAR_PERINSTANCE;
        }

        if (*pItem).magic == SLTG_VAR_WITH_FLAGS_MAGIC {
            (*pVarDesc).vardesc.wVarFlags = (*pItem).varflags;
        }

        if (*pItem).flags & 0x80 != 0 {
            (*pVarDesc).vardesc.wVarFlags |= VARFLAG_FREADONLY;
        }

        prevName = (*pVarDesc).Name;

        pItem = pBlk.add((*pItem).next as usize) as *mut SLTG_Variable;
        i += 1;
        pVarDesc = pVarDesc.add(1);
    }
    (*pTI).typeattr.cVars = cVars;
}

pub unsafe fn SLTG_DoFuncs(
    pBlk: *mut i8,
    pFirstItem: *mut i8,
    pTI: *mut ITypeInfoImpl,
    cFuncs: u16,
    pNameTable: *mut i8,
    ref_lookup: *const sltg_ref_lookup_t,
    hlp_strings: *const u8,
) {
    (*pTI).funcdescs = TLBFuncDesc_Alloc(cFuncs as UINT);

    let mut pFuncDesc = (*pTI).funcdescs;
    let mut pFunc = pFirstItem as *mut SLTG_Function;
    let mut i: u16 = 0;
    'funcs: while i < cFuncs && pFunc as usize != 0xFFFF {
        'next: {
            let magic = (*pFunc).magic;
            match magic & !SLTG_FUNCTION_FLAGS_PRESENT {
                m if m == SLTG_FUNCTION_MAGIC => {
                    (*pFuncDesc).funcdesc.funckind = FUNC_PUREVIRTUAL;
                }
                m if m == SLTG_DISPATCH_FUNCTION_MAGIC => {
                    (*pFuncDesc).funcdesc.funckind = FUNC_DISPATCH;
                }
                m if m == SLTG_STATIC_FUNCTION_MAGIC => {
                    (*pFuncDesc).funcdesc.funckind = FUNC_STATIC;
                }
                _ => {
                    FIXME!(
                        "unimplemented func magic = {:02x}",
                        magic & !SLTG_FUNCTION_FLAGS_PRESENT
                    );
                    break 'next; /* continue */
                }
            }
            (*pFuncDesc).Name =
                SLTG_ReadName(pNameTable, (*pFunc).name as i32, (*pTI).pTypeLib);

            (*pFuncDesc).funcdesc.memid = (*pFunc).dispid as MEMBERID;
            (*pFuncDesc).funcdesc.invkind = ((*pFunc).inv >> 4) as INVOKEKIND;
            (*pFuncDesc).funcdesc.callconv = ((*pFunc).nacc & 0x7) as CALLCONV;
            (*pFuncDesc).funcdesc.cParams = ((*pFunc).nacc >> 3) as SHORT;
            (*pFuncDesc).funcdesc.cParamsOpt = (((*pFunc).retnextopt & 0x7e) >> 1) as SHORT;
            // Wine 10.0: a dispatch function has no vtable slot, and the offset is
            // UNSIGNED: through `as i32` an offset >= 0x8000 went negative.
            (*pFuncDesc).funcdesc.oVft = if (*pFuncDesc).funcdesc.funckind == FUNC_DISPATCH {
                0
            } else {
                (((*pFunc).vtblpos as u16 as i32 & !1)
                    * size_of::<*mut c_void>() as i32
                    / (*(*pTI).pTypeLib).ptr_size) as SHORT
            };
            if (*pFunc).helpstring != 0xffff {
                (*pFuncDesc).HelpString = decode_string(
                    hlp_strings,
                    pBlk.add((*pFunc).helpstring as usize),
                    pNameTable.offset_from(pBlk) as DWORD,
                    (*pTI).pTypeLib,
                );
            }

            if (*pFunc).magic & SLTG_FUNCTION_FLAGS_PRESENT != 0 {
                (*pFuncDesc).funcdesc.wFuncFlags = (*pFunc).funcflags;
            }

            let mut pType: *mut WORD = if (*pFunc).retnextopt & 0x80 != 0 {
                &raw mut (*pFunc).rettype
            } else {
                pBlk.add((*pFunc).rettype as usize) as *mut WORD
            };

            SLTG_DoElem(pType, pBlk, &raw mut (*pFuncDesc).funcdesc.elemdescFunc, ref_lookup);

            (*pFuncDesc).funcdesc.lprgelemdescParam = heap_alloc_zero(
                (*pFuncDesc).funcdesc.cParams as usize * size_of::<ELEMDESC>(),
            ) as *mut ELEMDESC;
            (*pFuncDesc).pParamDesc =
                TLBParDesc_Constructor((*pFuncDesc).funcdesc.cParams as UINT);

            let mut pArg = pBlk.add((*pFunc).arg_off as usize) as *mut WORD;

            let mut param: i32 = 0;
            while param < (*pFuncDesc).funcdesc.cParams as i32 {
                let arg0 = read_unaligned(pArg);
                let mut paramName: *mut i8 = pNameTable.add((arg0 & !1) as usize);
                /* If arg type follows then paramName points to the 2nd letter
                 * of the name, else the next WORD is an offset to the arg
                 * type and paramName points to the first letter. So let's
                 * take one char off paramName and see if we're pointing at an
                 * alpha-numeric char. However if *pArg is 0xffff or 0xfffe
                 * then the param has no name, the former meaning that the
                 * next WORD is the type, the latter meaning that the next
                 * WORD is an offset to the type. */

                if arg0 == 0xffff || arg0 == 0xfffe {
                    paramName = null_mut();
                }

                let HaveOffs = (arg0 & 1) == 0;
                pArg = pArg.add(1);

                TRACE!();

                if HaveOffs {
                    /* the next word is an offset to type */
                    let pType = pBlk.add(read_unaligned(pArg) as usize) as *mut WORD;
                    SLTG_DoElem(
                        pType,
                        pBlk,
                        (*pFuncDesc).funcdesc.lprgelemdescParam.add(param as usize),
                        ref_lookup,
                    );
                    pArg = pArg.add(1);
                } else {
                    pArg = SLTG_DoElem(
                        pArg,
                        pBlk,
                        (*pFuncDesc).funcdesc.lprgelemdescParam.add(param as usize),
                        ref_lookup,
                    );
                }

                /* Are we an optional param ? */
                if (*pFuncDesc).funcdesc.cParams as i32 - param
                    <= (*pFuncDesc).funcdesc.cParamsOpt as i32
                {
                    (*(*pFuncDesc).funcdesc.lprgelemdescParam.add(param as usize))
                        .u
                        .paramdesc
                        .wParamFlags |= PARAMFLAG_FOPT;
                }

                if !paramName.is_null() {
                    (*(*pFuncDesc).pParamDesc.add(param as usize)).Name = SLTG_ReadName(
                        pNameTable,
                        paramName.offset_from(pNameTable) as i32,
                        (*pTI).pTypeLib,
                    );
                } else {
                    (*(*pFuncDesc).pParamDesc.add(param as usize)).Name = (*pFuncDesc).Name;
                }

                param += 1;
            }
        }

        /* loop increment (the C for-clause) */
        pFunc = pBlk.add((*pFunc).next as usize) as *mut SLTG_Function;
        i += 1;
        pFuncDesc = pFuncDesc.add(1);
    }
    (*pTI).typeattr.cFuncs = cFuncs;
}

pub unsafe fn SLTG_ProcessCoClass(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *mut i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
) {
    let mut ref_lookup: *mut sltg_ref_lookup_t = null_mut();

    let href_table = (*pTIHeader).href_table;
    if href_table != 0xffff_ffff {
        ref_lookup = SLTG_DoRefs(
            (pTIHeader as *mut i8).add(href_table as usize) as *mut SLTG_RefInfo,
            (*pTI).pTypeLib,
            pNameTable,
        );
    }

    let pFirstItem = pBlk;

    if read_unaligned(pFirstItem as *const WORD) == SLTG_IMPL_MAGIC {
        SLTG_DoImpls(pFirstItem, pTI, FALSE, ref_lookup);
    }
    heap_free(ref_lookup as *mut c_void);
}

pub unsafe fn SLTG_ProcessInterface(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *mut i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
    hlp_strings: *const u8,
) {
    let mut ref_lookup: *mut sltg_ref_lookup_t = null_mut();

    let href_table = (*pTIHeader).href_table;
    if href_table != 0xffff_ffff {
        ref_lookup = SLTG_DoRefs(
            (pTIHeader as *mut i8).add(href_table as usize) as *mut SLTG_RefInfo,
            (*pTI).pTypeLib,
            pNameTable,
        );
    }

    let pFirstItem = pBlk;

    if read_unaligned(pFirstItem as *const WORD) == SLTG_IMPL_MAGIC {
        SLTG_DoImpls(pFirstItem, pTI, TRUE, ref_lookup);
    }

    if (*pTITail).funcs_off != 0xffff {
        SLTG_DoFuncs(
            pBlk,
            pBlk.add((*pTITail).funcs_off as usize),
            pTI,
            (*pTITail).cFuncs,
            pNameTable,
            ref_lookup,
            hlp_strings,
        );
    }

    heap_free(ref_lookup as *mut c_void);

    /* TRACE_ON dump — compiled out */
}

pub unsafe fn SLTG_ProcessRecord(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *const i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
    hlp_strings: *const u8,
) {
    SLTG_DoVars(
        pBlk,
        pBlk.add((*pTITail).vars_off as usize),
        pTI,
        (*pTITail).cVars,
        pNameTable,
        null(),
        hlp_strings,
    );
}

pub unsafe fn SLTG_ProcessAlias(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *mut i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
) {
    let mut ref_lookup: *mut sltg_ref_lookup_t = null_mut();

    if (*pTITail).simple_alias != 0 {
        /* if simple alias, no more processing required */
        (*pTI).tdescAlias = heap_alloc_zero(size_of::<TYPEDESC>()) as *mut TYPEDESC;
        (*(*pTI).tdescAlias).vt = (*pTITail).tdescalias_vt as VARTYPE;
        return;
    }

    let href_table = (*pTIHeader).href_table;
    if href_table != 0xffff_ffff {
        ref_lookup = SLTG_DoRefs(
            (pTIHeader as *mut i8).add(href_table as usize) as *mut SLTG_RefInfo,
            (*pTI).pTypeLib,
            pNameTable,
        );
    }

    /* otherwise it is an offset to a type */
    let pType = pBlk.add((*pTITail).tdescalias_vt as usize) as *mut WORD;

    (*pTI).tdescAlias = heap_alloc(size_of::<TYPEDESC>()) as *mut TYPEDESC;
    SLTG_DoType(pType, pBlk, (*pTI).tdescAlias, ref_lookup);

    heap_free(ref_lookup as *mut c_void);
}

pub unsafe fn SLTG_ProcessDispatch(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *mut i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
    hlp_strings: *const u8,
) {
    let mut ref_lookup: *mut sltg_ref_lookup_t = null_mut();
    let href_table = (*pTIHeader).href_table;
    if href_table != 0xffff_ffff {
        ref_lookup = SLTG_DoRefs(
            (pTIHeader as *mut i8).add(href_table as usize) as *mut SLTG_RefInfo,
            (*pTI).pTypeLib,
            pNameTable,
        );
    }

    if (*pTITail).vars_off != 0xffff {
        SLTG_DoVars(
            pBlk,
            pBlk.add((*pTITail).vars_off as usize),
            pTI,
            (*pTITail).cVars,
            pNameTable,
            ref_lookup,
            hlp_strings,
        );
    }

    if (*pTITail).funcs_off != 0xffff {
        SLTG_DoFuncs(
            pBlk,
            pBlk.add((*pTITail).funcs_off as usize),
            pTI,
            (*pTITail).cFuncs,
            pNameTable,
            ref_lookup,
            hlp_strings,
        );
    }

    if (*pTITail).impls_off != 0xffff {
        SLTG_DoImpls(pBlk.add((*pTITail).impls_off as usize), pTI, FALSE, ref_lookup);
    }

    /* this is necessary to cope with MSFT typelibs that set cFuncs to the
     * number of dispinterface functions including the IDispatch ones, so
     * ITypeInfo::GetFuncDesc takes the real value for cFuncs from cbSizeVft */
    (*pTI).typeattr.cbSizeVft =
        ((*pTI).typeattr.cFuncs as i32 * (*(*pTI).pTypeLib).ptr_size) as WORD;

    heap_free(ref_lookup as *mut c_void);
    /* TRACE_ON dump — compiled out */
}

pub unsafe fn SLTG_ProcessEnum(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *const i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
    hlp_strings: *const u8,
) {
    SLTG_DoVars(
        pBlk,
        pBlk.add((*pTITail).vars_off as usize),
        pTI,
        (*pTITail).cVars,
        pNameTable,
        null(),
        hlp_strings,
    );
}

pub unsafe fn SLTG_ProcessModule(
    pBlk: *mut i8,
    pTI: *mut ITypeInfoImpl,
    pNameTable: *mut i8,
    pTIHeader: *mut SLTG_TypeInfoHeader,
    pTITail: *const SLTG_TypeInfoTail,
    hlp_strings: *const u8,
) {
    let mut ref_lookup: *mut sltg_ref_lookup_t = null_mut();
    let href_table = (*pTIHeader).href_table;
    if href_table != 0xffff_ffff {
        ref_lookup = SLTG_DoRefs(
            (pTIHeader as *mut i8).add(href_table as usize) as *mut SLTG_RefInfo,
            (*pTI).pTypeLib,
            pNameTable,
        );
    }

    if (*pTITail).vars_off != 0xffff {
        SLTG_DoVars(
            pBlk,
            pBlk.add((*pTITail).vars_off as usize),
            pTI,
            (*pTITail).cVars,
            pNameTable,
            ref_lookup,
            hlp_strings,
        );
    }

    if (*pTITail).funcs_off != 0xffff {
        SLTG_DoFuncs(
            pBlk,
            pBlk.add((*pTITail).funcs_off as usize),
            pTI,
            (*pTITail).cFuncs,
            pNameTable,
            ref_lookup,
            hlp_strings,
        );
    }
    heap_free(ref_lookup as *mut c_void);
    /* TRACE_ON dump — compiled out */
}

/// Because SLTG_OtherTypeInfo is such a painful struct, we make a more
/// manageable copy of it into this
#[repr(C)]
struct SLTG_InternalOtherTypeInfo {
    index_name: *mut i8,
    other_name: *mut i8,
    res1a: WORD,
    name_offs: WORD,
    hlpstr_len: WORD,
    extra: *mut i8,
    res20: WORD,
    helpcontext: DWORD,
    res26: WORD,
    uuid: GUID,
    typekind: WORD,
}

/****************************************************************************
 *	ITypeLib2_Constructor_SLTG
 *
 * loading a SLTG typelib from an in-memory image
 */
pub unsafe fn ITypeLib2_Constructor_SLTG(pLib: LPVOID, dwTLBLength: DWORD) -> *mut ITypeLib2 {
    TRACE!();

    let pTypeLibImpl = TypeLibImpl_Constructor();
    if pTypeLibImpl.is_null() {
        return null_mut();
    }

    let pHeader = pLib as *mut SLTG_Header;

    TRACE!();
    let magic = (*pHeader).SLTG_magic;
    if magic != SLTG_SIGNATURE {
        FIXME!("Header type magic 0x{:08x} not supported.", magic);
        return null_mut();
    }

    /* There are pHeader->nrOfFileBlks - 2 TypeInfo records in this typelib */
    (*pTypeLibImpl).TypeInfoCount = (*pHeader).nrOfFileBlks as i32 - 2;

    /* This points to pHeader->nrOfFileBlks - 1 of SLTG_BlkEntry */
    let pBlkEntry = pHeader.add(1) as *mut SLTG_BlkEntry;

    /* Next we have a magic block */
    let pMagic =
        pBlkEntry.add((*pHeader).nrOfFileBlks as usize - 1) as *mut SLTG_Magic;

    /* Let's see if we're still in sync */
    if memcmp(
        &raw const (*pMagic).CompObj_magic as *const c_void,
        SLTG_COMPOBJ_MAGIC.as_ptr() as *const c_void,
        SLTG_COMPOBJ_MAGIC.len(),
    ) != 0
    {
        FIXME!("CompObj magic mismatch");
        return null_mut();
    }
    if memcmp(
        &raw const (*pMagic).dir_magic as *const c_void,
        SLTG_DIR_MAGIC.as_ptr() as *const c_void,
        SLTG_DIR_MAGIC.len(),
    ) != 0
    {
        FIXME!("dir magic mismatch");
        return null_mut();
    }

    let pIndex = pMagic.add(1) as *mut SLTG_Index;

    let pPad9 = pIndex.add((*pTypeLibImpl).TypeInfoCount as usize) as *mut SLTG_Pad9;

    let pFirstBlk = pPad9.add(1) as LPVOID;

    /* We'll set up a ptr to the main library block, which is the last one. */

    let mut pBlk = pFirstBlk;
    let mut order: DWORD = (*pHeader).first_blk as DWORD - 1;
    loop {
        let be: SLTG_BlkEntry = read_unaligned(pBlkEntry.add(order as usize));
        if be.next == 0 {
            break;
        }
        pBlk = (pBlk as *mut i8).add(be.len as usize) as LPVOID;
        order = be.next as DWORD - 1;
    }
    let pLibBlk = pBlk as *mut SLTG_LibBlk;

    let mut len = SLTG_ReadLibBlk(pLibBlk as LPVOID, pTypeLibImpl);

    /* Now there are 0x40 bytes of 0xffff with the numbers 0 to TypeInfoCount
     * interspersed */

    len += 0x40;

    /* And now TypeInfoCount of SLTG_OtherTypeInfo */
    (*pTypeLibImpl).TypeInfoCount =
        read_unaligned((pLibBlk as *mut i8).add(len as usize) as *const WORD) as i32;
    len += size_of::<WORD>() as DWORD;

    let pOtherTypeInfoBlks = heap_alloc_zero(
        size_of::<SLTG_InternalOtherTypeInfo>() * (*pTypeLibImpl).TypeInfoCount as usize,
    ) as *mut SLTG_InternalOtherTypeInfo;

    let mut ptr = (pLibBlk as *mut i8).add(len as usize);

    let mut i: i32 = 0;
    while i < (*pTypeLibImpl).TypeInfoCount {
        let blk = pOtherTypeInfoBlks.add(i as usize);
        let mut len: DWORD = 0;

        let w = read_unaligned(ptr as *const WORD);
        if w != 0xffff {
            len += w as DWORD;
            (*blk).index_name = heap_alloc(w as usize + 1) as *mut i8;
            memcpy(
                (*blk).index_name as *mut c_void,
                ptr.add(2) as *const c_void,
                w as usize,
            );
            *(*blk).index_name.add(w as usize) = 0;
        }
        let w = read_unaligned(ptr.add(2 + len as usize) as *const WORD);
        if w != 0xffff {
            TRACE!();
            (*blk).other_name = heap_alloc(w as usize + 1) as *mut i8;
            memcpy(
                (*blk).other_name as *mut c_void,
                ptr.add(4 + len as usize) as *const c_void,
                w as usize,
            );
            *(*blk).other_name.add(w as usize) = 0;
            len += w as DWORD;
        }
        (*blk).res1a = read_unaligned(ptr.add(4 + len as usize) as *const WORD);
        (*blk).name_offs = read_unaligned(ptr.add(6 + len as usize) as *const WORD);
        let extra = read_unaligned(ptr.add(8 + len as usize) as *const WORD);
        (*blk).hlpstr_len = extra;
        if extra != 0 {
            (*blk).extra = heap_alloc(extra as usize) as *mut i8;
            memcpy(
                (*blk).extra as *mut c_void,
                ptr.add(10 + len as usize) as *const c_void,
                extra as usize,
            );
            len += extra as DWORD;
        }
        (*blk).res20 = read_unaligned(ptr.add(10 + len as usize) as *const WORD);
        (*blk).helpcontext = read_unaligned(ptr.add(12 + len as usize) as *const DWORD);
        (*blk).res26 = read_unaligned(ptr.add(16 + len as usize) as *const WORD);
        memcpy(
            &raw mut (*blk).uuid as *mut c_void,
            ptr.add(18 + len as usize) as *const c_void,
            size_of::<GUID>(),
        );
        (*blk).typekind =
            read_unaligned(ptr.add(18 + size_of::<GUID>() + len as usize) as *const WORD);
        len += size_of::<super::SLTG_OtherTypeInfo>() as DWORD;
        ptr = ptr.add(len as usize);
        i += 1;
    }

    /* Get the next DWORD */
    let len = read_unaligned(ptr as *const DWORD);

    let hlp_strings = ptr.add(size_of::<DWORD>()) as *const u8;
    TRACE!();

    /* Now add this to pLibBLk look at what we're pointing at and possibly add
     * 0x20, then add 0x216, sprinkle a bit a magic dust and we should be
     * pointing at the beginning of the name table */

    let mut pNameTable = (pLibBlk as *mut i8).add(len as usize);

    match read_unaligned(pNameTable as *const WORD) {
        0xffff => {}
        0x0200 => {
            pNameTable = pNameTable.add(0x20); // nolint: magic-offset (SLTG on-disk name-table jump, per wine typelib.c)
        }
        other => {
            FIXME!("pNameTable jump = {:x}", other);
        }
    }

    pNameTable = pNameTable.add(0x216); // nolint: magic-offset (SLTG on-disk name-table base, per wine typelib.c)

    pNameTable = pNameTable.add(2);

    TRACE!();

    (*pTypeLibImpl).Name =
        SLTG_ReadName(pNameTable, (*pLibBlk).name as i32, pTypeLibImpl);

    /* Hopefully we now have enough ptrs set up to actually read in some
     * TypeInfos. It's not clear which order to do them in, so I'll just
     * follow the links along the BlkEntry chain and read them in the order
     * in which they are in the file */

    (*pTypeLibImpl).typeinfos = heap_alloc_zero(
        (*pTypeLibImpl).TypeInfoCount as usize * size_of::<*mut ITypeInfoImpl>(),
    ) as *mut *mut ITypeInfoImpl;
    let mut ppTypeInfoImpl = (*pTypeLibImpl).typeinfos;

    let mut pBlk = pFirstBlk;
    let mut order: DWORD = (*pHeader).first_blk as DWORD - 1;
    let mut i: i32 = 0;
    loop {
        let be: SLTG_BlkEntry = read_unaligned(pBlkEntry.add(order as usize));
        if be.next == 0 {
            break;
        }

        let blk = pOtherTypeInfoBlks.add(i as usize);

        if {
            let index_str = (pMagic as *mut i8).add(be.index_string as usize);
            /* strcmp */
            let mut a = index_str;
            let mut b = (*blk).index_name;
            loop {
                if *a != *b {
                    break true;
                }
                if *a == 0 {
                    break false;
                }
                a = a.add(1);
                b = b.add(1);
            }
        } {
            FIXME!("Index strings don't match");
            heap_free(pOtherTypeInfoBlks as *mut c_void);
            return null_mut();
        }

        let pTIHeader = pBlk as *mut SLTG_TypeInfoHeader;
        let magic = (*pTIHeader).magic;
        if magic != SLTG_TIHEADER_MAGIC {
            FIXME!("TypeInfoHeader magic = {:04x}", magic);
            heap_free(pOtherTypeInfoBlks as *mut c_void);
            return null_mut();
        }
        TRACE!();

        *ppTypeInfoImpl = super::itypeinfo::ITypeInfoImpl_Constructor();
        let ti = *ppTypeInfoImpl;
        (*ti).pTypeLib = pTypeLibImpl;
        (*ti).index = i;
        (*ti).Name = SLTG_ReadName(pNameTable, (*blk).name_offs as i32, pTypeLibImpl);
        (*ti).dwHelpContext = (*blk).helpcontext;
        (*ti).DocString = decode_string(
            hlp_strings,
            (*blk).extra,
            (*blk).hlpstr_len as DWORD,
            pTypeLibImpl,
        );
        (*ti).guid = TLB_append_guid(
            &raw mut (*pTypeLibImpl).guid_list,
            &raw const (*blk).uuid,
            2,
        );
        (*ti).typeattr.typekind = (*pTIHeader).typekind as TYPEKIND;
        (*ti).typeattr.wMajorVerNum = (*pTIHeader).major_version;
        (*ti).typeattr.wMinorVerNum = (*pTIHeader).minor_version;
        (*ti).typeattr.wTypeFlags =
            (((*pTIHeader).typeflags1 as WORD) >> 3) | (((*pTIHeader).typeflags2 as WORD) << 5);

        if (*ti).typeattr.wTypeFlags & TYPEFLAG_FDUAL != 0 {
            (*ti).typeattr.typekind = TKIND_DISPATCH;
        }

        if ((*pTIHeader).typeflags1 & 7) != 2 {
            FIXME!("typeflags1 = {:02x}", (*pTIHeader).typeflags1);
        }
        if (*pTIHeader).typeflags3 != 2 {
            FIXME!("typeflags3 = {:02x}", (*pTIHeader).typeflags3);
        }

        TRACE!();

        let pMemHeader =
            (pBlk as *mut i8).add((*pTIHeader).elem_table as usize) as *mut SLTG_MemberHeader;

        let pTITail = (pMemHeader.add(1) as *mut i8)
            .add({
                let cbe = &raw const (*pMemHeader).cbExtra;
                read_unaligned(cbe) as usize
            }) as *mut SLTG_TypeInfoTail;

        (*ti).typeattr.cbAlignment = (*pTITail).cbAlignment;
        (*ti).typeattr.cbSizeInstance = (*pTITail).cbSizeInstance as ULONG;
        (*ti).typeattr.cbSizeVft = (*pTITail).cbSizeVft;

        match (*pTIHeader).typekind as TYPEKIND {
            k if k == TKIND_ENUM => SLTG_ProcessEnum(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
                hlp_strings,
            ),
            k if k == TKIND_RECORD => SLTG_ProcessRecord(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
                hlp_strings,
            ),
            k if k == TKIND_INTERFACE => SLTG_ProcessInterface(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
                hlp_strings,
            ),
            k if k == TKIND_COCLASS => SLTG_ProcessCoClass(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
            ),
            k if k == TKIND_ALIAS => SLTG_ProcessAlias(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
            ),
            k if k == TKIND_DISPATCH => SLTG_ProcessDispatch(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
                hlp_strings,
            ),
            k if k == TKIND_MODULE => SLTG_ProcessModule(
                pMemHeader.add(1) as *mut i8,
                ti,
                pNameTable,
                pTIHeader,
                pTITail,
                hlp_strings,
            ),
            _ => {
                FIXME!("Not processing typekind {}", (*pTIHeader).typekind);
            }
        }

        /* could get cFuncs, cVars and cImplTypes from here but we've already
         * set those (the X(..) res-field TRACEs are compiled out) */

        ppTypeInfoImpl = ppTypeInfoImpl.add(1);
        pBlk = (pBlk as *mut i8).add(be.len as usize) as LPVOID;
        order = be.next as DWORD - 1;
        i += 1;
    }

    if i != (*pTypeLibImpl).TypeInfoCount {
        FIXME!("Somehow processed {} TypeInfos", i);
        heap_free(pOtherTypeInfoBlks as *mut c_void);
        return null_mut();
    }

    heap_free(pOtherTypeInfoBlks as *mut c_void);
    &raw mut (*pTypeLibImpl).ITypeLib2_iface
}
