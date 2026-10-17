//! typecomp.rs — CreateDispTypeInfo (OLEAUT32.31) + the ITypeInfo-side
//! ITypeComp implementation (typelib.c lines ~8792–9041). Lane A2, 1:1.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_imports)]

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut};

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::itypeinfo::{ITypeInfoImpl_Constructor, TLB_AllocAndInitFuncDesc, TLB_AllocAndInitVarDesc};
use super::msft::TypeLibImpl_Constructor;
use super::sys::*;
use super::MSFT_TypeInfoBase;

// oleauto.h CreateDispTypeInfo input structures
#[repr(C)]
pub struct PARAMDATA {
    pub szName: LPOLESTR,
    pub vt: VARTYPE,
}
#[repr(C)]
pub struct METHODDATA {
    pub szName: LPOLESTR,
    pub ppdata: *mut PARAMDATA,
    pub dispid: DISPID,
    pub iMeth: UINT,
    pub cc: CALLCONV,
    pub cArgs: UINT,
    pub wFlags: WORD,
    pub vtReturn: VARTYPE,
}
#[repr(C)]
pub struct INTERFACEDATA {
    pub pmethdata: *mut METHODDATA,
    pub cMembers: UINT,
}

/******************************************************************************
 * CreateDispTypeInfo [OLEAUT32.31]
 *
 * Build type information for an object so it can be called through an
 * IDispatch interface.
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDispTypeInfo(
    pidata: *mut INTERFACEDATA,
    lcid: LCID,
    pptinfo: *mut *mut ITypeInfo,
) -> HRESULT {
    TRACE!();
    let pTypeLibImpl = TypeLibImpl_Constructor();
    if pTypeLibImpl.is_null() {
        return E_FAIL;
    }

    (*pTypeLibImpl).TypeInfoCount = 2;
    (*pTypeLibImpl).typeinfos = heap_alloc_zero(
        (*pTypeLibImpl).TypeInfoCount as usize * size_of::<*mut ITypeInfoImpl>(),
    ) as *mut *mut ITypeInfoImpl;

    let pTIIface = ITypeInfoImpl_Constructor();
    *(*pTypeLibImpl).typeinfos = pTIIface;
    (*pTIIface).pTypeLib = pTypeLibImpl;
    (*pTIIface).index = 0;
    (*pTIIface).Name = null();
    (*pTIIface).dwHelpContext = -1i32 as DWORD;
    (*pTIIface).guid = null_mut();
    (*pTIIface).typeattr.lcid = lcid;
    (*pTIIface).typeattr.typekind = TKIND_INTERFACE;
    (*pTIIface).typeattr.wMajorVerNum = 0;
    (*pTIIface).typeattr.wMinorVerNum = 0;
    (*pTIIface).typeattr.cbAlignment = 2;
    (*pTIIface).typeattr.cbSizeInstance = -1i32 as ULONG;
    (*pTIIface).typeattr.cbSizeVft = -1i32 as u32 as WORD;
    (*pTIIface).typeattr.cFuncs = 0;
    (*pTIIface).typeattr.cImplTypes = 0;
    (*pTIIface).typeattr.cVars = 0;
    (*pTIIface).typeattr.wTypeFlags = 0;
    (*pTIIface).hreftype = 0;

    (*pTIIface).funcdescs = TLBFuncDesc_Alloc((*pidata).cMembers);
    let mut pFuncDesc = (*pTIIface).funcdescs;
    let mut func: UINT = 0;
    while func < (*pidata).cMembers {
        let md = (*pidata).pmethdata.add(func as usize);
        (*pFuncDesc).Name = TLB_append_str(&raw mut (*pTypeLibImpl).name_list, (*md).szName);
        (*pFuncDesc).funcdesc.memid = (*md).dispid;
        (*pFuncDesc).funcdesc.lprgscode = null_mut();
        (*pFuncDesc).funcdesc.funckind = FUNC_VIRTUAL;
        (*pFuncDesc).funcdesc.invkind = (*md).wFlags as INVOKEKIND;
        (*pFuncDesc).funcdesc.callconv = (*md).cc;
        (*pFuncDesc).funcdesc.cParams = (*md).cArgs as SHORT;
        (*pFuncDesc).funcdesc.cParamsOpt = 0;
        (*pFuncDesc).funcdesc.oVft = ((*md).iMeth as usize * size_of::<*mut c_void>()) as SHORT;
        (*pFuncDesc).funcdesc.cScodes = 0;
        (*pFuncDesc).funcdesc.wFuncFlags = 0;
        (*pFuncDesc).funcdesc.elemdescFunc.tdesc.vt = (*md).vtReturn;
        (*pFuncDesc).funcdesc.elemdescFunc.u.paramdesc.wParamFlags = PARAMFLAG_NONE;
        (*pFuncDesc).funcdesc.elemdescFunc.u.paramdesc.pparamdescex = null_mut();
        (*pFuncDesc).funcdesc.lprgelemdescParam = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*md).cArgs as usize * size_of::<ELEMDESC>()) as _,
        ) as *mut ELEMDESC;
        (*pFuncDesc).pParamDesc = TLBParDesc_Constructor((*md).cArgs);
        let mut param: UINT = 0;
        while param < (*md).cArgs {
            (*(*pFuncDesc).funcdesc.lprgelemdescParam.add(param as usize)).tdesc.vt =
                (*(*md).ppdata.add(param as usize)).vt;
            (*(*pFuncDesc).pParamDesc.add(param as usize)).Name = TLB_append_str(
                &raw mut (*pTypeLibImpl).name_list,
                (*(*md).ppdata.add(param as usize)).szName,
            );
            param += 1;
        }
        (*pFuncDesc).helpcontext = 0;
        (*pFuncDesc).HelpStringContext = 0;
        (*pFuncDesc).HelpString = null();
        (*pFuncDesc).Entry = null();
        list_init(&raw mut (*pFuncDesc).custdata_list);
        (*pTIIface).typeattr.cFuncs += 1;
        pFuncDesc = pFuncDesc.add(1);
        func += 1;
    }

    /* dump_TypeInfo — compiled out */

    let pTIClass = ITypeInfoImpl_Constructor();
    *(*pTypeLibImpl).typeinfos.add(1) = pTIClass;
    (*pTIClass).pTypeLib = pTypeLibImpl;
    (*pTIClass).index = 1;
    (*pTIClass).Name = null();
    (*pTIClass).dwHelpContext = -1i32 as DWORD;
    (*pTIClass).guid = null_mut();
    (*pTIClass).typeattr.lcid = lcid;
    (*pTIClass).typeattr.typekind = TKIND_COCLASS;
    (*pTIClass).typeattr.wMajorVerNum = 0;
    (*pTIClass).typeattr.wMinorVerNum = 0;
    (*pTIClass).typeattr.cbAlignment = 2;
    (*pTIClass).typeattr.cbSizeInstance = -1i32 as ULONG;
    (*pTIClass).typeattr.cbSizeVft = -1i32 as u32 as WORD;
    (*pTIClass).typeattr.cFuncs = 0;
    (*pTIClass).typeattr.cImplTypes = 1;
    (*pTIClass).typeattr.cVars = 0;
    (*pTIClass).typeattr.wTypeFlags = 0;
    (*pTIClass).hreftype = size_of::<MSFT_TypeInfoBase>() as HREFTYPE;

    (*pTIClass).impltypes = TLBImplType_Alloc(1);

    let ref_ = heap_alloc_zero(size_of::<TLBRefType>()) as *mut TLBRefType;
    (*ref_).pImpTLInfo = TLB_REF_INTERNAL;
    list_add_head(&raw mut (*pTypeLibImpl).ref_list, &raw mut (*ref_).entry);

    /* dump_TypeInfo — compiled out */

    *pptinfo = &raw mut (*pTIClass).ITypeInfo2_iface as *mut ITypeInfo;

    let p = *pptinfo;
    ((*(*p).lpVtbl).AddRef)(p);
    let tl = &raw mut (*pTypeLibImpl).ITypeLib2_iface;
    ((*(*tl).lpVtbl).Release)(tl);

    S_OK
}

unsafe extern "system" fn ITypeComp_fnQueryInterface(
    iface: *mut ITypeComp,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = info_impl_from_ITypeComp(iface);

    let ti = &raw mut (*This).ITypeInfo2_iface;
    ((*(*ti).lpVtbl).QueryInterface)(ti, riid, ppv)
}

unsafe extern "system" fn ITypeComp_fnAddRef(iface: *mut ITypeComp) -> ULONG {
    let This = info_impl_from_ITypeComp(iface);

    let ti = &raw mut (*This).ITypeInfo2_iface;
    ((*(*ti).lpVtbl).AddRef)(ti)
}

unsafe extern "system" fn ITypeComp_fnRelease(iface: *mut ITypeComp) -> ULONG {
    let This = info_impl_from_ITypeComp(iface);

    let ti = &raw mut (*This).ITypeInfo2_iface;
    ((*(*ti).lpVtbl).Release)(ti)
}

unsafe extern "system" fn ITypeComp_fnBind(
    iface: *mut ITypeComp,
    szName: LPOLESTR,
    lHash: ULONG,
    wFlags: WORD,
    ppTInfo: *mut *mut ITypeInfo,
    pDescKind: *mut DESCKIND,
    pBindPtr: *mut BINDPTR,
) -> HRESULT {
    let This = info_impl_from_ITypeComp(iface);
    let mut pFDesc: *mut TLBFuncDesc = null_mut();
    let mut hr: HRESULT = DISP_E_MEMBERNOTFOUND;

    TRACE!();

    *pDescKind = DESCKIND_NONE;
    (*pBindPtr).lpfuncdesc = null_mut();
    *ppTInfo = null_mut();

    let mut fdc: UINT = 0;
    while fdc < (*This).typeattr.cFuncs as UINT {
        pFDesc = (*This).funcdescs.add(fdc as usize);
        if lstrcmpiW(TLB_get_bstr((*pFDesc).Name), szName) == 0 {
            if wFlags == 0 || ((*pFDesc).funcdesc.invkind & wFlags as INVOKEKIND) != 0 {
                break;
            } else {
                /* name found, but wrong flags */
                hr = TYPE_E_TYPEMISMATCH;
            }
        }
        fdc += 1;
    }

    if fdc < (*This).typeattr.cFuncs as UINT {
        let hr2 = TLB_AllocAndInitFuncDesc(
            &raw const (*pFDesc).funcdesc,
            &raw mut (*pBindPtr).lpfuncdesc,
            if (*This).typeattr.typekind == TKIND_DISPATCH { TRUE } else { FALSE },
        );
        if FAILED(hr2) {
            return hr2;
        }
        *pDescKind = DESCKIND_FUNCDESC;
        *ppTInfo = &raw mut (*This).ITypeInfo2_iface as *mut ITypeInfo;
        let p = *ppTInfo;
        ((*(*p).lpVtbl).AddRef)(p);
        return S_OK;
    } else {
        let pVDesc = TLB_get_vardesc_by_name(
            (*This).vardescs,
            (*This).typeattr.cVars as UINT,
            szName,
        );
        if !pVDesc.is_null() {
            let hr2 = TLB_AllocAndInitVarDesc(
                &raw const (*pVDesc).vardesc,
                &raw mut (*pBindPtr).lpvardesc,
            );
            if FAILED(hr2) {
                return hr2;
            }
            *pDescKind = DESCKIND_VARDESC;
            *ppTInfo = &raw mut (*This).ITypeInfo2_iface as *mut ITypeInfo;
            let p = *ppTInfo;
            ((*(*p).lpVtbl).AddRef)(p);
            return S_OK;
        }
    }

    if hr == DISP_E_MEMBERNOTFOUND && !(*This).impltypes.is_null() {
        /* recursive search */
        let mut pTInfo: *mut ITypeInfo = null_mut();
        let mut pTComp: *mut ITypeComp = null_mut();
        let ti2 = &raw mut (*This).ITypeInfo2_iface;
        let mut hr2 = ((*(*ti2).lpVtbl).GetRefTypeInfo)(
            ti2,
            (*(*This).impltypes).hRef,
            &raw mut pTInfo,
        );
        if SUCCEEDED(hr2) {
            hr2 = ((*(*pTInfo).lpVtbl).GetTypeComp)(pTInfo, &raw mut pTComp);
            ((*(*pTInfo).lpVtbl).Release)(pTInfo);
        }
        if SUCCEEDED(hr2) {
            hr2 = ((*(*pTComp).lpVtbl).Bind)(
                pTComp, szName, lHash, wFlags, ppTInfo, pDescKind, pBindPtr,
            );
            ((*(*pTComp).lpVtbl).Release)(pTComp);
            if SUCCEEDED(hr2)
                && *pDescKind == DESCKIND_FUNCDESC
                && (*This).typeattr.typekind == TKIND_DISPATCH
            {
                let tmp = (*pBindPtr).lpfuncdesc;
                hr2 = TLB_AllocAndInitFuncDesc(tmp, &raw mut (*pBindPtr).lpfuncdesc, TRUE);
                SysFreeString(tmp as BSTR);
            }
            return hr2;
        }
        WARN!();
    }
    if hr == DISP_E_MEMBERNOTFOUND {
        hr = S_OK;
    }
    TRACE!();
    hr
}

unsafe extern "system" fn ITypeComp_fnBindType(
    iface: *mut ITypeComp,
    szName: LPOLESTR,
    lHash: ULONG,
    ppTInfo: *mut *mut ITypeInfo,
    ppTComp: *mut *mut ITypeComp,
) -> HRESULT {
    TRACE!();

    /* strange behaviour (does nothing) but like the original */

    if ppTInfo.is_null() || ppTComp.is_null() {
        return E_POINTER;
    }

    *ppTInfo = null_mut();
    *ppTComp = null_mut();

    S_OK
}

pub static tcompvt: ITypeCompVtbl = ITypeCompVtbl {
    QueryInterface: ITypeComp_fnQueryInterface,
    AddRef: ITypeComp_fnAddRef,
    Release: ITypeComp_fnRelease,

    Bind: ITypeComp_fnBind,
    BindType: ITypeComp_fnBindType,
};
