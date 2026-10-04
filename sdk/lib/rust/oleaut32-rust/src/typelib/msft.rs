//! msft.rs — the MSFT typelib loader (typelib.c lines ~2030–3671): MSFT_Seek/
//! Read/ReadLE*, ReadAll{Guids,Names,Strings,Refs}, ReadValue, CustData,
//! GetTdesc, DoFuncs/DoVars/DoImplTypes, TLB_fix_32on64_typeinfo,
//! MSFT_DoTypeInfo, the tlb cache (cache_section), the TLB_PEFile/TLB_NEFile/
//! TLB_Mapping file mappers, TLB_ReadTypeLib, TypeLibImpl_Constructor and
//! ITypeLib2_Constructor_MSFT. Lane A2 continuation, faithful 1:1.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_assignments, unused_imports, unused_labels)]

use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{null, null_mut, read_unaligned};

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::sys::*;
use super::{
    FromLEDWord, FromLEDWords, FromLEWords, DO_NOT_SEEK, HELPDLLFLAG, MSFT_CDGuid,
    MSFT_FuncRecord, MSFT_GuidEntry, MSFT_Header, MSFT_ImpInfo, MSFT_NameIntro,
    MSFT_ParameterInfo, MSFT_RefRecord, MSFT_SegDir, MSFT_TypeInfoBase, MSFT_VarRecord,
    MSFT_IMPINFO_OFFSET_IS_GUID, MSFT_SIGNATURE, SLTG_SIGNATURE,
};

#[inline]
pub unsafe fn MSFT_Seek(pcx: *mut TLBContext, where_: LONG) {
    if where_ != DO_NOT_SEEK {
        let where_ = where_ as u32 + (*pcx).oStart;
        if where_ > (*pcx).length {
            /* FIXME */
            ERR!("seek beyond end ({}/{})", where_, (*pcx).length);
            TLB_abort();
        }
        (*pcx).pos = where_;
    }
}

/* read function */
pub unsafe fn MSFT_Read(buffer: *mut c_void, mut count: DWORD, pcx: *mut TLBContext, where_: LONG) -> DWORD {
    TRACE!();

    MSFT_Seek(pcx, where_);
    if (*pcx).pos + count > (*pcx).length {
        count = (*pcx).length - (*pcx).pos;
    }
    memcpy(
        buffer,
        ((*pcx).mapping as *const u8).add((*pcx).pos as usize) as *const c_void,
        count as usize,
    );
    (*pcx).pos += count;
    count
}

pub unsafe fn MSFT_ReadLEDWords(
    buffer: *mut c_void,
    count: DWORD,
    pcx: *mut TLBContext,
    where_: LONG,
) -> DWORD {
    let ret = MSFT_Read(buffer, count, pcx, where_);
    FromLEDWords(buffer, ret as i32);
    ret
}

pub unsafe fn MSFT_ReadLEWords(
    buffer: *mut c_void,
    count: DWORD,
    pcx: *mut TLBContext,
    where_: LONG,
) -> DWORD {
    let ret = MSFT_Read(buffer, count, pcx, where_);
    FromLEWords(buffer, ret as i32);
    ret
}

pub unsafe fn MSFT_ReadAllGuids(pcx: *mut TLBContext) -> HRESULT {
    let mut entry: MSFT_GuidEntry = core::mem::zeroed();
    let mut offs: i32 = 0;

    MSFT_Seek(pcx, (*(*pcx).pTblDir).pGuidTab.offset);
    loop {
        if offs >= (*(*pcx).pTblDir).pGuidTab.length {
            return S_OK;
        }

        MSFT_ReadLEWords(
            &raw mut entry as *mut c_void,
            size_of::<MSFT_GuidEntry>() as DWORD,
            pcx,
            DO_NOT_SEEK,
        );

        let guid = heap_alloc(size_of::<TLBGuid>()) as *mut TLBGuid;

        (*guid).offset = offs as UINT;
        (*guid).guid = entry.guid;
        (*guid).hreftype = entry.hreftype;

        list_add_tail(
            &raw mut (*(*pcx).pLibInfo).guid_list,
            &raw mut (*guid).entry,
        );

        offs += size_of::<MSFT_GuidEntry>() as i32;
    }
}

pub unsafe fn MSFT_ReadGuid(offset: i32, pcx: *mut TLBContext) -> *mut TLBGuid {
    let guid_list = &raw mut (*(*pcx).pLibInfo).guid_list;
    let mut cur = (*guid_list).next;
    while cur != guid_list {
        let ret = list_entry!(cur, TLBGuid, entry);
        if (*ret).offset == offset as UINT {
            TRACE!();
            return ret;
        }
        cur = (*cur).next;
    }
    null_mut()
}

pub unsafe fn MSFT_ReadHreftype(pcx: *mut TLBContext, offset: i32) -> HREFTYPE {
    let mut niName: MSFT_NameIntro = core::mem::zeroed();

    if offset < 0 {
        ERR!("bad offset {}", offset);
        return -1i32 as HREFTYPE;
    }

    MSFT_ReadLEDWords(
        &raw mut niName as *mut c_void,
        size_of::<MSFT_NameIntro>() as DWORD,
        pcx,
        (*(*pcx).pTblDir).pNametab.offset + offset,
    );

    niName.hreftype as HREFTYPE
}

pub unsafe fn MSFT_ReadAllNames(pcx: *mut TLBContext) -> HRESULT {
    let mut intro: MSFT_NameIntro = core::mem::zeroed();
    let mut offs: i32 = 0;

    MSFT_Seek(pcx, (*(*pcx).pTblDir).pNametab.offset);
    loop {
        if offs >= (*(*pcx).pTblDir).pNametab.length {
            return S_OK;
        }

        MSFT_ReadLEWords(
            &raw mut intro as *mut c_void,
            size_of::<MSFT_NameIntro>() as DWORD,
            pcx,
            DO_NOT_SEEK,
        );
        intro.namelen &= 0xFF;
        let mut len_piece: INT16 = (intro.namelen + size_of::<MSFT_NameIntro>() as i32) as INT16;
        if len_piece % 4 != 0 {
            len_piece = (len_piece + 4) & !0x3;
        }
        if len_piece < 8 {
            len_piece = 8;
        }

        let string = heap_alloc(len_piece as usize + 1) as *mut i8;
        MSFT_Read(
            string as *mut c_void,
            (len_piece as usize - size_of::<MSFT_NameIntro>()) as DWORD,
            pcx,
            DO_NOT_SEEK,
        );
        *string.add(intro.namelen as usize) = 0;

        let lengthInChars = MultiByteToWideChar(
            CP_ACP,
            MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
            string,
            -1,
            null_mut(),
            0,
        );
        if lengthInChars == 0 {
            heap_free(string as *mut c_void);
            return E_UNEXPECTED;
        }

        let tlbstr = heap_alloc(size_of::<TLBString>()) as *mut TLBString;

        (*tlbstr).offset = offs as UINT;
        (*tlbstr).str_ =
            SysAllocStringByteLen(null(), lengthInChars as UINT * size_of::<WCHAR>() as UINT);
        MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, string, -1, (*tlbstr).str_, lengthInChars);

        heap_free(string as *mut c_void);

        list_add_tail(
            &raw mut (*(*pcx).pLibInfo).name_list,
            &raw mut (*tlbstr).entry,
        );

        offs += len_piece as i32;
    }
}

pub unsafe fn MSFT_ReadName(pcx: *mut TLBContext, offset: i32) -> *mut TLBString {
    let name_list = &raw mut (*(*pcx).pLibInfo).name_list;
    let mut cur = (*name_list).next;
    while cur != name_list {
        let tlbstr = list_entry!(cur, TLBString, entry);
        if (*tlbstr).offset == offset as UINT {
            TRACE!();
            return tlbstr;
        }
        cur = (*cur).next;
    }
    null_mut()
}

pub unsafe fn MSFT_ReadString(pcx: *mut TLBContext, offset: i32) -> *mut TLBString {
    let string_list = &raw mut (*(*pcx).pLibInfo).string_list;
    let mut cur = (*string_list).next;
    while cur != string_list {
        let tlbstr = list_entry!(cur, TLBString, entry);
        if (*tlbstr).offset == offset as UINT {
            TRACE!();
            return tlbstr;
        }
        cur = (*cur).next;
    }
    null_mut()
}

/*
 * read a value and fill a VARIANT structure
 */
pub unsafe fn MSFT_ReadValue(pVar: *mut VARIANT, offset: i32, pcx: *mut TLBContext) {
    let mut size: i32;

    TRACE!();

    if offset < 0 {
        /* data are packed in here */
        (*pVar).vt = ((offset as u32 & 0x7c00_0000) >> 26) as VARTYPE;
        (*pVar).n3.lVal = offset & 0x3ff_ffff;
        return;
    }
    MSFT_ReadLEWords(
        &raw mut (*pVar).vt as *mut c_void,
        size_of::<VARTYPE>() as DWORD,
        pcx,
        (*(*pcx).pTblDir).pCustData.offset + offset,
    );
    TRACE!();
    match (*pVar).vt {
        VT_EMPTY   /* FIXME: is this right? */
        | VT_NULL  /* FIXME: is this right? */
        | VT_I2    /* this should not happen */
        | VT_I4
        | VT_R4
        | VT_ERROR
        | VT_BOOL
        | VT_I1
        | VT_UI1
        | VT_UI2
        | VT_UI4
        | VT_INT
        | VT_UINT
        | VT_VOID  /* FIXME: is this right? */
        | VT_HRESULT => {
            size = 4;
        }
        VT_R8 | VT_CY | VT_DATE | VT_I8 | VT_UI8
        | VT_DECIMAL /* FIXME: is this right? */
        | VT_FILETIME => {
            size = 8;
        }
        /* pointer types with known behaviour */
        VT_BSTR => {
            size = 0;
            MSFT_ReadLEDWords(&raw mut size as *mut c_void, size_of::<INT>() as DWORD, pcx, DO_NOT_SEEK);
            if size == -1 {
                (*pVar).n3.bstrVal = null_mut();
            } else {
                let ptr = heap_alloc_zero(size as usize) as *mut i8;
                MSFT_Read(ptr as *mut c_void, size as DWORD, pcx, DO_NOT_SEEK);
                // Wine 10.0 answers the old "do we need a AtoW conversion?"
                // FIXME: yes. The bytes are ANSI (CP_ACP); widening each one
                // (sign-extended) turned every non-ASCII constant to garbage.
                let len = MultiByteToWideChar(CP_ACP, 0, ptr, size, null_mut(), 0);
                (*pVar).n3.bstrVal = SysAllocStringLen(null(), len as UINT);
                MultiByteToWideChar(CP_ACP, 0, ptr, size, (*pVar).n3.bstrVal, len);
                heap_free(ptr as *mut c_void);
            }
            size = -4;
        }
        /* FIXME: this will not work AT ALL when the variant contains a pointer */
        _ => {
            size = 0;
            FIXME!(
                "VARTYPE {} is not supported, setting pointer to NULL",
                (*pVar).vt
            );
        }
    }

    if size > 0 {
        /* (big|small) endian correct? */
        MSFT_Read(
            &raw mut (*pVar).n3 as *mut c_void, /* &V_I2(pVar) — start of the data union */
            size as DWORD,
            pcx,
            DO_NOT_SEEK,
        );
    }
}

/*
 * create a linked list with custom data
 */
pub unsafe fn MSFT_CustData(pcx: *mut TLBContext, mut offset: i32, custdata_list: *mut list) -> i32 {
    let mut entry: MSFT_CDGuid = core::mem::zeroed();
    let mut count: i32 = 0;

    TRACE!();

    if (*(*pcx).pTblDir).pCDGuids.offset < 0 {
        return 0;
    }

    while offset >= 0 {
        count += 1;
        let pNew = heap_alloc_zero(size_of::<TLBCustData>()) as *mut TLBCustData;
        MSFT_ReadLEDWords(
            &raw mut entry as *mut c_void,
            size_of::<MSFT_CDGuid>() as DWORD,
            pcx,
            (*(*pcx).pTblDir).pCDGuids.offset + offset,
        );
        (*pNew).guid = MSFT_ReadGuid(entry.GuidOffset, pcx);
        MSFT_ReadValue(&raw mut (*pNew).data, entry.DataOffset, pcx);
        list_add_head(custdata_list, &raw mut (*pNew).entry);
        offset = entry.next;
    }
    count
}

pub unsafe fn MSFT_GetTdesc(pcx: *mut TLBContext, type_: INT, pTd: *mut TYPEDESC) {
    if type_ < 0 {
        (*pTd).vt = (type_ & VT_TYPEMASK as i32) as VARTYPE;
    } else {
        *pTd = *(*(*pcx).pLibInfo)
            .pTypeDesc
            .add(type_ as usize / (2 * size_of::<INT>()));
    }

    TRACE!();
}

pub unsafe fn MSFT_DoFuncs(
    pcx: *mut TLBContext,
    pTI: *mut ITypeInfoImpl,
    cFuncs: i32,
    cVars: i32,
    offset: i32,
    pptfd: *mut *mut TLBFuncDesc,
) {
    /*
     * member information is stored in a data structure at offset
     * indicated by the memoffset field of the typeinfo structure.
     * (See the long C comment for the layout.)
     */
    let mut infolen: i32 = 0;
    let mut nameoffset: i32 = 0;
    let mut reclength: i32 = 0;
    let mut recoffset: i32 = offset + size_of::<INT>() as i32;

    let recbuf = heap_alloc(0xffff) as *mut i8;
    let pFuncRec = recbuf as *mut MSFT_FuncRecord;
    let mut ptfd_prev: *mut TLBFuncDesc = null_mut();

    TRACE!();

    MSFT_ReadLEDWords(&raw mut infolen as *mut c_void, size_of::<INT>() as DWORD, pcx, offset);

    *pptfd = TLBFuncDesc_Alloc(cFuncs as UINT);
    let mut ptfd = *pptfd;
    let mut i: i32 = 0;
    while i < cFuncs {
        /* name, eventually add to a hash table */
        MSFT_ReadLEDWords(
            &raw mut nameoffset as *mut c_void,
            size_of::<INT>() as DWORD,
            pcx,
            offset + infolen + (cFuncs + cVars + i + 1) * size_of::<INT>() as i32,
        );

        /* read the function information record */
        MSFT_ReadLEDWords(
            &raw mut reclength as *mut c_void,
            size_of::<i32>() as DWORD, /* sizeof(pFuncRec->Info) */
            pcx,
            recoffset,
        );

        reclength &= 0xffff;

        MSFT_ReadLEDWords(
            &raw mut (*pFuncRec).DataType as *mut c_void,
            (reclength as usize - offset_of!(MSFT_FuncRecord, DataType)) as DWORD,
            pcx,
            DO_NOT_SEEK,
        );

        /* size without argument data */
        let mut optional: i32 =
            reclength - (*pFuncRec).nrargs as i32 * size_of::<MSFT_ParameterInfo>() as i32;
        if (*pFuncRec).FKCCIC & 0x1000 != 0 {
            optional -= (*pFuncRec).nrargs as i32 * size_of::<INT>() as i32;
        }

        if optional > offset_of!(MSFT_FuncRecord, HelpContext) as i32 {
            (*ptfd).helpcontext = (*pFuncRec).HelpContext;
        }

        if optional > offset_of!(MSFT_FuncRecord, oHelpString) as i32 {
            (*ptfd).HelpString = MSFT_ReadString(pcx, (*pFuncRec).oHelpString);
        }

        if optional > offset_of!(MSFT_FuncRecord, oEntry) as i32 {
            if (*pFuncRec).FKCCIC & 0x2000 != 0 {
                if !IS_INTRESOURCE_PTR((*pFuncRec).oEntry as isize as usize as *const c_void) {
                    ERR!("ordinal 0x{:08x} invalid, IS_INTRESOURCE is false", (*pFuncRec).oEntry);
                }
                (*ptfd).Entry = (LOWORD((*pFuncRec).oEntry as u32) as DWORD_PTR) as *const TLBString;
            } else {
                (*ptfd).Entry = MSFT_ReadString(pcx, (*pFuncRec).oEntry);
            }
        } else {
            (*ptfd).Entry = -1isize as usize as *const TLBString;
        }

        if optional > offset_of!(MSFT_FuncRecord, HelpStringContext) as i32 {
            (*ptfd).HelpStringContext = (*pFuncRec).HelpStringContext;
        }

        if optional > offset_of!(MSFT_FuncRecord, oCustData) as i32
            && (*pFuncRec).FKCCIC & 0x80 != 0
        {
            MSFT_CustData(pcx, (*pFuncRec).oCustData, &raw mut (*ptfd).custdata_list);
        }

        /* fill the FuncDesc Structure */
        MSFT_ReadLEDWords(
            &raw mut (*ptfd).funcdesc.memid as *mut c_void,
            size_of::<INT>() as DWORD,
            pcx,
            offset + infolen + (i + 1) * size_of::<INT>() as i32,
        );

        (*ptfd).funcdesc.funckind = (*pFuncRec).FKCCIC & 0x7;
        (*ptfd).funcdesc.invkind = ((*pFuncRec).FKCCIC >> 3) & 0xF;
        (*ptfd).funcdesc.callconv = ((*pFuncRec).FKCCIC >> 8) & 0xF;
        (*ptfd).funcdesc.cParams = (*pFuncRec).nrargs;
        (*ptfd).funcdesc.cParamsOpt = (*pFuncRec).nroargs;
        // Wine 10.0: a dispatch function has no vtable slot, and the offset is
        // UNSIGNED: through `as i32` an offset >= 0x8000 went negative.
        (*ptfd).funcdesc.oVft = if (*ptfd).funcdesc.funckind == FUNC_DISPATCH {
            0
        } else {
            (((*pFuncRec).VtableOffset as u16 as i32 & !1)
                * size_of::<*mut c_void>() as i32
                / (*(*pTI).pTypeLib).ptr_size) as SHORT
        };
        (*ptfd).funcdesc.wFuncFlags = LOWORD((*pFuncRec).Flags as u32);

        /* nameoffset is sometimes -1 on the second half of a propget/propput
         * pair of functions */
        if nameoffset == -1
            && i > 0
            && TLB_is_propgetput((*ptfd_prev).funcdesc.invkind)
            && TLB_is_propgetput((*ptfd).funcdesc.invkind)
        {
            (*ptfd).Name = (*ptfd_prev).Name;
        } else {
            (*ptfd).Name = MSFT_ReadName(pcx, nameoffset);
        }

        MSFT_GetTdesc(pcx, (*pFuncRec).DataType, &raw mut (*ptfd).funcdesc.elemdescFunc.tdesc);

        /* do the parameters/arguments */
        if (*pFuncRec).nrargs != 0 {
            let nrargs = (*pFuncRec).nrargs as i32;
            let mut paraminfo: MSFT_ParameterInfo = core::mem::zeroed();

            (*ptfd).funcdesc.lprgelemdescParam = heap_alloc_zero(
                nrargs as usize * (size_of::<ELEMDESC>() + size_of::<PARAMDESCEX>()),
            ) as *mut ELEMDESC;

            (*ptfd).pParamDesc = TLBParDesc_Constructor(nrargs as UINT);

            MSFT_ReadLEDWords(
                &raw mut paraminfo as *mut c_void,
                size_of::<MSFT_ParameterInfo>() as DWORD,
                pcx,
                recoffset + reclength - nrargs * size_of::<MSFT_ParameterInfo>() as i32,
            );

            let mut j: i32 = 0;
            while j < nrargs {
                let elemdesc = (*ptfd).funcdesc.lprgelemdescParam.add(j as usize);

                MSFT_GetTdesc(pcx, paraminfo.DataType, &raw mut (*elemdesc).tdesc);

                (*elemdesc).u.paramdesc.wParamFlags = paraminfo.Flags as USHORT;

                /* name */
                if paraminfo.oName != -1 {
                    (*(*ptfd).pParamDesc.add(j as usize)).Name =
                        MSFT_ReadName(pcx, paraminfo.oName);
                }
                TRACE!();

                /* default value */
                if ((*elemdesc).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT) != 0
                    && ((*pFuncRec).FKCCIC & 0x1000) != 0
                {
                    let pInt = (pFuncRec as *mut i8)
                        .add(reclength as usize - (nrargs as usize * 4) * size_of::<INT>())
                        as *mut INT;

                    let pParamDesc = &raw mut (*elemdesc).u.paramdesc;

                    (*pParamDesc).pparamdescex =
                        ((*ptfd).funcdesc.lprgelemdescParam.add(nrargs as usize)
                            as *mut PARAMDESCEX)
                            .add(j as usize);
                    (*(*pParamDesc).pparamdescex).cBytes = size_of::<PARAMDESCEX>() as ULONG;

                    MSFT_ReadValue(
                        &raw mut (*(*pParamDesc).pparamdescex).varDefaultValue,
                        *pInt.add(j as usize),
                        pcx,
                    );
                } else {
                    (*elemdesc).u.paramdesc.pparamdescex = null_mut();
                }

                /* custom info */
                if optional
                    > (offset_of!(MSFT_FuncRecord, oArgCustData)
                        + j as usize * size_of::<i32>()) as i32
                    && (*pFuncRec).FKCCIC & 0x80 != 0
                {
                    MSFT_CustData(
                        pcx,
                        *(&raw mut (*pFuncRec).oArgCustData as *mut i32).add(j as usize),
                        &raw mut (*(*ptfd).pParamDesc.add(j as usize)).custdata_list,
                    );
                }

                /* SEEK value = jump to offset,
                 * from there jump to the end of record,
                 * go back by (j-1) arguments */
                MSFT_ReadLEDWords(
                    &raw mut paraminfo as *mut c_void,
                    size_of::<MSFT_ParameterInfo>() as DWORD,
                    pcx,
                    recoffset + reclength
                        - (nrargs - j - 1) * size_of::<MSFT_ParameterInfo>() as i32,
                );

                j += 1;
            }
        }

        /* scode is not used: archaic win16 stuff FIXME: right? */
        (*ptfd).funcdesc.cScodes = 0;
        (*ptfd).funcdesc.lprgscode = null_mut();

        ptfd_prev = ptfd;
        ptfd = ptfd.add(1);
        recoffset += reclength;
        i += 1;
    }
    heap_free(recbuf as *mut c_void);
}

pub unsafe fn MSFT_DoVars(
    pcx: *mut TLBContext,
    pTI: *mut ITypeInfoImpl,
    cFuncs: i32,
    cVars: i32,
    offset: i32,
    pptvd: *mut *mut TLBVarDesc,
) {
    let mut infolen: i32 = 0;
    let mut nameoffset: i32 = 0;
    let mut reclength: i32 = 0;
    let mut recbuf: [i8; 256] = [0; 256];
    let pVarRec = recbuf.as_mut_ptr() as *mut MSFT_VarRecord;
    let mut recoffset: i32 = 0;

    TRACE!();

    *pptvd = TLBVarDesc_Alloc(cVars as UINT);
    let mut ptvd = *pptvd;
    MSFT_ReadLEDWords(&raw mut infolen as *mut c_void, size_of::<INT>() as DWORD, pcx, offset);
    MSFT_ReadLEDWords(
        &raw mut recoffset as *mut c_void,
        size_of::<INT>() as DWORD,
        pcx,
        offset + infolen + ((cFuncs + cVars) * 2 + cFuncs + 1) * size_of::<INT>() as i32,
    );
    recoffset += offset + size_of::<INT>() as i32;
    let mut i: i32 = 0;
    while i < cVars {
        /* name, eventually add to a hash table */
        MSFT_ReadLEDWords(
            &raw mut nameoffset as *mut c_void,
            size_of::<INT>() as DWORD,
            pcx,
            offset + infolen + (2 * cFuncs + cVars + i + 1) * size_of::<INT>() as i32,
        );
        (*ptvd).Name = MSFT_ReadName(pcx, nameoffset);
        /* read the variable information record */
        MSFT_ReadLEDWords(
            &raw mut reclength as *mut c_void,
            size_of::<i32>() as DWORD, /* sizeof(pVarRec->Info) */
            pcx,
            recoffset,
        );
        reclength &= 0xff;
        MSFT_ReadLEDWords(
            &raw mut (*pVarRec).DataType as *mut c_void,
            (reclength as usize - offset_of!(MSFT_VarRecord, DataType)) as DWORD,
            pcx,
            DO_NOT_SEEK,
        );

        /* optional data */
        if reclength > offset_of!(MSFT_VarRecord, HelpContext) as i32 {
            (*ptvd).HelpContext = (*pVarRec).HelpContext;
        }

        if reclength > offset_of!(MSFT_VarRecord, HelpString) as i32 {
            (*ptvd).HelpString = MSFT_ReadString(pcx, (*pVarRec).HelpString);
        }

        // Wine 10.0: variables carry custom data too (GetVarCustData /
        // GetAllVarCustData were always empty for a loaded typelib).
        if reclength > offset_of!(MSFT_VarRecord, oCustData) as i32 {
            MSFT_CustData(pcx, (*pVarRec).oCustData, &raw mut (*ptvd).custdata_list);
        }

        if reclength > offset_of!(MSFT_VarRecord, HelpStringContext) as i32 {
            (*ptvd).HelpStringContext = (*pVarRec).HelpStringContext;
        }

        /* fill the VarDesc Structure */
        MSFT_ReadLEDWords(
            &raw mut (*ptvd).vardesc.memid as *mut c_void,
            size_of::<INT>() as DWORD,
            pcx,
            offset + infolen + (cFuncs + i + 1) * size_of::<INT>() as i32,
        );
        (*ptvd).vardesc.varkind = (*pVarRec).VarKind as VARKIND;
        (*ptvd).vardesc.wVarFlags = LOWORD((*pVarRec).Flags as u32);
        MSFT_GetTdesc(pcx, (*pVarRec).DataType, &raw mut (*ptvd).vardesc.elemdescVar.tdesc);
        /* ptvd->vardesc.lpstrSchema; is reserved (SDK) FIXME?? */
        if (*pVarRec).VarKind as VARKIND == VAR_CONST {
            (*ptvd).vardesc.u.lpvarValue = heap_alloc_zero(size_of::<VARIANT>()) as *mut VARIANT;
            MSFT_ReadValue((*ptvd).vardesc.u.lpvarValue, (*pVarRec).OffsValue, pcx);
        } else {
            (*ptvd).vardesc.u.oInst = (*pVarRec).OffsValue as ULONG;
        }
        recoffset += reclength;
        i += 1;
        ptvd = ptvd.add(1);
    }
}

/* process Implemented Interfaces of a com class */
pub unsafe fn MSFT_DoImplTypes(pcx: *mut TLBContext, pTI: *mut ITypeInfoImpl, count: i32, mut offset: i32) {
    let mut refrec: MSFT_RefRecord = core::mem::zeroed();

    TRACE!();

    (*pTI).impltypes = TLBImplType_Alloc(count as UINT);
    let mut pImpl = (*pTI).impltypes;
    let mut i: i32 = 0;
    while i < count {
        if offset < 0 {
            break; /* paranoia */
        }
        MSFT_ReadLEDWords(
            &raw mut refrec as *mut c_void,
            size_of::<MSFT_RefRecord>() as DWORD,
            pcx,
            offset + (*(*pcx).pTblDir).pRefTab.offset,
        );
        (*pImpl).hRef = refrec.reftype as HREFTYPE;
        (*pImpl).implflags = refrec.flags;
        MSFT_CustData(pcx, refrec.oCustData, &raw mut (*pImpl).custdata_list);
        offset = refrec.onext;
        pImpl = pImpl.add(1);
        i += 1;
    }
}

/// when a 32-bit typelib is loaded in 64-bit mode, we need to resize pointers
/// and some structures, and fix the alignment (#ifdef _WIN64 in the C)
#[cfg(target_pointer_width = "64")]
pub unsafe fn TLB_fix_32on64_typeinfo(info: *mut ITypeInfoImpl) {
    if (*info).typeattr.typekind == TKIND_ALIAS {
        match (*(*info).tdescAlias).vt {
            VT_BSTR | VT_DISPATCH | VT_UNKNOWN | VT_PTR | VT_SAFEARRAY | VT_LPSTR | VT_LPWSTR => {
                (*info).typeattr.cbSizeInstance = size_of::<*mut c_void>() as ULONG;
                (*info).typeattr.cbAlignment = size_of::<*mut c_void>() as WORD;
            }
            VT_CARRAY | VT_USERDEFINED => {
                TLB_size_instance(
                    info,
                    SYS_WIN64,
                    (*info).tdescAlias,
                    &raw mut (*info).typeattr.cbSizeInstance,
                    &raw mut (*info).typeattr.cbAlignment,
                );
            }
            VT_VARIANT => {
                /* NOTE: preserved C fallthrough — VT_VARIANT sets size/align
                 * then FALLS THROUGH into the default arm (no break in the C). */
                (*info).typeattr.cbSizeInstance = size_of::<VARIANT>() as ULONG;
                (*info).typeattr.cbAlignment = 8;
                if (*info).typeattr.cbSizeInstance < size_of::<*mut c_void>() as ULONG {
                    (*info).typeattr.cbAlignment = (*info).typeattr.cbSizeInstance as WORD;
                } else {
                    (*info).typeattr.cbAlignment = size_of::<*mut c_void>() as WORD;
                }
            }
            _ => {
                if (*info).typeattr.cbSizeInstance < size_of::<*mut c_void>() as ULONG {
                    (*info).typeattr.cbAlignment = (*info).typeattr.cbSizeInstance as WORD;
                } else {
                    (*info).typeattr.cbAlignment = size_of::<*mut c_void>() as WORD;
                }
            }
        }
    } else if (*info).typeattr.typekind == TKIND_INTERFACE
        || (*info).typeattr.typekind == TKIND_DISPATCH
        || (*info).typeattr.typekind == TKIND_COCLASS
    {
        (*info).typeattr.cbSizeInstance = size_of::<*mut c_void>() as ULONG;
        (*info).typeattr.cbAlignment = size_of::<*mut c_void>() as WORD;
    }
}

/*
 * process a typeinfo record
 */
pub unsafe fn MSFT_DoTypeInfo(
    pcx: *mut TLBContext,
    count: i32,
    pLibInfo: *mut ITypeLibImpl,
) -> *mut ITypeInfoImpl {
    let mut tiBase: MSFT_TypeInfoBase = core::mem::zeroed();

    TRACE!();

    let ptiRet = super::itypeinfo::ITypeInfoImpl_Constructor();
    MSFT_ReadLEDWords(
        &raw mut tiBase as *mut c_void,
        size_of::<MSFT_TypeInfoBase>() as DWORD,
        pcx,
        (*(*pcx).pTblDir).pTypeInfoTab.offset + count * size_of::<MSFT_TypeInfoBase>() as i32,
    );

    /* this is where we are coming from */
    (*ptiRet).pTypeLib = pLibInfo;
    (*ptiRet).index = count;

    (*ptiRet).guid = MSFT_ReadGuid(tiBase.posguid, pcx);
    (*ptiRet).typeattr.lcid = (*pLibInfo).set_lcid; /* FIXME: correct? */
    (*ptiRet).typeattr.lpstrSchema = null_mut(); /* reserved */
    (*ptiRet).typeattr.cbSizeInstance = tiBase.size as ULONG;
    (*ptiRet).typeattr.typekind = tiBase.typekind & 0xF;
    (*ptiRet).typeattr.cFuncs = LOWORD(tiBase.cElement as u32);
    (*ptiRet).typeattr.cVars = HIWORD(tiBase.cElement as u32);
    (*ptiRet).typeattr.cbAlignment = ((tiBase.typekind >> 11) & 0x1F) as WORD; /* there are more flags there */
    (*ptiRet).typeattr.wTypeFlags = tiBase.flags as WORD;
    (*ptiRet).typeattr.wMajorVerNum = LOWORD(tiBase.version as u32);
    (*ptiRet).typeattr.wMinorVerNum = HIWORD(tiBase.version as u32);
    (*ptiRet).typeattr.cImplTypes = tiBase.cImplTypes as WORD;
    (*ptiRet).typeattr.cbSizeVft = tiBase.cbSizeVft as WORD;
    if (*ptiRet).typeattr.typekind == TKIND_ALIAS {
        let mut tmp: TYPEDESC = core::mem::zeroed();
        MSFT_GetTdesc(pcx, tiBase.datatype1, &raw mut tmp);
        (*ptiRet).tdescAlias =
            heap_alloc(TLB_SizeTypeDesc(&raw const tmp, TRUE)) as *mut TYPEDESC;
        TLB_CopyTypeDesc(null_mut(), &raw const tmp, (*ptiRet).tdescAlias as *mut c_void);
    }

    /*  FIXME: */
    /*    IDLDESC  idldescType; */ /* never saw this one != zero  */

    /* name, eventually add to a hash table */
    (*ptiRet).Name = MSFT_ReadName(pcx, tiBase.NameOffset);
    (*ptiRet).hreftype = MSFT_ReadHreftype(pcx, tiBase.NameOffset);
    TRACE!();
    /* help info */
    (*ptiRet).DocString = MSFT_ReadString(pcx, tiBase.docstringoffs);
    (*ptiRet).dwHelpStringContext = tiBase.helpstringcontext as DWORD;
    (*ptiRet).dwHelpContext = tiBase.helpcontext as DWORD;

    if (*ptiRet).typeattr.typekind == TKIND_MODULE {
        (*ptiRet).DllName = MSFT_ReadString(pcx, tiBase.datatype1);
    }

    /* note: InfoType's Help file and HelpStringDll come from the containing
     * library. Further HelpString and Docstring appear to be the same thing :(
     */
    /* functions */
    if (*ptiRet).typeattr.cFuncs > 0 {
        MSFT_DoFuncs(
            pcx,
            ptiRet,
            (*ptiRet).typeattr.cFuncs as i32,
            (*ptiRet).typeattr.cVars as i32,
            tiBase.memoffset,
            &raw mut (*ptiRet).funcdescs,
        );
    }
    /* variables */
    if (*ptiRet).typeattr.cVars > 0 {
        MSFT_DoVars(
            pcx,
            ptiRet,
            (*ptiRet).typeattr.cFuncs as i32,
            (*ptiRet).typeattr.cVars as i32,
            tiBase.memoffset,
            &raw mut (*ptiRet).vardescs,
        );
    }
    if (*ptiRet).typeattr.cImplTypes > 0 {
        match (*ptiRet).typeattr.typekind {
            k if k == TKIND_COCLASS => {
                MSFT_DoImplTypes(pcx, ptiRet, (*ptiRet).typeattr.cImplTypes as i32, tiBase.datatype1);
            }
            k if k == TKIND_DISPATCH => {
                /* This is not -1 when the interface is a non-base dual
                 * interface or when a dispinterface wraps an interface, i.e.,
                 * the idl 'dispinterface x {interface y;};'. Note however
                 * that GetRefTypeOfImplType(0) always returns a ref to
                 * IDispatch and not this interface. */
                if tiBase.datatype1 != -1 {
                    (*ptiRet).impltypes = TLBImplType_Alloc(1);
                    (*(*ptiRet).impltypes).hRef = tiBase.datatype1 as HREFTYPE;
                }
            }
            _ => {
                (*ptiRet).impltypes = TLBImplType_Alloc(1);
                (*(*ptiRet).impltypes).hRef = tiBase.datatype1 as HREFTYPE;
            }
        }
    }
    MSFT_CustData(pcx, tiBase.oCustData, (*ptiRet).pcustdata_list);

    TRACE!();

    ptiRet
}

pub unsafe fn MSFT_ReadAllStrings(pcx: *mut TLBContext) -> HRESULT {
    let mut len_str: INT16 = 0;
    let mut offs: i32 = 0;

    MSFT_Seek(pcx, (*(*pcx).pTblDir).pStringtab.offset);
    loop {
        if offs >= (*(*pcx).pTblDir).pStringtab.length {
            return S_OK;
        }

        MSFT_ReadLEWords(
            &raw mut len_str as *mut c_void,
            size_of::<INT16>() as DWORD,
            pcx,
            DO_NOT_SEEK,
        );
        let mut len_piece: INT16 = len_str + size_of::<INT16>() as INT16;
        if len_piece % 4 != 0 {
            len_piece = (len_piece + 4) & !0x3;
        }
        if len_piece < 8 {
            len_piece = 8;
        }

        let string = heap_alloc(len_piece as usize + 1) as *mut i8;
        MSFT_Read(
            string as *mut c_void,
            (len_piece as usize - size_of::<INT16>()) as DWORD,
            pcx,
            DO_NOT_SEEK,
        );
        *string.add(len_str as usize) = 0;

        let lengthInChars = MultiByteToWideChar(
            CP_ACP,
            MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
            string,
            -1,
            null_mut(),
            0,
        );
        if lengthInChars == 0 {
            heap_free(string as *mut c_void);
            return E_UNEXPECTED;
        }

        let tlbstr = heap_alloc(size_of::<TLBString>()) as *mut TLBString;

        (*tlbstr).offset = offs as UINT;
        (*tlbstr).str_ =
            SysAllocStringByteLen(null(), lengthInChars as UINT * size_of::<WCHAR>() as UINT);
        MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, string, -1, (*tlbstr).str_, lengthInChars);

        heap_free(string as *mut c_void);

        list_add_tail(
            &raw mut (*(*pcx).pLibInfo).string_list,
            &raw mut (*tlbstr).entry,
        );

        offs += len_piece as i32;
    }
}

pub unsafe fn MSFT_ReadAllRefs(pcx: *mut TLBContext) -> HRESULT {
    let mut offs: i32 = 0;

    MSFT_Seek(pcx, (*(*pcx).pTblDir).pImpInfo.offset);
    while offs < (*(*pcx).pTblDir).pImpInfo.length {
        let mut impinfo: MSFT_ImpInfo = core::mem::zeroed();

        MSFT_ReadLEDWords(
            &raw mut impinfo as *mut c_void,
            size_of::<MSFT_ImpInfo>() as DWORD,
            pcx,
            DO_NOT_SEEK,
        );

        let ref_ = heap_alloc_zero(size_of::<TLBRefType>()) as *mut TLBRefType;
        list_add_tail(&raw mut (*(*pcx).pLibInfo).ref_list, &raw mut (*ref_).entry);

        /* LIST_FOR_EACH_ENTRY with post-loop sentinel test, hand-rolled */
        let implib_list = &raw mut (*(*pcx).pLibInfo).implib_list;
        let mut pImpLib: *mut TLBImpLib = null_mut();
        let mut cur = (*implib_list).next;
        while cur != implib_list {
            let candidate = list_entry!(cur, TLBImpLib, entry);
            if (*candidate).offset == impinfo.oImpFile {
                pImpLib = candidate;
                break;
            }
            cur = (*cur).next;
        }

        if !pImpLib.is_null() {
            (*ref_).reference = offs as HREFTYPE;
            (*ref_).pImpTLInfo = pImpLib;
            if impinfo.flags & MSFT_IMPINFO_OFFSET_IS_GUID != 0 {
                (*ref_).guid = MSFT_ReadGuid(impinfo.oGuid, pcx);
                TRACE!();
                (*ref_).index = TLB_REF_USE_GUID;
            } else {
                (*ref_).index = impinfo.oGuid;
            }
        } else {
            ERR!("Cannot find a reference");
            (*ref_).reference = -1i32 as HREFTYPE;
            (*ref_).pImpTLInfo = TLB_REF_NOT_FOUND;
        }

        offs += size_of::<MSFT_ImpInfo>() as i32;
    }

    S_OK
}

// ---------------------------------------------------------------------------
// typelib cache.
//
// Because type library parsing has some degree of overhead, and some apps
// repeatedly load the same typelibs over and over, we cache them here.
// According to MSDN Microsoft have a similar scheme in place. This will cause
// a deliberate memory leak, but generally losing RAM for cycles is an
// acceptable tradeoff here.
//
// DEVIATION (same class as compobj DllMain CS init): the C initializes
// `cache_section` + `tlb_cache` with self-referential static initializers;
// Rust statics can't self-reference, and typelib.c has no DllMain hook, so
// both are lazily initialized through InitOnceExecuteOnce on first use.
// ---------------------------------------------------------------------------

static mut tlb_cache: list = list {
    next: null_mut(),
    prev: null_mut(),
};
static mut cache_section: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: null_mut(),
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: null_mut(),
    LockSemaphore: null_mut(),
    SpinCount: 0,
};
static mut cache_init_once: INIT_ONCE = INIT_ONCE { Ptr: null_mut() };

unsafe extern "system" fn cache_section_init(
    _once: *mut INIT_ONCE,
    _param: *mut c_void,
    _ctx: *mut *mut c_void,
) -> BOOL {
    InitializeCriticalSection(&raw mut cache_section);
    list_init(&raw mut tlb_cache);
    TRUE
}

pub unsafe fn tlb_cache_lock() -> (*mut list, *mut CRITICAL_SECTION) {
    InitOnceExecuteOnce(&raw mut cache_init_once, cache_section_init, null_mut(), null_mut());
    EnterCriticalSection(&raw mut cache_section);
    (&raw mut tlb_cache, &raw mut cache_section)
}

pub unsafe fn tlb_cache_unlock() {
    LeaveCriticalSection(&raw mut cache_section);
}

/// Remove a dying typelib from the cache (used by ITypeLib2 Release in
/// itypelib.rs — the list/CS statics live here).
pub unsafe fn tlb_cache_remove_entry(entry: *mut list) {
    let _ = tlb_cache_lock();
    list_remove(entry);
    tlb_cache_unlock();
}

// ---------------------------------------------------------------------------
// TLB_PEFile — typelib in a PE resource
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct TLB_PEFile {
    pub IUnknown_iface: IUnknown,
    pub refs: LONG,
    pub dll: HMODULE,
    pub typelib_resource: HRSRC,
    pub typelib_global: HGLOBAL,
    pub typelib_base: LPVOID,
}

#[inline]
unsafe fn pefile_impl_from_IUnknown(iface: *mut IUnknown) -> *mut TLB_PEFile {
    containing_record!(iface, TLB_PEFile, IUnknown_iface)
}

unsafe extern "system" fn TLB_PEFile_QueryInterface(
    iface: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if guid_eq(riid, &raw const IID_IUnknown) {
        *ppv = iface as *mut c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }
    *ppv = null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn TLB_PEFile_AddRef(iface: *mut IUnknown) -> ULONG {
    let This = pefile_impl_from_IUnknown(iface);
    InterlockedIncrement(&raw mut (*This).refs) as ULONG
}

unsafe extern "system" fn TLB_PEFile_Release(iface: *mut IUnknown) -> ULONG {
    let This = pefile_impl_from_IUnknown(iface);
    let refs = InterlockedDecrement(&raw mut (*This).refs) as ULONG;
    if refs == 0 {
        if !(*This).typelib_global.is_null() {
            FreeResource((*This).typelib_global);
        }
        if !(*This).dll.is_null() {
            FreeLibrary((*This).dll);
        }
        heap_free(This as *mut c_void);
    }
    refs
}

static TLB_PEFile_Vtable: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: TLB_PEFile_QueryInterface,
    AddRef: TLB_PEFile_AddRef,
    Release: TLB_PEFile_Release,
};

unsafe fn TLB_PEFile_Open(
    path: LPCWSTR,
    index: INT,
    ppBase: *mut LPVOID,
    pdwTLBLength: *mut DWORD,
    ppFile: *mut *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT = TYPE_E_CANTLOADLIBRARY;

    let This = heap_alloc(size_of::<TLB_PEFile>()) as *mut TLB_PEFile;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IUnknown_iface.lpVtbl = &TLB_PEFile_Vtable;
    (*This).refs = 1;
    (*This).dll = null_mut();
    (*This).typelib_resource = null_mut();
    (*This).typelib_global = null_mut();
    (*This).typelib_base = null_mut();

    (*This).dll = LoadLibraryExW(
        path,
        null_mut(),
        DONT_RESOLVE_DLL_REFERENCES | LOAD_LIBRARY_AS_DATAFILE | LOAD_WITH_ALTERED_SEARCH_PATH,
    );

    if !(*This).dll.is_null() {
        let TYPELIBW = wide!("TYPELIB");
        (*This).typelib_resource = FindResourceW((*This).dll, MAKEINTRESOURCEW(index), TYPELIBW.as_ptr());
        if !(*This).typelib_resource.is_null() {
            (*This).typelib_global = LoadResource((*This).dll, (*This).typelib_resource);
            if !(*This).typelib_global.is_null() {
                (*This).typelib_base = LockResource((*This).typelib_global);

                if !(*This).typelib_base.is_null() {
                    *pdwTLBLength = SizeofResource((*This).dll, (*This).typelib_resource);
                    *ppBase = (*This).typelib_base;
                    *ppFile = &raw mut (*This).IUnknown_iface;
                    return S_OK;
                }
            }
        }

        TRACE!();
        hr = E_FAIL;
    }

    TLB_PEFile_Release(&raw mut (*This).IUnknown_iface);
    hr
}

// ---------------------------------------------------------------------------
// TLB_NEFile — typelib in an NE (win16) resource, via lz32
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct TLB_NEFile {
    pub IUnknown_iface: IUnknown,
    pub refs: LONG,
    pub typelib_base: LPVOID,
}

#[inline]
unsafe fn nefile_impl_from_IUnknown(iface: *mut IUnknown) -> *mut TLB_NEFile {
    containing_record!(iface, TLB_NEFile, IUnknown_iface)
}

unsafe extern "system" fn TLB_NEFile_QueryInterface(
    iface: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if guid_eq(riid, &raw const IID_IUnknown) {
        *ppv = iface as *mut c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }
    *ppv = null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn TLB_NEFile_AddRef(iface: *mut IUnknown) -> ULONG {
    let This = nefile_impl_from_IUnknown(iface);
    InterlockedIncrement(&raw mut (*This).refs) as ULONG
}

unsafe extern "system" fn TLB_NEFile_Release(iface: *mut IUnknown) -> ULONG {
    let This = nefile_impl_from_IUnknown(iface);
    let refs = InterlockedDecrement(&raw mut (*This).refs) as ULONG;
    if refs == 0 {
        heap_free((*This).typelib_base);
        heap_free(This as *mut c_void);
    }
    refs
}

static TLB_NEFile_Vtable: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: TLB_NEFile_QueryInterface,
    AddRef: TLB_NEFile_AddRef,
    Release: TLB_NEFile_Release,
};

/***********************************************************************
 *           read_xx_header         [internal]
 */
unsafe fn read_xx_header(lzfd: HFILE) -> i32 {
    let mut mzh: IMAGE_DOS_HEADER = core::mem::zeroed();
    let mut magic: [i8; 3] = [0; 3];

    LZSeek(lzfd, 0, SEEK_SET);
    if size_of::<IMAGE_DOS_HEADER>() as INT
        != LZRead(lzfd, &raw mut mzh as *mut i8, size_of::<IMAGE_DOS_HEADER>() as INT)
    {
        return 0;
    }
    if mzh.e_magic != IMAGE_DOS_SIGNATURE {
        return 0;
    }

    LZSeek(lzfd, mzh.e_lfanew, SEEK_SET);
    if 2 != LZRead(lzfd, magic.as_mut_ptr(), 2) {
        return 0;
    }

    LZSeek(lzfd, mzh.e_lfanew, SEEK_SET);

    if magic[0] == b'N' as i8 && magic[1] == b'E' as i8 {
        return IMAGE_OS2_SIGNATURE;
    }
    if magic[0] == b'P' as i8 && magic[1] == b'E' as i8 {
        return IMAGE_NT_SIGNATURE;
    }

    magic[2] = 0;
    WARN!();
    0
}

/***********************************************************************
 *           find_ne_resource         [internal]
 */
unsafe fn find_ne_resource(
    lzfd: HFILE,
    typeid: LPCSTR,
    resid: LPCSTR,
    resLen: *mut DWORD,
    resOff: *mut DWORD,
) -> BOOL {
    let mut nehd: IMAGE_OS2_HEADER = core::mem::zeroed();

    /* Read in NE header */
    let nehdoffset: DWORD = LZSeek(lzfd, 0, SEEK_CUR) as DWORD;
    if size_of::<IMAGE_OS2_HEADER>() as INT
        != LZRead(lzfd, &raw mut nehd as *mut i8, size_of::<IMAGE_OS2_HEADER>() as INT)
    {
        return FALSE;
    }

    let resTabSize: DWORD = (nehd.ne_restab - nehd.ne_rsrctab) as DWORD;
    if resTabSize == 0 {
        TRACE!();
        return FALSE;
    }

    /* Read in resource table */
    let resTab = heap_alloc(resTabSize as usize) as *mut u8;
    if resTab.is_null() {
        return FALSE;
    }

    LZSeek(lzfd, nehd.ne_rsrctab as LONG + nehdoffset as LONG, SEEK_SET);
    if resTabSize as INT != LZRead(lzfd, resTab as *mut i8, resTabSize as INT) {
        heap_free(resTab as *mut c_void);
        return FALSE;
    }

    /* Find resource */
    let mut typeInfoPtr = resTab.add(2) as *mut NE_TYPEINFO;
    let mut found_type = false;

    if !IS_INTRESOURCE_PTR(typeid as *const c_void) {
        /* named type */
        let len = strlen(typeid) as u8;
        loop {
            let ti: NE_TYPEINFO = read_unaligned(typeInfoPtr);
            if ti.type_id == 0 {
                break;
            }
            if ti.type_id & 0x8000 == 0 {
                let p = resTab.add(ti.type_id as usize);
                if *p == len && _strnicmp(p.add(1) as *const i8, typeid, len as usize) == 0 {
                    found_type = true;
                    break;
                }
            }
            typeInfoPtr = (typeInfoPtr.add(1) as *mut u8)
                .add(ti.count as usize * size_of::<NE_NAMEINFO>())
                as *mut NE_TYPEINFO;
        }
    } else {
        /* numeric type id */
        let id: WORD = (typeid as usize as u32 & 0xffff) as WORD | 0x8000;
        loop {
            let ti: NE_TYPEINFO = read_unaligned(typeInfoPtr);
            if ti.type_id == 0 {
                break;
            }
            if ti.type_id == id {
                found_type = true;
                break;
            }
            typeInfoPtr = (typeInfoPtr.add(1) as *mut u8)
                .add(ti.count as usize * size_of::<NE_NAMEINFO>())
                as *mut NE_TYPEINFO;
        }
    }
    if !found_type {
        TRACE!();
        heap_free(resTab as *mut c_void);
        return FALSE;
    }

    let ti: NE_TYPEINFO = read_unaligned(typeInfoPtr);
    let mut nameInfoPtr = typeInfoPtr.add(1) as *mut NE_NAMEINFO;
    let mut found_name = false;
    let mut ni: NE_NAMEINFO = core::mem::zeroed();

    if !IS_INTRESOURCE_PTR(resid as *const c_void) {
        /* named resource */
        let len = strlen(resid) as u8;
        let mut count = ti.count as i32;
        while count > 0 {
            ni = read_unaligned(nameInfoPtr);
            let p = resTab.add(ni.id as usize);
            if ni.id & 0x8000 == 0
                && *p == len
                && _strnicmp(p.add(1) as *const i8, resid, len as usize) == 0
            {
                found_name = true;
                break;
            }
            count -= 1;
            nameInfoPtr = nameInfoPtr.add(1);
        }
    } else {
        /* numeric resource id */
        let id: WORD = (resid as usize as u32 & 0xffff) as WORD | 0x8000;
        let mut count = ti.count as i32;
        while count > 0 {
            ni = read_unaligned(nameInfoPtr);
            if ni.id == id {
                found_name = true;
                break;
            }
            count -= 1;
            nameInfoPtr = nameInfoPtr.add(1);
        }
    }
    if !found_name {
        TRACE!();
        heap_free(resTab as *mut c_void);
        return FALSE;
    }

    /* Return resource data */
    let shift = read_unaligned(resTab as *const WORD);
    if !resLen.is_null() {
        *resLen = (ni.length as DWORD) << shift;
    }
    if !resOff.is_null() {
        *resOff = (ni.offset as DWORD) << shift;
    }

    heap_free(resTab as *mut c_void);
    TRUE
}

unsafe fn TLB_NEFile_Open(
    path: LPCWSTR,
    index: INT,
    ppBase: *mut LPVOID,
    pdwTLBLength: *mut DWORD,
    ppFile: *mut *mut IUnknown,
) -> HRESULT {
    let mut ofs: OFSTRUCT = core::mem::zeroed();
    let mut hr: HRESULT = TYPE_E_CANTLOADLIBRARY;

    let This = heap_alloc(size_of::<TLB_NEFile>()) as *mut TLB_NEFile;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IUnknown_iface.lpVtbl = &TLB_NEFile_Vtable;
    (*This).refs = 1;
    (*This).typelib_base = null_mut();

    let lzfd: HFILE = LZOpenFileW(path as LPWSTR, &raw mut ofs, OF_READ as WORD);
    if lzfd >= 0 && read_xx_header(lzfd) == IMAGE_OS2_SIGNATURE {
        let mut reslen: DWORD = 0;
        let mut offset: DWORD = 0;
        if find_ne_resource(
            lzfd,
            b"TYPELIB\0".as_ptr() as LPCSTR,
            MAKEINTRESOURCEA(index),
            &raw mut reslen,
            &raw mut offset,
        ) != FALSE
        {
            (*This).typelib_base = heap_alloc(reslen as usize);
            if (*This).typelib_base.is_null() {
                hr = E_OUTOFMEMORY;
            } else {
                LZSeek(lzfd, offset as LONG, SEEK_SET);
                let reslen = LZRead(lzfd, (*This).typelib_base as *mut i8, reslen as INT) as DWORD;
                LZClose(lzfd);
                *ppBase = (*This).typelib_base;
                *pdwTLBLength = reslen;
                *ppFile = &raw mut (*This).IUnknown_iface;
                return S_OK;
            }
        }
    }

    if lzfd >= 0 {
        LZClose(lzfd);
    }
    TLB_NEFile_Release(&raw mut (*This).IUnknown_iface);
    hr
}

// ---------------------------------------------------------------------------
// TLB_Mapping — plain .tlb file, memory-mapped
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct TLB_Mapping {
    pub IUnknown_iface: IUnknown,
    pub refs: LONG,
    pub file: HANDLE,
    pub mapping: HANDLE,
    pub typelib_base: LPVOID,
}

#[inline]
unsafe fn mapping_impl_from_IUnknown(iface: *mut IUnknown) -> *mut TLB_Mapping {
    containing_record!(iface, TLB_Mapping, IUnknown_iface)
}

unsafe extern "system" fn TLB_Mapping_QueryInterface(
    iface: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if guid_eq(riid, &raw const IID_IUnknown) {
        *ppv = iface as *mut c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }
    *ppv = null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn TLB_Mapping_AddRef(iface: *mut IUnknown) -> ULONG {
    let This = mapping_impl_from_IUnknown(iface);
    InterlockedIncrement(&raw mut (*This).refs) as ULONG
}

unsafe extern "system" fn TLB_Mapping_Release(iface: *mut IUnknown) -> ULONG {
    let This = mapping_impl_from_IUnknown(iface);
    let refs = InterlockedDecrement(&raw mut (*This).refs) as ULONG;
    if refs == 0 {
        if !(*This).typelib_base.is_null() {
            UnmapViewOfFile((*This).typelib_base);
        }
        if !(*This).mapping.is_null() {
            CloseHandle((*This).mapping);
        }
        if (*This).file != INVALID_HANDLE_VALUE {
            CloseHandle((*This).file);
        }
        heap_free(This as *mut c_void);
    }
    refs
}

static TLB_Mapping_Vtable: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: TLB_Mapping_QueryInterface,
    AddRef: TLB_Mapping_AddRef,
    Release: TLB_Mapping_Release,
};

unsafe fn TLB_Mapping_Open(
    path: LPCWSTR,
    ppBase: *mut LPVOID,
    pdwTLBLength: *mut DWORD,
    ppFile: *mut *mut IUnknown,
) -> HRESULT {
    let This = heap_alloc(size_of::<TLB_Mapping>()) as *mut TLB_Mapping;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IUnknown_iface.lpVtbl = &TLB_Mapping_Vtable;
    (*This).refs = 1;
    (*This).file = INVALID_HANDLE_VALUE;
    (*This).mapping = null_mut();
    (*This).typelib_base = null_mut();

    (*This).file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, null_mut(), OPEN_EXISTING, 0, null_mut());
    if INVALID_HANDLE_VALUE != (*This).file {
        (*This).mapping =
            CreateFileMappingW((*This).file, null_mut(), PAGE_READONLY | SEC_COMMIT, 0, 0, null());
        if !(*This).mapping.is_null() {
            (*This).typelib_base = MapViewOfFile((*This).mapping, FILE_MAP_READ, 0, 0, 0);
            if !(*This).typelib_base.is_null() {
                /* retrieve file size */
                *pdwTLBLength = GetFileSize((*This).file, null_mut());
                *ppBase = (*This).typelib_base;
                *ppFile = &raw mut (*This).IUnknown_iface;
                return S_OK;
            }
        }
    }

    IUnknown_Release(&raw mut (*This).IUnknown_iface);
    TYPE_E_CANTLOADLIBRARY
}

/****************************************************************************
 *	TLB_ReadTypeLib
 *
 * find the type of the typelib file and map the typelib resource into
 * the memory
 */
pub unsafe fn TLB_ReadTypeLib(
    pszFileName: LPCWSTR,
    pszPath: LPWSTR,
    cchPath: UINT,
    ppTypeLib: *mut *mut ITypeLib2,
) -> HRESULT {
    let mut ret: HRESULT;
    let mut index: INT = 1;
    let mut file: LPWSTR = pszFileName as LPWSTR;
    let mut pBase: LPVOID = null_mut();
    let mut dwTLBLength: DWORD = 0;
    let mut pFile: *mut IUnknown = null_mut();

    *ppTypeLib = null_mut();

    let mut index_str = wcsrchr(pszFileName, '\\' as u16);
    if !index_str.is_null() {
        index_str = index_str.add(1);
        if *index_str != 0 {
            let mut end_ptr: *mut u16 = null_mut();
            let idx = wcstol(index_str, &raw mut end_ptr, 10);
            if *end_ptr == 0 {
                let str_len = index_str.offset_from(pszFileName) as usize - 1;
                index = idx;
                file = heap_alloc((str_len + 1) * size_of::<WCHAR>()) as LPWSTR;
                memcpy(
                    file as *mut c_void,
                    pszFileName as *const c_void,
                    str_len * size_of::<WCHAR>(),
                );
                *file.add(str_len) = 0;
            }
        }
    }

    if SearchPathW(null(), file, null(), cchPath, pszPath, null_mut()) == 0 {
        if !wcschr(file, '\\' as u16).is_null() {
            lstrcpyW(pszPath, file);
        } else {
            let len = GetSystemDirectoryW(pszPath, cchPath) as usize;
            *pszPath.add(len) = '\\' as u16;
            memcpy(
                pszPath.add(len + 1) as *mut c_void,
                file as *const c_void,
                (lstrlenW(file) as usize + 1) * size_of::<WCHAR>(),
            );
        }
    }

    if file != pszFileName as LPWSTR {
        heap_free(file as *mut c_void);
    }

    let h = CreateFileW(
        pszPath,
        GENERIC_READ,
        0,
        null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        null_mut(),
    );
    if h != INVALID_HANDLE_VALUE {
        let mut size_info: FILE_NAME_INFORMATION = core::mem::zeroed();

        /* GetFileInformationByHandleEx returns the path of the file without
         * WOW64 redirection */
        let br = GetFileInformationByHandleEx(
            h,
            FileNameInfo,
            &raw mut size_info as *mut c_void,
            size_of::<FILE_NAME_INFORMATION>() as DWORD,
        );
        if br != FALSE || GetLastError() == ERROR_MORE_DATA {
            let size = size_of::<FILE_NAME_INFORMATION>() as DWORD
                + size_info.FileNameLength
                + size_of::<WCHAR>() as DWORD;

            let info =
                HeapAlloc(GetProcessHeap(), 0, size as usize) as *mut FILE_NAME_INFORMATION;

            let br = GetFileInformationByHandleEx(h, FileNameInfo, info as *mut c_void, size);
            if br != FALSE {
                *(&raw mut (*info).FileName as *mut WCHAR)
                    .add((*info).FileNameLength as usize / size_of::<WCHAR>()) = 0;
                lstrcpynW(
                    pszPath.add(2),
                    &raw const (*info).FileName as *const WCHAR,
                    (cchPath - 2) as INT,
                );
            }

            HeapFree(GetProcessHeap(), 0, info as *mut c_void);
        }

        CloseHandle(h);
    }

    TRACE!();

    /* We look the path up in the typelib cache. If found, we just addref it,
     * and return the pointer. */
    let (cache, _) = tlb_cache_lock();
    let mut cur = (*cache).next;
    while cur != cache {
        let entry = list_entry!(cur, ITypeLibImpl, entry);
        if _wcsicmp((*entry).path, pszPath) == 0 && (*entry).index == index {
            TRACE!();
            *ppTypeLib = &raw mut (*entry).ITypeLib2_iface;
            let tl = *ppTypeLib;
            ((*(*tl).lpVtbl).AddRef)(tl);
            tlb_cache_unlock();
            return S_OK;
        }
        cur = (*cur).next;
    }
    tlb_cache_unlock();

    /* now actually load and parse the typelib */

    ret = TLB_PEFile_Open(pszPath, index, &raw mut pBase, &raw mut dwTLBLength, &raw mut pFile);
    if ret == TYPE_E_CANTLOADLIBRARY {
        ret = TLB_NEFile_Open(pszPath, index, &raw mut pBase, &raw mut dwTLBLength, &raw mut pFile);
    }
    if ret == TYPE_E_CANTLOADLIBRARY {
        ret = TLB_Mapping_Open(pszPath, &raw mut pBase, &raw mut dwTLBLength, &raw mut pFile);
    }
    if SUCCEEDED(ret) {
        if dwTLBLength >= 4 {
            let dwSignature = FromLEDWord(*(pBase as *const DWORD));
            if dwSignature == MSFT_SIGNATURE {
                *ppTypeLib = ITypeLib2_Constructor_MSFT(pBase, dwTLBLength);
            } else if dwSignature == SLTG_SIGNATURE {
                *ppTypeLib = super::sltg::ITypeLib2_Constructor_SLTG(pBase, dwTLBLength);
            } else {
                FIXME!("Header type magic 0x{:08x} not supported.", dwSignature);
                ret = TYPE_E_CANTLOADLIBRARY;
            }
        } else {
            ret = TYPE_E_CANTLOADLIBRARY;
        }
        IUnknown_Release(pFile);
    }

    if !(*ppTypeLib).is_null() {
        let impl_ = impl_from_ITypeLib2(*ppTypeLib);

        TRACE!();
        (*impl_).path =
            heap_alloc((lstrlenW(pszPath) as usize + 1) * size_of::<WCHAR>()) as *mut WCHAR;
        lstrcpyW((*impl_).path, pszPath);
        /* We should really canonicalise the path here. */
        (*impl_).index = index;

        /* FIXME: check if it has added already in the meantime */
        let (cache, _) = tlb_cache_lock();
        list_add_head(cache, &raw mut (*impl_).entry);
        tlb_cache_unlock();
        ret = S_OK;
    } else {
        if ret != E_FAIL {
            ERR!(
                "Loading of typelib {} failed with error {}",
                DbgW(pszFileName),
                GetLastError()
            );
        }
        ret = TYPE_E_CANTLOADLIBRARY;
    }

    ret
}

/*================== ITypeLib(2) Methods ===================================*/

pub unsafe fn TypeLibImpl_Constructor() -> *mut ITypeLibImpl {
    let pTypeLibImpl = heap_alloc_zero(size_of::<ITypeLibImpl>()) as *mut ITypeLibImpl;
    if pTypeLibImpl.is_null() {
        return null_mut();
    }

    (*pTypeLibImpl).ITypeLib2_iface.lpVtbl = &super::itypelib::tlbvt;
    (*pTypeLibImpl).ITypeComp_iface.lpVtbl = &super::itypelib::tlbtcvt;
    (*pTypeLibImpl).ICreateTypeLib2_iface.lpVtbl = &super::writer::CreateTypeLib2Vtbl;
    (*pTypeLibImpl).ref_ = 1;

    list_init(&raw mut (*pTypeLibImpl).implib_list);
    list_init(&raw mut (*pTypeLibImpl).custdata_list);
    list_init(&raw mut (*pTypeLibImpl).name_list);
    list_init(&raw mut (*pTypeLibImpl).string_list);
    list_init(&raw mut (*pTypeLibImpl).guid_list);
    list_init(&raw mut (*pTypeLibImpl).ref_list);
    (*pTypeLibImpl).dispatch_href = -1i32 as HREFTYPE;

    pTypeLibImpl
}

/****************************************************************************
 *	ITypeLib2_Constructor_MSFT
 *
 * loading an MSFT typelib from an in-memory image
 */
pub unsafe fn ITypeLib2_Constructor_MSFT(pLib: LPVOID, dwTLBLength: DWORD) -> *mut ITypeLib2 {
    let mut cx: TLBContext = core::mem::zeroed();
    let mut tlbHeader: MSFT_Header = core::mem::zeroed();
    let mut tlbSegDir: MSFT_SegDir = core::mem::zeroed();

    TRACE!();

    let pTypeLibImpl = TypeLibImpl_Constructor();
    if pTypeLibImpl.is_null() {
        return null_mut();
    }

    /* get pointer to beginning of typelib data */
    cx.pos = 0;
    cx.oStart = 0;
    cx.mapping = pLib;
    cx.pLibInfo = pTypeLibImpl;
    cx.length = dwTLBLength;

    /* read header */
    MSFT_ReadLEDWords(
        &raw mut tlbHeader as *mut c_void,
        size_of::<MSFT_Header>() as DWORD,
        &raw mut cx,
        0,
    );
    TRACE!();
    if tlbHeader.magic1 as u32 != MSFT_SIGNATURE {
        FIXME!("Header type magic 0x{:08x} not supported.", tlbHeader.magic1);
        return null_mut();
    }
    TRACE!();

    /* there is a small amount of information here until the next important
     * part: the segment directory. Try to calculate the amount of data */
    let lPSegDir: LONG = (size_of::<MSFT_Header>()
        + tlbHeader.nrtypeinfos as usize * 4
        + if tlbHeader.varflags & HELPDLLFLAG as i32 != 0 { 4 } else { 0 }) as LONG;

    /* now read the segment directory */
    TRACE!();
    MSFT_ReadLEDWords(
        &raw mut tlbSegDir as *mut c_void,
        size_of::<MSFT_SegDir>() as DWORD,
        &raw mut cx,
        lPSegDir,
    );
    cx.pTblDir = &raw mut tlbSegDir;

    /* just check two entries */
    if tlbSegDir.pTypeInfoTab.res0c != 0x0F || tlbSegDir.pImpInfo.res0c != 0x0F {
        ERR!("cannot find the table directory, ptr=0x{:x}", lPSegDir);
        heap_free(pTypeLibImpl as *mut c_void);
        return null_mut();
    }

    MSFT_ReadAllNames(&raw mut cx);
    MSFT_ReadAllStrings(&raw mut cx);
    MSFT_ReadAllGuids(&raw mut cx);

    /* now fill our internal data */
    /* TLIBATTR fields */
    (*pTypeLibImpl).guid = MSFT_ReadGuid(tlbHeader.posguid, &raw mut cx);

    (*pTypeLibImpl).syskind = tlbHeader.varflags & 0x0f; /* check the mask */
    (*pTypeLibImpl).ptr_size = get_ptr_size((*pTypeLibImpl).syskind);
    (*pTypeLibImpl).ver_major = LOWORD(tlbHeader.version as u32);
    (*pTypeLibImpl).ver_minor = HIWORD(tlbHeader.version as u32);
    (*pTypeLibImpl).libflags =
        ((tlbHeader.flags as WORD) & 0xffff) /* check mask */ | LIBFLAG_FHASDISKIMAGE;

    (*pTypeLibImpl).set_lcid = tlbHeader.lcid2 as LCID;
    (*pTypeLibImpl).lcid = tlbHeader.lcid as LCID;

    /* name, eventually add to a hash table */
    (*pTypeLibImpl).Name = MSFT_ReadName(&raw mut cx, tlbHeader.NameOffset);

    /* help info */
    (*pTypeLibImpl).DocString = MSFT_ReadString(&raw mut cx, tlbHeader.helpstring);
    (*pTypeLibImpl).HelpFile = MSFT_ReadString(&raw mut cx, tlbHeader.helpfile);

    if tlbHeader.varflags & HELPDLLFLAG as i32 != 0 {
        let mut offset: i32 = 0;
        MSFT_ReadLEDWords(
            &raw mut offset as *mut c_void,
            size_of::<i32>() as DWORD,
            &raw mut cx,
            size_of::<MSFT_Header>() as LONG,
        );
        (*pTypeLibImpl).HelpStringDll = MSFT_ReadString(&raw mut cx, offset);
    }

    (*pTypeLibImpl).dwHelpContext = tlbHeader.helpstringcontext as DWORD;

    /* custom data */
    if tlbHeader.CustomDataOffset >= 0 {
        MSFT_CustData(
            &raw mut cx,
            tlbHeader.CustomDataOffset,
            &raw mut (*pTypeLibImpl).custdata_list,
        );
    }

    /* fill in type descriptions */
    if tlbSegDir.pTypdescTab.length > 0 {
        let cTD = tlbSegDir.pTypdescTab.length as usize / (2 * size_of::<INT>());
        let mut td: [INT16; 4] = [0; 4];
        (*pTypeLibImpl).ctTypeDesc = cTD as i32;
        (*pTypeLibImpl).pTypeDesc =
            heap_alloc_zero(cTD * size_of::<TYPEDESC>()) as *mut TYPEDESC;
        MSFT_ReadLEWords(
            td.as_mut_ptr() as *mut c_void,
            size_of::<[INT16; 4]>() as DWORD,
            &raw mut cx,
            tlbSegDir.pTypdescTab.offset,
        );
        let mut i: usize = 0;
        while i < cTD {
            /* FIXME: add several sanity checks here */
            (*(*pTypeLibImpl).pTypeDesc.add(i)).vt = (td[0] & VT_TYPEMASK as i16) as VARTYPE;
            if td[0] as i32 == VT_PTR as i32 || td[0] as i32 == VT_SAFEARRAY as i32 {
                /* FIXME: check safearray */
                if td[3] < 0 {
                    (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lptdesc =
                        (&raw mut std_typedesc).cast::<TYPEDESC>().offset(td[2] as isize);
                } else {
                    (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lptdesc =
                        (*pTypeLibImpl).pTypeDesc.offset(td[2] as isize / 8);
                }
            } else if td[0] as i32 == VT_CARRAY as i32 {
                /* array descr table here */
                (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lpadesc =
                    td[2] as INT_PTR as *mut ARRAYDESC; /* temp store offset in */
            } else if td[0] as i32 == VT_USERDEFINED as i32 {
                (*(*pTypeLibImpl).pTypeDesc.add(i)).u.hreftype =
                    MAKELONG(td[2] as u16, td[3] as u16);
            }
            i += 1;
            if i < cTD {
                MSFT_ReadLEWords(
                    td.as_mut_ptr() as *mut c_void,
                    size_of::<[INT16; 4]>() as DWORD,
                    &raw mut cx,
                    DO_NOT_SEEK,
                );
            }
        }

        /* second time around to fill the array subscript info */
        let mut i: usize = 0;
        while i < cTD {
            if (*(*pTypeLibImpl).pTypeDesc.add(i)).vt != VT_CARRAY {
                i += 1;
                continue;
            }
            if tlbSegDir.pArrayDescriptions.offset > 0 {
                MSFT_ReadLEWords(
                    td.as_mut_ptr() as *mut c_void,
                    size_of::<[INT16; 4]>() as DWORD,
                    &raw mut cx,
                    tlbSegDir.pArrayDescriptions.offset
                        + (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lpadesc as INT_PTR as i32,
                );
                (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lpadesc = heap_alloc_zero(
                    size_of::<ARRAYDESC>()
                        + size_of::<SAFEARRAYBOUND>() * (td[3] as usize).wrapping_sub(1),
                ) as *mut ARRAYDESC;

                let adesc = (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lpadesc;

                if td[1] < 0 {
                    (*adesc).tdescElem.vt = (td[0] & VT_TYPEMASK as i16) as VARTYPE;
                } else {
                    (*adesc).tdescElem =
                        *(*cx.pLibInfo).pTypeDesc.add(td[0] as usize / (2 * size_of::<INT>()));
                }

                (*adesc).cDims = td[2] as USHORT;

                let mut j: i32 = 0;
                while j < td[2] as i32 {
                    let bound = (&raw mut (*adesc).rgbounds)
                        .cast::<SAFEARRAYBOUND>()
                        .add(j as usize);
                    MSFT_ReadLEDWords(
                        &raw mut (*bound).cElements as *mut c_void,
                        size_of::<INT>() as DWORD,
                        &raw mut cx,
                        DO_NOT_SEEK,
                    );
                    MSFT_ReadLEDWords(
                        &raw mut (*bound).lLbound as *mut c_void,
                        size_of::<INT>() as DWORD,
                        &raw mut cx,
                        DO_NOT_SEEK,
                    );
                    j += 1;
                }
            } else {
                (*(*pTypeLibImpl).pTypeDesc.add(i)).u.lpadesc = null_mut();
                ERR!("didn't find array description data");
            }
            i += 1;
        }
    }

    /* imported type libs */
    if tlbSegDir.pImpFiles.offset > 0 {
        let mut oGuid: i32 = 0;
        let mut offset: i32 = tlbSegDir.pImpFiles.offset;
        let mut size: UINT16 = 0;

        while offset < tlbSegDir.pImpFiles.offset + tlbSegDir.pImpFiles.length {
            let pImpLib = heap_alloc_zero(size_of::<TLBImpLib>()) as *mut TLBImpLib;
            (*pImpLib).offset = offset - tlbSegDir.pImpFiles.offset;
            MSFT_ReadLEDWords(&raw mut oGuid as *mut c_void, size_of::<INT>() as DWORD, &raw mut cx, offset);

            MSFT_ReadLEDWords(
                &raw mut (*pImpLib).lcid as *mut c_void,
                size_of::<LCID>() as DWORD,
                &raw mut cx,
                DO_NOT_SEEK,
            );
            MSFT_ReadLEWords(
                &raw mut (*pImpLib).wVersionMajor as *mut c_void,
                size_of::<WORD>() as DWORD,
                &raw mut cx,
                DO_NOT_SEEK,
            );
            MSFT_ReadLEWords(
                &raw mut (*pImpLib).wVersionMinor as *mut c_void,
                size_of::<WORD>() as DWORD,
                &raw mut cx,
                DO_NOT_SEEK,
            );
            MSFT_ReadLEWords(
                &raw mut size as *mut c_void,
                size_of::<UINT16>() as DWORD,
                &raw mut cx,
                DO_NOT_SEEK,
            );

            size >>= 2;
            let name = heap_alloc_zero(size as usize + 1) as *mut i8;
            MSFT_Read(name as *mut c_void, size as DWORD, &raw mut cx, DO_NOT_SEEK);
            (*pImpLib).name = TLB_MultiByteToBSTR(name);
            heap_free(name as *mut c_void);

            (*pImpLib).guid = MSFT_ReadGuid(oGuid, &raw mut cx);
            offset = (offset
                + size_of::<INT>() as i32
                + size_of::<DWORD>() as i32
                + size_of::<LCID>() as i32
                + size_of::<UINT16>() as i32
                + size as i32
                + 3)
                & !3;

            list_add_tail(
                &raw mut (*pTypeLibImpl).implib_list,
                &raw mut (*pImpLib).entry,
            );
        }
    }

    MSFT_ReadAllRefs(&raw mut cx);

    (*pTypeLibImpl).dispatch_href = tlbHeader.dispatchpos as HREFTYPE;

    /* type infos */
    if tlbHeader.nrtypeinfos >= 0 {
        (*pTypeLibImpl).typeinfos = heap_alloc_zero(
            size_of::<*mut ITypeInfoImpl>() * tlbHeader.nrtypeinfos as usize,
        ) as *mut *mut ITypeInfoImpl;
        let mut ppTI = (*pTypeLibImpl).typeinfos;

        let mut i: i32 = 0;
        while i < tlbHeader.nrtypeinfos {
            *ppTI = MSFT_DoTypeInfo(&raw mut cx, i, pTypeLibImpl);

            ppTI = ppTI.add(1);
            (*pTypeLibImpl).TypeInfoCount += 1;
            i += 1;
        }
    }

    #[cfg(target_pointer_width = "64")]
    {
        if (*pTypeLibImpl).syskind == SYS_WIN32 {
            let mut i: i32 = 0;
            while i < (*pTypeLibImpl).TypeInfoCount {
                TLB_fix_32on64_typeinfo(*(*pTypeLibImpl).typeinfos.add(i as usize));
                i += 1;
            }
        }
    }

    TRACE!();
    &raw mut (*pTypeLibImpl).ITypeLib2_iface
}
