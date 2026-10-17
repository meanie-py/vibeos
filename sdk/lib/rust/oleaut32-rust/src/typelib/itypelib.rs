//! itypelib.rs — the ITypeLib2 + ITypeLibComp implementations (typelib.c
//! lines ~4782–5620) + TLB_copy_all_custdata. Lane A2 continuation, 1:1.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_assignments, unused_imports)]

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut};

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::sys::*;

pub const STG_E_INSUFFICIENTMEMORY: HRESULT = 0x8003_0008u32 as HRESULT;

unsafe extern "system" fn ITypeLib2_fnQueryInterface(
    iface: *mut ITypeLib2,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();

    if guid_eq(riid, &raw const IID_IUnknown)
        || guid_eq(riid, &raw const IID_ITypeLib)
        || guid_eq(riid, &raw const IID_ITypeLib2)
    {
        *ppv = &raw mut (*This).ITypeLib2_iface as *mut c_void;
    } else if guid_eq(riid, &raw const IID_ICreateTypeLib)
        || guid_eq(riid, &raw const IID_ICreateTypeLib2)
    {
        *ppv = &raw mut (*This).ICreateTypeLib2_iface as *mut c_void;
    } else {
        *ppv = null_mut();
        TRACE!();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppv as *mut IUnknown);
    S_OK
}

unsafe extern "system" fn ITypeLib2_fnAddRef(iface: *mut ITypeLib2) -> ULONG {
    let This = impl_from_ITypeLib2(iface);
    let ref_ = InterlockedIncrement(&raw mut (*This).ref_) as ULONG;

    TRACE!();

    ref_
}

pub(super) unsafe extern "system" fn ITypeLib2_fnRelease(iface: *mut ITypeLib2) -> ULONG {
    let This = impl_from_ITypeLib2(iface);
    // Wine 10.0: take the cache lock BEFORE the decrement and hold it through
    // the teardown. Locking only around list_remove left a window in which
    // TLB_ReadTypeLib, holding the lock, found this typelib in the cache and
    // AddRef'd it while we were already freeing it. The lock is a
    // CRITICAL_SECTION, so the recursive Release of imported typelibs below
    // re-enters it safely.
    let _ = super::msft::tlb_cache_lock();
    let ref_ = InterlockedDecrement(&raw mut (*This).ref_) as ULONG;

    TRACE!();

    if ref_ == 0 {
        /* remove cache entry */
        if !(*This).path.is_null() {
            TRACE!();
            if !(*This).entry.next.is_null() {
                list_remove(&raw mut (*This).entry);
            }
            heap_free((*This).path as *mut c_void);
        }
        TRACE!();

        /* LIST_FOR_EACH_ENTRY_SAFE over string_list */
        let l = &raw mut (*This).string_list;
        let mut cur = (*l).next;
        while cur != l {
            let tlbstr = list_entry!(cur, TLBString, entry);
            let next = (*cur).next;
            list_remove(&raw mut (*tlbstr).entry);
            SysFreeString((*tlbstr).str_);
            heap_free(tlbstr as *mut c_void);
            cur = next;
        }

        let l = &raw mut (*This).name_list;
        let mut cur = (*l).next;
        while cur != l {
            let tlbstr = list_entry!(cur, TLBString, entry);
            let next = (*cur).next;
            list_remove(&raw mut (*tlbstr).entry);
            SysFreeString((*tlbstr).str_);
            heap_free(tlbstr as *mut c_void);
            cur = next;
        }

        let l = &raw mut (*This).guid_list;
        let mut cur = (*l).next;
        while cur != l {
            let tlbguid = list_entry!(cur, TLBGuid, entry);
            let next = (*cur).next;
            list_remove(&raw mut (*tlbguid).entry);
            heap_free(tlbguid as *mut c_void);
            cur = next;
        }

        TLB_FreeCustData(&raw mut (*This).custdata_list);

        let mut i: i32 = 0;
        while i < (*This).ctTypeDesc {
            let td = (*This).pTypeDesc.add(i as usize);
            if (*td).vt == VT_CARRAY {
                heap_free((*td).u.lpadesc as *mut c_void);
            }
            i += 1;
        }

        heap_free((*This).pTypeDesc as *mut c_void);

        let l = &raw mut (*This).implib_list;
        let mut cur = (*l).next;
        while cur != l {
            let pImpLib = list_entry!(cur, TLBImpLib, entry);
            let next = (*cur).next;
            if !(*pImpLib).pImpTypeLib.is_null() {
                let tl = &raw mut (*(*pImpLib).pImpTypeLib).ITypeLib2_iface;
                ((*(*tl).lpVtbl).Release)(tl);
            }
            SysFreeString((*pImpLib).name);

            list_remove(&raw mut (*pImpLib).entry);
            heap_free(pImpLib as *mut c_void);
            cur = next;
        }

        let l = &raw mut (*This).ref_list;
        let mut cur = (*l).next;
        while cur != l {
            let ref_type = list_entry!(cur, TLBRefType, entry);
            let next = (*cur).next;
            list_remove(&raw mut (*ref_type).entry);
            heap_free(ref_type as *mut c_void);
            cur = next;
        }

        let mut i: i32 = 0;
        while i < (*This).TypeInfoCount {
            let ti = *(*This).typeinfos.add(i as usize);
            heap_free((*ti).tdescAlias as *mut c_void);
            super::itypeinfo::ITypeInfoImpl_Destroy(ti);
            i += 1;
        }
        heap_free((*This).typeinfos as *mut c_void);
        heap_free(This as *mut c_void);
        super::msft::tlb_cache_unlock();
        return 0;
    }

    super::msft::tlb_cache_unlock();
    ref_
}

/* ITypeLib::GetTypeInfoCount
 *
 * Returns the number of type descriptions in the type library
 */
unsafe extern "system" fn ITypeLib2_fnGetTypeInfoCount(iface: *mut ITypeLib2) -> UINT {
    let This = impl_from_ITypeLib2(iface);
    TRACE!();
    (*This).TypeInfoCount as UINT
}

/* ITypeLib::GetTypeInfo
 *
 * retrieves the specified type description in the library.
 */
unsafe extern "system" fn ITypeLib2_fnGetTypeInfo(
    iface: *mut ITypeLib2,
    index: UINT,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();

    if ppTInfo.is_null() {
        return E_INVALIDARG;
    }

    if index >= (*This).TypeInfoCount as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    *ppTInfo = &raw mut (*(*(*This).typeinfos.add(index as usize))).ITypeInfo2_iface
        as *mut ITypeInfo;
    let ti = *ppTInfo;
    ((*(*ti).lpVtbl).AddRef)(ti);

    S_OK
}

/* ITypeLibs::GetTypeInfoType
 *
 * Retrieves the type of a type description.
 */
unsafe extern "system" fn ITypeLib2_fnGetTypeInfoType(
    iface: *mut ITypeLib2,
    index: UINT,
    pTKind: *mut TYPEKIND,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();

    if pTKind.is_null() {
        return E_INVALIDARG;
    }

    if index >= (*This).TypeInfoCount as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    *pTKind = (*(*(*This).typeinfos.add(index as usize))).typeattr.typekind;

    S_OK
}

/* ITypeLib::GetTypeInfoOfGuid
 *
 * Retrieves the type description that corresponds to the specified GUID.
 */
unsafe extern "system" fn ITypeLib2_fnGetTypeInfoOfGuid(
    iface: *mut ITypeLib2,
    guid: REFGUID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();

    let mut i: i32 = 0;
    while i < (*This).TypeInfoCount {
        let ti = *(*This).typeinfos.add(i as usize);
        if guid_eq(TLB_get_guid_null((*ti).guid), guid) {
            *ppTInfo = &raw mut (*ti).ITypeInfo2_iface as *mut ITypeInfo;
            let p = *ppTInfo;
            ((*(*p).lpVtbl).AddRef)(p);
            return S_OK;
        }
        i += 1;
    }

    TYPE_E_ELEMENTNOTFOUND
}

/* ITypeLib::GetLibAttr
 *
 * Retrieves the structure that contains the library's attributes.
 */
unsafe extern "system" fn ITypeLib2_fnGetLibAttr(
    iface: *mut ITypeLib2,
    attr: *mut *mut TLIBATTR,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();

    if attr.is_null() {
        return E_INVALIDARG;
    }

    *attr = heap_alloc(size_of::<TLIBATTR>()) as *mut TLIBATTR;
    if (*attr).is_null() {
        return E_OUTOFMEMORY;
    }

    (**attr).guid = *TLB_get_guid_null((*This).guid);
    (**attr).lcid = (*This).set_lcid;
    (**attr).syskind = (*This).syskind;
    (**attr).wMajorVerNum = (*This).ver_major;
    (**attr).wMinorVerNum = (*This).ver_minor;
    (**attr).wLibFlags = (*This).libflags;

    S_OK
}

/* ITypeLib::GetTypeComp
 *
 * Enables a client compiler to bind to a library's types, variables,
 * constants, and global functions.
 */
unsafe extern "system" fn ITypeLib2_fnGetTypeComp(
    iface: *mut ITypeLib2,
    ppTComp: *mut *mut ITypeComp,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();
    *ppTComp = &raw mut (*This).ITypeComp_iface;
    let p = *ppTComp;
    ((*(*p).lpVtbl).AddRef)(p);

    S_OK
}

/* ITypeLib::GetDocumentation
 *
 * Retrieves the library's documentation string, the complete Help file name
 * and path, and the context identifier for the library Help topic in the Help
 * file.
 *
 * On a successful return all non-null BSTR pointers will have been set,
 * possibly to NULL.
 */
unsafe extern "system" fn ITypeLib2_fnGetDocumentation(
    iface: *mut ITypeLib2,
    index: INT,
    pBstrName: *mut BSTR,
    pBstrDocString: *mut BSTR,
    pdwHelpContext: *mut DWORD,
    pBstrHelpFile: *mut BSTR,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);
    let mut result: HRESULT = E_INVALIDARG;
    let mut pTInfo: *mut ITypeInfo = null_mut();

    TRACE!();

    if index < 0 {
        /* documentation for the typelib */
        // goto-ladder memerr1/memerr2/memerr3 reproduced with a labeled block
        'memerr: {
            if !pBstrName.is_null() {
                if !(*This).Name.is_null() {
                    *pBstrName = SysAllocString(TLB_get_bstr((*This).Name));
                    if (*pBstrName).is_null() {
                        /* memerr1 */
                        break 'memerr;
                    }
                } else {
                    *pBstrName = null_mut();
                }
            }
            if !pBstrDocString.is_null() {
                if !(*This).DocString.is_null() {
                    *pBstrDocString = SysAllocString(TLB_get_bstr((*This).DocString));
                    if (*pBstrDocString).is_null() {
                        /* memerr2 */
                        if !pBstrName.is_null() {
                            SysFreeString(*pBstrName);
                        }
                        break 'memerr;
                    }
                } else {
                    *pBstrDocString = null_mut();
                }
            }
            if !pdwHelpContext.is_null() {
                *pdwHelpContext = (*This).dwHelpContext;
            }
            if !pBstrHelpFile.is_null() {
                if !(*This).HelpFile.is_null() {
                    *pBstrHelpFile = SysAllocString(TLB_get_bstr((*This).HelpFile));
                    if (*pBstrHelpFile).is_null() {
                        /* memerr3 */
                        if !pBstrDocString.is_null() {
                            SysFreeString(*pBstrDocString);
                        }
                        if !pBstrName.is_null() {
                            SysFreeString(*pBstrName);
                        }
                        break 'memerr;
                    }
                } else {
                    *pBstrHelpFile = null_mut();
                }
            }

            return S_OK;
        }
        return STG_E_INSUFFICIENTMEMORY;
    } else {
        /* for a typeinfo */
        result = ITypeLib2_fnGetTypeInfo(iface, index as UINT, &raw mut pTInfo);

        if SUCCEEDED(result) {
            result = ((*(*pTInfo).lpVtbl).GetDocumentation)(
                pTInfo,
                MEMBERID_NIL,
                pBstrName,
                pBstrDocString,
                pdwHelpContext,
                pBstrHelpFile,
            );

            ((*(*pTInfo).lpVtbl).Release)(pTInfo);
        }
    }
    result
}

/* ITypeLib::IsName
 *
 * Indicates whether a passed-in string contains the name of a type or member
 * described in the library.
 */
unsafe extern "system" fn ITypeLib2_fnIsName(
    iface: *mut ITypeLib2,
    szNameBuf: LPOLESTR,
    lHashVal: ULONG,
    pfName: *mut BOOL,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);
    let nNameBufLen: UINT = ((lstrlenW(szNameBuf) + 1) as usize * size_of::<WCHAR>()) as UINT;

    TRACE!();

    *pfName = TRUE;
    'exit: {
        let mut tic: i32 = 0;
        while tic < (*This).TypeInfoCount {
            let pTInfo = *(*This).typeinfos.add(tic as usize);
            if TLB_str_memcmp(szNameBuf as *mut c_void, (*pTInfo).Name, nNameBufLen) == 0 {
                break 'exit;
            }
            let mut fdc: UINT = 0;
            while fdc < (*pTInfo).typeattr.cFuncs as UINT {
                let pFInfo = (*pTInfo).funcdescs.add(fdc as usize);
                if TLB_str_memcmp(szNameBuf as *mut c_void, (*pFInfo).Name, nNameBufLen) == 0 {
                    break 'exit;
                }
                let mut pc: i32 = 0;
                while pc < (*pFInfo).funcdesc.cParams as i32 {
                    if TLB_str_memcmp(
                        szNameBuf as *mut c_void,
                        (*(*pFInfo).pParamDesc.add(pc as usize)).Name,
                        nNameBufLen,
                    ) == 0
                    {
                        break 'exit;
                    }
                    pc += 1;
                }
                fdc += 1;
            }
            let mut vrc: UINT = 0;
            while vrc < (*pTInfo).typeattr.cVars as UINT {
                let pVInfo = (*pTInfo).vardescs.add(vrc as usize);
                if TLB_str_memcmp(szNameBuf as *mut c_void, (*pVInfo).Name, nNameBufLen) == 0 {
                    break 'exit;
                }
                vrc += 1;
            }
            tic += 1;
        }
        *pfName = FALSE;
    }

    TRACE!();

    S_OK
}

/* ITypeLib::FindName
 *
 * Finds occurrences of a type description in a type library. This may be used
 * to quickly verify that a name exists in a type library.
 */
unsafe extern "system" fn ITypeLib2_fnFindName(
    iface: *mut ITypeLib2,
    name: LPOLESTR,
    hash: ULONG,
    ppTInfo: *mut *mut ITypeInfo,
    memid: *mut MEMBERID,
    found: *mut USHORT,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);
    let mut count: UINT = 0;

    TRACE!();

    if (name.is_null() && hash == 0) || ppTInfo.is_null() || memid.is_null() || found.is_null() {
        return E_INVALIDARG;
    }

    let len: UINT = ((lstrlenW(name) + 1) as usize * size_of::<WCHAR>()) as UINT;
    let mut tic: i32 = 0;
    while count < *found as UINT && tic < (*This).TypeInfoCount {
        let pTInfo = *(*This).typeinfos.add(tic as usize);

        let matched = 'find: {
            if TLB_str_memcmp(name as *mut c_void, (*pTInfo).Name, len) == 0 {
                *memid.add(count as usize) = MEMBERID_NIL;
                break 'find true;
            }

            let mut fdc: UINT = 0;
            while fdc < (*pTInfo).typeattr.cFuncs as UINT {
                let func = (*pTInfo).funcdescs.add(fdc as usize);

                if TLB_str_memcmp(name as *mut c_void, (*func).Name, len) == 0 {
                    *memid.add(count as usize) = (*func).funcdesc.memid;
                    break 'find true;
                }
                fdc += 1;
            }

            let var = TLB_get_vardesc_by_name(
                (*pTInfo).vardescs,
                (*pTInfo).typeattr.cVars as UINT,
                name,
            );
            if !var.is_null() {
                *memid.add(count as usize) = (*var).vardesc.memid;
                break 'find true;
            }

            false
        };

        if matched {
            let ti2 = &raw mut (*pTInfo).ITypeInfo2_iface;
            ((*(*ti2).lpVtbl).AddRef)(ti2);
            *ppTInfo.add(count as usize) = ti2 as *mut ITypeInfo;
            count += 1;
        }
        tic += 1;
    }
    TRACE!();

    *found = count as USHORT;

    S_OK
}

/* ITypeLib::ReleaseTLibAttr
 *
 * Releases the TLIBATTR originally obtained from ITypeLib::GetLibAttr.
 */
unsafe extern "system" fn ITypeLib2_fnReleaseTLibAttr(
    iface: *mut ITypeLib2,
    pTLibAttr: *mut TLIBATTR,
) {
    let This = impl_from_ITypeLib2(iface);
    TRACE!();
    heap_free(pTLibAttr as *mut c_void);
}

/* ITypeLib2::GetCustData
 *
 * gets the custom data
 */
unsafe extern "system" fn ITypeLib2_fnGetCustData(
    iface: *mut ITypeLib2,
    guid: REFGUID,
    pVarVal: *mut VARIANT,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    TRACE!();

    let pCData = TLB_get_custdata_by_guid(&raw mut (*This).custdata_list, guid);
    if pCData.is_null() {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    VariantInit(pVarVal);
    VariantCopy(pVarVal, &raw const (*pCData).data);

    S_OK
}

/* ITypeLib2::GetLibStatistics
 *
 * Returns statistics about a type library that are required for efficient
 * sizing of hash tables.
 */
unsafe extern "system" fn ITypeLib2_fnGetLibStatistics(
    iface: *mut ITypeLib2,
    pcUniqueNames: *mut ULONG,
    pcchUniqueNames: *mut ULONG,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);

    FIXME!("({:p}): stub!", This);

    if !pcUniqueNames.is_null() {
        *pcUniqueNames = 1;
    }
    if !pcchUniqueNames.is_null() {
        *pcchUniqueNames = 1;
    }
    S_OK
}

/* ITypeLib2::GetDocumentation2
 *
 * Retrieves the library's documentation string, the complete Help file name
 * and path, the localization context to use, and the context ID for the
 * library Help topic in the Help file.
 */
unsafe extern "system" fn ITypeLib2_fnGetDocumentation2(
    iface: *mut ITypeLib2,
    index: INT,
    lcid: LCID,
    pbstrHelpString: *mut BSTR,
    pdwHelpStringContext: *mut DWORD,
    pbstrHelpStringDll: *mut BSTR,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);
    let mut result: HRESULT;
    let mut pTInfo: *mut ITypeInfo = null_mut();

    FIXME!("({:p}) index {} lcid {} half implemented stub!", This, index, lcid);

    /* the help string should be obtained from the helpstringdll, using the
     * _DLLGetDocumentation function, based on the supplied lcid. Nice to do
     * sometime... */
    if index < 0 {
        /* documentation for the typelib */
        if !pbstrHelpString.is_null() {
            *pbstrHelpString = SysAllocString(TLB_get_bstr((*This).DocString));
        }
        if !pdwHelpStringContext.is_null() {
            *pdwHelpStringContext = (*This).dwHelpContext;
        }
        if !pbstrHelpStringDll.is_null() {
            *pbstrHelpStringDll = SysAllocString(TLB_get_bstr((*This).HelpStringDll));
        }

        result = S_OK;
    } else {
        /* for a typeinfo */
        result = ITypeLib2_fnGetTypeInfo(iface, index as UINT, &raw mut pTInfo);

        if SUCCEEDED(result) {
            let mut pTInfo2: *mut ITypeInfo2 = null_mut();
            result = ((*(*pTInfo).lpVtbl).QueryInterface)(
                pTInfo,
                &raw const IID_ITypeInfo2,
                &raw mut pTInfo2 as *mut *mut c_void,
            );

            if SUCCEEDED(result) {
                result = ((*(*pTInfo2).lpVtbl).GetDocumentation2)(
                    pTInfo2,
                    MEMBERID_NIL,
                    lcid,
                    pbstrHelpString,
                    pdwHelpStringContext,
                    pbstrHelpStringDll,
                );

                ((*(*pTInfo2).lpVtbl).Release)(pTInfo2);
            }

            ((*(*pTInfo).lpVtbl).Release)(pTInfo);
        }
    }
    result
}

pub unsafe fn TLB_copy_all_custdata(
    custdata_list: *mut list,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    let ct = list_count(custdata_list);

    (*pCustData).prgCustData =
        CoTaskMemAlloc((ct as usize * size_of::<CUSTDATAITEM>()) as _) as *mut CUSTDATAITEM;
    if (*pCustData).prgCustData.is_null() {
        return E_OUTOFMEMORY;
    }

    (*pCustData).cCustData = ct;

    let mut cdi = (*pCustData).prgCustData;
    let mut cur = (*custdata_list).next;
    let mut hr = S_OK;
    while cur != custdata_list {
        let pCData = list_entry!(cur, TLBCustData, entry);
        (*cdi).guid = *TLB_get_guid_null((*pCData).guid);
        // Wine 10.0: VariantCopy CLEARS its destination first, and this one
        // is fresh CoTaskMemAlloc garbage, so initialise it; and report a
        // failed copy instead of S_OK.
        VariantInit(&raw mut (*cdi).varValue);
        hr = VariantCopy(&raw mut (*cdi).varValue, &raw const (*pCData).data);
        if FAILED(hr) {
            break;
        }
        cdi = cdi.add(1);
        cur = (*cur).next;
    }

    hr
}

/* ITypeLib2::GetAllCustData
 *
 * Gets all custom data items for the library.
 */
unsafe extern "system" fn ITypeLib2_fnGetAllCustData(
    iface: *mut ITypeLib2,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    let This = impl_from_ITypeLib2(iface);
    TRACE!();
    TLB_copy_all_custdata(&raw mut (*This).custdata_list, pCustData)
}

pub static tlbvt: ITypeLib2Vtbl = ITypeLib2Vtbl {
    QueryInterface: ITypeLib2_fnQueryInterface,
    AddRef: ITypeLib2_fnAddRef,
    Release: ITypeLib2_fnRelease,
    GetTypeInfoCount: ITypeLib2_fnGetTypeInfoCount,
    GetTypeInfo: ITypeLib2_fnGetTypeInfo,
    GetTypeInfoType: ITypeLib2_fnGetTypeInfoType,
    GetTypeInfoOfGuid: ITypeLib2_fnGetTypeInfoOfGuid,
    GetLibAttr: ITypeLib2_fnGetLibAttr,
    GetTypeComp: ITypeLib2_fnGetTypeComp,
    GetDocumentation: ITypeLib2_fnGetDocumentation,
    IsName: ITypeLib2_fnIsName,
    FindName: ITypeLib2_fnFindName,
    ReleaseTLibAttr: ITypeLib2_fnReleaseTLibAttr,

    GetCustData: ITypeLib2_fnGetCustData,
    GetLibStatistics: ITypeLib2_fnGetLibStatistics,
    GetDocumentation2: ITypeLib2_fnGetDocumentation2,
    GetAllCustData: ITypeLib2_fnGetAllCustData,
};

// ---------------------------------------------------------------------------
// ITypeLibComp (the typelib's ITypeComp face)
// ---------------------------------------------------------------------------

unsafe extern "system" fn ITypeLibComp_fnQueryInterface(
    iface: *mut ITypeComp,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = tlb_impl_from_ITypeComp(iface);
    let tl = &raw mut (*This).ITypeLib2_iface;
    ((*(*tl).lpVtbl).QueryInterface)(tl, riid, ppv)
}

unsafe extern "system" fn ITypeLibComp_fnAddRef(iface: *mut ITypeComp) -> ULONG {
    let This = tlb_impl_from_ITypeComp(iface);
    let tl = &raw mut (*This).ITypeLib2_iface;
    ((*(*tl).lpVtbl).AddRef)(tl)
}

unsafe extern "system" fn ITypeLibComp_fnRelease(iface: *mut ITypeComp) -> ULONG {
    let This = tlb_impl_from_ITypeComp(iface);
    let tl = &raw mut (*This).ITypeLib2_iface;
    ((*(*tl).lpVtbl).Release)(tl)
}

unsafe extern "system" fn ITypeLibComp_fnBind(
    iface: *mut ITypeComp,
    szName: LPOLESTR,
    lHash: ULONG,
    wFlags: WORD,
    ppTInfo: *mut *mut ITypeInfo,
    pDescKind: *mut DESCKIND,
    pBindPtr: *mut BINDPTR,
) -> HRESULT {
    let This = tlb_impl_from_ITypeComp(iface);
    let mut typemismatch = false;

    TRACE!();

    *pDescKind = DESCKIND_NONE;
    (*pBindPtr).lptcomp = null_mut();
    *ppTInfo = null_mut();

    let mut i: i32 = 0;
    while i < (*This).TypeInfoCount {
        let pTypeInfo = *(*This).typeinfos.add(i as usize);
        TRACE!();

        /* FIXME: check wFlags here? */
        /* FIXME: we should use a hash table to look this info up using lHash
         * instead of an O(n) search */
        if (*pTypeInfo).typeattr.typekind == TKIND_ENUM
            || (*pTypeInfo).typeattr.typekind == TKIND_MODULE
        {
            if !(*pTypeInfo).Name.is_null() && wcscmp((*(*pTypeInfo).Name).str_, szName) == 0 {
                *pDescKind = DESCKIND_TYPECOMP;
                (*pBindPtr).lptcomp = &raw mut (*pTypeInfo).ITypeComp_iface;
                let tc = (*pBindPtr).lptcomp;
                ((*(*tc).lpVtbl).AddRef)(tc);
                TRACE!();
                return S_OK;
            }
        }

        if (*pTypeInfo).typeattr.typekind == TKIND_MODULE
            || (*pTypeInfo).typeattr.typekind == TKIND_ENUM
        {
            let pSubTypeComp = &raw mut (*pTypeInfo).ITypeComp_iface;

            let hr = ((*(*pSubTypeComp).lpVtbl).Bind)(
                pSubTypeComp,
                szName,
                lHash,
                wFlags,
                ppTInfo,
                pDescKind,
                pBindPtr,
            );
            if SUCCEEDED(hr) && *pDescKind != DESCKIND_NONE {
                TRACE!();
                return S_OK;
            } else if hr == TYPE_E_TYPEMISMATCH {
                typemismatch = true;
            }
        }

        if (*pTypeInfo).typeattr.typekind == TKIND_COCLASS
            && (*pTypeInfo).typeattr.wTypeFlags & TYPEFLAG_FAPPOBJECT != 0
        {
            let pSubTypeComp = &raw mut (*pTypeInfo).ITypeComp_iface;
            let mut subtypeinfo: *mut ITypeInfo = null_mut();
            let mut subbindptr: BINDPTR = BINDPTR { lptcomp: null_mut() };
            let mut subdesckind: DESCKIND = DESCKIND_NONE;

            let mut hr = ((*(*pSubTypeComp).lpVtbl).Bind)(
                pSubTypeComp,
                szName,
                lHash,
                wFlags,
                &raw mut subtypeinfo,
                &raw mut subdesckind,
                &raw mut subbindptr,
            );
            if SUCCEEDED(hr) && subdesckind != DESCKIND_NONE {
                let mut tdesc_appobject: TYPEDESC = core::mem::zeroed();
                tdesc_appobject.u.hreftype = (*pTypeInfo).hreftype;
                tdesc_appobject.vt = VT_USERDEFINED;

                let mut vardesc_appobject: VARDESC = core::mem::zeroed();
                vardesc_appobject.memid = -2;
                vardesc_appobject.lpstrSchema = null_mut();
                vardesc_appobject.u.oInst = 0;
                vardesc_appobject.elemdescVar.tdesc.u.lptdesc = &raw mut tdesc_appobject;
                vardesc_appobject.elemdescVar.tdesc.vt = VT_PTR;
                vardesc_appobject.wVarFlags = 0;
                vardesc_appobject.varkind = VAR_STATIC;

                TRACE!();

                /* cleanup things filled in by Bind call so we can put our
                 * application object data in there instead */
                match subdesckind {
                    k if k == DESCKIND_FUNCDESC => {
                        ((*(*subtypeinfo).lpVtbl).ReleaseFuncDesc)(
                            subtypeinfo,
                            subbindptr.lpfuncdesc,
                        );
                    }
                    k if k == DESCKIND_VARDESC => {
                        ((*(*subtypeinfo).lpVtbl).ReleaseVarDesc)(
                            subtypeinfo,
                            subbindptr.lpvardesc,
                        );
                    }
                    _ => {}
                }
                if !subtypeinfo.is_null() {
                    ((*(*subtypeinfo).lpVtbl).Release)(subtypeinfo);
                }

                if (*pTypeInfo).hreftype == -1i32 as HREFTYPE {
                    FIXME!("no hreftype for interface {:p}", pTypeInfo);
                }

                hr = super::itypeinfo::TLB_AllocAndInitVarDesc(
                    &raw const vardesc_appobject,
                    &raw mut (*pBindPtr).lpvardesc,
                );
                if FAILED(hr) {
                    return hr;
                }

                *pDescKind = DESCKIND_IMPLICITAPPOBJ;
                *ppTInfo = &raw mut (*pTypeInfo).ITypeInfo2_iface as *mut ITypeInfo;
                let p = *ppTInfo;
                ((*(*p).lpVtbl).AddRef)(p);
                return S_OK;
            } else if hr == TYPE_E_TYPEMISMATCH {
                typemismatch = true;
            }
        }

        i += 1;
    }

    if typemismatch {
        TRACE!();
        TYPE_E_TYPEMISMATCH
    } else {
        TRACE!();
        S_OK
    }
}

unsafe extern "system" fn ITypeLibComp_fnBindType(
    iface: *mut ITypeComp,
    szName: LPOLESTR,
    lHash: ULONG,
    ppTInfo: *mut *mut ITypeInfo,
    ppTComp: *mut *mut ITypeComp,
) -> HRESULT {
    let This = tlb_impl_from_ITypeComp(iface);

    TRACE!();

    if szName.is_null() || ppTInfo.is_null() || ppTComp.is_null() {
        return E_INVALIDARG;
    }

    let info = TLB_get_typeinfo_by_name(
        (*This).typeinfos,
        (*This).TypeInfoCount as UINT,
        szName,
    );
    if info.is_null() {
        *ppTInfo = null_mut();
        *ppTComp = null_mut();
        return S_OK;
    }

    *ppTInfo = &raw mut (*info).ITypeInfo2_iface as *mut ITypeInfo;
    let p = *ppTInfo;
    ((*(*p).lpVtbl).AddRef)(p);
    *ppTComp = &raw mut (*info).ITypeComp_iface;
    let tc = *ppTComp;
    ((*(*tc).lpVtbl).AddRef)(tc);

    S_OK
}

pub static tlbtcvt: ITypeCompVtbl = ITypeCompVtbl {
    QueryInterface: ITypeLibComp_fnQueryInterface,
    AddRef: ITypeLibComp_fnAddRef,
    Release: ITypeLibComp_fnRelease,

    Bind: ITypeLibComp_fnBind,
    BindType: ITypeLibComp_fnBindType,
};
