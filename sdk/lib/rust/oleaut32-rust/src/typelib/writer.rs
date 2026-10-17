//! writer.rs — CreateTypeLib2, the ICreateTypeLib2 / ICreateTypeInfo2
//! implementations and the WMSFT .tlb serializer (typelib.c lines
//! ~9043–11622) + ClearCustData (11635–11651). Lane A2 continuation, 1:1
//! (the C's own `- stub` FIXMEs stay stubs, matching wine).

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_assignments, unused_imports)]

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut};

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::itypeinfo::{
    ITypeInfoImpl_Constructor, TLB_AllocAndInitVarDesc, TLB_CopyElemDesc, TLB_SanitizeVariant,
    TLB_SizeElemDesc,
};
use super::msft::TypeLibImpl_Constructor;
use super::sys::*;
use super::{
    MSFT_CDGuid, MSFT_FuncRecord, MSFT_GuidEntry, MSFT_Header, MSFT_ImpInfo, MSFT_NameIntro,
    MSFT_ParameterInfo, MSFT_RefRecord, MSFT_SegDir, MSFT_TypeInfoBase, MSFT_VarRecord,
    MSFT_pSeg, HELPDLLFLAG, MSFT_IMPINFO_OFFSET_IS_GUID,
};

pub const TYPE_E_NAMECONFLICT: HRESULT = 0x8002_802Du32 as HRESULT;
pub const TYPE_E_INCONSISTENTPROPFUNCS: HRESULT = 0x8002_9C83u32 as HRESULT;
pub const GENERIC_WRITE: DWORD = 0x4000_0000;
pub const CREATE_ALWAYS: DWORD = 2;
pub const LANG_ENGLISH: u16 = 0x09;
pub const SUBLANG_ENGLISH_US: u16 = 0x01;

extern "system" {
    fn WriteFile(
        hFile: HANDLE, lpBuffer: *const c_void, nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD, lpOverlapped: PVOID,
    ) -> BOOL;
    fn GetSystemDefaultLCID() -> LCID;
    // oleaut32 other TU (hash.c)
    fn LHashValOfNameSysA(syskind: SYSKIND, lcid: LCID, szName: LPCSTR) -> ULONG;
}

/// CreateTypeLib2 (OLEAUT32.180)
#[no_mangle]
pub unsafe extern "system" fn CreateTypeLib2(
    syskind: SYSKIND,
    szFile: LPCOLESTR,
    ppctlib: *mut *mut ICreateTypeLib2,
) -> HRESULT {
    TRACE!();

    if szFile.is_null() {
        return E_INVALIDARG;
    }

    let This = TypeLibImpl_Constructor();
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).lcid = GetSystemDefaultLCID();
    (*This).syskind = syskind;
    (*This).ptr_size = get_ptr_size(syskind);

    (*This).path = heap_alloc((lstrlenW(szFile) as usize + 1) * size_of::<WCHAR>()) as *mut WCHAR;
    if (*This).path.is_null() {
        let tl = &raw mut (*This).ITypeLib2_iface;
        ((*(*tl).lpVtbl).Release)(tl);
        return E_OUTOFMEMORY;
    }
    lstrcpyW((*This).path, szFile);

    let tl = &raw mut (*This).ITypeLib2_iface;
    let hres = ((*(*tl).lpVtbl).QueryInterface)(
        tl,
        &raw const IID_ICreateTypeLib2,
        ppctlib as *mut *mut c_void,
    );
    ((*(*tl).lpVtbl).Release)(tl);
    hres
}

unsafe extern "system" fn ICreateTypeLib2_fnQueryInterface(
    iface: *mut ICreateTypeLib2,
    riid: REFIID,
    object: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    let tl = &raw mut (*This).ITypeLib2_iface;
    ((*(*tl).lpVtbl).QueryInterface)(tl, riid, object)
}

unsafe extern "system" fn ICreateTypeLib2_fnAddRef(iface: *mut ICreateTypeLib2) -> ULONG {
    let This = impl_from_ICreateTypeLib2(iface);
    let tl = &raw mut (*This).ITypeLib2_iface;
    ((*(*tl).lpVtbl).AddRef)(tl)
}

unsafe extern "system" fn ICreateTypeLib2_fnRelease(iface: *mut ICreateTypeLib2) -> ULONG {
    let This = impl_from_ICreateTypeLib2(iface);
    let tl = &raw mut (*This).ITypeLib2_iface;
    ((*(*tl).lpVtbl).Release)(tl)
}

unsafe extern "system" fn ICreateTypeLib2_fnCreateTypeInfo(
    iface: *mut ICreateTypeLib2,
    name: LPOLESTR,
    kind: TYPEKIND,
    ctinfo: *mut *mut ICreateTypeInfo2,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    if ctinfo.is_null() || name.is_null() {
        return E_INVALIDARG;
    }

    let mut info =
        TLB_get_typeinfo_by_name((*This).typeinfos, (*This).TypeInfoCount as UINT, name);
    if !info.is_null() {
        return TYPE_E_NAMECONFLICT;
    }

    if !(*This).typeinfos.is_null() {
        (*This).typeinfos = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).typeinfos as *mut c_void,
            (size_of::<*mut ITypeInfoImpl>() * ((*This).TypeInfoCount as usize + 1)) as _,
        ) as *mut *mut ITypeInfoImpl;
    } else {
        (*This).typeinfos =
            heap_alloc_zero(size_of::<*mut ITypeInfoImpl>()) as *mut *mut ITypeInfoImpl;
    }

    info = ITypeInfoImpl_Constructor();
    *(*This).typeinfos.add((*This).TypeInfoCount as usize) = info;

    (*info).pTypeLib = This;
    (*info).Name = TLB_append_str(&raw mut (*This).name_list, name);
    (*info).index = (*This).TypeInfoCount;
    (*info).typeattr.typekind = kind;
    (*info).typeattr.cbAlignment = 4;

    match (*info).typeattr.typekind {
        k if k == TKIND_ENUM
            || k == TKIND_INTERFACE
            || k == TKIND_DISPATCH
            || k == TKIND_COCLASS =>
        {
            (*info).typeattr.cbSizeInstance = (*This).ptr_size as ULONG;
        }
        k if k == TKIND_RECORD || k == TKIND_UNION => {
            (*info).typeattr.cbSizeInstance = 0;
        }
        k if k == TKIND_MODULE => {
            (*info).typeattr.cbSizeInstance = 2;
        }
        k if k == TKIND_ALIAS => {
            (*info).typeattr.cbSizeInstance = -0x75i32 as ULONG;
        }
        k => {
            FIXME!("unrecognized typekind {}", k);
            (*info).typeattr.cbSizeInstance = 0xdead_beef;
        }
    }

    let ti = &raw mut (*info).ITypeInfo2_iface;
    let hres = ((*(*ti).lpVtbl).QueryInterface)(
        ti,
        &raw const IID_ICreateTypeInfo,
        ctinfo as *mut *mut c_void,
    );
    if FAILED(hres) {
        ((*(*ti).lpVtbl).Release)(ti);
        return hres;
    }

    (*info).hreftype = ((*info).index as usize * size_of::<MSFT_TypeInfoBase>()) as HREFTYPE;

    (*This).TypeInfoCount += 1;

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetName(
    iface: *mut ICreateTypeLib2,
    name: LPOLESTR,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    if name.is_null() {
        return E_INVALIDARG;
    }

    (*This).Name = TLB_append_str(&raw mut (*This).name_list, name);

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetVersion(
    iface: *mut ICreateTypeLib2,
    majorVerNum: WORD,
    minorVerNum: WORD,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    (*This).ver_major = majorVerNum;
    (*This).ver_minor = minorVerNum;

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetGuid(
    iface: *mut ICreateTypeLib2,
    guid: REFGUID,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    (*This).guid = TLB_append_guid(&raw mut (*This).guid_list, guid, -2i32 as HREFTYPE);

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetDocString(
    iface: *mut ICreateTypeLib2,
    doc: LPOLESTR,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    if doc.is_null() {
        return E_INVALIDARG;
    }

    (*This).DocString = TLB_append_str(&raw mut (*This).string_list, doc);

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetHelpFileName(
    iface: *mut ICreateTypeLib2,
    helpFileName: LPOLESTR,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    if helpFileName.is_null() {
        return E_INVALIDARG;
    }

    (*This).HelpFile = TLB_append_str(&raw mut (*This).string_list, helpFileName);

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetHelpContext(
    iface: *mut ICreateTypeLib2,
    helpContext: DWORD,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    TRACE!();
    (*This).dwHelpContext = helpContext;
    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetLcid(
    iface: *mut ICreateTypeLib2,
    lcid: LCID,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    TRACE!();
    (*This).set_lcid = lcid;
    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnSetLibFlags(
    iface: *mut ICreateTypeLib2,
    libFlags: UINT,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    TRACE!();
    (*This).libflags = libFlags as WORD;
    S_OK
}

// ---------------------------------------------------------------------------
// WMSFT .tlb serializer
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct WMSFT_SegContents {
    pub len: DWORD,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct WMSFT_TLBFile {
    pub header: MSFT_Header,
    pub typeinfo_seg: WMSFT_SegContents,
    pub impfile_seg: WMSFT_SegContents,
    pub impinfo_seg: WMSFT_SegContents,
    pub ref_seg: WMSFT_SegContents,
    pub guidhash_seg: WMSFT_SegContents,
    pub guid_seg: WMSFT_SegContents,
    pub namehash_seg: WMSFT_SegContents,
    pub name_seg: WMSFT_SegContents,
    pub string_seg: WMSFT_SegContents,
    pub typdesc_seg: WMSFT_SegContents,
    pub arraydesc_seg: WMSFT_SegContents,
    pub custdata_seg: WMSFT_SegContents,
    pub cdguids_seg: WMSFT_SegContents,
    pub segdir: MSFT_SegDir,
    pub aux_seg: WMSFT_SegContents,
}

unsafe fn WMSFT_compile_strings(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) -> HRESULT {
    (*file).string_seg.len = 0;
    let l = &raw mut (*This).string_list;
    let mut cur = (*l).next;
    while cur != l {
        let str_ = list_entry!(cur, TLBString, entry);

        let mut size = WideCharToMultiByte(
            CP_ACP, 0, (*str_).str_, lstrlenW((*str_).str_), null_mut(), 0, null(), null_mut(),
        );
        if size == 0 {
            return E_UNEXPECTED;
        }

        size += size_of::<INT16>() as i32;
        if size % 4 != 0 {
            size = (size + 4) & !0x3;
        }
        if size < 8 {
            size = 8;
        }

        (*file).string_seg.len += size as DWORD;

        /* temporarily use str->offset to store the length of the aligned,
         * converted string */
        (*str_).offset = size as UINT;
        cur = (*cur).next;
    }

    let mut data = heap_alloc((*file).string_seg.len as usize) as *mut i8;
    (*file).string_seg.data = data as *mut c_void;

    let mut last_offs: UINT = 0;
    let mut cur = (*l).next;
    while cur != l {
        let str_ = list_entry!(cur, TLBString, entry);

        let size = WideCharToMultiByte(
            CP_ACP,
            0,
            (*str_).str_,
            lstrlenW((*str_).str_),
            data.add(size_of::<INT16>()),
            ((*file).string_seg.len - last_offs - size_of::<INT16>() as u32) as INT,
            null(),
            null_mut(),
        );
        if size == 0 {
            heap_free((*file).string_seg.data);
            // Wine 10.0: null it. The caller (SaveAllChanges) runs
            // WMSFT_free_file on failure, which freed it a second time.
            (*file).string_seg.data = null_mut();
            return E_UNEXPECTED;
        }

        *(data as *mut INT16) = size as INT16;

        memset(
            data.add(size_of::<INT16>() + size as usize) as *mut c_void,
            0x57,
            (*str_).offset as usize - size as usize - size_of::<INT16>(),
        );

        let size = (*str_).offset;
        data = data.add(size as usize);
        (*str_).offset = last_offs;
        last_offs += size;
        cur = (*cur).next;
    }

    S_OK
}

unsafe fn WMSFT_compile_names(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) -> HRESULT {
    let mut last_intro: *mut MSFT_NameIntro = null_mut();

    (*file).header.nametablecount = 0;
    (*file).header.nametablechars = 0;

    (*file).name_seg.len = 0;
    let l = &raw mut (*This).name_list;
    let mut cur = (*l).next;
    while cur != l {
        let str_ = list_entry!(cur, TLBString, entry);

        let mut size = lstrlenW((*str_).str_);
        (*file).header.nametablechars += size;
        (*file).header.nametablecount += 1;

        size = WideCharToMultiByte(CP_ACP, 0, (*str_).str_, size, null_mut(), 0, null(), null_mut());
        if size == 0 {
            return E_UNEXPECTED;
        }

        size += size_of::<MSFT_NameIntro>() as i32;
        if size % 4 != 0 {
            size = (size + 4) & !0x3;
        }
        if size < 8 {
            size = 8;
        }

        (*file).name_seg.len += size as DWORD;

        /* temporarily use str->offset to store the length of the aligned,
         * converted string */
        (*str_).offset = size as UINT;
        cur = (*cur).next;
    }

    /* Allocate bigger buffer so we can temporarily NULL terminate the name */
    let mut data = heap_alloc((*file).name_seg.len as usize + 1) as *mut i8;
    (*file).name_seg.data = data as *mut c_void;

    let mut last_offs: UINT = 0;
    let mut cur = (*l).next;
    while cur != l {
        let str_ = list_entry!(cur, TLBString, entry);
        let intro = data as *mut MSFT_NameIntro;

        let size = WideCharToMultiByte(
            CP_ACP,
            0,
            (*str_).str_,
            lstrlenW((*str_).str_),
            data.add(size_of::<MSFT_NameIntro>()),
            ((*file).name_seg.len - last_offs - size_of::<MSFT_NameIntro>() as u32) as INT,
            null(),
            null_mut(),
        );
        if size == 0 {
            heap_free((*file).name_seg.data);
            return E_UNEXPECTED;
        }
        *data.add(size_of::<MSFT_NameIntro>() + size as usize) = 0;

        (*intro).hreftype = -1; /* TODO? */
        (*intro).namelen = size & 0xFF;
        /* TODO: namelen & 0xFF00 == ??? maybe HREF type indicator? */
        let hash = LHashValOfNameSysA(
            (*This).syskind,
            (*This).lcid,
            data.add(size_of::<MSFT_NameIntro>()),
        ) as i32;
        (*intro).namelen |= hash << 16;
        (*intro).next_hash =
            *((*file).namehash_seg.data as *mut i32).add((hash & 0x7f) as usize);
        *((*file).namehash_seg.data as *mut i32).add((hash & 0x7f) as usize) = last_offs as i32;

        memset(
            data.add(size_of::<MSFT_NameIntro>() + size as usize) as *mut c_void,
            0x57,
            (*str_).offset as usize - size as usize - size_of::<MSFT_NameIntro>(),
        );

        /* update str->offset to actual value to use in other compilation
         * functions that require positions within the string table */
        last_intro = intro;
        let size = (*str_).offset;
        data = data.add(size as usize);
        (*str_).offset = last_offs;
        last_offs += size;
        cur = (*cur).next;
    }

    if !last_intro.is_null() {
        (*last_intro).hreftype = 0; /* last one is 0? */
    }

    S_OK
}

#[inline]
unsafe fn hash_guid(guid: *mut GUID) -> i32 {
    let mut hash: i32 = 0;
    let p = guid as *const i16;
    let mut i = 0;
    while i < 8 {
        hash ^= *p.add(i) as i32;
        i += 1;
    }
    hash & 0x1f
}

unsafe fn WMSFT_compile_guids(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) -> HRESULT {
    (*file).guid_seg.len =
        (size_of::<MSFT_GuidEntry>() as u32) * list_count(&raw const (*This).guid_list);
    (*file).guid_seg.data = heap_alloc((*file).guid_seg.len as usize);

    let mut entry = (*file).guid_seg.data as *mut MSFT_GuidEntry;
    let mut offs: DWORD = 0;
    let guidhashtab = (*file).guidhash_seg.data as *mut i32;
    let l = &raw mut (*This).guid_list;
    let mut cur = (*l).next;
    while cur != l {
        let guid = list_entry!(cur, TLBGuid, entry);
        memcpy(
            &raw mut (*entry).guid as *mut c_void,
            &raw const (*guid).guid as *const c_void,
            size_of::<GUID>(),
        );
        (*entry).hreftype = (*guid).hreftype;

        let hash_key = hash_guid(&raw mut (*guid).guid);
        (*entry).next_hash = *guidhashtab.add(hash_key as usize);
        *guidhashtab.add(hash_key as usize) = offs as i32;

        (*guid).offset = offs;
        offs += size_of::<MSFT_GuidEntry>() as DWORD;
        entry = entry.add(1);
        cur = (*cur).next;
    }

    S_OK
}

unsafe fn WMSFT_encode_variant(value: *mut VARIANT, file: *mut WMSFT_TLBFile) -> DWORD {
    let mut v: VARIANT = *value;
    let mut arg_type: VARTYPE = (*value).vt;
    let mut mask: u32 = 0;
    let ret: DWORD = (*file).custdata_seg.len;

    if arg_type == VT_INT {
        arg_type = VT_I4;
    }
    if arg_type == VT_UINT {
        arg_type = VT_UI4;
    }

    v = *value;
    if (*value).vt != arg_type {
        let hres = VariantChangeType(&raw mut v, value, 0, arg_type);
        if FAILED(hres) {
            ERR!("VariantChangeType failed: {:08x}", hres as u32);
            return -1i32 as DWORD;
        }
    }

    /* Check if default value can be stored in-place (replicating the C's
     * case-fallthrough chain) */
    {
        let mut in_place = false;
        match arg_type {
            VT_I4 | VT_UI4 => {
                mask = 0x3ff_ffff;
                if v.n3.ulVal <= 0x3ff_ffff {
                    in_place = true;
                }
            }
            VT_I1 | VT_UI1 | VT_BOOL => {
                mask = 0xff;
                in_place = true;
            }
            VT_I2 | VT_UI2 => {
                mask = 0xffff;
                in_place = true;
            }
            _ => {}
        }
        if in_place {
            return (((0x80u32 + 0x4 * (*value).vt as u32) << 24) | (v.n3.ulVal & mask)) as DWORD;
        }
    }

    /* have to allocate space in custdata_seg */
    match arg_type {
        VT_I4 | VT_R4 | VT_UI4 | VT_INT | VT_UINT | VT_HRESULT | VT_PTR => {
            /* Construct the data to be allocated */
            let data: *mut i32;

            if !(*file).custdata_seg.data.is_null() {
                (*file).custdata_seg.data = heap_realloc(
                    (*file).custdata_seg.data,
                    (*file).custdata_seg.len as usize + size_of::<i32>() * 2,
                );
                data = ((*file).custdata_seg.data as *mut i8)
                    .add((*file).custdata_seg.len as usize) as *mut i32;
                (*file).custdata_seg.len += (size_of::<i32>() * 2) as DWORD;
            } else {
                (*file).custdata_seg.len = (size_of::<i32>() * 2) as DWORD;
                (*file).custdata_seg.data = heap_alloc((*file).custdata_seg.len as usize);
                data = (*file).custdata_seg.data as *mut i32;
            }

            *data = ((*value).vt as u32).wrapping_add(v.n3.ulVal << 16) as i32;
            *data.add(1) = (v.n3.ulVal >> 16).wrapping_add(0x5757_0000) as i32;

            /* TODO: Check if the encoded data is already present in custdata_seg */

            ret
        }

        VT_BSTR => {
            // Wine 10.0: the stored string is the CP_ACP encoding and its
            // length is the ENCODED length. The pre-10 code sized it by
            // SysStringLen and pre-filled '?' for every non-ASCII character,
            // so multibyte code pages wrote the wrong length.
            let bstr = v.n3.bstrVal;
            let mb_len = WideCharToMultiByte(
                CP_ACP,
                0,
                bstr,
                SysStringLen(bstr) as INT,
                null_mut(),
                0,
                null(),
                null_mut(),
            );
            let len: i32 = (6 + mb_len + 3) & !0x3;
            let data: *mut i8;

            if !(*file).custdata_seg.data.is_null() {
                (*file).custdata_seg.data = heap_realloc(
                    (*file).custdata_seg.data,
                    (*file).custdata_seg.len as usize + len as usize,
                );
                data = ((*file).custdata_seg.data as *mut i8)
                    .add((*file).custdata_seg.len as usize);
                (*file).custdata_seg.len += len as DWORD;
            } else {
                (*file).custdata_seg.len = len as DWORD;
                (*file).custdata_seg.data = heap_alloc((*file).custdata_seg.len as usize);
                data = (*file).custdata_seg.data as *mut i8;
            }

            *(data as *mut u16) = (*value).vt;
            core::ptr::write_unaligned(data.add(2) as *mut u32, mb_len as u32);
            WideCharToMultiByte(
                CP_ACP,
                0,
                bstr,
                SysStringLen(bstr) as INT,
                data.add(6),
                mb_len,
                null(),
                null_mut(),
            );
            let mut i = 6 + mb_len;
            while i < len {
                *data.add(i as usize) = 0x57;
                i += 1;
            }

            /* TODO: Check if the encoded data is already present in custdata_seg */

            ret
        }
        _ => {
            FIXME!("Argument type not yet handled");
            -1i32 as DWORD
        }
    }
}

unsafe fn WMSFT_append_arraydesc(desc: *mut ARRAYDESC, file: *mut WMSFT_TLBFile) -> DWORD {
    let offs: DWORD = (*file).arraydesc_seg.len;

    /* TODO: we should check for duplicates, but that's harder because each
     * chunk is variable length (really we should store TYPEDESC and ARRAYDESC
     * at the library-level) */

    let cDims = (*desc).cDims;
    (*file).arraydesc_seg.len += ((2 + cDims as usize * 2) * size_of::<DWORD>()) as DWORD;
    if (*file).arraydesc_seg.data.is_null() {
        (*file).arraydesc_seg.data = heap_alloc((*file).arraydesc_seg.len as usize);
    } else {
        (*file).arraydesc_seg.data =
            heap_realloc((*file).arraydesc_seg.data, (*file).arraydesc_seg.len as usize);
    }
    let encoded =
        ((*file).arraydesc_seg.data as *mut i8).add(offs as usize) as *mut DWORD;

    *encoded = WMSFT_append_typedesc(&raw mut (*desc).tdescElem, file, null_mut(), null_mut());
    *encoded.add(1) =
        cDims as DWORD | (((cDims as usize * 2 * size_of::<DWORD>()) as DWORD) << 16);
    let mut i: USHORT = 0;
    while i < cDims {
        let bound = (&raw const (*desc).rgbounds).cast::<SAFEARRAYBOUND>().add(i as usize);
        *encoded.add(2 + i as usize * 2) = (*bound).cElements;
        *encoded.add(2 + i as usize * 2 + 1) = (*bound).lLbound as DWORD;
        i += 1;
    }

    offs
}

unsafe fn WMSFT_append_typedesc(
    desc: *mut TYPEDESC,
    file: *mut WMSFT_TLBFile,
    out_mix: *mut DWORD,
    out_size: *mut INT16,
) -> DWORD {
    let mut junk: DWORD = 0;
    let mut junk2: INT16 = 0;
    let mut offs: DWORD = 0;
    let mut encoded: [DWORD; 2] = [0; 2];

    if desc.is_null() {
        return -1i32 as DWORD;
    }

    let out_mix = if out_mix.is_null() { &raw mut junk } else { out_mix };
    let out_size = if out_size.is_null() { &raw mut junk2 } else { out_size };

    let vt: VARTYPE = (*desc).vt & VT_TYPEMASK;

    if vt == VT_PTR || vt == VT_SAFEARRAY {
        let mut mix: DWORD = 0;
        encoded[1] = WMSFT_append_typedesc((*desc).u.lptdesc, file, &raw mut mix, out_size);
        encoded[0] = (*desc).vt as DWORD | ((mix | VT_BYREF as DWORD) << 16);
        *out_mix = 0x7FFF;
        *out_size += (2 * size_of::<DWORD>()) as INT16;
    } else if vt == VT_CARRAY {
        encoded[0] = (*desc).vt as DWORD | (0x7FFEu32 << 16);
        encoded[1] = WMSFT_append_arraydesc((*desc).u.lpadesc, file);
        *out_mix = 0x7FFE;
    } else if vt == VT_USERDEFINED {
        encoded[0] = (*desc).vt as DWORD | (0x7FFFu32 << 16);
        encoded[1] = (*desc).u.hreftype;
        *out_mix = 0x7FFF; /* FIXME: Should get TYPEKIND of the hreftype, e.g. TKIND_ENUM => VT_I4 */
    } else {
        TRACE!();

        let subtype: VARTYPE = match vt {
            VT_INT => VT_I4,
            VT_UINT => VT_UI4,
            VT_VOID => VT_EMPTY,
            _ => vt,
        };

        *out_mix = subtype as DWORD;
        return 0x8000_0000 | ((subtype as DWORD) << 16) | (*desc).vt as DWORD;
    }

    let mut data = (*file).typdesc_seg.data as *mut i8;
    while offs < (*file).typdesc_seg.len {
        if memcmp(
            data.add(offs as usize) as *const c_void,
            encoded.as_ptr() as *const c_void,
            size_of::<[DWORD; 2]>(),
        ) == 0
        {
            return offs;
        }
        offs += size_of::<[DWORD; 2]>() as DWORD;
    }

    (*file).typdesc_seg.len += size_of::<[DWORD; 2]>() as DWORD;
    if (*file).typdesc_seg.data.is_null() {
        (*file).typdesc_seg.data = heap_alloc((*file).typdesc_seg.len as usize);
    } else {
        (*file).typdesc_seg.data =
            heap_realloc((*file).typdesc_seg.data, (*file).typdesc_seg.len as usize);
    }
    data = (*file).typdesc_seg.data as *mut i8;

    memcpy(
        data.add(offs as usize) as *mut c_void,
        encoded.as_ptr() as *const c_void,
        size_of::<[DWORD; 2]>(),
    );

    offs
}

unsafe fn WMSFT_compile_custdata(custdata_list: *mut list, file: *mut WMSFT_TLBFile) -> DWORD {
    let cdguids_seg = &raw mut (*file).cdguids_seg;
    let ret: DWORD = (*cdguids_seg).len;

    if list_empty(custdata_list) != 0 {
        return -1i32 as DWORD;
    }

    (*cdguids_seg).len += (size_of::<MSFT_CDGuid>() as u32) * list_count(custdata_list);
    let mut cdguid: *mut MSFT_CDGuid;
    if (*cdguids_seg).data.is_null() {
        (*cdguids_seg).data = heap_alloc((*cdguids_seg).len as usize);
        cdguid = (*cdguids_seg).data as *mut MSFT_CDGuid;
    } else {
        (*cdguids_seg).data = heap_realloc((*cdguids_seg).data, (*cdguids_seg).len as usize);
        cdguid = ((*cdguids_seg).data as *mut i8).add(ret as usize) as *mut MSFT_CDGuid;
    }

    let mut offs: DWORD = ret + size_of::<MSFT_CDGuid>() as DWORD;
    let mut cur = (*custdata_list).next;
    while cur != custdata_list {
        let cd = list_entry!(cur, TLBCustData, entry);
        (*cdguid).GuidOffset = (*(*cd).guid).offset as i32;
        (*cdguid).DataOffset = WMSFT_encode_variant(&raw mut (*cd).data, file) as i32;
        (*cdguid).next = offs as i32;
        offs += size_of::<MSFT_CDGuid>() as DWORD;
        cdguid = cdguid.add(1);
        cur = (*cur).next;
    }

    cdguid = cdguid.sub(1);
    (*cdguid).next = -1;

    ret
}

unsafe fn WMSFT_compile_typeinfo_aux(info: *mut ITypeInfoImpl, file: *mut WMSFT_TLBFile) -> DWORD {
    let aux_seg = &raw mut (*file).aux_seg;
    let ret: DWORD = (*aux_seg).len;
    let mut recorded_size: DWORD = 0;
    let mut extra_size: DWORD = 0;

    let mut i: DWORD = 0;
    while i < (*info).typeattr.cFuncs as DWORD {
        let desc = (*info).funcdescs.add(i as usize);

        recorded_size += 6 * size_of::<INT>() as DWORD; /* mandatory fields */

        /* optional fields */
        /* TODO: oArgCustData - FuncSetCustData not impl yet */
        if list_empty(&raw const (*desc).custdata_list) == 0 {
            recorded_size += 7 * size_of::<INT>() as DWORD;
        } else if (*desc).HelpStringContext != 0 {
            recorded_size += 6 * size_of::<INT>() as DWORD;
        }
        /* res9? resA? */
        else if !(*desc).Entry.is_null() {
            recorded_size += 3 * size_of::<INT>() as DWORD;
        } else if !(*desc).HelpString.is_null() {
            recorded_size += 2 * size_of::<INT>() as DWORD;
        } else if (*desc).helpcontext != 0 {
            recorded_size += size_of::<INT>() as DWORD;
        }

        recorded_size +=
            (*desc).funcdesc.cParams as DWORD * size_of::<MSFT_ParameterInfo>() as DWORD;

        let mut j: i32 = 0;
        while j < (*desc).funcdesc.cParams as i32 {
            if ((*(*desc).funcdesc.lprgelemdescParam.add(j as usize))
                .u
                .paramdesc
                .wParamFlags
                & PARAMFLAG_FHASDEFAULT)
                != 0
            {
                recorded_size += (*desc).funcdesc.cParams as DWORD * size_of::<INT>() as DWORD;
                break;
            }
            j += 1;
        }

        extra_size += 2 * size_of::<INT>() as DWORD; /* memberid, name offs */
        i += 1;
    }

    let mut i: DWORD = 0;
    while i < (*info).typeattr.cVars as DWORD {
        let desc = (*info).vardescs.add(i as usize);

        recorded_size += 5 * size_of::<INT>() as DWORD; /* mandatory fields */

        /* optional fields */
        if (*desc).HelpStringContext != 0 {
            recorded_size += 5 * size_of::<INT>() as DWORD;
        } else if list_empty(&raw const (*desc).custdata_list) == 0 {
            recorded_size += 4 * size_of::<INT>() as DWORD;
        }
        /* res9? */
        else if !(*desc).HelpString.is_null() {
            recorded_size += 2 * size_of::<INT>() as DWORD;
        } else if (*desc).HelpContext != 0 {
            recorded_size += size_of::<INT>() as DWORD;
        }

        extra_size += 2 * size_of::<INT>() as DWORD; /* memberid, name offs */
        i += 1;
    }

    if recorded_size == 0 && extra_size == 0 {
        return ret;
    }

    extra_size += size_of::<INT>() as DWORD; /* total aux size for this typeinfo */

    (*aux_seg).len += recorded_size + extra_size;

    (*aux_seg).len += (size_of::<INT>() as DWORD)
        * ((*info).typeattr.cVars as DWORD + (*info).typeattr.cFuncs as DWORD); /* offsets at the end */

    if !(*aux_seg).data.is_null() {
        (*aux_seg).data = heap_realloc((*aux_seg).data, (*aux_seg).len as usize);
    } else {
        (*aux_seg).data = heap_alloc((*aux_seg).len as usize);
    }

    *(((*aux_seg).data as *mut i8).add(ret as usize) as *mut DWORD) = recorded_size;

    let mut offsets = ((*aux_seg).data as *mut i8)
        .add((ret + recorded_size + extra_size) as usize) as *mut DWORD;
    let mut offs: DWORD = 0;

    let mut funcrecord = ((*aux_seg).data as *mut i8)
        .add((ret + size_of::<INT>() as DWORD) as usize) as *mut MSFT_FuncRecord;
    let mut i: DWORD = 0;
    while i < (*info).typeattr.cFuncs as DWORD {
        let desc = (*info).funcdescs.add(i as usize);
        let mut size: DWORD = 6 * size_of::<INT>() as DWORD;
        let mut paramdefault_size: DWORD = 0;

        (*funcrecord).funcdescsize = (size_of::<FUNCDESC>()
            + (*desc).funcdesc.cParams as usize * size_of::<ELEMDESC>())
            as INT16;
        (*funcrecord).DataType = WMSFT_append_typedesc(
            &raw mut (*desc).funcdesc.elemdescFunc.tdesc,
            file,
            null_mut(),
            &raw mut (*funcrecord).funcdescsize,
        ) as i32;
        (*funcrecord).Flags = (*desc).funcdesc.wFuncFlags as i32;
        (*funcrecord).VtableOffset = (*desc).funcdesc.oVft;

        /* FKCCIC:
         * XXXX XXXX XXXX XXXX  XXXX XXXX XXXX XXXX
         *                                      ^^^funckind
         *                                 ^^^ ^invkind
         *                                ^has_cust_data
         *                           ^^^^callconv
         *                         ^has_param_defaults
         *                        ^oEntry_is_intresource
         */
        (*funcrecord).FKCCIC = (*desc).funcdesc.funckind
            | ((*desc).funcdesc.invkind << 3)
            | (if list_empty(&raw const (*desc).custdata_list) != 0 { 0 } else { 0x80 })
            | ((*desc).funcdesc.callconv << 8);

        if !(*desc).Entry.is_null()
            && (*desc).Entry != -1isize as usize as *const TLBString
            && IS_INTRESOURCE_PTR((*desc).Entry as *const c_void)
        {
            (*funcrecord).FKCCIC |= 0x2000;
        }

        let mut j: i32 = 0;
        while j < (*desc).funcdesc.cParams as i32 {
            if ((*(*desc).funcdesc.lprgelemdescParam.add(j as usize))
                .u
                .paramdesc
                .wParamFlags
                & PARAMFLAG_FHASDEFAULT)
                != 0
            {
                paramdefault_size =
                    (size_of::<INT>() as DWORD) * (*desc).funcdesc.cParams as DWORD;
                (*funcrecord).funcdescsize += size_of::<PARAMDESCEX>() as INT16;
            }
            j += 1;
        }
        if paramdefault_size > 0 {
            (*funcrecord).FKCCIC |= 0x1000;
        }

        (*funcrecord).nrargs = (*desc).funcdesc.cParams;
        (*funcrecord).nroargs = (*desc).funcdesc.cParamsOpt;

        /* optional fields */
        /* res9? resA? */
        if list_empty(&raw const (*desc).custdata_list) == 0 {
            size += 7 * size_of::<INT>() as DWORD;
            (*funcrecord).HelpContext = (*desc).helpcontext;
            if !(*desc).HelpString.is_null() {
                (*funcrecord).oHelpString = (*(*desc).HelpString).offset as i32;
            } else {
                (*funcrecord).oHelpString = -1;
            }
            if (*desc).Entry.is_null() {
                (*funcrecord).oEntry = -1;
            } else if IS_INTRESOURCE_PTR((*desc).Entry as *const c_void) {
                (*funcrecord).oEntry = ((*desc).Entry as usize & 0xffff) as i32;
            } else {
                (*funcrecord).oEntry = (*(*desc).Entry).offset as i32;
            }
            (*funcrecord).res9 = -1;
            (*funcrecord).resA = -1;
            (*funcrecord).HelpStringContext = (*desc).HelpStringContext;
            (*funcrecord).oCustData =
                WMSFT_compile_custdata(&raw mut (*desc).custdata_list, file) as i32;
        } else if (*desc).HelpStringContext != 0 {
            size += 6 * size_of::<INT>() as DWORD;
            (*funcrecord).HelpContext = (*desc).helpcontext;
            if !(*desc).HelpString.is_null() {
                (*funcrecord).oHelpString = (*(*desc).HelpString).offset as i32;
            } else {
                (*funcrecord).oHelpString = -1;
            }
            if (*desc).Entry.is_null() {
                (*funcrecord).oEntry = -1;
            } else if IS_INTRESOURCE_PTR((*desc).Entry as *const c_void) {
                (*funcrecord).oEntry = ((*desc).Entry as usize & 0xffff) as i32;
            } else {
                (*funcrecord).oEntry = (*(*desc).Entry).offset as i32;
            }
            (*funcrecord).res9 = -1;
            (*funcrecord).resA = -1;
            (*funcrecord).HelpStringContext = (*desc).HelpStringContext;
        } else if !(*desc).Entry.is_null() {
            size += 3 * size_of::<INT>() as DWORD;
            (*funcrecord).HelpContext = (*desc).helpcontext;
            if !(*desc).HelpString.is_null() {
                (*funcrecord).oHelpString = (*(*desc).HelpString).offset as i32;
            } else {
                (*funcrecord).oHelpString = -1;
            }
            if (*desc).Entry.is_null() {
                (*funcrecord).oEntry = -1;
            } else if IS_INTRESOURCE_PTR((*desc).Entry as *const c_void) {
                (*funcrecord).oEntry = ((*desc).Entry as usize & 0xffff) as i32;
            } else {
                (*funcrecord).oEntry = (*(*desc).Entry).offset as i32;
            }
        } else if !(*desc).HelpString.is_null() {
            size += 2 * size_of::<INT>() as DWORD;
            (*funcrecord).HelpContext = (*desc).helpcontext;
            (*funcrecord).oHelpString = (*(*desc).HelpString).offset as i32;
        } else if (*desc).helpcontext != 0 {
            size += size_of::<INT>() as DWORD;
            (*funcrecord).HelpContext = (*desc).helpcontext;
        }

        let mut paramdefault = (funcrecord as *mut i8).add(size as usize) as *mut DWORD;
        size += paramdefault_size;

        let mut j: i32 = 0;
        while j < (*desc).funcdesc.cParams as i32 {
            let pinfo = (funcrecord as *mut i8).add(size as usize) as *mut MSFT_ParameterInfo;

            (*pinfo).DataType = WMSFT_append_typedesc(
                &raw mut (*(*desc).funcdesc.lprgelemdescParam.add(j as usize)).tdesc,
                file,
                null_mut(),
                &raw mut (*funcrecord).funcdescsize,
            ) as i32;
            if !(*(*desc).pParamDesc.add(j as usize)).Name.is_null() {
                (*pinfo).oName =
                    (*(*(*desc).pParamDesc.add(j as usize)).Name).offset as i32;
            } else {
                (*pinfo).oName = -1;
            }
            (*pinfo).Flags = (*(*desc).funcdesc.lprgelemdescParam.add(j as usize))
                .u
                .paramdesc
                .wParamFlags as i32;

            if paramdefault_size != 0 {
                if ((*(*desc).funcdesc.lprgelemdescParam.add(j as usize))
                    .u
                    .paramdesc
                    .wParamFlags
                    & PARAMFLAG_FHASDEFAULT)
                    != 0
                {
                    *paramdefault = WMSFT_encode_variant(
                        &raw mut (*(*(*desc).funcdesc.lprgelemdescParam.add(j as usize))
                            .u
                            .paramdesc
                            .pparamdescex)
                            .varDefaultValue,
                        file,
                    );
                } else {
                    *paramdefault = -1i32 as DWORD;
                }
                paramdefault = paramdefault.add(1);
            }

            size += size_of::<MSFT_ParameterInfo>() as DWORD;
            j += 1;
        }

        (*funcrecord).Info = (size | (i << 16)) as i32; /* is it just the index? */

        *offsets = offs;
        offs += size;
        offsets = offsets.add(1);

        funcrecord = (funcrecord as *mut i8).add(size as usize) as *mut MSFT_FuncRecord;
        i += 1;
    }

    let mut varrecord = funcrecord as *mut MSFT_VarRecord;
    let mut i: DWORD = 0;
    while i < (*info).typeattr.cVars as DWORD {
        let desc = (*info).vardescs.add(i as usize);
        let mut size: DWORD = 5 * size_of::<INT>() as DWORD;

        (*varrecord).vardescsize = size_of::<VARDESC>() as INT16;
        (*varrecord).DataType = WMSFT_append_typedesc(
            &raw mut (*desc).vardesc.elemdescVar.tdesc,
            file,
            null_mut(),
            &raw mut (*varrecord).vardescsize,
        ) as i32;
        (*varrecord).Flags = (*desc).vardesc.wVarFlags as i32;
        (*varrecord).VarKind = (*desc).vardesc.varkind as i16;

        if (*desc).vardesc.varkind == VAR_CONST {
            (*varrecord).vardescsize += size_of::<VARIANT>() as INT16;
            (*varrecord).OffsValue =
                WMSFT_encode_variant((*desc).vardesc.u.lpvarValue, file) as i32;
        } else {
            (*varrecord).OffsValue = (*desc).vardesc.u.oInst as i32;
        }

        /* res9? */
        if (*desc).HelpStringContext != 0 {
            size += 5 * size_of::<INT>() as DWORD;
            (*varrecord).HelpContext = (*desc).HelpContext;
            if !(*desc).HelpString.is_null() {
                (*varrecord).HelpString = (*(*desc).HelpString).offset as i32;
            } else {
                (*varrecord).HelpString = -1;
            }
            (*varrecord).res9 = -1;
            (*varrecord).oCustData =
                WMSFT_compile_custdata(&raw mut (*desc).custdata_list, file) as i32;
            (*varrecord).HelpStringContext = (*desc).HelpStringContext;
        } else if list_empty(&raw const (*desc).custdata_list) == 0 {
            size += 4 * size_of::<INT>() as DWORD;
            (*varrecord).HelpContext = (*desc).HelpContext;
            if !(*desc).HelpString.is_null() {
                (*varrecord).HelpString = (*(*desc).HelpString).offset as i32;
            } else {
                (*varrecord).HelpString = -1;
            }
            (*varrecord).res9 = -1;
            (*varrecord).oCustData =
                WMSFT_compile_custdata(&raw mut (*desc).custdata_list, file) as i32;
        } else if !(*desc).HelpString.is_null() {
            size += 2 * size_of::<INT>() as DWORD;
            (*varrecord).HelpContext = (*desc).HelpContext;
            if !(*desc).HelpString.is_null() {
                (*varrecord).HelpString = (*(*desc).HelpString).offset as i32;
            } else {
                (*varrecord).HelpString = -1;
            }
        } else if (*desc).HelpContext != 0 {
            size += size_of::<INT>() as DWORD;
            (*varrecord).HelpContext = (*desc).HelpContext;
        }

        (*varrecord).Info = (size | (i << 16)) as i32;

        *offsets = offs;
        offs += size;
        offsets = offsets.add(1);

        varrecord = (varrecord as *mut i8).add(size as usize) as *mut MSFT_VarRecord;
        i += 1;
    }

    let mut memid = varrecord as *mut MEMBERID;
    let mut i: DWORD = 0;
    while i < (*info).typeattr.cFuncs as DWORD {
        let desc = (*info).funcdescs.add(i as usize);
        *memid = (*desc).funcdesc.memid;
        memid = memid.add(1);
        i += 1;
    }
    let mut i: DWORD = 0;
    while i < (*info).typeattr.cVars as DWORD {
        let desc = (*info).vardescs.add(i as usize);
        *memid = (*desc).vardesc.memid;
        memid = memid.add(1);
        i += 1;
    }

    let mut name = memid as *mut UINT;
    let mut i: DWORD = 0;
    while i < (*info).typeattr.cFuncs as DWORD {
        let desc = (*info).funcdescs.add(i as usize);
        if !(*desc).Name.is_null() {
            *name = (*(*desc).Name).offset;
        } else {
            *name = -1i32 as UINT;
        }
        name = name.add(1);
        i += 1;
    }
    let mut i: DWORD = 0;
    while i < (*info).typeattr.cVars as DWORD {
        let desc = (*info).vardescs.add(i as usize);
        if !(*desc).Name.is_null() {
            *name = (*(*desc).Name).offset;
        } else {
            *name = -1i32 as UINT;
        }
        name = name.add(1);
        i += 1;
    }

    ret
}

#[repr(C)]
struct WMSFT_RefChunk {
    href: DWORD,
    res04: DWORD,
    res08: DWORD,
    next: DWORD,
}

unsafe fn WMSFT_compile_typeinfo_ref(info: *mut ITypeInfoImpl, file: *mut WMSFT_TLBFile) -> DWORD {
    let offs: DWORD = (*file).ref_seg.len;

    (*file).ref_seg.len +=
        (*info).typeattr.cImplTypes as DWORD * size_of::<WMSFT_RefChunk>() as DWORD;
    if (*file).ref_seg.data.is_null() {
        (*file).ref_seg.data = heap_alloc((*file).ref_seg.len as usize);
    } else {
        (*file).ref_seg.data = heap_realloc((*file).ref_seg.data, (*file).ref_seg.len as usize);
    }

    let mut chunk = ((*file).ref_seg.data as *mut i8).add(offs as usize) as *mut WMSFT_RefChunk;

    let mut i: DWORD = 0;
    while i < (*info).typeattr.cImplTypes as DWORD {
        (*chunk).href = (*(*info).impltypes.add(i as usize)).hRef;
        (*chunk).res04 = (*(*info).impltypes.add(i as usize)).implflags as DWORD;
        (*chunk).res08 = -1i32 as DWORD;
        if i < (*info).typeattr.cImplTypes as DWORD - 1 {
            (*chunk).next = offs + size_of::<WMSFT_RefChunk>() as DWORD * (i + 1);
        } else {
            (*chunk).next = -1i32 as DWORD;
        }
        chunk = chunk.add(1);
        i += 1;
    }

    offs
}

unsafe fn WMSFT_compile_typeinfo(
    info: *mut ITypeInfoImpl,
    index: INT16,
    file: *mut WMSFT_TLBFile,
    data: *mut i8,
) -> DWORD {
    let size = size_of::<MSFT_TypeInfoBase>() as DWORD;

    if !data.is_null() {
        let base = data as *mut MSFT_TypeInfoBase;
        if (*info).typeattr.wTypeFlags & TYPEFLAG_FDUAL != 0 {
            (*base).typekind = TKIND_DISPATCH;
        } else {
            (*base).typekind = (*info).typeattr.typekind;
        }
        (*base).typekind |= (index as i32) << 16; /* TODO: There are some other flags here */
        (*base).typekind |= ((*info).typeattr.cbAlignment as i32) << 11
            | ((*info).typeattr.cbAlignment as i32) << 6;
        (*base).memoffset = WMSFT_compile_typeinfo_aux(info, file) as i32;
        (*base).res2 = 0;
        (*base).res3 = 0;
        (*base).res4 = 3;
        (*base).res5 = 0;
        (*base).cElement =
            (((*info).typeattr.cVars as i32) << 16) | (*info).typeattr.cFuncs as i32;
        (*base).res7 = 0;
        (*base).res8 = 0;
        (*base).res9 = 0;
        (*base).resA = 0;
        if !(*info).guid.is_null() {
            (*base).posguid = (*(*info).guid).offset as i32;
        } else {
            (*base).posguid = -1;
        }
        (*base).flags = (*info).typeattr.wTypeFlags as i32;
        if !(*info).Name.is_null() {
            (*base).NameOffset = (*(*info).Name).offset as i32;

            *((*file).name_seg.data as *mut u8).add((*(*info).Name).offset as usize + 9) = 0x38;
            core::ptr::write_unaligned(
                ((*file).name_seg.data as *mut u8).add((*(*info).Name).offset as usize)
                    as *mut HREFTYPE,
                (*info).hreftype,
            );
        } else {
            (*base).NameOffset = -1;
        }
        (*base).version = (((*info).typeattr.wMinorVerNum as i32) << 16)
            | (*info).typeattr.wMajorVerNum as i32;
        if !(*info).DocString.is_null() {
            (*base).docstringoffs = (*(*info).DocString).offset as i32;
        } else {
            (*base).docstringoffs = -1;
        }
        (*base).helpstringcontext = (*info).dwHelpStringContext as i32;
        (*base).helpcontext = (*info).dwHelpContext as i32;
        (*base).oCustData = WMSFT_compile_custdata((*info).pcustdata_list, file) as i32;
        (*base).cImplTypes = (*info).typeattr.cImplTypes as i16;
        (*base).cbSizeVft = (*info).typeattr.cbSizeVft as i16;
        (*base).size = (*info).typeattr.cbSizeInstance as i32;
        if (*info).typeattr.typekind == TKIND_COCLASS {
            (*base).datatype1 = WMSFT_compile_typeinfo_ref(info, file) as i32;
        } else if (*info).typeattr.typekind == TKIND_ALIAS {
            (*base).datatype1 =
                WMSFT_append_typedesc((*info).tdescAlias, file, null_mut(), null_mut()) as i32;
        } else if (*info).typeattr.typekind == TKIND_MODULE {
            if !(*info).DllName.is_null() {
                (*base).datatype1 = (*(*info).DllName).offset as i32;
            } else {
                (*base).datatype1 = -1;
            }
        } else {
            if (*info).typeattr.cImplTypes > 0 {
                (*base).datatype1 = (*(*info).impltypes).hRef as i32;
            } else {
                (*base).datatype1 = -1;
            }
        }
        (*base).datatype2 = index as i32; /* FIXME: i think there's more here */
        (*base).res18 = 0;
        (*base).res19 = -1;
    }

    size
}

unsafe fn WMSFT_compile_typeinfo_seg(
    This: *mut ITypeLibImpl,
    file: *mut WMSFT_TLBFile,
    mut junk: *mut DWORD,
) {
    (*file).typeinfo_seg.len = 0;
    let mut i: i32 = 0;
    while i < (*This).TypeInfoCount {
        let info = *(*This).typeinfos.add(i as usize);
        *junk = (*file).typeinfo_seg.len;
        junk = junk.add(1);
        (*file).typeinfo_seg.len +=
            WMSFT_compile_typeinfo(info, i as INT16, null_mut(), null_mut());
        i += 1;
    }

    (*file).typeinfo_seg.data = heap_alloc((*file).typeinfo_seg.len as usize);
    memset((*file).typeinfo_seg.data, 0x96, (*file).typeinfo_seg.len as usize);

    (*file).aux_seg.len = 0;
    (*file).aux_seg.data = null_mut();

    (*file).typeinfo_seg.len = 0;
    let mut i: i32 = 0;
    while i < (*This).TypeInfoCount {
        let info = *(*This).typeinfos.add(i as usize);
        (*file).typeinfo_seg.len += WMSFT_compile_typeinfo(
            info,
            i as INT16,
            file,
            ((*file).typeinfo_seg.data as *mut i8).add((*file).typeinfo_seg.len as usize),
        );
        i += 1;
    }
}

#[repr(C)]
struct WMSFT_ImpFile {
    guid_offs: INT,
    lcid: LCID,
    version: DWORD,
}

unsafe fn WMSFT_compile_impfile(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) {
    let mut last_offs: DWORD = 0;

    (*file).impfile_seg.len = 0;
    let l = &raw mut (*This).implib_list;
    let mut cur = (*l).next;
    while cur != l {
        let implib = list_entry!(cur, TLBImpLib, entry);
        let mut size: i32 = 0;

        if !(*implib).name.is_null() {
            let mut path = wcsrchr((*implib).name, '\\' as u16);
            if !path.is_null() {
                path = path.add(1);
            } else {
                path = (*implib).name;
            }
            size = WideCharToMultiByte(
                CP_ACP, 0, path, lstrlenW(path), null_mut(), 0, null(), null_mut(),
            );
            if size == 0 {
                ERR!("failed to convert wide string: {}", DbgW(path));
            }
        }

        size += size_of::<INT16>() as i32;
        if size % 4 != 0 {
            size = (size + 4) & !0x3;
        }
        if size < 8 {
            size = 8;
        }

        (*file).impfile_seg.len += size_of::<WMSFT_ImpFile>() as DWORD + size as DWORD;
        cur = (*cur).next;
    }

    let mut data = heap_alloc((*file).impfile_seg.len as usize) as *mut i8;
    (*file).impfile_seg.data = data as *mut c_void;

    let mut cur = (*l).next;
    while cur != l {
        let implib = list_entry!(cur, TLBImpLib, entry);
        let mut strlen_: i32 = 0;

        let impfile = data as *mut WMSFT_ImpFile;
        (*impfile).guid_offs = (*(*implib).guid).offset as INT;
        (*impfile).lcid = (*implib).lcid;
        (*impfile).version =
            (((*implib).wVersionMinor as DWORD) << 16) | (*implib).wVersionMajor as DWORD;

        data = data.add(size_of::<WMSFT_ImpFile>());

        if !(*implib).name.is_null() {
            let mut path = wcsrchr((*implib).name, '\\' as u16);
            if !path.is_null() {
                path = path.add(1);
            } else {
                path = (*implib).name;
            }
            strlen_ = WideCharToMultiByte(
                CP_ACP,
                0,
                path,
                lstrlenW(path),
                data.add(size_of::<INT16>()),
                ((*file).impfile_seg.len - last_offs - size_of::<INT16>() as u32) as INT,
                null(),
                null_mut(),
            );
            if strlen_ == 0 {
                ERR!("failed to convert wide string: {}", DbgW(path));
            }
        }

        *(data as *mut INT16) = ((strlen_ << 2) | 1) as INT16; /* FIXME: is that a flag, or what? */

        let mut size = strlen_ + size_of::<INT16>() as i32;
        if size % 4 != 0 {
            size = (size + 4) & !0x3;
        }
        if size < 8 {
            size = 8;
        }
        memset(
            data.add(size_of::<INT16>() + strlen_ as usize) as *mut c_void,
            0x57,
            (size - strlen_ - size_of::<INT16>() as i32) as usize,
        );

        data = data.add(size as usize);
        (*implib).offset = last_offs as i32;
        last_offs += size as DWORD + size_of::<WMSFT_ImpFile>() as DWORD;
        cur = (*cur).next;
    }
}

unsafe fn WMSFT_compile_impinfo(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) {
    let mut i: UINT = 0;

    WMSFT_compile_impfile(This, file);

    (*file).impinfo_seg.len =
        (size_of::<MSFT_ImpInfo>() as u32) * list_count(&raw const (*This).ref_list);
    let mut info = heap_alloc((*file).impinfo_seg.len as usize) as *mut MSFT_ImpInfo;
    (*file).impinfo_seg.data = info as *mut c_void;

    let l = &raw mut (*This).ref_list;
    let mut cur = (*l).next;
    while cur != l {
        let ref_type = list_entry!(cur, TLBRefType, entry);
        (*info).flags = i as i32 | (((*ref_type).tkind & 0xFF) << 24);
        if (*ref_type).index == TLB_REF_USE_GUID {
            (*info).flags |= MSFT_IMPINFO_OFFSET_IS_GUID;
            (*info).oGuid = (*(*ref_type).guid).offset as i32;
        } else {
            (*info).oGuid = (*ref_type).index;
        }
        (*info).oImpFile = (*(*ref_type).pImpTLInfo).offset;
        i += 1;
        info = info.add(1);
        cur = (*cur).next;
    }
}

unsafe fn WMSFT_compile_guidhash(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) {
    (*file).guidhash_seg.len = 0x80;
    (*file).guidhash_seg.data = heap_alloc((*file).guidhash_seg.len as usize);
    memset((*file).guidhash_seg.data, 0xFF, (*file).guidhash_seg.len as usize);
}

unsafe fn WMSFT_compile_namehash(This: *mut ITypeLibImpl, file: *mut WMSFT_TLBFile) {
    (*file).namehash_seg.len = 0x200;
    (*file).namehash_seg.data = heap_alloc((*file).namehash_seg.len as usize);
    memset((*file).namehash_seg.data, 0xFF, (*file).namehash_seg.len as usize);
}

unsafe fn tmp_fill_segdir_seg(
    segdir: *mut MSFT_pSeg,
    contents: *mut WMSFT_SegContents,
    running_offset: *mut DWORD,
) {
    if !contents.is_null() && (*contents).len != 0 {
        (*segdir).offset = *running_offset as i32;
        (*segdir).length = (*contents).len as i32;
        *running_offset += (*segdir).length as DWORD;
    } else {
        (*segdir).offset = -1;
        (*segdir).length = 0;
    }

    /* TODO: do these ever change? */
    (*segdir).res08 = -1;
    (*segdir).res0c = 0xf;
}

unsafe fn WMSFT_write_segment(outfile: HANDLE, segment: *mut WMSFT_SegContents) {
    let mut written: DWORD = 0;
    if !segment.is_null() {
        WriteFile(outfile, (*segment).data, (*segment).len, &raw mut written, null_mut());
    }
}

unsafe fn WMSFT_fixup_typeinfos(
    This: *mut ITypeLibImpl,
    file: *mut WMSFT_TLBFile,
    file_len: DWORD,
) -> HRESULT {
    let mut base = (*file).typeinfo_seg.data as *mut MSFT_TypeInfoBase;

    let mut i: i32 = 0;
    while i < (*This).TypeInfoCount {
        (*base).memoffset += file_len as i32;
        base = base.add(1);
        i += 1;
    }

    S_OK
}

unsafe fn WMSFT_free_file(file: *mut WMSFT_TLBFile) {
    HeapFree(GetProcessHeap(), 0, (*file).typeinfo_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).guidhash_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).guid_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).ref_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).impinfo_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).impfile_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).namehash_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).name_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).string_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).typdesc_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).arraydesc_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).custdata_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).cdguids_seg.data);
    HeapFree(GetProcessHeap(), 0, (*file).aux_seg.data);
}

unsafe extern "system" fn ICreateTypeLib2_fnSaveAllChanges(iface: *mut ICreateTypeLib2) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    let mut file: WMSFT_TLBFile = core::mem::zeroed();
    let mut written: DWORD = 0;
    let junk_size: DWORD;
    let junk_offs: DWORD;
    let mut running_offset: DWORD;
    let mut hres: HRESULT;
    let junk: *mut DWORD;

    TRACE!();

    let mut i: i32 = 0;
    while i < (*This).TypeInfoCount {
        let ti = *(*This).typeinfos.add(i as usize);
        if (*ti).needs_layout != FALSE {
            let cti = &raw mut (*ti).ICreateTypeInfo2_iface;
            ((*(*cti).lpVtbl).LayOut)(cti);
        }
        i += 1;
    }

    memset(&raw mut file as *mut c_void, 0, size_of::<WMSFT_TLBFile>());

    file.header.magic1 = 0x5446_534Du32 as i32;
    file.header.magic2 = 0x0001_0002;
    file.header.lcid = if (*This).set_lcid != 0 {
        (*This).set_lcid as i32
    } else {
        (((SUBLANG_ENGLISH_US as u32) << 10) | LANG_ENGLISH as u32) as i32 /* MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US) */
    };
    file.header.lcid2 = (*This).set_lcid as i32;
    file.header.varflags = 0x40 | (*This).syskind;
    if !(*This).HelpFile.is_null() {
        file.header.varflags |= 0x10;
    }
    if !(*This).HelpStringDll.is_null() {
        file.header.varflags |= HELPDLLFLAG as i32;
    }
    file.header.version = (((*This).ver_minor as i32) << 16) | (*This).ver_major as i32;
    file.header.flags = (*This).libflags as i32;
    file.header.helpstringcontext = 0; /* TODO - SetHelpStringContext not implemented yet */
    file.header.helpcontext = (*This).dwHelpContext as i32;
    file.header.res44 = 0x20;
    file.header.res48 = 0x80;
    file.header.dispatchpos = (*This).dispatch_href as i32;

    WMSFT_compile_namehash(This, &raw mut file);
    /* do name and string compilation to get offsets for other compilations */
    hres = WMSFT_compile_names(This, &raw mut file);
    if FAILED(hres) {
        WMSFT_free_file(&raw mut file);
        return hres;
    }

    hres = WMSFT_compile_strings(This, &raw mut file);
    if FAILED(hres) {
        WMSFT_free_file(&raw mut file);
        return hres;
    }

    WMSFT_compile_guidhash(This, &raw mut file);
    hres = WMSFT_compile_guids(This, &raw mut file);
    if FAILED(hres) {
        WMSFT_free_file(&raw mut file);
        return hres;
    }

    if !(*This).HelpFile.is_null() {
        file.header.helpfile = (*(*This).HelpFile).offset as i32;
    } else {
        file.header.helpfile = -1;
    }

    if !(*This).DocString.is_null() {
        file.header.helpstring = (*(*This).DocString).offset as i32;
    } else {
        file.header.helpstring = -1;
    }

    /* do some more segment compilation */
    file.header.nimpinfos = list_count(&raw const (*This).ref_list) as i32;
    file.header.nrtypeinfos = (*This).TypeInfoCount;

    if !(*This).Name.is_null() {
        file.header.NameOffset = (*(*This).Name).offset as i32;
    } else {
        file.header.NameOffset = -1;
    }

    file.header.CustomDataOffset =
        WMSFT_compile_custdata(&raw mut (*This).custdata_list, &raw mut file) as i32;

    if !(*This).guid.is_null() {
        file.header.posguid = (*(*This).guid).offset as i32;
    } else {
        file.header.posguid = -1;
    }

    junk_size = file.header.nrtypeinfos as DWORD * size_of::<DWORD>() as DWORD
        + if file.header.varflags & HELPDLLFLAG as i32 != 0 {
            size_of::<DWORD>() as DWORD
        } else {
            0
        };
    if junk_size != 0 {
        junk = heap_alloc_zero(junk_size as usize) as *mut DWORD;
        if file.header.varflags & HELPDLLFLAG as i32 != 0 {
            *junk = (*(*This).HelpStringDll).offset;
            junk_offs = 1;
        } else {
            junk_offs = 0;
        }
    } else {
        junk = null_mut();
        junk_offs = 0;
    }

    WMSFT_compile_typeinfo_seg(This, &raw mut file, junk.wrapping_add(junk_offs as usize));
    WMSFT_compile_impinfo(This, &raw mut file);

    running_offset = 0;

    TRACE!();
    running_offset += size_of::<MSFT_Header>() as DWORD;

    TRACE!();
    running_offset += junk_size;

    TRACE!();
    running_offset += size_of::<MSFT_SegDir>() as DWORD;

    tmp_fill_segdir_seg(&raw mut file.segdir.pTypeInfoTab, &raw mut file.typeinfo_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pGuidHashTab, &raw mut file.guidhash_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pGuidTab, &raw mut file.guid_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pRefTab, &raw mut file.ref_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pImpInfo, &raw mut file.impinfo_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pImpFiles, &raw mut file.impfile_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pNameHashTab, &raw mut file.namehash_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pNametab, &raw mut file.name_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pStringtab, &raw mut file.string_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pTypdescTab, &raw mut file.typdesc_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pArrayDescriptions, &raw mut file.arraydesc_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pCustData, &raw mut file.custdata_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.pCDGuids, &raw mut file.cdguids_seg, &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.res0e, null_mut(), &raw mut running_offset);
    tmp_fill_segdir_seg(&raw mut file.segdir.res0f, null_mut(), &raw mut running_offset);

    TRACE!();

    WMSFT_fixup_typeinfos(This, &raw mut file, running_offset);

    let outfile = CreateFileW(
        (*This).path,
        GENERIC_WRITE,
        0,
        null_mut(),
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        null_mut(),
    );
    if outfile == INVALID_HANDLE_VALUE {
        WMSFT_free_file(&raw mut file);
        heap_free(junk as *mut c_void);
        return TYPE_E_IOERROR;
    }

    let br = WriteFile(
        outfile,
        &raw const file.header as *const c_void,
        size_of::<MSFT_Header>() as DWORD,
        &raw mut written,
        null_mut(),
    );
    if br == FALSE {
        WMSFT_free_file(&raw mut file);
        CloseHandle(outfile);
        heap_free(junk as *mut c_void);
        return TYPE_E_IOERROR;
    }

    let br = WriteFile(outfile, junk as *const c_void, junk_size, &raw mut written, null_mut());
    heap_free(junk as *mut c_void);
    if br == FALSE {
        WMSFT_free_file(&raw mut file);
        CloseHandle(outfile);
        return TYPE_E_IOERROR;
    }

    let br = WriteFile(
        outfile,
        &raw const file.segdir as *const c_void,
        size_of::<MSFT_SegDir>() as DWORD,
        &raw mut written,
        null_mut(),
    );
    if br == FALSE {
        WMSFT_free_file(&raw mut file);
        CloseHandle(outfile);
        return TYPE_E_IOERROR;
    }

    WMSFT_write_segment(outfile, &raw mut file.typeinfo_seg);
    WMSFT_write_segment(outfile, &raw mut file.guidhash_seg);
    WMSFT_write_segment(outfile, &raw mut file.guid_seg);
    WMSFT_write_segment(outfile, &raw mut file.ref_seg);
    WMSFT_write_segment(outfile, &raw mut file.impinfo_seg);
    WMSFT_write_segment(outfile, &raw mut file.impfile_seg);
    WMSFT_write_segment(outfile, &raw mut file.namehash_seg);
    WMSFT_write_segment(outfile, &raw mut file.name_seg);
    WMSFT_write_segment(outfile, &raw mut file.string_seg);
    WMSFT_write_segment(outfile, &raw mut file.typdesc_seg);
    WMSFT_write_segment(outfile, &raw mut file.arraydesc_seg);
    WMSFT_write_segment(outfile, &raw mut file.custdata_seg);
    WMSFT_write_segment(outfile, &raw mut file.cdguids_seg);
    WMSFT_write_segment(outfile, &raw mut file.aux_seg);

    WMSFT_free_file(&raw mut file);

    CloseHandle(outfile);

    S_OK
}

unsafe extern "system" fn ICreateTypeLib2_fnDeleteTypeInfo(
    iface: *mut ICreateTypeLib2,
    name: LPOLESTR,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    FIXME!("{:p} - stub (DeleteTypeInfo)", This);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeLib2_fnSetCustData(
    iface: *mut ICreateTypeLib2,
    guid: REFGUID,
    varVal: *mut VARIANT,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);

    TRACE!();

    if guid.is_null() || varVal.is_null() {
        return E_INVALIDARG;
    }

    let tlbguid = TLB_append_guid(&raw mut (*This).guid_list, guid, -1i32 as HREFTYPE);

    TLB_set_custdata(&raw mut (*This).custdata_list, tlbguid, varVal)
}

unsafe extern "system" fn ICreateTypeLib2_fnSetHelpStringContext(
    iface: *mut ICreateTypeLib2,
    helpStringContext: ULONG,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    FIXME!("{:p} {} - stub (SetHelpStringContext)", This, helpStringContext);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeLib2_fnSetHelpStringDll(
    iface: *mut ICreateTypeLib2,
    filename: LPOLESTR,
) -> HRESULT {
    let This = impl_from_ICreateTypeLib2(iface);
    TRACE!();

    if filename.is_null() {
        return E_INVALIDARG;
    }

    (*This).HelpStringDll = TLB_append_str(&raw mut (*This).string_list, filename);

    S_OK
}

pub static CreateTypeLib2Vtbl: ICreateTypeLib2Vtbl = ICreateTypeLib2Vtbl {
    QueryInterface: ICreateTypeLib2_fnQueryInterface,
    AddRef: ICreateTypeLib2_fnAddRef,
    Release: ICreateTypeLib2_fnRelease,
    CreateTypeInfo: ICreateTypeLib2_fnCreateTypeInfo,
    SetName: ICreateTypeLib2_fnSetName,
    SetVersion: ICreateTypeLib2_fnSetVersion,
    SetGuid: ICreateTypeLib2_fnSetGuid,
    SetDocString: ICreateTypeLib2_fnSetDocString,
    SetHelpFileName: ICreateTypeLib2_fnSetHelpFileName,
    SetHelpContext: ICreateTypeLib2_fnSetHelpContext,
    SetLcid: ICreateTypeLib2_fnSetLcid,
    SetLibFlags: ICreateTypeLib2_fnSetLibFlags,
    SaveAllChanges: ICreateTypeLib2_fnSaveAllChanges,
    DeleteTypeInfo: ICreateTypeLib2_fnDeleteTypeInfo,
    SetCustData: ICreateTypeLib2_fnSetCustData,
    SetHelpStringContext: ICreateTypeLib2_fnSetHelpStringContext,
    SetHelpStringDll: ICreateTypeLib2_fnSetHelpStringDll,
};

// ---------------------------------------------------------------------------
// ICreateTypeInfo2
// ---------------------------------------------------------------------------

unsafe extern "system" fn ICreateTypeInfo2_fnQueryInterface(
    iface: *mut ICreateTypeInfo2,
    riid: REFIID,
    object: *mut *mut c_void,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let ti = &raw mut (*This).ITypeInfo2_iface;
    ((*(*ti).lpVtbl).QueryInterface)(ti, riid, object)
}

unsafe extern "system" fn ICreateTypeInfo2_fnAddRef(iface: *mut ICreateTypeInfo2) -> ULONG {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let ti = &raw mut (*This).ITypeInfo2_iface;
    ((*(*ti).lpVtbl).AddRef)(ti)
}

unsafe extern "system" fn ICreateTypeInfo2_fnRelease(iface: *mut ICreateTypeInfo2) -> ULONG {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let ti = &raw mut (*This).ITypeInfo2_iface;
    ((*(*ti).lpVtbl).Release)(ti)
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetGuid(
    iface: *mut ICreateTypeInfo2,
    guid: REFGUID,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    (*This).guid = TLB_append_guid(
        &raw mut (*(*This).pTypeLib).guid_list,
        guid,
        (*This).hreftype,
    );

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetTypeFlags(
    iface: *mut ICreateTypeInfo2,
    typeFlags: UINT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if typeFlags as WORD & TYPEFLAG_FDUAL != 0 {
        let stdole2tlb = wide!("stdole2.tlb");
        let mut stdole: *mut ITypeLib = null_mut();
        let mut dispatch: *mut ITypeInfo = null_mut();
        let mut hreftype: HREFTYPE = 0;

        let hres = super::super::typelib::LoadTypeLib(stdole2tlb.as_ptr(), &raw mut stdole);
        if FAILED(hres) {
            return hres;
        }

        let hres = ((*(*stdole).lpVtbl).GetTypeInfoOfGuid)(
            stdole,
            &raw const IID_IDispatch,
            &raw mut dispatch,
        );
        ((*(*stdole).lpVtbl).Release)(stdole);
        if FAILED(hres) {
            return hres;
        }

        let hres = ICreateTypeInfo2_fnAddRefTypeInfo(iface, dispatch, &raw mut hreftype);
        ((*(*dispatch).lpVtbl).Release)(dispatch);
        if FAILED(hres) {
            return hres;
        }
    }

    let old_flags = (*This).typeattr.wTypeFlags;
    (*This).typeattr.wTypeFlags = typeFlags as WORD;

    let hres = ICreateTypeInfo2_fnLayOut(iface);
    if FAILED(hres) {
        (*This).typeattr.wTypeFlags = old_flags;
        return hres;
    }

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetDocString(
    iface: *mut ICreateTypeInfo2,
    doc: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if doc.is_null() {
        return E_INVALIDARG;
    }

    (*This).DocString = TLB_append_str(&raw mut (*(*This).pTypeLib).string_list, doc);

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetHelpContext(
    iface: *mut ICreateTypeInfo2,
    helpContext: DWORD,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    (*This).dwHelpContext = helpContext;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetVersion(
    iface: *mut ICreateTypeInfo2,
    majorVerNum: WORD,
    minorVerNum: WORD,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    (*This).typeattr.wMajorVerNum = majorVerNum;
    (*This).typeattr.wMinorVerNum = minorVerNum;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnAddRefTypeInfo(
    iface: *mut ICreateTypeInfo2,
    typeInfo: *mut ITypeInfo,
    refType: *mut HREFTYPE,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let mut index: UINT = 0;
    let mut container: *mut ITypeLib = null_mut();
    let mut typeattr: *mut TYPEATTR = null_mut();
    let mut libattr: *mut TLIBATTR = null_mut();

    TRACE!();

    if typeInfo.is_null() || refType.is_null() {
        return E_INVALIDARG;
    }

    let hres = ((*(*typeInfo).lpVtbl).GetContainingTypeLib)(
        typeInfo,
        &raw mut container,
        &raw mut index,
    );
    if FAILED(hres) {
        return hres;
    }

    if container == &raw mut (*(*This).pTypeLib).ITypeLib2_iface as *mut ITypeLib {
        let target = impl_from_ITypeInfo(typeInfo);

        ((*(*container).lpVtbl).Release)(container);

        *refType = (*target).hreftype;

        return S_OK;
    }

    let hres = ((*(*container).lpVtbl).GetLibAttr)(container, &raw mut libattr);
    if FAILED(hres) {
        ((*(*container).lpVtbl).Release)(container);
        return hres;
    }

    /* LIST_FOR_EACH_ENTRY with completion sentinel */
    let implib_list = &raw mut (*(*This).pTypeLib).implib_list;
    let mut implib: *mut TLBImpLib = null_mut();
    let mut cur = (*implib_list).next;
    while cur != implib_list {
        let candidate = list_entry!(cur, TLBImpLib, entry);
        if guid_eq(&raw const (*(*candidate).guid).guid, &raw const (*libattr).guid)
            && (*candidate).lcid == (*libattr).lcid
            && (*candidate).wVersionMajor == (*libattr).wMajorVerNum
            && (*candidate).wVersionMinor == (*libattr).wMinorVerNum
        {
            implib = candidate;
            break;
        }
        cur = (*cur).next;
    }

    if implib.is_null() {
        implib = heap_alloc_zero(size_of::<TLBImpLib>()) as *mut TLBImpLib;

        if (*container).lpVtbl as *const ITypeLib2Vtbl == &raw const super::itypelib::tlbvt {
            let our_container = impl_from_ITypeLib2(container as *mut ITypeLib2);
            (*implib).name = SysAllocString((*our_container).path);
        } else {
            let hres = super::super::typelib::QueryPathOfRegTypeLib(
                &raw const (*libattr).guid,
                (*libattr).wMajorVerNum,
                (*libattr).wMinorVerNum,
                (*libattr).lcid,
                &raw mut (*implib).name,
            );
            if FAILED(hres) {
                (*implib).name = null_mut();
                TRACE!();
            }
        }

        (*implib).guid = TLB_append_guid(
            &raw mut (*(*This).pTypeLib).guid_list,
            &raw const (*libattr).guid,
            2,
        );
        (*implib).lcid = (*libattr).lcid;
        (*implib).wVersionMajor = (*libattr).wMajorVerNum;
        (*implib).wVersionMinor = (*libattr).wMinorVerNum;

        list_add_tail(implib_list, &raw mut (*implib).entry);
    }

    ((*(*container).lpVtbl).ReleaseTLibAttr)(container, libattr);
    ((*(*container).lpVtbl).Release)(container);

    let hres = ((*(*typeInfo).lpVtbl).GetTypeAttr)(typeInfo, &raw mut typeattr);
    if FAILED(hres) {
        return hres;
    }

    let mut index: UINT = 0;
    let ref_list = &raw mut (*(*This).pTypeLib).ref_list;
    let mut ref_type: *mut TLBRefType = null_mut();
    let mut cur = (*ref_list).next;
    while cur != ref_list {
        let candidate = list_entry!(cur, TLBRefType, entry);
        if (*candidate).index == TLB_REF_USE_GUID
            && guid_eq(&raw const (*(*candidate).guid).guid, &raw const (*typeattr).guid)
            && (*candidate).tkind == (*typeattr).typekind
        {
            ref_type = candidate;
            break;
        }
        index += 1;
        cur = (*cur).next;
    }

    if ref_type.is_null() {
        ref_type = heap_alloc_zero(size_of::<TLBRefType>()) as *mut TLBRefType;

        (*ref_type).tkind = (*typeattr).typekind;
        (*ref_type).pImpTLInfo = implib;
        (*ref_type).reference = (index as usize * size_of::<MSFT_ImpInfo>()) as HREFTYPE;

        (*ref_type).index = TLB_REF_USE_GUID;

        (*ref_type).guid = TLB_append_guid(
            &raw mut (*(*This).pTypeLib).guid_list,
            &raw const (*typeattr).guid,
            (*ref_type).reference + 1,
        );

        list_add_tail(ref_list, &raw mut (*ref_type).entry);
    }

    ((*(*typeInfo).lpVtbl).ReleaseTypeAttr)(typeInfo, typeattr);

    *refType = (*ref_type).reference | 0x1;

    if guid_eq(&raw const (*(*ref_type).guid).guid, &raw const IID_IDispatch) {
        (*(*This).pTypeLib).dispatch_href = *refType;
    }

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnAddFuncDesc(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    funcDesc: *mut FUNCDESC,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let mut tmp_func_desc: TLBFuncDesc = core::mem::zeroed();
    let func_desc: *mut TLBFuncDesc;

    TRACE!();

    if funcDesc.is_null() || (*funcDesc).oVft & 3 != 0 {
        return E_INVALIDARG;
    }

    match (*This).typeattr.typekind {
        k if k == TKIND_MODULE => {
            if (*funcDesc).funckind != FUNC_STATIC {
                return TYPE_E_BADMODULEKIND;
            }
        }
        k if k == TKIND_DISPATCH => {
            if (*funcDesc).funckind != FUNC_DISPATCH {
                return TYPE_E_BADMODULEKIND;
            }
        }
        _ => {
            if (*funcDesc).funckind != FUNC_PUREVIRTUAL {
                return TYPE_E_BADMODULEKIND;
            }
        }
    }

    if index > (*This).typeattr.cFuncs as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    if (*funcDesc).invkind & (INVOKE_PROPERTYPUT | INVOKE_PROPERTYPUTREF) != 0
        && (*funcDesc).cParams == 0
    {
        return TYPE_E_INCONSISTENTPROPFUNCS;
    }

    // Wine 10.0: on any bitness, a win64 typelib's vtable offsets are
    // pointer-aligned.
    if (*(*This).pTypeLib).syskind == SYS_WIN64 && (*funcDesc).oVft % 8 != 0 {
        return E_INVALIDARG;
    }

    memset(&raw mut tmp_func_desc as *mut c_void, 0, size_of::<TLBFuncDesc>());
    TLBFuncDesc_Constructor(&raw mut tmp_func_desc);

    core::ptr::copy_nonoverlapping(funcDesc, &raw mut tmp_func_desc.funcdesc, 1);

    if tmp_func_desc.funcdesc.oVft != 0 {
        tmp_func_desc.funcdesc.oVft |= 1;
    }

    if (*funcDesc).cScodes != 0 && !(*funcDesc).lprgscode.is_null() {
        tmp_func_desc.funcdesc.lprgscode =
            heap_alloc(size_of::<SCODE>() * (*funcDesc).cScodes as usize) as *mut SCODE;
        memcpy(
            tmp_func_desc.funcdesc.lprgscode as *mut c_void,
            (*funcDesc).lprgscode as *const c_void,
            size_of::<SCODE>() * (*funcDesc).cScodes as usize,
        );
    } else {
        tmp_func_desc.funcdesc.lprgscode = null_mut();
        tmp_func_desc.funcdesc.cScodes = 0;
    }

    let mut buf_size = TLB_SizeElemDesc(&raw const (*funcDesc).elemdescFunc);
    let mut i: i32 = 0;
    while i < (*funcDesc).cParams as i32 {
        buf_size += size_of::<ELEMDESC>();
        buf_size += TLB_SizeElemDesc((*funcDesc).lprgelemdescParam.add(i as usize));
        i += 1;
    }
    tmp_func_desc.funcdesc.lprgelemdescParam = heap_alloc(buf_size) as *mut ELEMDESC;
    let mut buffer = tmp_func_desc
        .funcdesc
        .lprgelemdescParam
        .add((*funcDesc).cParams as usize) as *mut i8;

    let mut hres = TLB_CopyElemDesc(
        &raw const (*funcDesc).elemdescFunc,
        &raw mut tmp_func_desc.funcdesc.elemdescFunc,
        &raw mut buffer,
    );
    if FAILED(hres) {
        heap_free(tmp_func_desc.funcdesc.lprgelemdescParam as *mut c_void);
        heap_free(tmp_func_desc.funcdesc.lprgscode as *mut c_void);
        return hres;
    }

    let mut i: i32 = 0;
    while i < (*funcDesc).cParams as i32 {
        hres = TLB_CopyElemDesc(
            (*funcDesc).lprgelemdescParam.add(i as usize),
            tmp_func_desc.funcdesc.lprgelemdescParam.add(i as usize),
            &raw mut buffer,
        );
        if FAILED(hres) {
            heap_free(tmp_func_desc.funcdesc.lprgelemdescParam as *mut c_void);
            heap_free(tmp_func_desc.funcdesc.lprgscode as *mut c_void);
            return hres;
        }
        let ed = tmp_func_desc.funcdesc.lprgelemdescParam.add(i as usize);
        if (*ed).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT != 0
            && (*ed).tdesc.vt != VT_VARIANT
            && (*ed).tdesc.vt != VT_USERDEFINED
        {
            hres = TLB_SanitizeVariant(&raw mut (*(*ed).u.paramdesc.pparamdescex).varDefaultValue);
            if FAILED(hres) {
                heap_free(tmp_func_desc.funcdesc.lprgelemdescParam as *mut c_void);
                heap_free(tmp_func_desc.funcdesc.lprgscode as *mut c_void);
                return hres;
            }
        }
        i += 1;
    }

    tmp_func_desc.pParamDesc = TLBParDesc_Constructor((*funcDesc).cParams as UINT);

    if !(*This).funcdescs.is_null() {
        (*This).funcdescs = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).funcdescs as *mut c_void,
            (size_of::<TLBFuncDesc>() * ((*This).typeattr.cFuncs as usize + 1)) as _,
        ) as *mut TLBFuncDesc;

        if index < (*This).typeattr.cFuncs as UINT {
            memmove(
                (*This).funcdescs.add(index as usize + 1) as *mut c_void,
                (*This).funcdescs.add(index as usize) as *const c_void,
                ((*This).typeattr.cFuncs as usize - index as usize) * size_of::<TLBFuncDesc>(),
            );
            func_desc = (*This).funcdescs.add(index as usize);
        } else {
            func_desc = (*This).funcdescs.add((*This).typeattr.cFuncs as usize);
        }

        /* move custdata lists to the new memory location */
        let mut i: UINT = 0;
        while i < (*This).typeattr.cFuncs as UINT + 1 {
            if index != i {
                let fd = (*This).funcdescs.add(i as usize);
                if (*fd).custdata_list.prev == (*fd).custdata_list.next {
                    list_init(&raw mut (*fd).custdata_list);
                } else {
                    (*(*fd).custdata_list.prev).next = &raw mut (*fd).custdata_list;
                    (*(*fd).custdata_list.next).prev = &raw mut (*fd).custdata_list;
                }
            }
            i += 1;
        }
    } else {
        (*This).funcdescs = heap_alloc(size_of::<TLBFuncDesc>()) as *mut TLBFuncDesc;
        func_desc = (*This).funcdescs;
    }

    memcpy(
        func_desc as *mut c_void,
        &raw const tmp_func_desc as *const c_void,
        size_of::<TLBFuncDesc>(),
    );
    list_init(&raw mut (*func_desc).custdata_list);

    (*This).typeattr.cFuncs += 1;

    (*This).needs_layout = TRUE;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnAddImplType(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    refType: HREFTYPE,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let impl_type: *mut TLBImplType;

    TRACE!();

    match (*This).typeattr.typekind {
        k if k == TKIND_COCLASS => {
            if index == u32::MAX {
                FIXME!("Unhandled index: -1");
                return E_NOTIMPL;
            }

            if index != (*This).typeattr.cImplTypes as UINT {
                return TYPE_E_ELEMENTNOTFOUND;
            }
        }
        k if k == TKIND_INTERFACE || k == TKIND_DISPATCH => {
            if index != 0 || (*This).typeattr.cImplTypes != 0 {
                return TYPE_E_ELEMENTNOTFOUND;
            }
        }
        k => {
            FIXME!("Unimplemented typekind: {}", k);
            return E_NOTIMPL;
        }
    }

    if !(*This).impltypes.is_null() {
        (*This).impltypes = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).impltypes as *mut c_void,
            (size_of::<TLBImplType>() * ((*This).typeattr.cImplTypes as usize + 1)) as _,
        ) as *mut TLBImplType;

        if index < (*This).typeattr.cImplTypes as UINT {
            memmove(
                (*This).impltypes.add(index as usize + 1) as *mut c_void,
                (*This).impltypes.add(index as usize) as *const c_void,
                ((*This).typeattr.cImplTypes as usize - index as usize)
                    * size_of::<TLBImplType>(),
            );
            impl_type = (*This).impltypes.add(index as usize);
        } else {
            impl_type = (*This).impltypes.add((*This).typeattr.cImplTypes as usize);
        }

        /* move custdata lists to the new memory location */
        let mut i: UINT = 0;
        while i < (*This).typeattr.cImplTypes as UINT + 1 {
            if index != i {
                let it = (*This).impltypes.add(i as usize);
                if (*it).custdata_list.prev == (*it).custdata_list.next {
                    list_init(&raw mut (*it).custdata_list);
                } else {
                    (*(*it).custdata_list.prev).next = &raw mut (*it).custdata_list;
                    (*(*it).custdata_list.next).prev = &raw mut (*it).custdata_list;
                }
            }
            i += 1;
        }
    } else {
        (*This).impltypes = heap_alloc(size_of::<TLBImplType>()) as *mut TLBImplType;
        impl_type = (*This).impltypes;
    }

    memset(impl_type as *mut c_void, 0, size_of::<TLBImplType>());
    TLBImplType_Constructor(impl_type);
    (*impl_type).hRef = refType;

    (*This).typeattr.cImplTypes += 1;

    if (refType & !0x3) == ((*(*This).pTypeLib).dispatch_href & !0x3) {
        (*This).typeattr.wTypeFlags |= TYPEFLAG_FDISPATCHABLE;
    }

    let hres = ICreateTypeInfo2_fnLayOut(iface);
    if FAILED(hres) {
        return hres;
    }

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetImplTypeFlags(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    implTypeFlags: INT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let impl_type = (*This).impltypes.add(index as usize);

    TRACE!();

    if (*This).typeattr.typekind != TKIND_COCLASS {
        return TYPE_E_BADMODULEKIND;
    }

    if index >= (*This).typeattr.cImplTypes as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    (*impl_type).implflags = implTypeFlags;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetAlignment(
    iface: *mut ICreateTypeInfo2,
    alignment: WORD,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    (*This).typeattr.cbAlignment = alignment;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetSchema(
    iface: *mut ICreateTypeInfo2,
    schema: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if schema.is_null() {
        return E_INVALIDARG;
    }

    (*This).Schema = TLB_append_str(&raw mut (*(*This).pTypeLib).string_list, schema);

    (*This).typeattr.lpstrSchema = (*(*This).Schema).str_;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnAddVarDesc(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    varDesc: *mut VARDESC,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let var_desc: *mut TLBVarDesc;

    TRACE!();

    if !(*This).vardescs.is_null() {
        (*This).vardescs = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).vardescs as *mut c_void,
            (size_of::<TLBVarDesc>() * ((*This).typeattr.cVars as usize + 1)) as _,
        ) as *mut TLBVarDesc;

        if index < (*This).typeattr.cVars as UINT {
            memmove(
                (*This).vardescs.add(index as usize + 1) as *mut c_void,
                (*This).vardescs.add(index as usize) as *const c_void,
                ((*This).typeattr.cVars as usize - index as usize) * size_of::<TLBVarDesc>(),
            );
            var_desc = (*This).vardescs.add(index as usize);
        } else {
            var_desc = (*This).vardescs.add((*This).typeattr.cVars as usize);
        }

        /* move custdata lists to the new memory location */
        let mut i: UINT = 0;
        while i < (*This).typeattr.cVars as UINT + 1 {
            if index != i {
                let var = (*This).vardescs.add(i as usize);
                if (*var).custdata_list.prev == (*var).custdata_list.next {
                    list_init(&raw mut (*var).custdata_list);
                } else {
                    (*(*var).custdata_list.prev).next = &raw mut (*var).custdata_list;
                    (*(*var).custdata_list.next).prev = &raw mut (*var).custdata_list;
                }
            }
            i += 1;
        }
    } else {
        (*This).vardescs = heap_alloc_zero(size_of::<TLBVarDesc>()) as *mut TLBVarDesc;
        var_desc = (*This).vardescs;
    }

    TLBVarDesc_Constructor(var_desc);
    // Wine 10.0: a failed copy (out of memory, VariantCopy failing) must not
    // fall through to dereferencing the null vardesc_create.
    let hr = TLB_AllocAndInitVarDesc(varDesc, &raw mut (*var_desc).vardesc_create);
    if FAILED(hr) {
        return hr;
    }
    core::ptr::copy_nonoverlapping((*var_desc).vardesc_create, &raw mut (*var_desc).vardesc, 1);

    (*This).typeattr.cVars += 1;

    (*This).needs_layout = TRUE;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetFuncAndParamNames(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    names: *mut LPOLESTR,
    numNames: UINT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let func_desc = (*This).funcdescs.add(index as usize);

    TRACE!();

    if names.is_null() {
        return E_INVALIDARG;
    }

    if index >= (*This).typeattr.cFuncs as UINT || numNames == 0 {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    if (*func_desc).funcdesc.invkind & (INVOKE_PROPERTYPUT | INVOKE_PROPERTYPUTREF) != 0 {
        if numNames > (*func_desc).funcdesc.cParams as UINT {
            return TYPE_E_ELEMENTNOTFOUND;
        }
    } else if numNames > (*func_desc).funcdesc.cParams as UINT + 1 {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    let mut i: i32 = 0;
    while i < (*This).typeattr.cFuncs as i32 {
        let iter = (*This).funcdescs.add(i as usize);
        if !(*iter).Name.is_null() && wcscmp(TLB_get_bstr((*iter).Name), *names) == 0 {
            if (*iter).funcdesc.invkind
                & (INVOKE_PROPERTYPUT | INVOKE_PROPERTYPUTREF | INVOKE_PROPERTYGET)
                != 0
                && (*func_desc).funcdesc.invkind
                    & (INVOKE_PROPERTYPUT | INVOKE_PROPERTYPUTREF | INVOKE_PROPERTYGET)
                    != 0
                && (*func_desc).funcdesc.invkind != (*iter).funcdesc.invkind
            {
                i += 1;
                continue;
            }
            return TYPE_E_AMBIGUOUSNAME;
        }
        i += 1;
    }

    (*func_desc).Name = TLB_append_str(&raw mut (*(*This).pTypeLib).name_list, *names);

    let mut i: UINT = 1;
    while i < numNames {
        let par_desc = (*func_desc).pParamDesc.add(i as usize - 1);
        (*par_desc).Name =
            TLB_append_str(&raw mut (*(*This).pTypeLib).name_list, *names.add(i as usize));
        i += 1;
    }

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetVarName(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    name: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if name.is_null() {
        return E_INVALIDARG;
    }

    if index >= (*This).typeattr.cVars as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    (*(*This).vardescs.add(index as usize)).Name =
        TLB_append_str(&raw mut (*(*This).pTypeLib).name_list, name);
    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetTypeDescAlias(
    iface: *mut ICreateTypeInfo2,
    tdescAlias: *mut TYPEDESC,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if tdescAlias.is_null() {
        return E_INVALIDARG;
    }

    if (*This).typeattr.typekind != TKIND_ALIAS {
        return TYPE_E_BADMODULEKIND;
    }

    let hr = TLB_size_instance(
        This,
        (*(*This).pTypeLib).syskind,
        tdescAlias,
        &raw mut (*This).typeattr.cbSizeInstance,
        &raw mut (*This).typeattr.cbAlignment,
    );
    if FAILED(hr) {
        return hr;
    }

    heap_free((*This).tdescAlias as *mut c_void);
    (*This).tdescAlias = heap_alloc(TLB_SizeTypeDesc(tdescAlias, TRUE)) as *mut TYPEDESC;
    TLB_CopyTypeDesc(null_mut(), tdescAlias, (*This).tdescAlias as *mut c_void);

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnDefineFuncAsDllEntry(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    dllName: LPOLESTR,
    procName: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} - stub (DefineFuncAsDllEntry)", This, index);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetFuncDocString(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    docString: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let func_desc = (*This).funcdescs.add(index as usize);

    TRACE!();

    if docString.is_null() {
        return E_INVALIDARG;
    }

    if index >= (*This).typeattr.cFuncs as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    (*func_desc).HelpString =
        TLB_append_str(&raw mut (*(*This).pTypeLib).string_list, docString);

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetVarDocString(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    docString: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let var_desc = (*This).vardescs.add(index as usize);

    TRACE!();

    if docString.is_null() {
        return E_INVALIDARG;
    }

    if index >= (*This).typeattr.cVars as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    (*var_desc).HelpString =
        TLB_append_str(&raw mut (*(*This).pTypeLib).string_list, docString);

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetFuncHelpContext(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    helpContext: DWORD,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let func_desc = (*This).funcdescs.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cFuncs as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    (*func_desc).helpcontext = helpContext as i32;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetVarHelpContext(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    helpContext: DWORD,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let var_desc = (*This).vardescs.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cVars as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    (*var_desc).HelpContext = helpContext as i32;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetMops(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    bstrMops: BSTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} - stub (SetMops)", This, index);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetTypeIdldesc(
    iface: *mut ICreateTypeInfo2,
    idlDesc: *mut IDLDESC,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if idlDesc.is_null() {
        return E_INVALIDARG;
    }

    (*This).typeattr.idldescType.dwReserved = (*idlDesc).dwReserved;
    (*This).typeattr.idldescType.wIDLFlags = (*idlDesc).wIDLFlags;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnLayOut(iface: *mut ICreateTypeInfo2) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    let mut tinfo: *mut ITypeInfo = null_mut();
    let mut user_vft: UINT = 0;
    let mut depth: UINT = 0;
    let mut hres: HRESULT = S_OK;

    TRACE!();

    (*This).needs_layout = FALSE;

    hres = ((*(*iface).lpVtbl).QueryInterface)(
        iface,
        &raw const IID_ITypeInfo,
        &raw mut tinfo as *mut *mut c_void,
    );
    if FAILED(hres) {
        return hres;
    }

    if (*This).typeattr.typekind == TKIND_INTERFACE {
        let mut inh: *mut ITypeInfo = null_mut();
        let mut attr: *mut TYPEATTR = null_mut();
        let mut inh_href: HREFTYPE = 0;

        hres = ((*(*tinfo).lpVtbl).GetRefTypeOfImplType)(tinfo, 0, &raw mut inh_href);

        if SUCCEEDED(hres) {
            hres = ((*(*tinfo).lpVtbl).GetRefTypeInfo)(tinfo, inh_href, &raw mut inh);

            if SUCCEEDED(hres) {
                hres = ((*(*inh).lpVtbl).GetTypeAttr)(inh, &raw mut attr);
                if FAILED(hres) {
                    ((*(*inh).lpVtbl).Release)(inh);
                    ((*(*tinfo).lpVtbl).Release)(tinfo);
                    return hres;
                }
                (*This).typeattr.cbSizeVft = (*attr).cbSizeVft;
                ((*(*inh).lpVtbl).ReleaseTypeAttr)(inh, attr);

                loop {
                    depth += 1;
                    hres = ((*(*inh).lpVtbl).GetRefTypeOfImplType)(inh, 0, &raw mut inh_href);
                    if SUCCEEDED(hres) {
                        let mut next: *mut ITypeInfo = null_mut();
                        hres = ((*(*inh).lpVtbl).GetRefTypeInfo)(inh, inh_href, &raw mut next);
                        if SUCCEEDED(hres) {
                            ((*(*inh).lpVtbl).Release)(inh);
                            inh = next;
                        }
                    }
                    if FAILED(hres) {
                        break;
                    }
                }
                hres = S_OK;

                ((*(*inh).lpVtbl).Release)(inh);
            } else if hres == TYPE_E_ELEMENTNOTFOUND {
                (*This).typeattr.cbSizeVft = 0;
                hres = S_OK;
            } else {
                ((*(*tinfo).lpVtbl).Release)(tinfo);
                return hres;
            }
        } else if hres == TYPE_E_ELEMENTNOTFOUND {
            (*This).typeattr.cbSizeVft = 0;
            hres = S_OK;
        } else {
            ((*(*tinfo).lpVtbl).Release)(tinfo);
            return hres;
        }
    } else if (*This).typeattr.typekind == TKIND_DISPATCH {
        (*This).typeattr.cbSizeVft = (7 * (*(*This).pTypeLib).ptr_size) as WORD;
    } else {
        (*This).typeattr.cbSizeVft = 0;
    }

    let mut func_desc = (*This).funcdescs;
    let mut i: UINT = 0;
    while i < (*This).typeattr.cFuncs as UINT {
        if (*func_desc).funcdesc.oVft & 0x1 == 0 {
            (*func_desc).funcdesc.oVft = (*This).typeattr.cbSizeVft as SHORT;
        }

        if ((*func_desc).funcdesc.oVft as u16 & 0xFFFC) as UINT > user_vft {
            user_vft = ((*func_desc).funcdesc.oVft as u16 & 0xFFFC) as UINT;
        }

        (*This).typeattr.cbSizeVft += (*(*This).pTypeLib).ptr_size as WORD;

        if (*func_desc).funcdesc.memid == MEMBERID_NIL {
            let mut j: UINT = 0;
            let mut reset = false;

            (*func_desc).funcdesc.memid =
                0x6000_0000 + ((depth as i32) << 16) + i as i32;

            let mut iter = (*This).funcdescs;
            while j < (*This).typeattr.cFuncs as UINT {
                if iter != func_desc && (*iter).funcdesc.memid == (*func_desc).funcdesc.memid {
                    if !reset {
                        (*func_desc).funcdesc.memid = 0x6000_0000
                            + ((depth as i32) << 16)
                            + (*This).typeattr.cFuncs as i32;
                        reset = true;
                    } else {
                        (*func_desc).funcdesc.memid += 1;
                    }
                    iter = (*This).funcdescs;
                    j = 0;
                } else {
                    iter = iter.add(1);
                    j += 1;
                }
            }
        }

        func_desc = func_desc.add(1);
        i += 1;
    }

    if user_vft > (*This).typeattr.cbSizeVft as UINT {
        (*This).typeattr.cbSizeVft = (user_vft + (*(*This).pTypeLib).ptr_size as UINT) as WORD;
    }

    let mut i: UINT = 0;
    while i < (*This).typeattr.cVars as UINT {
        let var_desc = (*This).vardescs.add(i as usize);
        if (*var_desc).vardesc.memid == MEMBERID_NIL {
            let mut j: UINT = 0;
            let mut reset = false;

            (*var_desc).vardesc.memid = 0x4000_0000 + ((depth as i32) << 16) + i as i32;

            let mut iter = (*This).vardescs;
            while j < (*This).typeattr.cVars as UINT {
                if iter != var_desc && (*iter).vardesc.memid == (*var_desc).vardesc.memid {
                    if !reset {
                        (*var_desc).vardesc.memid = 0x4000_0000
                            + ((depth as i32) << 16)
                            + (*This).typeattr.cVars as i32;
                        reset = true;
                    } else {
                        (*var_desc).vardesc.memid += 1;
                    }
                    iter = (*This).vardescs;
                    j = 0;
                } else {
                    iter = iter.add(1);
                    j += 1;
                }
            }
        }
        i += 1;
    }

    ((*(*tinfo).lpVtbl).Release)(tinfo);
    hres
}

/// Wine 10.0 TLB_relink_custdata: a list head that moved (memmove of the
/// array that holds it) must have its neighbours re-pointed at it.
unsafe fn tlb_relink_custdata(custdata_list: *mut list) {
    if (*custdata_list).prev == (*custdata_list).next {
        list_init(custdata_list);
    } else {
        (*(*custdata_list).prev).next = custdata_list;
        (*(*custdata_list).next).prev = custdata_list;
    }
}

/// Wine 10.0 typeinfo_release_funcdesc: what ITypeInfoImpl_Destroy frees for
/// each function, for one function.
unsafe fn typeinfo_release_funcdesc(func: *mut TLBFuncDesc) {
    let mut j: i32 = 0;
    while j < (*func).funcdesc.cParams as i32 {
        let elemdesc = (*func).funcdesc.lprgelemdescParam.add(j as usize);
        if (*elemdesc).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
            VariantClear(&raw mut (*(*elemdesc).u.paramdesc.pparamdescex).varDefaultValue);
        }
        TLB_FreeCustData(&raw mut (*(*func).pParamDesc.add(j as usize)).custdata_list);
        j += 1;
    }
    heap_free((*func).funcdesc.lprgelemdescParam as *mut c_void);
    heap_free((*func).pParamDesc as *mut c_void);
    TLB_FreeCustData(&raw mut (*func).custdata_list);
}

/// Implemented in Wine 10.0 (was an E_NOTIMPL stub).
unsafe extern "system" fn ICreateTypeInfo2_fnDeleteFuncDesc(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    TRACE!();

    if index >= (*This).typeattr.cFuncs as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    typeinfo_release_funcdesc((*This).funcdescs.add(index as usize));

    (*This).typeattr.cFuncs -= 1;
    let n = (*This).typeattr.cFuncs as UINT;
    if index != n {
        memmove(
            (*This).funcdescs.add(index as usize) as *mut c_void,
            (*This).funcdescs.add(index as usize + 1) as *const c_void,
            size_of::<TLBFuncDesc>() * (n - index) as usize,
        );
        let mut i = index;
        while i < n {
            tlb_relink_custdata(&raw mut (*(*This).funcdescs.add(i as usize)).custdata_list);
            i += 1;
        }
    }

    (*This).needs_layout = TRUE;
    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnDeleteFuncDescByMemId(
    iface: *mut ICreateTypeInfo2,
    memid: MEMBERID,
    invKind: INVOKEKIND,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {:x} {} - stub (DeleteFuncDescByMemId)", This, memid, invKind);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnDeleteVarDesc(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} - stub (DeleteVarDesc)", This, index);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnDeleteVarDescByMemId(
    iface: *mut ICreateTypeInfo2,
    memid: MEMBERID,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {:x} - stub (DeleteVarDescByMemId)", This, memid);
    E_NOTIMPL
}

/// Implemented in Wine 10.0 (was an E_NOTIMPL stub).
unsafe extern "system" fn ICreateTypeInfo2_fnDeleteImplType(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    TRACE!();

    if index >= (*This).typeattr.cImplTypes as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    TLB_FreeCustData(&raw mut (*(*This).impltypes.add(index as usize)).custdata_list);
    (*This).typeattr.cImplTypes -= 1;
    let n = (*This).typeattr.cImplTypes as UINT;

    if index < n {
        memmove(
            (*This).impltypes.add(index as usize) as *mut c_void,
            (*This).impltypes.add(index as usize + 1) as *const c_void,
            size_of::<TLBImplType>() * (n - index) as usize,
        );
        let mut i = index;
        while i < n {
            tlb_relink_custdata(&raw mut (*(*This).impltypes.add(i as usize)).custdata_list);
            i += 1;
        }
    }

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetCustData(
    iface: *mut ICreateTypeInfo2,
    guid: REFGUID,
    varVal: *mut VARIANT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if guid.is_null() || varVal.is_null() {
        return E_INVALIDARG;
    }

    let tlbguid = TLB_append_guid(
        &raw mut (*(*This).pTypeLib).guid_list,
        guid,
        -1i32 as HREFTYPE,
    );

    TLB_set_custdata((*This).pcustdata_list, tlbguid, varVal)
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetFuncCustData(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    guid: REFGUID,
    varVal: *mut VARIANT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} - stub (SetFuncCustData)", This, index);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetParamCustData(
    iface: *mut ICreateTypeInfo2,
    funcIndex: UINT,
    paramIndex: UINT,
    guid: REFGUID,
    varVal: *mut VARIANT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} {} - stub (SetParamCustData)", This, funcIndex, paramIndex);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetVarCustData(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    guid: REFGUID,
    varVal: *mut VARIANT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} - stub (SetVarCustData)", This, index);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetImplTypeCustData(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    guid: REFGUID,
    varVal: *mut VARIANT,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} - stub (SetImplTypeCustData)", This, index);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetHelpStringContext(
    iface: *mut ICreateTypeInfo2,
    helpStringContext: ULONG,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    (*This).dwHelpStringContext = helpStringContext;

    S_OK
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetFuncHelpStringContext(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    helpStringContext: ULONG,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} {} - stub (SetFuncHelpStringContext)", This, index, helpStringContext);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetVarHelpStringContext(
    iface: *mut ICreateTypeInfo2,
    index: UINT,
    helpStringContext: ULONG,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} {} {} - stub (SetVarHelpStringContext)", This, index, helpStringContext);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnInvalidate(iface: *mut ICreateTypeInfo2) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);
    FIXME!("{:p} - stub (Invalidate)", This);
    E_NOTIMPL
}

unsafe extern "system" fn ICreateTypeInfo2_fnSetName(
    iface: *mut ICreateTypeInfo2,
    name: LPOLESTR,
) -> HRESULT {
    let This = info_impl_from_ICreateTypeInfo2(iface);

    TRACE!();

    if name.is_null() {
        return E_INVALIDARG;
    }

    (*This).Name = TLB_append_str(&raw mut (*(*This).pTypeLib).name_list, name);

    S_OK
}

pub static CreateTypeInfo2Vtbl: ICreateTypeInfo2Vtbl = ICreateTypeInfo2Vtbl {
    QueryInterface: ICreateTypeInfo2_fnQueryInterface,
    AddRef: ICreateTypeInfo2_fnAddRef,
    Release: ICreateTypeInfo2_fnRelease,
    SetGuid: ICreateTypeInfo2_fnSetGuid,
    SetTypeFlags: ICreateTypeInfo2_fnSetTypeFlags,
    SetDocString: ICreateTypeInfo2_fnSetDocString,
    SetHelpContext: ICreateTypeInfo2_fnSetHelpContext,
    SetVersion: ICreateTypeInfo2_fnSetVersion,
    AddRefTypeInfo: ICreateTypeInfo2_fnAddRefTypeInfo,
    AddFuncDesc: ICreateTypeInfo2_fnAddFuncDesc,
    AddImplType: ICreateTypeInfo2_fnAddImplType,
    SetImplTypeFlags: ICreateTypeInfo2_fnSetImplTypeFlags,
    SetAlignment: ICreateTypeInfo2_fnSetAlignment,
    SetSchema: ICreateTypeInfo2_fnSetSchema,
    AddVarDesc: ICreateTypeInfo2_fnAddVarDesc,
    SetFuncAndParamNames: ICreateTypeInfo2_fnSetFuncAndParamNames,
    SetVarName: ICreateTypeInfo2_fnSetVarName,
    SetTypeDescAlias: ICreateTypeInfo2_fnSetTypeDescAlias,
    DefineFuncAsDllEntry: ICreateTypeInfo2_fnDefineFuncAsDllEntry,
    SetFuncDocString: ICreateTypeInfo2_fnSetFuncDocString,
    SetVarDocString: ICreateTypeInfo2_fnSetVarDocString,
    SetFuncHelpContext: ICreateTypeInfo2_fnSetFuncHelpContext,
    SetVarHelpContext: ICreateTypeInfo2_fnSetVarHelpContext,
    SetMops: ICreateTypeInfo2_fnSetMops,
    SetTypeIdldesc: ICreateTypeInfo2_fnSetTypeIdldesc,
    LayOut: ICreateTypeInfo2_fnLayOut,
    DeleteFuncDesc: ICreateTypeInfo2_fnDeleteFuncDesc,
    DeleteFuncDescByMemId: ICreateTypeInfo2_fnDeleteFuncDescByMemId,
    DeleteVarDesc: ICreateTypeInfo2_fnDeleteVarDesc,
    DeleteVarDescByMemId: ICreateTypeInfo2_fnDeleteVarDescByMemId,
    DeleteImplType: ICreateTypeInfo2_fnDeleteImplType,
    SetCustData: ICreateTypeInfo2_fnSetCustData,
    SetFuncCustData: ICreateTypeInfo2_fnSetFuncCustData,
    SetParamCustData: ICreateTypeInfo2_fnSetParamCustData,
    SetVarCustData: ICreateTypeInfo2_fnSetVarCustData,
    SetImplTypeCustData: ICreateTypeInfo2_fnSetImplTypeCustData,
    SetHelpStringContext: ICreateTypeInfo2_fnSetHelpStringContext,
    SetFuncHelpStringContext: ICreateTypeInfo2_fnSetFuncHelpStringContext,
    SetVarHelpStringContext: ICreateTypeInfo2_fnSetVarHelpStringContext,
    Invalidate: ICreateTypeInfo2_fnInvalidate,
    SetName: ICreateTypeInfo2_fnSetName,
};

/******************************************************************************
 * ClearCustData (OLEAUT32.171)
 *
 * Clear a custom data type's data.
 */
#[no_mangle]
pub unsafe extern "system" fn ClearCustData(lpCust: *mut CUSTDATA) {
    if !lpCust.is_null() && (*lpCust).cCustData != 0 {
        if !(*lpCust).prgCustData.is_null() {
            let mut i: DWORD = 0;
            while i < (*lpCust).cCustData {
                VariantClear(&raw mut (*(*lpCust).prgCustData.add(i as usize)).varValue);
                i += 1;
            }

            CoTaskMemFree((*lpCust).prgCustData as *mut c_void);
            (*lpCust).prgCustData = null_mut();
        }
        (*lpCust).cCustData = 0;
    }
}
