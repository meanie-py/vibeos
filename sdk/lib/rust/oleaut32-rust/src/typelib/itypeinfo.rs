//! itypeinfo.rs — the ITypeInfo2 implementation (typelib.c lines ~5623–8776):
//! ITypeInfoImpl constructor/destructor, the TLB_*ElemDesc/FuncDesc/VarDesc
//! deep-copy helpers, the typedesc→VARTYPE converters, fnInvoke, and the full
//! tinfvt vtbl. DispCallFunc lives in dispcall.rs. Lane A2 continuation, 1:1.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_assignments, unused_imports)]

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut};

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::dispcall::DispCallFunc;
use super::sltg::SAFEARRAY;
use super::sys::*;
use super::{DISPATCH_HREF_MASK, DISPATCH_HREF_OFFSET};

// HRESULTs / constants used only by this section
pub const DISP_E_NOTACOLLECTION: HRESULT = 0x8002_0011u32 as HRESULT;
pub const STG_E_FILENOTFOUND: HRESULT = 0x8003_0002u32 as HRESULT;
pub const TYPE_E_DLLFUNCTIONNOTFOUND: HRESULT = 0x8002_802Fu32 as HRESULT;
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x8004_0110u32 as HRESULT;
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;
pub const CLSCTX_LOCAL_SERVER: DWORD = 0x4;
pub const LOCALE_USER_DEFAULT: LCID = 0x400;

// IErrorInfo (oaidl.idl) — only the slots fnInvoke touches.
#[repr(C)]
pub struct IErrorInfoVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IErrorInfo, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IErrorInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IErrorInfo) -> ULONG,
    pub GetGUID: unsafe extern "system" fn(*mut IErrorInfo, *mut GUID) -> HRESULT,
    pub GetSource: unsafe extern "system" fn(*mut IErrorInfo, *mut BSTR) -> HRESULT,
    pub GetDescription: unsafe extern "system" fn(*mut IErrorInfo, *mut BSTR) -> HRESULT,
    pub GetHelpFile: unsafe extern "system" fn(*mut IErrorInfo, *mut BSTR) -> HRESULT,
    pub GetHelpContext: unsafe extern "system" fn(*mut IErrorInfo, *mut DWORD) -> HRESULT,
}
#[repr(C)]
pub struct IErrorInfo {
    pub lpVtbl: *const IErrorInfoVtbl,
}

pub type ENUMRESNAMEPROCW =
    unsafe extern "system" fn(HMODULE, LPCWSTR, LPWSTR, isize) -> BOOL;

extern "system" {
    // kernel32
    fn GetSystemDefaultLCID() -> LCID;
    fn EnumResourceNamesW(
        hModule: HMODULE, lpType: LPCWSTR, lpEnumFunc: ENUMRESNAMEPROCW, lParam: isize,
    ) -> BOOL;
    // ole32 (oleaut32 forwards GetErrorInfo to it)
    fn GetErrorInfo(dwReserved: ULONG, pperrinfo: *mut *mut IErrorInfo) -> HRESULT;
    // oleaut32 other TUs
    fn GetActiveObject(rclsid: REFCLSID, pvReserved: PVOID, ppunk: *mut *mut IUnknown) -> HRESULT;
    fn SafeArrayCreate(vt: VARTYPE, cDims: UINT, rgsabound: *mut SAFEARRAYBOUND) -> *mut SAFEARRAY;
    fn SafeArrayAllocDescriptorEx(vt: VARTYPE, cDims: UINT, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT;
    fn SafeArrayAccessData(psa: *mut SAFEARRAY, ppvData: *mut PVOID) -> HRESULT;
    fn SafeArrayUnaccessData(psa: *mut SAFEARRAY) -> HRESULT;
    fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
    fn SafeArrayGetUBound(psa: *mut SAFEARRAY, nDim: UINT, plUbound: *mut LONG) -> HRESULT;
}

extern "C" {
    // oleaut32 variant.c internal (variant.h, DECLSPEC_HIDDEN)
    fn VARIANT_ClearInd(pVarg: *mut VARIANTARG) -> HRESULT;
}

/*================== ITypeInfo(2) Methods ===================================*/

pub unsafe fn ITypeInfoImpl_Constructor() -> *mut ITypeInfoImpl {
    let pTypeInfoImpl = heap_alloc_zero(size_of::<ITypeInfoImpl>()) as *mut ITypeInfoImpl;
    if !pTypeInfoImpl.is_null() {
        (*pTypeInfoImpl).ITypeInfo2_iface.lpVtbl = &tinfvt;
        (*pTypeInfoImpl).ITypeComp_iface.lpVtbl = &super::typecomp::tcompvt;
        (*pTypeInfoImpl).ICreateTypeInfo2_iface.lpVtbl = &super::writer::CreateTypeInfo2Vtbl;
        (*pTypeInfoImpl).ref_ = 0;
        (*pTypeInfoImpl).hreftype = -1i32 as HREFTYPE;
        (*pTypeInfoImpl).typeattr.memidConstructor = MEMBERID_NIL;
        (*pTypeInfoImpl).typeattr.memidDestructor = MEMBERID_NIL;
        (*pTypeInfoImpl).pcustdata_list = &raw mut (*pTypeInfoImpl).custdata_list;
        list_init((*pTypeInfoImpl).pcustdata_list);
    }
    TRACE!();
    pTypeInfoImpl
}

/* ITypeInfo::QueryInterface
 */
unsafe extern "system" fn ITypeInfo_fnQueryInterface(
    iface: *mut ITypeInfo2,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    *ppvObject = null_mut();
    if guid_eq(riid, &raw const IID_IUnknown)
        || guid_eq(riid, &raw const IID_ITypeInfo)
        || guid_eq(riid, &raw const IID_ITypeInfo2)
    {
        *ppvObject = &raw mut (*This).ITypeInfo2_iface as *mut c_void;
    } else if guid_eq(riid, &raw const IID_ICreateTypeInfo)
        || guid_eq(riid, &raw const IID_ICreateTypeInfo2)
    {
        *ppvObject = &raw mut (*This).ICreateTypeInfo2_iface as *mut c_void;
    } else if guid_eq(riid, &raw const IID_ITypeComp) {
        *ppvObject = &raw mut (*This).ITypeComp_iface as *mut c_void;
    }

    if !(*ppvObject).is_null() {
        IUnknown_AddRef(*ppvObject as *mut IUnknown);
        TRACE!();
        return S_OK;
    }
    TRACE!();
    E_NOINTERFACE
}

/* ITypeInfo::AddRef
 */
unsafe extern "system" fn ITypeInfo_fnAddRef(iface: *mut ITypeInfo2) -> ULONG {
    let This = impl_from_ITypeInfo2(iface);
    let ref_ = InterlockedIncrement(&raw mut (*This).ref_) as ULONG;

    TRACE!();

    if ref_ == 1 {
        /* incremented from 0 */
        let tl = &raw mut (*(*This).pTypeLib).ITypeLib2_iface;
        ((*(*tl).lpVtbl).AddRef)(tl);
    }

    ref_
}

pub unsafe fn ITypeInfoImpl_Destroy(This: *mut ITypeInfoImpl) {
    TRACE!();

    let mut i: UINT = 0;
    while i < (*This).typeattr.cFuncs as UINT {
        let pFInfo = (*This).funcdescs.add(i as usize);
        let mut j: i32 = 0;
        while j < (*pFInfo).funcdesc.cParams as i32 {
            let elemdesc = (*pFInfo).funcdesc.lprgelemdescParam.add(j as usize);
            if (*elemdesc).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
                VariantClear(
                    &raw mut (*(*elemdesc).u.paramdesc.pparamdescex).varDefaultValue,
                );
            }
            TLB_FreeCustData(&raw mut (*(*pFInfo).pParamDesc.add(j as usize)).custdata_list);
            j += 1;
        }
        heap_free((*pFInfo).funcdesc.lprgelemdescParam as *mut c_void);
        heap_free((*pFInfo).pParamDesc as *mut c_void);
        TLB_FreeCustData(&raw mut (*pFInfo).custdata_list);
        i += 1;
    }
    heap_free((*This).funcdescs as *mut c_void);

    let mut i: UINT = 0;
    while i < (*This).typeattr.cVars as UINT {
        let pVInfo = (*This).vardescs.add(i as usize);
        if !(*pVInfo).vardesc_create.is_null() {
            TLB_FreeVarDesc((*pVInfo).vardesc_create);
        } else if (*pVInfo).vardesc.varkind == VAR_CONST {
            VariantClear((*pVInfo).vardesc.u.lpvarValue);
            heap_free((*pVInfo).vardesc.u.lpvarValue as *mut c_void);
        }
        TLB_FreeCustData(&raw mut (*pVInfo).custdata_list);
        i += 1;
    }
    heap_free((*This).vardescs as *mut c_void);

    if !(*This).impltypes.is_null() {
        let mut i: UINT = 0;
        while i < (*This).typeattr.cImplTypes as UINT {
            let pImpl = (*This).impltypes.add(i as usize);
            TLB_FreeCustData(&raw mut (*pImpl).custdata_list);
            i += 1;
        }
        heap_free((*This).impltypes as *mut c_void);
    }

    TLB_FreeCustData(&raw mut (*This).custdata_list);

    heap_free(This as *mut c_void);
}

/* ITypeInfo::Release
 */
unsafe extern "system" fn ITypeInfo_fnRelease(iface: *mut ITypeInfo2) -> ULONG {
    let This = impl_from_ITypeInfo2(iface);
    let ref_ = InterlockedDecrement(&raw mut (*This).ref_) as ULONG;

    TRACE!();

    if ref_ == 0 {
        let not_attached_to_typelib = (*This).not_attached_to_typelib;
        let tl = &raw mut (*(*This).pTypeLib).ITypeLib2_iface;
        ((*(*tl).lpVtbl).Release)(tl);
        if not_attached_to_typelib != FALSE {
            heap_free(This as *mut c_void);
        }
        /* otherwise This will be freed when typelib is freed */
    }

    ref_
}

/* ITypeInfo::GetTypeAttr
 *
 * Retrieves a TYPEATTR structure that contains the attributes of the type
 * description.
 */
unsafe extern "system" fn ITypeInfo_fnGetTypeAttr(
    iface: *mut ITypeInfo2,
    ppTypeAttr: *mut *mut TYPEATTR,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    let mut size: SIZE_T = size_of::<TYPEATTR>();
    if (*This).typeattr.typekind == TKIND_ALIAS && !(*This).tdescAlias.is_null() {
        size += TLB_SizeTypeDesc((*This).tdescAlias, FALSE);
    }

    *ppTypeAttr = heap_alloc(size) as *mut TYPEATTR;
    if (*ppTypeAttr).is_null() {
        return E_OUTOFMEMORY;
    }

    // **ppTypeAttr = This->typeattr (struct copy)
    core::ptr::copy_nonoverlapping(
        &raw const (*This).typeattr,
        *ppTypeAttr,
        1,
    );
    (**ppTypeAttr).guid = *TLB_get_guid_null((*This).guid);

    if !(*This).tdescAlias.is_null() {
        TLB_CopyTypeDesc(
            &raw mut (**ppTypeAttr).tdescAlias,
            (*This).tdescAlias,
            (*ppTypeAttr).add(1) as *mut c_void,
        );
    }

    if (**ppTypeAttr).typekind == TKIND_DISPATCH {
        /* This should include all the inherited funcs */
        (**ppTypeAttr).cFuncs =
            ((**ppTypeAttr).cbSizeVft as i32 / (*(*This).pTypeLib).ptr_size) as WORD;
        /* This is always the size of IDispatch's vtbl */
        (**ppTypeAttr).cbSizeVft = size_of::<IDispatchVtbl>() as WORD;
        (**ppTypeAttr).wTypeFlags &= !TYPEFLAG_FOLEAUTOMATION;
    }
    S_OK
}

/* ITypeInfo::GetTypeComp
 *
 * Retrieves the ITypeComp interface for the type description, which enables a
 * client compiler to bind to the type description's members.
 */
unsafe extern "system" fn ITypeInfo_fnGetTypeComp(
    iface: *mut ITypeInfo2,
    ppTComp: *mut *mut ITypeComp,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    *ppTComp = &raw mut (*This).ITypeComp_iface;
    let tc = *ppTComp;
    ((*(*tc).lpVtbl).AddRef)(tc);
    S_OK
}

pub unsafe fn TLB_SizeElemDesc(elemdesc: *const ELEMDESC) -> SIZE_T {
    let mut size = TLB_SizeTypeDesc(&raw const (*elemdesc).tdesc, FALSE);
    if (*elemdesc).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
        size += size_of::<PARAMDESCEX>();
    }
    size
}

pub unsafe fn TLB_CopyElemDesc(
    src: *const ELEMDESC,
    dest: *mut ELEMDESC,
    buffer: *mut *mut i8,
) -> HRESULT {
    *dest = *src;
    *buffer = TLB_CopyTypeDesc(
        &raw mut (*dest).tdesc,
        &raw const (*src).tdesc,
        *buffer as *mut c_void,
    ) as *mut i8;
    if (*src).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
        let pparamdescex_src = (*src).u.paramdesc.pparamdescex;
        let pparamdescex_dest = *buffer as *mut PARAMDESCEX;
        (*dest).u.paramdesc.pparamdescex = pparamdescex_dest;
        *buffer = (*buffer).add(size_of::<PARAMDESCEX>());
        *pparamdescex_dest = *pparamdescex_src;
        (*pparamdescex_dest).cBytes = size_of::<PARAMDESCEX>() as ULONG;
        VariantInit(&raw mut (*pparamdescex_dest).varDefaultValue);
        return VariantCopy(
            &raw mut (*pparamdescex_dest).varDefaultValue,
            &raw const (*pparamdescex_src).varDefaultValue,
        );
    } else {
        (*dest).u.paramdesc.pparamdescex = null_mut();
    }
    S_OK
}

/// Wine 10.0: only INT/UINT are normalised. The '?'-for-non-ASCII BSTR
/// rewrite (TLB_SanitizeBSTR) is gone: the writer now encodes BSTRs to the
/// code page properly (WMSFT_encode_variant), so defaults keep their text.
pub unsafe fn TLB_SanitizeVariant(var: *mut VARIANT) -> HRESULT {
    if (*var).vt == VT_INT {
        VariantChangeType(var, var, 0, VT_I4)
    } else if (*var).vt == VT_UINT {
        VariantChangeType(var, var, 0, VT_UI4)
    } else {
        S_OK
    }
}

pub unsafe fn TLB_FreeElemDesc(elemdesc: *mut ELEMDESC) {
    if (*elemdesc).u.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
        VariantClear(&raw mut (*(*elemdesc).u.paramdesc.pparamdescex).varDefaultValue);
    }
}

pub unsafe fn TLB_AllocAndInitFuncDesc(
    src: *const FUNCDESC,
    dest_ptr: *mut *mut FUNCDESC,
    dispinterface: BOOL,
) -> HRESULT {
    let mut size: SIZE_T = size_of::<FUNCDESC>();
    let mut hr: HRESULT;

    size += size_of::<SCODE>() * (*src).cScodes as usize;
    size += TLB_SizeElemDesc(&raw const (*src).elemdescFunc);
    let mut i: i32 = 0;
    while i < (*src).cParams as i32 {
        size += size_of::<ELEMDESC>();
        size += TLB_SizeElemDesc((*src).lprgelemdescParam.add(i as usize));
        i += 1;
    }

    let dest = SysAllocStringByteLen(null(), size as UINT) as *mut FUNCDESC;
    if dest.is_null() {
        return E_OUTOFMEMORY;
    }

    core::ptr::copy_nonoverlapping(src, dest, 1);
    if dispinterface != FALSE {
        /* overwrite funckind */
        (*dest).funckind = FUNC_DISPATCH;
    }
    let mut buffer = dest.add(1) as *mut i8;

    (*dest).oVft = ((*dest).oVft as u16 & 0xFFFC) as SHORT;

    if (*dest).cScodes != 0 {
        (*dest).lprgscode = buffer as *mut SCODE;
        memcpy(
            (*dest).lprgscode as *mut c_void,
            (*src).lprgscode as *const c_void,
            size_of::<SCODE>() * (*src).cScodes as usize,
        );
        buffer = buffer.add(size_of::<SCODE>() * (*src).cScodes as usize);
    } else {
        (*dest).lprgscode = null_mut();
    }

    hr = TLB_CopyElemDesc(
        &raw const (*src).elemdescFunc,
        &raw mut (*dest).elemdescFunc,
        &raw mut buffer,
    );
    if FAILED(hr) {
        SysFreeString(dest as BSTR);
        return hr;
    }

    if (*dest).cParams != 0 {
        (*dest).lprgelemdescParam = buffer as *mut ELEMDESC;
        buffer = buffer.add(size_of::<ELEMDESC>() * (*src).cParams as usize);
        let mut i: i32 = 0;
        while i < (*src).cParams as i32 {
            hr = TLB_CopyElemDesc(
                (*src).lprgelemdescParam.add(i as usize),
                (*dest).lprgelemdescParam.add(i as usize),
                &raw mut buffer,
            );
            if FAILED(hr) {
                break;
            }
            i += 1;
        }
        if FAILED(hr) {
            /* undo the above actions */
            let mut j: i32 = i - 1;
            while j >= 0 {
                TLB_FreeElemDesc((*dest).lprgelemdescParam.add(j as usize));
                j -= 1;
            }
            TLB_FreeElemDesc(&raw mut (*dest).elemdescFunc);
            SysFreeString(dest as BSTR);
            return hr;
        }
    } else {
        (*dest).lprgelemdescParam = null_mut();
    }

    /* special treatment for dispinterface FUNCDESC based on an interface
     * FUNCDESC (Wine 10.0): the arguments IDispatch::Invoke passes separately
     * ([retval] in pVarResult, [lcid] from Invoke's lcid) leave the signature.
     * Keyed on the SOURCE not being a dispatch function (dest->funckind was
     * already overwritten above), where the pre-10 code keyed on an HRESULT
     * return and never removed [lcid]. */
    if dispinterface != FALSE && (*src).funckind != FUNC_DISPATCH {
        if (*dest).cParams != 0
            && ((*(*dest).lprgelemdescParam.add((*dest).cParams as usize - 1))
                .u
                .paramdesc
                .wParamFlags
                & PARAMFLAG_FRETVAL)
                != 0
        {
            let elemdesc = (*dest).lprgelemdescParam.add((*dest).cParams as usize - 1);
            if (*elemdesc).tdesc.vt != VT_PTR {
                ERR!("elemdesc should have started with VT_PTR instead of:");
                /* ERR_ON dump — compiled out */
                return E_UNEXPECTED;
            }

            /* copy last parameter to the return value. we are using a flat
             * buffer so there is no danger of leaking memory in elemdescFunc */
            (*dest).elemdescFunc.tdesc = *(*elemdesc).tdesc.u.lptdesc;

            /* remove the last parameter */
            (*dest).cParams -= 1;
        } else if (*dest).elemdescFunc.tdesc.vt == VT_HRESULT {
            /* an HRESULT is returned in pExcepInfo->scode, not pVarResult, so
             * the signature shows no return value */
            (*dest).elemdescFunc.tdesc.vt = VT_VOID;
        }

        /* the now-last parameter may be [lcid], supplied from Invoke(lcid) */
        if (*dest).cParams != 0
            && ((*(*dest).lprgelemdescParam.add((*dest).cParams as usize - 1))
                .u
                .paramdesc
                .wParamFlags
                & PARAMFLAG_FLCID)
                != 0
        {
            (*dest).cParams -= 1;
        }
    }

    *dest_ptr = dest;
    S_OK
}

pub unsafe fn TLB_FreeVarDesc(var_desc: *mut VARDESC) {
    TLB_FreeElemDesc(&raw mut (*var_desc).elemdescVar);
    if (*var_desc).varkind == VAR_CONST {
        VariantClear((*var_desc).u.lpvarValue);
    }
    SysFreeString(var_desc as BSTR);
}

/// non-static in the C (consumed by tmarshal.c) — keep the cdecl symbol.
#[no_mangle]
pub unsafe extern "C" fn ITypeInfoImpl_GetInternalFuncDesc(
    iface: *mut ITypeInfo,
    index: UINT,
    ppFuncDesc: *mut *const FUNCDESC,
) -> HRESULT {
    let This = impl_from_ITypeInfo(iface);

    if index >= (*This).typeattr.cFuncs as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    *ppFuncDesc = &raw const (*(*This).funcdescs.add(index as usize)).funcdesc;
    S_OK
}

/// Wine 10.0's ITypeInfoImpl_GetInternalFuncDesc, which returns the
/// TLBFuncDesc (so callers reach its custdata) and resolves a DISPINTERFACE
/// index through the inherited interfaces, as GetFuncDesc already did. The
/// custom-data getters indexed funcdescs[] directly, so on a dispinterface
/// they answered for the wrong function.
unsafe fn internal_tlb_funcdesc(iface: *mut ITypeInfo, index: UINT) -> Result<*mut TLBFuncDesc, HRESULT> {
    // funcdesc is TLBFuncDesc's first field (repr(C)), so the FUNCDESC the
    // resolvers hand back IS the TLBFuncDesc.
    const _: () = assert!(core::mem::offset_of!(TLBFuncDesc, funcdesc) == 0);
    let This = impl_from_ITypeInfo(iface);
    let mut fd: *const FUNCDESC = null();
    let mut hrefoffset: UINT = 0;
    let hr = if (*This).typeattr.typekind == TKIND_DISPATCH {
        ITypeInfoImpl_GetInternalDispatchFuncDesc(iface, index, &raw mut fd, null_mut(), &raw mut hrefoffset)
    } else {
        ITypeInfoImpl_GetInternalFuncDesc(iface, index, &raw mut fd)
    };
    if FAILED(hr) {
        Err(hr)
    } else {
        Ok(fd as *mut TLBFuncDesc)
    }
}

/* internal function to make the inherited interfaces' methods appear part of
 * the interface */
unsafe fn ITypeInfoImpl_GetInternalDispatchFuncDesc(
    iface: *mut ITypeInfo,
    index: UINT,
    ppFuncDesc: *mut *const FUNCDESC,
    funcs: *mut UINT,
    hrefoffset: *mut UINT,
) -> HRESULT {
    let This = impl_from_ITypeInfo(iface);
    let mut implemented_funcs: UINT = 0;

    if !funcs.is_null() {
        *funcs = 0;
    } else {
        *hrefoffset = DISPATCH_HREF_OFFSET;
    }

    if !(*This).impltypes.is_null() {
        let mut pSubTypeInfo: *mut ITypeInfo = null_mut();
        let mut sub_funcs: UINT = 0;

        let hr = ((*(*iface).lpVtbl).GetRefTypeInfo)(
            iface,
            (*(*This).impltypes).hRef,
            &raw mut pSubTypeInfo,
        );
        if FAILED(hr) {
            return hr;
        }

        let hr = ITypeInfoImpl_GetInternalDispatchFuncDesc(
            pSubTypeInfo,
            index,
            ppFuncDesc,
            &raw mut sub_funcs,
            hrefoffset,
        );
        implemented_funcs += sub_funcs;
        ((*(*pSubTypeInfo).lpVtbl).Release)(pSubTypeInfo);
        if SUCCEEDED(hr) {
            return hr;
        }
        *hrefoffset += DISPATCH_HREF_OFFSET;
    }

    if !funcs.is_null() {
        *funcs = implemented_funcs + (*This).typeattr.cFuncs as UINT;
    } else {
        *hrefoffset = 0;
    }

    if index < implemented_funcs {
        return E_INVALIDARG;
    }
    ITypeInfoImpl_GetInternalFuncDesc(iface, index - implemented_funcs, ppFuncDesc)
}

#[inline]
unsafe fn ITypeInfoImpl_ElemDescAddHrefOffset(pElemDesc: *mut ELEMDESC, hrefoffset: UINT) {
    let mut pTypeDesc = &raw mut (*pElemDesc).tdesc;
    loop {
        match (*pTypeDesc).vt {
            VT_USERDEFINED => {
                (*pTypeDesc).u.hreftype += hrefoffset;
                return;
            }
            VT_PTR | VT_SAFEARRAY => {
                pTypeDesc = (*pTypeDesc).u.lptdesc;
            }
            VT_CARRAY => {
                pTypeDesc = &raw mut (*(*pTypeDesc).u.lpadesc).tdescElem;
            }
            _ => return,
        }
    }
}

#[inline]
unsafe fn ITypeInfoImpl_FuncDescAddHrefOffset(pFuncDesc: *mut FUNCDESC, hrefoffset: UINT) {
    let mut i: i32 = 0;
    while i < (*pFuncDesc).cParams as i32 {
        ITypeInfoImpl_ElemDescAddHrefOffset(
            (*pFuncDesc).lprgelemdescParam.add(i as usize),
            hrefoffset,
        );
        i += 1;
    }
    ITypeInfoImpl_ElemDescAddHrefOffset(&raw mut (*pFuncDesc).elemdescFunc, hrefoffset);
}

/* ITypeInfo::GetFuncDesc
 *
 * Retrieves the FUNCDESC structure that contains information about a
 * specified function.
 */
unsafe extern "system" fn ITypeInfo_fnGetFuncDesc(
    iface: *mut ITypeInfo2,
    index: UINT,
    ppFuncDesc: *mut *mut FUNCDESC,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut internal_funcdesc: *const FUNCDESC = null();
    let mut hr: HRESULT;
    let mut hrefoffset: UINT = 0;

    TRACE!();

    if ppFuncDesc.is_null() {
        return E_INVALIDARG;
    }

    if (*This).needs_layout != FALSE {
        let cti = &raw mut (*This).ICreateTypeInfo2_iface;
        ((*(*cti).lpVtbl).LayOut)(cti);
    }

    if (*This).typeattr.typekind == TKIND_DISPATCH {
        hr = ITypeInfoImpl_GetInternalDispatchFuncDesc(
            iface as *mut ITypeInfo,
            index,
            &raw mut internal_funcdesc,
            null_mut(),
            &raw mut hrefoffset,
        );
    } else {
        hr = ITypeInfoImpl_GetInternalFuncDesc(
            iface as *mut ITypeInfo,
            index,
            &raw mut internal_funcdesc,
        );
    }
    if FAILED(hr) {
        WARN!();
        return hr;
    }

    hr = TLB_AllocAndInitFuncDesc(
        internal_funcdesc,
        ppFuncDesc,
        if (*This).typeattr.typekind == TKIND_DISPATCH { TRUE } else { FALSE },
    );

    if (*This).typeattr.typekind == TKIND_DISPATCH && hrefoffset != 0 {
        ITypeInfoImpl_FuncDescAddHrefOffset(*ppFuncDesc, hrefoffset);
    }

    TRACE!();
    hr
}

pub unsafe fn TLB_AllocAndInitVarDesc(
    src: *const VARDESC,
    dest_ptr: *mut *mut VARDESC,
) -> HRESULT {
    let mut size: SIZE_T = size_of::<VARDESC>();

    if !(*src).lpstrSchema.is_null() {
        size += (lstrlenW((*src).lpstrSchema) as usize + 1) * size_of::<WCHAR>();
    }
    if (*src).varkind == VAR_CONST {
        size += size_of::<VARIANT>();
    }
    size += TLB_SizeElemDesc(&raw const (*src).elemdescVar);

    let dest = SysAllocStringByteLen(null(), size as UINT) as *mut VARDESC;
    if dest.is_null() {
        return E_OUTOFMEMORY;
    }

    core::ptr::copy_nonoverlapping(src, dest, 1);
    let mut buffer = dest.add(1) as *mut i8;
    if !(*src).lpstrSchema.is_null() {
        (*dest).lpstrSchema = buffer as LPOLESTR;
        let len = lstrlenW((*src).lpstrSchema);
        memcpy(
            (*dest).lpstrSchema as *mut c_void,
            (*src).lpstrSchema as *const c_void,
            (len as usize + 1) * size_of::<WCHAR>(),
        );
        buffer = buffer.add((len as usize + 1) * size_of::<WCHAR>());
    }

    if (*src).varkind == VAR_CONST {
        (*dest).u.lpvarValue = buffer as *mut VARIANT;
        *(*dest).u.lpvarValue = *(*src).u.lpvarValue;
        buffer = buffer.add(size_of::<VARIANT>());
        VariantInit((*dest).u.lpvarValue);
        let hr = VariantCopy((*dest).u.lpvarValue, (*src).u.lpvarValue);
        if FAILED(hr) {
            SysFreeString(dest as BSTR);
            return hr;
        }
    }
    let hr = TLB_CopyElemDesc(
        &raw const (*src).elemdescVar,
        &raw mut (*dest).elemdescVar,
        &raw mut buffer,
    );
    if FAILED(hr) {
        if (*src).varkind == VAR_CONST {
            VariantClear((*dest).u.lpvarValue);
        }
        SysFreeString(dest as BSTR);
        return hr;
    }
    *dest_ptr = dest;
    S_OK
}

/* ITypeInfo::GetVarDesc
 *
 * Retrieves a VARDESC structure that describes the specified variable.
 */
unsafe extern "system" fn ITypeInfo_fnGetVarDesc(
    iface: *mut ITypeInfo2,
    index: UINT,
    ppVarDesc: *mut *mut VARDESC,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let pVDesc = (*This).vardescs.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cVars as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    if (*This).needs_layout != FALSE {
        let cti = &raw mut (*This).ICreateTypeInfo2_iface;
        ((*(*cti).lpVtbl).LayOut)(cti);
    }

    TLB_AllocAndInitVarDesc(&raw const (*pVDesc).vardesc, ppVarDesc)
}

/* ITypeInfo_GetNames
 *
 * Retrieves the variable with the specified member ID (or the name of the
 * property or method and its parameters) that correspond to the specified
 * function ID.
 */
unsafe extern "system" fn ITypeInfo_fnGetNames(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    rgBstrNames: *mut BSTR,
    cMaxNames: UINT,
    pcNames: *mut UINT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    TRACE!();

    if rgBstrNames.is_null() {
        return E_INVALIDARG;
    }

    typeinfo_getnames(
        iface as *mut ITypeInfo,
        memid,
        rgBstrNames,
        cMaxNames,
        pcNames,
        (*This).typeattr.typekind == TKIND_DISPATCH,
    )
}

/// Wine 10.0's typeinfo_getnames. `dispinterface` is decided once, by the
/// typeinfo the CALLER asked, and carried through the inheritance recursion:
/// names of trailing [retval]/[lcid] parameters are hidden exactly as
/// TLB_AllocAndInitFuncDesc hides the parameters, so GetNames and GetFuncDesc
/// agree on a dispinterface (the pre-10 code listed names for parameters
/// GetFuncDesc no longer reported).
unsafe fn typeinfo_getnames(
    iface: *mut ITypeInfo,
    memid: MEMBERID,
    rgBstrNames: *mut BSTR,
    cMaxNames: UINT,
    pcNames: *mut UINT,
    dispinterface: bool,
) -> HRESULT {
    let This = impl_from_ITypeInfo(iface);

    *pcNames = 0;

    let pFDesc = TLB_get_funcdesc_by_memberid(
        (*This).funcdescs,
        (*This).typeattr.cFuncs as UINT,
        memid,
    );
    if !pFDesc.is_null() {
        let mut params = (*pFDesc).funcdesc.cParams as i32;
        if cMaxNames == 0 || (*pFDesc).Name.is_null() {
            return S_OK;
        }

        *rgBstrNames = SysAllocString(TLB_get_bstr((*pFDesc).Name));
        *pcNames += 1;

        if dispinterface && (*pFDesc).funcdesc.funckind != FUNC_DISPATCH {
            let flags = |k: i32| {
                (*(*pFDesc).funcdesc.lprgelemdescParam.add(k as usize)).u.paramdesc.wParamFlags
            };
            if params > 0 && (flags(params - 1) & PARAMFLAG_FRETVAL) != 0 {
                params -= 1; /* Invoke(pVarResult) supplies [retval] */
            }
            if params > 0 && (flags(params - 1) & PARAMFLAG_FLCID) != 0 {
                params -= 1; /* Invoke(lcid) supplies [lcid] */
            }
        }

        let mut i: i32 = 0;
        while i < params {
            if *pcNames >= cMaxNames || (*(*pFDesc).pParamDesc.add(i as usize)).Name.is_null() {
                return S_OK;
            }
            *rgBstrNames.add(*pcNames as usize) =
                SysAllocString(TLB_get_bstr((*(*pFDesc).pParamDesc.add(i as usize)).Name));
            *pcNames += 1;
            i += 1;
        }
        return S_OK;
    }

    let pVDesc = TLB_get_vardesc_by_memberid(
        (*This).vardescs,
        (*This).typeattr.cVars as UINT,
        memid,
    );
    if !pVDesc.is_null() {
        *rgBstrNames = SysAllocString(TLB_get_bstr((*pVDesc).Name));
        *pcNames = 1;
    } else {
        if !(*This).impltypes.is_null()
            && ((*This).typeattr.typekind == TKIND_INTERFACE
                || (*This).typeattr.typekind == TKIND_DISPATCH)
        {
            /* recursive search */
            let mut pTInfo: *mut ITypeInfo = null_mut();
            let result = ((*(*iface).lpVtbl).GetRefTypeInfo)(
                iface,
                (*(*This).impltypes).hRef,
                &raw mut pTInfo,
            );
            if SUCCEEDED(result) {
                let result =
                    typeinfo_getnames(pTInfo, memid, rgBstrNames, cMaxNames, pcNames, dispinterface);
                ((*(*pTInfo).lpVtbl).Release)(pTInfo);
                return result;
            }
            WARN!();
        } else {
            WARN!();
        }
        *pcNames = 0;
        return TYPE_E_ELEMENTNOTFOUND;
    }
    S_OK
}

/* ITypeInfo::GetRefTypeOfImplType
 *
 * If a type description describes a COM class, it retrieves the type
 * description of the implemented interface types. For an interface,
 * GetRefTypeOfImplType returns the type information for inherited interfaces,
 * if any exist.
 */
unsafe extern "system" fn ITypeInfo_fnGetRefTypeOfImplType(
    iface: *mut ITypeInfo2,
    index: UINT,
    pRefType: *mut HREFTYPE,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!();

    if index == u32::MAX {
        /* only valid on dual interfaces; retrieve the associated
         * TKIND_INTERFACE handle for the current TKIND_DISPATCH */
        if (*This).typeattr.wTypeFlags & TYPEFLAG_FDUAL != 0 {
            *pRefType = -2i32 as HREFTYPE;
        } else {
            hr = TYPE_E_ELEMENTNOTFOUND;
        }
    } else if index == 0 && (*This).typeattr.typekind == TKIND_DISPATCH {
        /* All TKIND_DISPATCHs are made to look like they inherit from
         * IDispatch */
        *pRefType = (*(*This).pTypeLib).dispatch_href;
    } else {
        if index >= (*This).typeattr.cImplTypes as UINT {
            hr = TYPE_E_ELEMENTNOTFOUND;
        } else {
            *pRefType = (*(*This).impltypes.add(index as usize)).hRef;
            if (*This).typeattr.typekind == TKIND_INTERFACE {
                *pRefType |= 0x2;
            }
        }
    }

    hr
}

/* ITypeInfo::GetImplTypeFlags
 *
 * Retrieves the IMPLTYPEFLAGS enumeration for one implemented interface or
 * base interface in a type description.
 */
unsafe extern "system" fn ITypeInfo_fnGetImplTypeFlags(
    iface: *mut ITypeInfo2,
    index: UINT,
    pImplTypeFlags: *mut INT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    if pImplTypeFlags.is_null() {
        return E_INVALIDARG;
    }

    if (*This).typeattr.typekind == TKIND_DISPATCH && index == 0 {
        *pImplTypeFlags = 0;
        return S_OK;
    }

    if index >= (*This).typeattr.cImplTypes as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    *pImplTypeFlags = (*(*This).impltypes.add(index as usize)).implflags;

    S_OK
}

/* GetIDsOfNames
 * Maps between member names and member IDs, and parameter names and
 * parameter IDs.
 */
unsafe extern "system" fn ITypeInfo_fnGetIDsOfNames(
    iface: *mut ITypeInfo2,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    pMemId: *mut MEMBERID,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut ret: HRESULT = S_OK;

    TRACE!();

    /* init out parameters in case of failure */
    let mut i: UINT = 0;
    while i < cNames {
        *pMemId.add(i as usize) = MEMBERID_NIL;
        i += 1;
    }

    let mut fdc: UINT = 0;
    while fdc < (*This).typeattr.cFuncs as UINT {
        let pFDesc = (*This).funcdescs.add(fdc as usize);
        if lstrcmpiW(*rgszNames, TLB_get_bstr((*pFDesc).Name)) == 0 {
            if cNames != 0 {
                *pMemId = (*pFDesc).funcdesc.memid;
            }
            let mut i: UINT = 1;
            while i < cNames {
                let mut j: i32 = 0;
                while j < (*pFDesc).funcdesc.cParams as i32 {
                    if lstrcmpiW(
                        *rgszNames.add(i as usize),
                        TLB_get_bstr((*(*pFDesc).pParamDesc.add(j as usize)).Name),
                    ) == 0
                    {
                        break;
                    }
                    j += 1;
                }
                if j < (*pFDesc).funcdesc.cParams as i32 {
                    *pMemId.add(i as usize) = j;
                } else {
                    ret = DISP_E_UNKNOWNNAME;
                }
                i += 1;
            }
            TRACE!();
            return ret;
        }
        fdc += 1;
    }
    let pVDesc = TLB_get_vardesc_by_name(
        (*This).vardescs,
        (*This).typeattr.cVars as UINT,
        *rgszNames,
    );
    if !pVDesc.is_null() {
        if cNames != 0 {
            *pMemId = (*pVDesc).vardesc.memid;
        }
        return ret;
    }
    /* not found, see if it can be found in an inherited interface */
    if !(*This).impltypes.is_null() {
        /* recursive search */
        let mut pTInfo: *mut ITypeInfo = null_mut();
        ret = ((*(*iface).lpVtbl).GetRefTypeInfo)(
            iface,
            (*(*This).impltypes).hRef,
            &raw mut pTInfo,
        );
        if SUCCEEDED(ret) {
            ret = ((*(*pTInfo).lpVtbl).GetIDsOfNames)(pTInfo, rgszNames, cNames, pMemId);
            ((*(*pTInfo).lpVtbl).Release)(pTInfo);
            return ret;
        }
        WARN!();
    } else {
        WARN!();
    }
    DISP_E_UNKNOWNNAME
}

unsafe fn userdefined_to_variantvt(
    tinfo: *mut ITypeInfo,
    tdesc: *const TYPEDESC,
    vt: *mut VARTYPE,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut tinfo2: *mut ITypeInfo = null_mut();
    let mut tattr: *mut TYPEATTR = null_mut();

    hr = ((*(*tinfo).lpVtbl).GetRefTypeInfo)(tinfo, (*tdesc).u.hreftype, &raw mut tinfo2);
    if hr != 0 {
        ERR!(
            "Could not get typeinfo of hreftype {:x} for VT_USERDEFINED, hr = 0x{:08x}",
            (*tdesc).u.hreftype,
            hr as u32
        );
        return hr;
    }
    hr = ((*(*tinfo2).lpVtbl).GetTypeAttr)(tinfo2, &raw mut tattr);
    if hr != 0 {
        ERR!("ITypeInfo_GetTypeAttr failed, hr = 0x{:08x}", hr as u32);
        ((*(*tinfo2).lpVtbl).Release)(tinfo2);
        return hr;
    }

    match (*tattr).typekind {
        k if k == TKIND_ENUM => {
            *vt |= VT_I4;
        }
        k if k == TKIND_ALIAS => {
            hr = typedescvt_to_variantvt(tinfo2, &raw const (*tattr).tdescAlias, vt);
        }
        k if k == TKIND_INTERFACE => {
            if (*tattr).wTypeFlags & TYPEFLAG_FDISPATCHABLE != 0 {
                *vt |= VT_DISPATCH;
            } else {
                *vt |= VT_UNKNOWN;
            }
        }
        k if k == TKIND_DISPATCH => {
            *vt |= VT_DISPATCH;
        }
        k if k == TKIND_COCLASS => {
            *vt |= VT_DISPATCH;
        }
        k if k == TKIND_RECORD => {
            FIXME!("TKIND_RECORD unhandled.");
            hr = E_NOTIMPL;
        }
        k if k == TKIND_UNION => {
            FIXME!("TKIND_UNION unhandled.");
            hr = E_NOTIMPL;
        }
        k => {
            FIXME!("TKIND {} unhandled.", k);
            hr = E_NOTIMPL;
        }
    }
    ((*(*tinfo2).lpVtbl).ReleaseTypeAttr)(tinfo2, tattr);
    ((*(*tinfo2).lpVtbl).Release)(tinfo2);
    hr
}

pub(super) unsafe fn typedescvt_to_variantvt(
    tinfo: *mut ITypeInfo,
    tdesc: *const TYPEDESC,
    vt: *mut VARTYPE,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut tdesc = tdesc;

    /* enforce only one level of pointer indirection */
    if (*vt & VT_BYREF) == 0 && (*vt & VT_ARRAY) == 0 && (*tdesc).vt == VT_PTR {
        tdesc = (*tdesc).u.lptdesc;

        /* munch VT_PTR -> VT_USERDEFINED(interface) into VT_UNKNOWN or
         * VT_DISPATCH and VT_PTR -> VT_PTR -> VT_USERDEFINED(interface) into
         * VT_BYREF|VT_DISPATCH or VT_BYREF|VT_UNKNOWN */
        if (*tdesc).vt == VT_USERDEFINED
            || ((*tdesc).vt == VT_PTR && (*(*tdesc).u.lptdesc).vt == VT_USERDEFINED)
        {
            let mut vt_userdefined: VARTYPE = 0;
            let mut tdesc_userdefined: *const TYPEDESC = tdesc;
            if (*tdesc).vt == VT_PTR {
                vt_userdefined = VT_BYREF;
                tdesc_userdefined = (*tdesc).u.lptdesc;
            }
            hr = userdefined_to_variantvt(tinfo, tdesc_userdefined, &raw mut vt_userdefined);
            if hr == S_OK
                && ((vt_userdefined & VT_TYPEMASK) == VT_UNKNOWN
                    || (vt_userdefined & VT_TYPEMASK) == VT_DISPATCH)
            {
                *vt |= vt_userdefined;
                return S_OK;
            }
        }
        *vt = VT_BYREF;
    }

    match (*tdesc).vt {
        VT_HRESULT => {
            *vt |= VT_ERROR;
        }
        VT_USERDEFINED => {
            hr = userdefined_to_variantvt(tinfo, tdesc, vt);
        }
        VT_VOID | VT_CARRAY | VT_PTR | VT_LPSTR | VT_LPWSTR => {
            ERR!("cannot convert type {} into variant VT", (*tdesc).vt);
            hr = DISP_E_BADVARTYPE;
        }
        VT_SAFEARRAY => {
            *vt |= VT_ARRAY;
            hr = typedescvt_to_variantvt(tinfo, (*tdesc).u.lptdesc, vt);
        }
        VT_INT => {
            *vt |= VT_I4;
        }
        VT_UINT => {
            *vt |= VT_UI4;
        }
        _ => {
            *vt |= (*tdesc).vt;
        }
    }
    hr
}

pub(super) unsafe fn get_iface_guid(
    tinfo: *mut ITypeInfo,
    href: HREFTYPE,
    guid: *mut GUID,
) -> HRESULT {
    let mut tinfo2: *mut ITypeInfo = null_mut();
    let mut tattr: *mut TYPEATTR = null_mut();
    let mut hres: HRESULT;
    let mut flags: INT = 0;
    let mut href = href;

    hres = ((*(*tinfo).lpVtbl).GetRefTypeInfo)(tinfo, href, &raw mut tinfo2);
    if FAILED(hres) {
        return hres;
    }

    hres = ((*(*tinfo2).lpVtbl).GetTypeAttr)(tinfo2, &raw mut tattr);
    if FAILED(hres) {
        ((*(*tinfo2).lpVtbl).Release)(tinfo2);
        return hres;
    }

    match (*tattr).typekind {
        k if k == TKIND_ALIAS => {
            hres = get_iface_guid(tinfo2, (*tattr).tdescAlias.u.hreftype, guid);
        }
        k if k == TKIND_INTERFACE || k == TKIND_DISPATCH => {
            *guid = (*tattr).guid;
        }
        k if k == TKIND_COCLASS => {
            let mut i: i32 = 0;
            while i < (*tattr).cImplTypes as i32 {
                ((*(*tinfo2).lpVtbl).GetImplTypeFlags)(tinfo2, i as UINT, &raw mut flags);
                if flags & IMPLTYPEFLAG_FDEFAULT != 0 {
                    break;
                }
                i += 1;
            }

            if i == (*tattr).cImplTypes as i32 {
                i = 0;
            }

            hres = ((*(*tinfo2).lpVtbl).GetRefTypeOfImplType)(tinfo2, i as UINT, &raw mut href);
            if SUCCEEDED(hres) {
                hres = get_iface_guid(tinfo2, href, guid);
            }
        }
        k => {
            ERR!("Unexpected typekind {}", k);
            hres = E_UNEXPECTED;
        }
    }

    ((*(*tinfo2).lpVtbl).ReleaseTypeAttr)(tinfo2, tattr);
    ((*(*tinfo2).lpVtbl).Release)(tinfo2);
    hres
}

#[inline]
unsafe fn func_restricted(desc: *const FUNCDESC) -> bool {
    ((*desc).wFuncFlags & FUNCFLAG_FRESTRICTED) != 0 && (*desc).memid >= 0
}

const INVBUF_ELEMENT_SIZE: usize =
    size_of::<VARIANTARG>() + size_of::<VARIANTARG>() + size_of::<*mut VARIANTARG>()
        + size_of::<VARTYPE>();
#[inline]
unsafe fn INVBUF_GET_ARG_ARRAY(buffer: *mut c_void, _params: i32) -> *mut VARIANTARG {
    buffer as *mut VARIANTARG
}
#[inline]
unsafe fn INVBUF_GET_MISSING_ARG_ARRAY(buffer: *mut c_void, params: i32) -> *mut VARIANTARG {
    (buffer as *mut u8).add(size_of::<VARIANTARG>() * params as usize) as *mut VARIANTARG
}
#[inline]
unsafe fn INVBUF_GET_ARG_PTR_ARRAY(buffer: *mut c_void, params: i32) -> *mut *mut VARIANTARG {
    (buffer as *mut u8)
        .add((size_of::<VARIANTARG>() + size_of::<VARIANTARG>()) * params as usize)
        as *mut *mut VARIANTARG
}
#[inline]
unsafe fn INVBUF_GET_ARG_TYPE_ARRAY(buffer: *mut c_void, params: i32) -> *mut VARTYPE {
    (buffer as *mut u8).add(
        (size_of::<VARIANTARG>() + size_of::<VARIANTARG>() + size_of::<*mut VARIANTARG>())
            * params as usize,
    ) as *mut VARTYPE
}

unsafe extern "system" fn ITypeInfo_fnInvoke(
    iface: *mut ITypeInfo2,
    pIUnk: PVOID,
    memid: MEMBERID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExcepInfo: *mut EXCEPINFO,
    pArgErr: *mut UINT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut var_index: UINT = 0;
    let mut type_kind: TYPEKIND = 0;
    let mut hres: HRESULT;
    let mut pFuncInfo: *mut TLBFuncDesc = null_mut();

    TRACE!();

    if (*This).typeattr.wTypeFlags & TYPEFLAG_FRESTRICTED != 0 {
        return DISP_E_MEMBERNOTFOUND;
    }

    if pDispParams.is_null() {
        ERR!("NULL pDispParams not allowed");
        return E_INVALIDARG;
    }

    /* dump_DispParms — TRACE-only, compiled out */

    if (*pDispParams).cNamedArgs > (*pDispParams).cArgs {
        ERR!(
            "named argument array cannot be bigger than argument array ({}/{})",
            (*pDispParams).cNamedArgs,
            (*pDispParams).cArgs
        );
        return E_INVALIDARG;
    }

    /* we do this instead of using GetFuncDesc since it will return a fake
     * FUNCDESC for dispinterfaces and we want the real function description */
    let mut fdc: UINT = 0;
    while fdc < (*This).typeattr.cFuncs as UINT {
        pFuncInfo = (*This).funcdescs.add(fdc as usize);
        if memid == (*pFuncInfo).funcdesc.memid
            && (wFlags as i32 & (*pFuncInfo).funcdesc.invkind) != 0
            && !func_restricted(&raw const (*pFuncInfo).funcdesc)
        {
            break;
        }
        fdc += 1;
    }

    if fdc < (*This).typeattr.cFuncs as UINT {
        let func_desc: *const FUNCDESC = &raw const (*pFuncInfo).funcdesc;

        match (*func_desc).funckind {
            k if k == FUNC_PUREVIRTUAL || k == FUNC_VIRTUAL => {
                let buffer =
                    heap_alloc_zero(INVBUF_ELEMENT_SIZE * (*func_desc).cParams as usize);
                let mut varresult: VARIANT = core::mem::zeroed();
                let mut retval: VARIANT = core::mem::zeroed(); /* pointer for storing byref retvals in */
                let prgpvarg = INVBUF_GET_ARG_PTR_ARRAY(buffer, (*func_desc).cParams as i32);
                let rgvarg = INVBUF_GET_ARG_ARRAY(buffer, (*func_desc).cParams as i32);
                let rgvt = INVBUF_GET_ARG_TYPE_ARRAY(buffer, (*func_desc).cParams as i32);
                let cNamedArgs: UINT = (*pDispParams).cNamedArgs;
                let rgdispidNamedArgs: *mut DISPID = (*pDispParams).rgdispidNamedArgs;
                let mut vargs_converted: UINT = 0;
                /* C keeps a single `SAFEARRAY *a` stack slot whose ADDRESS can
                 * be stored into rgvarg (V_BYREF(&rgvarg[i]) = &a) — one slot
                 * hoisted here reproduces the exact aliasing. */
                let mut safearray_slot: *mut SAFEARRAY = null_mut();

                hres = S_OK;

                'func_fail: {
                    if (*func_desc).invkind
                        & (INVOKE_PROPERTYPUT | INVOKE_PROPERTYPUTREF) as INVOKEKIND
                        != 0
                    {
                        if cNamedArgs == 0 || *rgdispidNamedArgs != DISPID_PROPERTYPUT {
                            ERR!("first named arg for property put invocation must be DISPID_PROPERTYPUT");
                            hres = DISP_E_PARAMNOTFOUND;
                            break 'func_fail;
                        }
                    }

                    if (*func_desc).cParamsOpt < 0 && cNamedArgs != 0 {
                        ERR!("functions with the vararg attribute do not support named arguments");
                        hres = DISP_E_NONAMEDARGS;
                        break 'func_fail;
                    }

                    let mut i: i32 = 0;
                    while i < (*func_desc).cParams as i32 {
                        let tdesc =
                            &raw mut (*(*func_desc).lprgelemdescParam.add(i as usize)).tdesc;
                        hres = typedescvt_to_variantvt(
                            iface as *mut ITypeInfo,
                            tdesc,
                            rgvt.add(i as usize),
                        );
                        if FAILED(hres) {
                            break 'func_fail;
                        }
                        i += 1;
                    }

                    TRACE!();
                    let mut i: i32 = 0;
                    'args: while i < (*func_desc).cParams as i32 {
                        let wParamFlags = (*(*func_desc).lprgelemdescParam.add(i as usize))
                            .u
                            .paramdesc
                            .wParamFlags;
                        let mut tdesc =
                            &raw mut (*(*func_desc).lprgelemdescParam.add(i as usize)).tdesc
                                as *mut TYPEDESC;

                        if wParamFlags & PARAMFLAG_FLCID != 0 {
                            let arg = rgvarg.add(i as usize);
                            *prgpvarg.add(i as usize) = arg;
                            (*arg).vt = VT_I4;
                            (*arg).n3.lVal = (*(*This).pTypeLib).lcid as i32;
                            i += 1;
                            continue;
                        }

                        let mut src_arg: *mut VARIANTARG = null_mut();

                        if cNamedArgs != 0 {
                            let mut j: UINT = 0;
                            while j < cNamedArgs {
                                if *rgdispidNamedArgs.add(j as usize) == i
                                    || (i == (*func_desc).cParams as i32 - 1
                                        && *rgdispidNamedArgs.add(j as usize)
                                            == DISPID_PROPERTYPUT)
                                {
                                    src_arg = (*pDispParams).rgvarg.add(j as usize);
                                    break;
                                }
                                j += 1;
                            }
                        }

                        if src_arg.is_null()
                            && vargs_converted + cNamedArgs < (*pDispParams).cArgs
                        {
                            src_arg = (*pDispParams)
                                .rgvarg
                                .add(((*pDispParams).cArgs - 1 - vargs_converted) as usize);
                            vargs_converted += 1;
                        }

                        if wParamFlags & PARAMFLAG_FRETVAL != 0 {
                            /* under most conditions the caller is not allowed to
                             * pass in a dispparam arg in the index of what would be
                             * the retval parameter. however, there is an exception
                             * where the extra parameter is used in an extra
                             * IDispatch::Invoke below */
                            if (i as UINT) < (*pDispParams).cArgs
                                && ((*func_desc).cParams != 1
                                    || pVarResult.is_null()
                                    || (*func_desc).invkind & INVOKE_PROPERTYGET == 0)
                            {
                                hres = DISP_E_BADPARAMCOUNT;
                                break 'args;
                            }

                            /* note: this check is placed so that if the caller passes
                             * in a VARIANTARG for the retval we just ignore it, like
                             * native does */
                            if i == (*func_desc).cParams as i32 - 1 {
                                let arg = rgvarg.add(i as usize);
                                *prgpvarg.add(i as usize) = arg;
                                memset(arg as *mut c_void, 0, size_of::<VARIANTARG>());
                                (*arg).vt = *rgvt.add(i as usize);
                                memset(
                                    &raw mut retval as *mut c_void,
                                    0,
                                    size_of::<VARIANT>(),
                                );
                                (*arg).n3.byref = &raw mut retval as *mut c_void;
                            } else {
                                ERR!(
                                    "[retval] parameter must be the last parameter of the method ({}/{})",
                                    i,
                                    (*func_desc).cParams
                                );
                                hres = E_UNEXPECTED;
                                break 'args;
                            }
                        } else if !src_arg.is_null()
                            && !(wParamFlags & PARAMFLAG_FOPT != 0
                                && (*src_arg).vt == VT_ERROR
                                && (*src_arg).n3.scode == DISP_E_PARAMNOTFOUND)
                        {
                            TRACE!();

                            if *rgvt.add(i as usize) != (*src_arg).vt {
                                if *rgvt.add(i as usize) == VT_VARIANT {
                                    hres = VariantCopy(rgvarg.add(i as usize), src_arg);
                                } else if *rgvt.add(i as usize) == (VT_VARIANT | VT_BYREF) {
                                    if *rgvt.add(i as usize) == (*src_arg).vt {
                                        (*rgvarg.add(i as usize)).n3.byref =
                                            (*src_arg).n3.byref;
                                    } else {
                                        let missing_arg = INVBUF_GET_MISSING_ARG_ARRAY(
                                            buffer,
                                            (*func_desc).cParams as i32,
                                        );
                                        if wParamFlags & PARAMFLAG_FIN != 0 {
                                            hres = VariantCopy(
                                                missing_arg.add(i as usize),
                                                src_arg,
                                            );
                                        }
                                        (*rgvarg.add(i as usize)).n3.byref =
                                            missing_arg.add(i as usize) as *mut c_void;
                                    }
                                    (*rgvarg.add(i as usize)).vt = *rgvt.add(i as usize);
                                } else if (*rgvt.add(i as usize) == (VT_VARIANT | VT_ARRAY)
                                    || *rgvt.add(i as usize)
                                        == (VT_VARIANT | VT_ARRAY | VT_BYREF))
                                    && (*func_desc).cParamsOpt < 0
                                {
                                    let mut bound = SAFEARRAYBOUND {
                                        cElements: (*pDispParams).cArgs - i as UINT,
                                        lLbound: 0,
                                    };
                                    let a = SafeArrayCreate(VT_VARIANT, 1, &raw mut bound);
                                    if a.is_null() {
                                        ERR!("SafeArrayCreate failed");
                                        break 'args;
                                    }
                                    safearray_slot = a;
                                    let mut v: *mut VARIANT = null_mut();
                                    hres = SafeArrayAccessData(
                                        a,
                                        &raw mut v as *mut PVOID,
                                    );
                                    if hres != S_OK {
                                        ERR!("SafeArrayAccessData failed with {:x}", hres as u32);
                                        SafeArrayDestroy(a);
                                        break 'args;
                                    }
                                    let mut j: LONG = 0;
                                    while (j as ULONG) < bound.cElements {
                                        VariantCopy(
                                            v.add(j as usize),
                                            (*pDispParams).rgvarg.add(
                                                ((*pDispParams).cArgs as i32 - 1 - i - j)
                                                    as usize,
                                            ),
                                        );
                                        j += 1;
                                    }
                                    hres = SafeArrayUnaccessData(a);
                                    if hres != S_OK {
                                        ERR!("SafeArrayUnaccessData failed with {:x}", hres as u32);
                                        SafeArrayDestroy(a);
                                        break 'args;
                                    }
                                    if *rgvt.add(i as usize) & VT_BYREF != 0 {
                                        (*rgvarg.add(i as usize)).n3.byref =
                                            &raw mut safearray_slot as *mut c_void;
                                    } else {
                                        (*rgvarg.add(i as usize)).n3.byref =
                                            safearray_slot as *mut c_void; /* V_ARRAY */
                                    }
                                    (*rgvarg.add(i as usize)).vt = *rgvt.add(i as usize);
                                } else if *rgvt.add(i as usize) & VT_BYREF != 0
                                    && (*src_arg).vt & VT_BYREF == 0
                                {
                                    let missing_arg = INVBUF_GET_MISSING_ARG_ARRAY(
                                        buffer,
                                        (*func_desc).cParams as i32,
                                    );
                                    if wParamFlags & PARAMFLAG_FIN != 0 {
                                        hres = VariantChangeType(
                                            missing_arg.add(i as usize),
                                            src_arg,
                                            0,
                                            *rgvt.add(i as usize) & !VT_BYREF,
                                        );
                                    } else {
                                        (*missing_arg.add(i as usize)).vt =
                                            *rgvt.add(i as usize) & !VT_BYREF;
                                    }
                                    /* V_BYREF(&rgvarg[i]) = &V_NONE(&missing_arg[i]) —
                                     * address of the data union */
                                    (*rgvarg.add(i as usize)).n3.byref =
                                        &raw mut (*missing_arg.add(i as usize)).n3
                                            as *mut c_void;
                                    (*rgvarg.add(i as usize)).vt = *rgvt.add(i as usize);
                                } else if *rgvt.add(i as usize) & VT_BYREF != 0
                                    && *rgvt.add(i as usize) == (*src_arg).vt
                                {
                                    (*rgvarg.add(i as usize)).n3.byref = (*src_arg).n3.byref;
                                    (*rgvarg.add(i as usize)).vt = *rgvt.add(i as usize);
                                } else {
                                    /* FIXME: this doesn't work for VT_BYREF arguments if
                                     * they are not the same type as in the paramdesc */
                                    (*rgvarg.add(i as usize)).vt = (*src_arg).vt;
                                    hres = VariantChangeType(
                                        rgvarg.add(i as usize),
                                        src_arg,
                                        0,
                                        *rgvt.add(i as usize),
                                    );
                                    (*rgvarg.add(i as usize)).vt = *rgvt.add(i as usize);
                                }

                                if FAILED(hres) {
                                    ERR!("failed to convert param {}", i);
                                    break 'args;
                                }
                                *prgpvarg.add(i as usize) = rgvarg.add(i as usize);
                            } else {
                                *prgpvarg.add(i as usize) = src_arg;
                            }

                            if ((*tdesc).vt == VT_USERDEFINED
                                || ((*tdesc).vt == VT_PTR
                                    && (*(*tdesc).u.lptdesc).vt == VT_USERDEFINED))
                                && ((*(*prgpvarg.add(i as usize))).vt == VT_DISPATCH
                                    || (*(*prgpvarg.add(i as usize))).vt == VT_UNKNOWN)
                                && !(*(*prgpvarg.add(i as usize))).n3.punkVal.is_null()
                            {
                                let mut userdefined_iface: *mut IUnknown = null_mut();
                                let mut guid: GUID = core::mem::zeroed();

                                if (*tdesc).vt == VT_PTR {
                                    tdesc = (*tdesc).u.lptdesc;
                                }

                                hres = get_iface_guid(
                                    iface as *mut ITypeInfo,
                                    (*tdesc).u.hreftype,
                                    &raw mut guid,
                                );
                                if FAILED(hres) {
                                    break 'args;
                                }

                                hres = IUnknown_QueryInterface(
                                    (*(*prgpvarg.add(i as usize))).n3.punkVal,
                                    &raw const guid,
                                    &raw mut userdefined_iface as *mut *mut c_void,
                                );
                                if FAILED(hres) {
                                    ERR!(
                                        "argument does not support {} interface",
                                        DbgGuid(&raw const guid)
                                    );
                                    break 'args;
                                }

                                IUnknown_Release((*(*prgpvarg.add(i as usize))).n3.punkVal);
                                (*(*prgpvarg.add(i as usize))).n3.punkVal = userdefined_iface;
                            }
                        } else if wParamFlags & PARAMFLAG_FOPT != 0 {
                            let arg = rgvarg.add(i as usize);
                            *prgpvarg.add(i as usize) = arg;
                            if wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
                                hres = VariantCopy(
                                    arg,
                                    &raw const (*(*(*func_desc)
                                        .lprgelemdescParam
                                        .add(i as usize))
                                    .u
                                    .paramdesc
                                    .pparamdescex)
                                        .varDefaultValue,
                                );
                                if FAILED(hres) {
                                    break 'args;
                                }
                            } else {
                                let missing_arg: *mut VARIANTARG;
                                /* if the function wants a pointer to a variant then
                                 * set that up, otherwise just pass the VT_ERROR in
                                 * the argument by value */
                                if *rgvt.add(i as usize) & VT_BYREF != 0 {
                                    missing_arg = INVBUF_GET_MISSING_ARG_ARRAY(
                                        buffer,
                                        (*func_desc).cParams as i32,
                                    )
                                    .add(i as usize);
                                    (*arg).vt = VT_VARIANT | VT_BYREF;
                                    (*arg).n3.byref = missing_arg as *mut c_void;
                                } else {
                                    missing_arg = arg;
                                }
                                (*missing_arg).vt = VT_ERROR;
                                (*missing_arg).n3.scode = DISP_E_PARAMNOTFOUND;
                            }
                        } else if (*func_desc).cParamsOpt < 0
                            && (*rgvt.add(i as usize) & !VT_BYREF) == (VT_VARIANT | VT_ARRAY)
                        {
                            // Wine 10.0: a vararg method called with NO varargs
                            // gets an empty SAFEARRAY, as Windows passes; it used
                            // to fail DISP_E_BADPARAMCOUNT.
                            hres = SafeArrayAllocDescriptorEx(VT_EMPTY, 1, &raw mut safearray_slot);
                            if FAILED(hres) {
                                break 'args;
                            }
                            let arg = rgvarg.add(i as usize);
                            if *rgvt.add(i as usize) & VT_BYREF != 0 {
                                (*arg).n3.byref = &raw mut safearray_slot as *mut c_void;
                            } else {
                                (*arg).n3.byref = safearray_slot as *mut c_void; /* V_ARRAY */
                            }
                            (*arg).vt = *rgvt.add(i as usize);
                            *prgpvarg.add(i as usize) = arg;
                        } else {
                            hres = DISP_E_BADPARAMCOUNT;
                            break 'args;
                        }

                        i += 1;
                    }
                    if FAILED(hres) {
                        break 'func_fail; /* FIXME: we don't free changed types here */
                    }

                    /* VT_VOID is a special case for return types, so it is not
                     * handled in the general function */
                    if (*func_desc).elemdescFunc.tdesc.vt == VT_VOID {
                        varresult.vt = VT_EMPTY;
                    } else {
                        varresult.vt = 0;
                        hres = typedescvt_to_variantvt(
                            iface as *mut ITypeInfo,
                            &raw const (*func_desc).elemdescFunc.tdesc,
                            &raw mut varresult.vt,
                        );
                        if FAILED(hres) {
                            break 'func_fail; /* FIXME: we don't free changed types here */
                        }
                    }

                    hres = DispCallFunc(
                        pIUnk,
                        ((*func_desc).oVft as u16 & 0xFFFC) as ULONG_PTR,
                        (*func_desc).callconv,
                        varresult.vt,
                        (*func_desc).cParams as UINT,
                        rgvt,
                        prgpvarg,
                        &raw mut varresult,
                    );

                    vargs_converted = 0;

                    let mut i: i32 = 0;
                    'cleanup: while i < (*func_desc).cParams as i32 {
                        let wParamFlags = (*(*func_desc).lprgelemdescParam.add(i as usize))
                            .u
                            .paramdesc
                            .wParamFlags;
                        let missing_arg =
                            INVBUF_GET_MISSING_ARG_ARRAY(buffer, (*func_desc).cParams as i32);

                        if wParamFlags & PARAMFLAG_FLCID != 0 {
                            i += 1;
                            continue;
                        } else if wParamFlags & PARAMFLAG_FRETVAL != 0 {
                            TRACE!();

                            if !pVarResult.is_null() {
                                VariantInit(pVarResult);
                                /* deref return value */
                                hres = VariantCopyInd(pVarResult, *prgpvarg.add(i as usize));
                            }

                            VARIANT_ClearInd(*prgpvarg.add(i as usize));
                        } else if vargs_converted < (*pDispParams).cArgs {
                            let arg = (*pDispParams)
                                .rgvarg
                                .add(((*pDispParams).cArgs - 1 - vargs_converted) as usize);
                            if wParamFlags & PARAMFLAG_FOUT != 0 {
                                if (*rgvt.add(i as usize) & VT_BYREF) != 0
                                    && ((*arg).vt & VT_BYREF) == 0
                                {
                                    hres = VariantChangeType(
                                        arg,
                                        rgvarg.add(i as usize),
                                        0,
                                        (*arg).vt,
                                    );

                                    if FAILED(hres) {
                                        ERR!("failed to convert param {} back", i);
                                        break 'cleanup;
                                    }
                                }
                            } else if (*(*prgpvarg.add(i as usize))).vt
                                == (VT_VARIANT | VT_ARRAY)
                                && (*func_desc).cParamsOpt < 0
                                && i == (*func_desc).cParams as i32 - 1
                            {
                                let a = (*(*prgpvarg.add(i as usize))).n3.byref
                                    as *mut SAFEARRAY; /* V_ARRAY */
                                let mut ubound: LONG = 0;
                                let mut v: *mut VARIANT = null_mut();
                                hres = SafeArrayGetUBound(a, 1, &raw mut ubound);
                                if hres != S_OK {
                                    ERR!("SafeArrayGetUBound failed with {:x}", hres as u32);
                                    break 'cleanup;
                                }
                                hres = SafeArrayAccessData(a, &raw mut v as *mut PVOID);
                                if hres != S_OK {
                                    ERR!("SafeArrayAccessData failed with {:x}", hres as u32);
                                    break 'cleanup;
                                }
                                let mut j: LONG = 0;
                                while j <= ubound {
                                    VariantClear(v.add(j as usize));
                                    j += 1;
                                }
                                hres = SafeArrayUnaccessData(a);
                                if hres != S_OK {
                                    ERR!("SafeArrayUnaccessData failed with {:x}", hres as u32);
                                    break 'cleanup;
                                }
                            }
                            VariantClear(rgvarg.add(i as usize));
                            vargs_converted += 1;
                        } else if wParamFlags & PARAMFLAG_FOPT != 0 {
                            if wParamFlags & PARAMFLAG_FHASDEFAULT != 0 {
                                VariantClear(rgvarg.add(i as usize));
                            }
                        }

                        VariantClear(missing_arg.add(i as usize));
                        i += 1;
                    }

                    if varresult.vt == VT_ERROR && FAILED(varresult.n3.scode) {
                        WARN!();
                        hres = DISP_E_EXCEPTION;
                        if !pExcepInfo.is_null() {
                            let mut pErrorInfo: *mut IErrorInfo = null_mut();
                            (*pExcepInfo).scode = varresult.n3.scode;
                            if GetErrorInfo(0, &raw mut pErrorInfo) == S_OK {
                                ((*(*pErrorInfo).lpVtbl).GetDescription)(
                                    pErrorInfo,
                                    &raw mut (*pExcepInfo).bstrDescription,
                                );
                                ((*(*pErrorInfo).lpVtbl).GetHelpFile)(
                                    pErrorInfo,
                                    &raw mut (*pExcepInfo).bstrHelpFile,
                                );
                                ((*(*pErrorInfo).lpVtbl).GetSource)(
                                    pErrorInfo,
                                    &raw mut (*pExcepInfo).bstrSource,
                                );
                                ((*(*pErrorInfo).lpVtbl).GetHelpContext)(
                                    pErrorInfo,
                                    &raw mut (*pExcepInfo).dwHelpContext,
                                );

                                ((*(*pErrorInfo).lpVtbl).Release)(pErrorInfo);
                            }
                        }
                    }
                    if varresult.vt != VT_ERROR {
                        TRACE!();

                        if !pVarResult.is_null() {
                            VariantClear(pVarResult);
                            *pVarResult = varresult;
                        } else {
                            VariantClear(&raw mut varresult);
                        }
                    }

                    if SUCCEEDED(hres)
                        && !pVarResult.is_null()
                        && (*func_desc).cParams == 1
                        && (*func_desc).invkind & INVOKE_PROPERTYGET != 0
                        && ((*(*func_desc).lprgelemdescParam).u.paramdesc.wParamFlags
                            & PARAMFLAG_FRETVAL)
                            != 0
                        && (*pDispParams).cArgs != 0
                    {
                        if (*pVarResult).vt == VT_DISPATCH {
                            let pDispatch = (*pVarResult).n3.pdispVal as *mut IDispatch;
                            /* Note: not VariantClear; we still need the dispatch
                             * pointer to be valid */
                            VariantInit(pVarResult);
                            hres = ((*(*pDispatch).lpVtbl).Invoke)(
                                pDispatch,
                                DISPID_VALUE,
                                &raw const IID_NULL,
                                GetSystemDefaultLCID(),
                                wFlags,
                                pDispParams,
                                pVarResult,
                                pExcepInfo,
                                pArgErr,
                            );
                            ((*(*pDispatch).lpVtbl).Release)(pDispatch);
                        } else {
                            VariantClear(pVarResult);
                            hres = DISP_E_NOTACOLLECTION;
                        }
                    }
                }

                /* func_fail: */
                heap_free(buffer);
            }
            k if k == FUNC_DISPATCH => {
                let mut disp: *mut IDispatch = null_mut();

                hres = IUnknown_QueryInterface(
                    pIUnk as *mut IUnknown,
                    &raw const IID_IDispatch,
                    &raw mut disp as *mut *mut c_void,
                );
                if SUCCEEDED(hres) {
                    FIXME!("Calling Invoke in IDispatch iface. untested!");
                    hres = ((*(*disp).lpVtbl).Invoke)(
                        disp,
                        memid,
                        &raw const IID_NULL,
                        LOCALE_USER_DEFAULT,
                        wFlags,
                        pDispParams,
                        pVarResult,
                        pExcepInfo,
                        pArgErr,
                    );
                    if FAILED(hres) {
                        FIXME!(
                            "IDispatch::Invoke failed with {:08x}. (Could be not a real error?)",
                            hres as u32
                        );
                    }
                    ((*(*disp).lpVtbl).Release)(disp);
                } else {
                    FIXME!("FUNC_DISPATCH used on object without IDispatch iface?");
                }
            }
            k => {
                FIXME!("Unknown function invocation type {}", k);
                hres = E_FAIL;
            }
        }

        TRACE!();
        return hres;
    }

    hres = ((*(*iface).lpVtbl).GetVarIndexOfMemId)(iface, memid, &raw mut var_index);
    if SUCCEEDED(hres) {
        let mut var_desc: *mut VARDESC = null_mut();

        hres = ((*(*iface).lpVtbl).GetVarDesc)(iface, var_index, &raw mut var_desc);
        if FAILED(hres) {
            return hres;
        }

        FIXME!("varseek: Found memid, but variable-based invoking not supported");
        /* dump_VARDESC — compiled out */
        ((*(*iface).lpVtbl).ReleaseVarDesc)(iface, var_desc);
        return E_NOTIMPL;
    }

    /* not found, check for special error cases (Wine 10.0): a property PUT on
     * a member that only has a GET, with the argument count of a put, is a
     * parameter-count error on Windows, not DISP_E_MEMBERNOTFOUND. */
    let mut fdc: UINT = 0;
    while fdc < (*This).typeattr.cFuncs as UINT {
        let func_desc = &raw const (*(*This).funcdescs.add(fdc as usize)).funcdesc;
        if memid == (*func_desc).memid
            && (wFlags as u32 & INVOKE_PROPERTYPUT as u32) != 0
            && ((*func_desc).invkind as u32 & INVOKE_PROPERTYGET as u32) != 0
        {
            let mut count_inputs: UINT = 0;
            let mut k: i32 = 0;
            while k < (*func_desc).cParams as i32 {
                let flags = (*(*func_desc).lprgelemdescParam.add(k as usize)).u.paramdesc.wParamFlags;
                if (flags as u32 & PARAMFLAG_FRETVAL as u32) == 0 {
                    count_inputs += 1;
                }
                k += 1;
            }
            if count_inputs == 0 || (*pDispParams).cArgs == count_inputs + 1 {
                return DISP_E_BADPARAMCOUNT;
            }
        }
        fdc += 1;
    }

    /* not found, look for it in inherited interfaces */
    ((*(*iface).lpVtbl).GetTypeKind)(iface, &raw mut type_kind);
    if type_kind == TKIND_INTERFACE || type_kind == TKIND_DISPATCH {
        if !(*This).impltypes.is_null() {
            /* recursive search */
            let mut pTInfo: *mut ITypeInfo = null_mut();
            hres = ((*(*iface).lpVtbl).GetRefTypeInfo)(
                iface,
                (*(*This).impltypes).hRef,
                &raw mut pTInfo,
            );
            if SUCCEEDED(hres) {
                hres = ((*(*pTInfo).lpVtbl).Invoke)(
                    pTInfo,
                    pIUnk,
                    memid,
                    wFlags,
                    pDispParams,
                    pVarResult,
                    pExcepInfo,
                    pArgErr,
                );
                ((*(*pTInfo).lpVtbl).Release)(pTInfo);
                return hres;
            }
            WARN!();
        }
    }
    WARN!();
    DISP_E_MEMBERNOTFOUND
}

/* ITypeInfo::GetDocumentation
 *
 * Retrieves the documentation string, the complete Help file name and path,
 * and the context ID for the Help topic for a specified type description.
 */
unsafe extern "system" fn ITypeInfo_fnGetDocumentation(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    pBstrName: *mut BSTR,
    pBstrDocString: *mut BSTR,
    pdwHelpContext: *mut DWORD,
    pBstrHelpFile: *mut BSTR,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    TRACE!();
    if memid == MEMBERID_NIL {
        /* documentation for the typeinfo */
        if !pBstrName.is_null() {
            *pBstrName = SysAllocString(TLB_get_bstr((*This).Name));
        }
        if !pBstrDocString.is_null() {
            *pBstrDocString = SysAllocString(TLB_get_bstr((*This).DocString));
        }
        if !pdwHelpContext.is_null() {
            *pdwHelpContext = (*This).dwHelpContext;
        }
        if !pBstrHelpFile.is_null() {
            *pBstrHelpFile = SysAllocString(TLB_get_bstr((*(*This).pTypeLib).HelpFile));
        }
        return S_OK;
    } else {
        /* for a member */
        let pFDesc = TLB_get_funcdesc_by_memberid(
            (*This).funcdescs,
            (*This).typeattr.cFuncs as UINT,
            memid,
        );
        if !pFDesc.is_null() {
            if !pBstrName.is_null() {
                *pBstrName = SysAllocString(TLB_get_bstr((*pFDesc).Name));
            }
            if !pBstrDocString.is_null() {
                *pBstrDocString = SysAllocString(TLB_get_bstr((*pFDesc).HelpString));
            }
            if !pdwHelpContext.is_null() {
                *pdwHelpContext = (*pFDesc).helpcontext as DWORD;
            }
            if !pBstrHelpFile.is_null() {
                *pBstrHelpFile = SysAllocString(TLB_get_bstr((*(*This).pTypeLib).HelpFile));
            }
            return S_OK;
        }
        let pVDesc = TLB_get_vardesc_by_memberid(
            (*This).vardescs,
            (*This).typeattr.cVars as UINT,
            memid,
        );
        if !pVDesc.is_null() {
            if !pBstrName.is_null() {
                *pBstrName = SysAllocString(TLB_get_bstr((*pVDesc).Name));
            }
            if !pBstrDocString.is_null() {
                *pBstrDocString = SysAllocString(TLB_get_bstr((*pVDesc).HelpString));
            }
            if !pdwHelpContext.is_null() {
                *pdwHelpContext = (*pVDesc).HelpContext as DWORD;
            }
            if !pBstrHelpFile.is_null() {
                *pBstrHelpFile = SysAllocString(TLB_get_bstr((*(*This).pTypeLib).HelpFile));
            }
            return S_OK;
        }
    }

    if !(*This).impltypes.is_null()
        && ((*This).typeattr.typekind == TKIND_INTERFACE
            || (*This).typeattr.typekind == TKIND_DISPATCH)
    {
        /* recursive search */
        let mut pTInfo: *mut ITypeInfo = null_mut();
        let result = ((*(*iface).lpVtbl).GetRefTypeInfo)(
            iface,
            (*(*This).impltypes).hRef,
            &raw mut pTInfo,
        );
        if SUCCEEDED(result) {
            let result = ((*(*pTInfo).lpVtbl).GetDocumentation)(
                pTInfo,
                memid,
                pBstrName,
                pBstrDocString,
                pdwHelpContext,
                pBstrHelpFile,
            );
            ((*(*pTInfo).lpVtbl).Release)(pTInfo);
            return result;
        }
        WARN!();
    }

    WARN!();
    TYPE_E_ELEMENTNOTFOUND
}

/*  ITypeInfo::GetDllEntry
 *
 * Retrieves a description or specification of an entry point for a function
 * in a DLL.
 */
unsafe extern "system" fn ITypeInfo_fnGetDllEntry(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    invKind: INVOKEKIND,
    pBstrDllName: *mut BSTR,
    pBstrName: *mut BSTR,
    pwOrdinal: *mut WORD,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    if !pBstrDllName.is_null() {
        *pBstrDllName = null_mut();
    }
    if !pBstrName.is_null() {
        *pBstrName = null_mut();
    }
    if !pwOrdinal.is_null() {
        *pwOrdinal = 0;
    }

    if (*This).typeattr.typekind != TKIND_MODULE {
        return TYPE_E_BADMODULEKIND;
    }

    // Wine 10.0: match the INVOKEKIND too. A module's property get and put
    // share a memid, and the first match handed back the wrong entry point.
    let mut pFDesc: *mut TLBFuncDesc = null_mut();
    let mut k: UINT = 0;
    while k < (*This).typeattr.cFuncs as UINT {
        let f = (*This).funcdescs.add(k as usize);
        if (*f).funcdesc.memid == memid && (*f).funcdesc.invkind == invKind {
            pFDesc = f;
            break;
        }
        k += 1;
    }
    if !pFDesc.is_null() {
        /* dump_TypeInfo / dump_TLBFuncDescOne — compiled out */

        if !pBstrDllName.is_null() {
            *pBstrDllName = SysAllocString(TLB_get_bstr((*This).DllName));
        }

        if !IS_INTRESOURCE_PTR((*pFDesc).Entry as *const c_void)
            && (*pFDesc).Entry != -1isize as usize as *const TLBString
        {
            if !pBstrName.is_null() {
                *pBstrName = SysAllocString(TLB_get_bstr((*pFDesc).Entry));
            }
            if !pwOrdinal.is_null() {
                *pwOrdinal = -1i32 as WORD;
            }
            return S_OK;
        }
        if !pBstrName.is_null() {
            *pBstrName = null_mut();
        }
        if !pwOrdinal.is_null() {
            *pwOrdinal = ((*pFDesc).Entry as usize & 0xffff) as WORD;
        }
        return S_OK;
    }
    TYPE_E_ELEMENTNOTFOUND
}

/* internal function to make the inherited interfaces' methods appear part of
 * the interface */
unsafe fn ITypeInfoImpl_GetDispatchRefTypeInfo(
    iface: *mut ITypeInfo,
    hRefType: *mut HREFTYPE,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let This = impl_from_ITypeInfo(iface);

    TRACE!();

    if !(*This).impltypes.is_null() && (*hRefType & DISPATCH_HREF_MASK) != 0 {
        let mut pSubTypeInfo: *mut ITypeInfo = null_mut();

        let hr = ((*(*iface).lpVtbl).GetRefTypeInfo)(
            iface,
            (*(*This).impltypes).hRef,
            &raw mut pSubTypeInfo,
        );
        if FAILED(hr) {
            return hr;
        }

        let hr = ITypeInfoImpl_GetDispatchRefTypeInfo(pSubTypeInfo, hRefType, ppTInfo);
        ((*(*pSubTypeInfo).lpVtbl).Release)(pSubTypeInfo);
        if SUCCEEDED(hr) {
            return hr;
        }
    }
    *hRefType -= DISPATCH_HREF_OFFSET;

    if (*hRefType & DISPATCH_HREF_MASK) == 0 {
        ((*(*iface).lpVtbl).GetRefTypeInfo)(iface, *hRefType, ppTInfo)
    } else {
        E_FAIL
    }
}

#[repr(C)]
struct search_res_tlb_params {
    guid: *const GUID,
    pTLib: *mut ITypeLib,
}

unsafe extern "system" fn search_res_tlb(
    hModule: HMODULE,
    lpszType: LPCWSTR,
    lpszName: LPWSTR,
    lParam: isize,
) -> BOOL {
    let params = lParam as *mut search_res_tlb_params;
    let mut szPath: [WCHAR; MAX_PATH + 1] = [0; MAX_PATH + 1];
    let mut pTLib: *mut ITypeLib = null_mut();

    if !IS_INTRESOURCE_PTR(lpszName as *const c_void) {
        return TRUE;
    }

    let len = GetModuleFileNameW(hModule, szPath.as_mut_ptr(), MAX_PATH as DWORD) as usize;
    if len == 0 {
        return TRUE;
    }

    /* swprintf(szPath + len, "\\%d", LOWORD(lpszName)) — hand-rolled */
    {
        let mut pos = len;
        szPath[pos] = b'\\' as u16;
        pos += 1;
        let mut val = (lpszName as usize & 0xffff) as u32;
        let mut digits = [0u16; 5];
        let mut n = 0usize;
        if val == 0 {
            digits[0] = b'0' as u16;
            n = 1;
        } else {
            while val != 0 {
                digits[n] = b'0' as u16 + (val % 10) as u16;
                val /= 10;
                n += 1;
            }
        }
        while n > 0 {
            n -= 1;
            szPath[pos] = digits[n];
            pos += 1;
        }
        szPath[pos] = 0;
    }

    let ret = super::super::typelib::LoadTypeLibEx(szPath.as_ptr(), REGKIND_NONE, &raw mut pTLib);
    if SUCCEEDED(ret) {
        let impl_ = impl_from_ITypeLib(pTLib);
        if guid_eq((*params).guid, TLB_get_guidref((*impl_).guid)) {
            (*params).pTLib = pTLib;
            return FALSE; /* stop enumeration */
        }
        ((*(*pTLib).lpVtbl).Release)(pTLib);
    }

    TRUE
}

/* ITypeInfo::GetRefTypeInfo
 *
 * If a type description references other type descriptions, it retrieves the
 * referenced type descriptions.
 */
unsafe extern "system" fn ITypeInfo_fnGetRefTypeInfo(
    iface: *mut ITypeInfo2,
    hRefType: HREFTYPE,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut result: HRESULT = E_FAIL;

    if ppTInfo.is_null() {
        return E_INVALIDARG;
    }

    if (hRefType as INT) < 0 {
        if (*This).typeattr.wTypeFlags & TYPEFLAG_FDUAL == 0
            || !((*This).typeattr.typekind == TKIND_INTERFACE
                || (*This).typeattr.typekind == TKIND_DISPATCH)
        {
            return TYPE_E_ELEMENTNOTFOUND;
        }

        /* when we meet a DUAL typeinfo, we must create the alternate version
         * of it. */
        let pTypeInfoImpl = ITypeInfoImpl_Constructor();

        /* *pTypeInfoImpl = *This (struct copy) */
        core::ptr::copy_nonoverlapping(This as *const u8, pTypeInfoImpl as *mut u8, size_of::<ITypeInfoImpl>());
        (*pTypeInfoImpl).ref_ = 0;
        list_init(&raw mut (*pTypeInfoImpl).custdata_list);

        if (*This).typeattr.typekind == TKIND_INTERFACE {
            (*pTypeInfoImpl).typeattr.typekind = TKIND_DISPATCH;
        } else {
            (*pTypeInfoImpl).typeattr.typekind = TKIND_INTERFACE;
        }

        *ppTInfo = &raw mut (*pTypeInfoImpl).ITypeInfo2_iface as *mut ITypeInfo;
        /* the AddRef implicitly adds a reference to the parent typelib, which
         * stops the copied data from being destroyed until the new typeinfo's
         * refcount goes to zero, but we need to signal to the new instance to
         * not free its data structures when it is destroyed */
        (*pTypeInfoImpl).not_attached_to_typelib = TRUE;

        let p = *ppTInfo;
        ((*(*p).lpVtbl).AddRef)(p);

        result = S_OK;
    } else if (hRefType & DISPATCH_HREF_MASK) != 0
        && (*This).typeattr.typekind == TKIND_DISPATCH
    {
        let mut href_dispatch: HREFTYPE = hRefType;
        result = ITypeInfoImpl_GetDispatchRefTypeInfo(
            iface as *mut ITypeInfo,
            &raw mut href_dispatch,
            ppTInfo,
        );
    } else {
        'end: {
            let mut pTLib: *mut ITypeLib = null_mut();

            if hRefType & 0x1 == 0 {
                let mut i: i32 = 0;
                while i < (*(*This).pTypeLib).TypeInfoCount {
                    let ti = *(*(*This).pTypeLib).typeinfos.add(i as usize);
                    if (*ti).hreftype == (hRefType & !0x3) {
                        result = S_OK;
                        *ppTInfo = &raw mut (*ti).ITypeInfo2_iface as *mut ITypeInfo;
                        let p = *ppTInfo;
                        ((*(*p).lpVtbl).AddRef)(p);
                        break 'end;
                    }
                    i += 1;
                }
            }

            /* LIST_FOR_EACH_ENTRY with completion sentinel, hand-rolled */
            let ref_list = &raw mut (*(*This).pTypeLib).ref_list;
            let mut ref_type: *mut TLBRefType = null_mut();
            let mut cur = (*ref_list).next;
            while cur != ref_list {
                let candidate = list_entry!(cur, TLBRefType, entry);
                if (*candidate).reference == (hRefType & !0x3) {
                    ref_type = candidate;
                    break;
                }
                cur = (*cur).next;
            }
            if ref_type.is_null() {
                FIXME!("Can't find pRefType for ref {:x}", hRefType);
                break 'end;
            }

            if (*ref_type).pImpTLInfo == TLB_REF_INTERNAL {
                let mut Index: UINT = 0;
                TRACE!();
                result = ((*(*iface).lpVtbl).GetContainingTypeLib)(
                    iface,
                    &raw mut pTLib,
                    &raw mut Index,
                );
            } else {
                if !(*(*ref_type).pImpTLInfo).pImpTypeLib.is_null() {
                    TRACE!();
                    pTLib = &raw mut (*(*(*ref_type).pImpTLInfo).pImpTypeLib).ITypeLib2_iface
                        as *mut ITypeLib;
                    ((*(*pTLib).lpVtbl).AddRef)(pTLib);
                    result = S_OK;
                } else {
                    // Wine 10.0: an already-loaded typelib with the right
                    // libid, version and lcid is reused from the cache before
                    // anything is searched or loaded again.
                    let imp = (*ref_type).pImpTLInfo;
                    let want = TLB_get_guid_null((*imp).guid);
                    let (cache, _) = super::msft::tlb_cache_lock();
                    let mut cur = (*cache).next;
                    while cur != cache {
                        let entry = list_entry!(cur, ITypeLibImpl, entry);
                        if !(*entry).guid.is_null()
                            && (*(*entry).guid).guid == *want
                            && (*entry).ver_major == (*imp).wVersionMajor
                            && (*entry).ver_minor == (*imp).wVersionMinor
                            && (*entry).set_lcid == (*imp).lcid
                        {
                            pTLib = &raw mut (*entry).ITypeLib2_iface as *mut ITypeLib;
                            ((*(*pTLib).lpVtbl).AddRef)(pTLib);
                            break;
                        }
                        cur = (*cur).next;
                    }
                    super::msft::tlb_cache_unlock();
                }
                if !pTLib.is_null() && (*(*ref_type).pImpTLInfo).pImpTypeLib.is_null() {
                    result = S_OK;
                    (*(*ref_type).pImpTLInfo).pImpTypeLib = impl_from_ITypeLib(pTLib);
                    ((*(*pTLib).lpVtbl).AddRef)(pTLib);
                } else if (*(*ref_type).pImpTLInfo).pImpTypeLib.is_null() {
                    // (resource table, then disk: the pre-10 search, kept as
                    // the fallback; wine-10 dropped the resource search, which
                    // came from Wine Staging and still finds embedded typelibs.)
                    let mut params = search_res_tlb_params {
                        guid: TLB_get_guid_null((*(*ref_type).pImpTLInfo).guid),
                        pTLib: null_mut(),
                    };

                    TRACE!();

                    /* Search in resource table */
                    let TYPELIBW = wide!("TYPELIB");
                    EnumResourceNamesW(
                        null_mut(),
                        TYPELIBW.as_ptr(),
                        search_res_tlb,
                        &raw mut params as isize,
                    );
                    pTLib = params.pTLib;
                    result = S_OK;

                    if pTLib.is_null() {
                        let mut libnam: BSTR = null_mut();

                        /* Search on disk */
                        result = super::super::typelib::query_typelib_path(
                            TLB_get_guid_null((*(*ref_type).pImpTLInfo).guid),
                            (*(*ref_type).pImpTLInfo).wVersionMajor,
                            (*(*ref_type).pImpTLInfo).wVersionMinor,
                            (*(*This).pTypeLib).syskind,
                            (*(*ref_type).pImpTLInfo).lcid,
                            &raw mut libnam,
                            TRUE,
                        );
                        if FAILED(result) {
                            libnam = SysAllocString((*(*ref_type).pImpTLInfo).name);
                        }

                        result = super::super::typelib::LoadTypeLib(libnam, &raw mut pTLib);
                        SysFreeString(libnam);
                    }

                    if SUCCEEDED(result) {
                        (*(*ref_type).pImpTLInfo).pImpTypeLib = impl_from_ITypeLib(pTLib);
                        ((*(*pTLib).lpVtbl).AddRef)(pTLib);
                    }
                }
            }
            if SUCCEEDED(result) {
                if (*ref_type).index == TLB_REF_USE_GUID {
                    result = ((*(*pTLib).lpVtbl).GetTypeInfoOfGuid)(
                        pTLib,
                        TLB_get_guid_null((*ref_type).guid),
                        ppTInfo,
                    );
                } else {
                    result =
                        ((*(*pTLib).lpVtbl).GetTypeInfo)(pTLib, (*ref_type).index as UINT, ppTInfo);
                }
            }
            if !pTLib.is_null() {
                ((*(*pTLib).lpVtbl).Release)(pTLib);
            }
        }

        // Wine 10.0: bit 0x2 of an hreftype names the OTHER half of a dual
        // interface. Hand back that alternate (GetRefTypeInfo(-2) on what was
        // found); before, the plain typeinfo came back for both halves.
        if SUCCEEDED(result) && (hRefType & 0x2) != 0 && !(*ppTInfo).is_null() {
            let found = *ppTInfo;
            let mut alt: *mut ITypeInfo = null_mut();
            if SUCCEEDED(((*(*found).lpVtbl).GetRefTypeInfo)(found, -2i32 as HREFTYPE, &raw mut alt)) {
                ((*(*found).lpVtbl).Release)(found);
                *ppTInfo = alt;
            }
        }
    }

    /* end: */
    TRACE!();
    result
}

/* ITypeInfo::AddressOfMember
 *
 * Retrieves the addresses of static functions or variables, such as those
 * defined in a DLL.
 */
unsafe extern "system" fn ITypeInfo_fnAddressOfMember(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    invKind: INVOKEKIND,
    ppv: *mut PVOID,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut dll: BSTR = null_mut();
    let mut entry: BSTR = null_mut();
    let mut ordinal: WORD = 0;

    TRACE!();

    let hr = ((*(*iface).lpVtbl).GetDllEntry)(
        iface,
        memid,
        invKind,
        &raw mut dll,
        &raw mut entry,
        &raw mut ordinal,
    );
    if FAILED(hr) {
        return hr;
    }

    let module = LoadLibraryW(dll);
    if module.is_null() {
        ERR!("couldn't load {}", DbgW(dll));
        SysFreeString(dll);
        SysFreeString(entry);
        return STG_E_FILENOTFOUND;
    }
    /* FIXME: store library somewhere where we can free it */

    if !entry.is_null() {
        let len = WideCharToMultiByte(CP_ACP, 0, entry, -1, null_mut(), 0, null(), null_mut());
        let entryA = heap_alloc(len as usize) as LPSTR;
        WideCharToMultiByte(CP_ACP, 0, entry, -1, entryA, len, null(), null_mut());

        *ppv = GetProcAddress(module, entryA);
        if (*ppv).is_null() {
            ERR!("function not found (named entry)");
        }

        heap_free(entryA as *mut c_void);
    } else {
        *ppv = GetProcAddress(module, MAKEINTRESOURCEA(ordinal as INT));
        if (*ppv).is_null() {
            ERR!("function not found {}", ordinal);
        }
    }

    SysFreeString(dll);
    SysFreeString(entry);

    if (*ppv).is_null() {
        return TYPE_E_DLLFUNCTIONNOTFOUND;
    }

    S_OK
}

/* ITypeInfo::CreateInstance
 *
 * Creates a new instance of a type that describes a component object class
 * (coclass).
 */
unsafe extern "system" fn ITypeInfo_fnCreateInstance(
    iface: *mut ITypeInfo2,
    pOuterUnk: *mut IUnknown,
    riid: REFIID,
    ppvObj: *mut PVOID,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let mut pTA: *mut TYPEATTR = null_mut();

    TRACE!();

    *ppvObj = null_mut();

    if !pOuterUnk.is_null() {
        WARN!();
        return CLASS_E_NOAGGREGATION;
    }

    let mut hr = ((*(*iface).lpVtbl).GetTypeAttr)(iface, &raw mut pTA);
    if FAILED(hr) {
        return hr;
    }

    'end: {
        if (*pTA).typekind != TKIND_COCLASS {
            WARN!();
            hr = E_INVALIDARG;
            break 'end;
        }

        hr = S_FALSE;
        if (*pTA).wTypeFlags & TYPEFLAG_FAPPOBJECT != 0 {
            let mut pUnk: *mut IUnknown = null_mut();
            hr = GetActiveObject(&raw const (*pTA).guid, null_mut(), &raw mut pUnk);
            TRACE!();
            if hr == S_OK {
                hr = IUnknown_QueryInterface(pUnk, riid, ppvObj);
                IUnknown_Release(pUnk);
            }
        }

        if hr != S_OK {
            hr = CoCreateInstance(
                &raw const (*pTA).guid,
                null_mut(),
                CLSCTX_INPROC_SERVER | CLSCTX_LOCAL_SERVER,
                riid,
                ppvObj,
            );
        }
    }

    /* end: */
    ((*(*iface).lpVtbl).ReleaseTypeAttr)(iface, pTA);
    hr
}

/* ITypeInfo::GetMops
 *
 * Retrieves marshalling information.
 */
unsafe extern "system" fn ITypeInfo_fnGetMops(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    pBstrMops: *mut BSTR,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    FIXME!("({:p} {}) stub!", This, memid);
    *pBstrMops = null_mut();
    S_OK
}

/* ITypeInfo::GetContainingTypeLib
 *
 * Retrieves the containing type library and the index of the type description
 * within that type library.
 */
unsafe extern "system" fn ITypeInfo_fnGetContainingTypeLib(
    iface: *mut ITypeInfo2,
    ppTLib: *mut *mut ITypeLib,
    pIndex: *mut UINT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    /* If a pointer is null, we simply ignore it, the ATL in particular passes
     * pIndex as 0 */
    if !pIndex.is_null() {
        *pIndex = (*This).index as UINT;
        TRACE!();
    }

    if !ppTLib.is_null() {
        *ppTLib = &raw mut (*(*This).pTypeLib).ITypeLib2_iface as *mut ITypeLib;
        let p = *ppTLib;
        ((*(*p).lpVtbl).AddRef)(p);
        TRACE!();
    }

    S_OK
}

/* ITypeInfo::ReleaseTypeAttr
 *
 * Releases a TYPEATTR previously returned by Get
 */
unsafe extern "system" fn ITypeInfo_fnReleaseTypeAttr(
    iface: *mut ITypeInfo2,
    pTypeAttr: *mut TYPEATTR,
) {
    let This = impl_from_ITypeInfo2(iface);
    TRACE!();
    heap_free(pTypeAttr as *mut c_void);
}

/* ITypeInfo::ReleaseFuncDesc
 *
 * Releases a FUNCDESC previously returned by GetFuncDesc.
 */
unsafe extern "system" fn ITypeInfo_fnReleaseFuncDesc(
    iface: *mut ITypeInfo2,
    pFuncDesc: *mut FUNCDESC,
) {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    let mut i: i32 = 0;
    while i < (*pFuncDesc).cParams as i32 {
        TLB_FreeElemDesc((*pFuncDesc).lprgelemdescParam.add(i as usize));
        i += 1;
    }
    TLB_FreeElemDesc(&raw mut (*pFuncDesc).elemdescFunc);

    SysFreeString(pFuncDesc as BSTR);
}

/* ITypeInfo::ReleaseVarDesc
 *
 * Releases a VARDESC previously returned by GetVarDesc.
 */
unsafe extern "system" fn ITypeInfo_fnReleaseVarDesc(
    iface: *mut ITypeInfo2,
    pVarDesc: *mut VARDESC,
) {
    let This = impl_from_ITypeInfo2(iface);
    TRACE!();

    TLB_FreeVarDesc(pVarDesc);
}

/* ITypeInfo2::GetTypeKind
 *
 * Returns the TYPEKIND enumeration quickly, without doing any allocations.
 */
unsafe extern "system" fn ITypeInfo2_fnGetTypeKind(
    iface: *mut ITypeInfo2,
    pTypeKind: *mut TYPEKIND,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    *pTypeKind = (*This).typeattr.typekind;
    TRACE!();
    S_OK
}

/* ITypeInfo2::GetTypeFlags
 *
 * Returns the type flags without any allocations. This returns a DWORD type
 * flag, which expands the type flags without growing the TYPEATTR (type
 * attribute).
 */
unsafe extern "system" fn ITypeInfo2_fnGetTypeFlags(
    iface: *mut ITypeInfo2,
    pTypeFlags: *mut ULONG,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    *pTypeFlags = (*This).typeattr.wTypeFlags as ULONG;
    TRACE!();
    S_OK
}

/* ITypeInfo2::GetFuncIndexOfMemId
 * Binds to a specific member based on a known DISPID, where the member name
 * is not known (for example, when binding to a default member).
 */
unsafe extern "system" fn ITypeInfo2_fnGetFuncIndexOfMemId(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    invKind: INVOKEKIND,
    pFuncIndex: *mut UINT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let result: HRESULT;

    let mut fdc: UINT = 0;
    while fdc < (*This).typeattr.cFuncs as UINT {
        let pFuncInfo = (*This).funcdescs.add(fdc as usize);
        if memid == (*pFuncInfo).funcdesc.memid
            && (invKind & (*pFuncInfo).funcdesc.invkind) != 0
        {
            break;
        }
        fdc += 1;
    }
    if fdc < (*This).typeattr.cFuncs as UINT {
        *pFuncIndex = fdc;
        result = S_OK;
    } else {
        result = TYPE_E_ELEMENTNOTFOUND;
    }

    TRACE!();
    result
}

/* TypeInfo2::GetVarIndexOfMemId
 *
 * Binds to a specific member based on a known DISPID, where the member name
 * is not known (for example, when binding to a default member).
 */
unsafe extern "system" fn ITypeInfo2_fnGetVarIndexOfMemId(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    pVarIndex: *mut UINT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    let pVarInfo = TLB_get_vardesc_by_memberid(
        (*This).vardescs,
        (*This).typeattr.cVars as UINT,
        memid,
    );
    if pVarInfo.is_null() {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    *pVarIndex = pVarInfo.offset_from((*This).vardescs) as UINT;

    S_OK
}

/* ITypeInfo2::GetCustData
 *
 * Gets the custom data
 */
unsafe extern "system" fn ITypeInfo2_fnGetCustData(
    iface: *mut ITypeInfo2,
    guid: REFGUID,
    pVarVal: *mut VARIANT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    if guid.is_null() || pVarVal.is_null() {
        return E_INVALIDARG;
    }

    let pCData = TLB_get_custdata_by_guid((*This).pcustdata_list, guid);

    VariantInit(pVarVal);
    if !pCData.is_null() {
        VariantCopy(pVarVal, &raw const (*pCData).data);
    } else {
        VariantClear(pVarVal);
    }
    S_OK
}

/* ITypeInfo2::GetFuncCustData
 *
 * Gets the custom data
 */
unsafe extern "system" fn ITypeInfo2_fnGetFuncCustData(
    iface: *mut ITypeInfo2,
    index: UINT,
    guid: REFGUID,
    pVarVal: *mut VARIANT,
) -> HRESULT {
    TRACE!();

    let pFDesc = match internal_tlb_funcdesc(iface as *mut ITypeInfo, index) {
        Ok(f) => f,
        Err(hr) => return hr,
    };

    // Wine 10.0: a GUID with no data is S_OK with an EMPTY variant, not
    // TYPE_E_ELEMENTNOTFOUND.
    VariantInit(pVarVal);
    let pCData = TLB_get_custdata_by_guid(&raw mut (*pFDesc).custdata_list, guid);
    if pCData.is_null() {
        S_OK
    } else {
        VariantCopy(pVarVal, &raw const (*pCData).data)
    }
}

/* ITypeInfo2::GetParamCustData
 *
 * Gets the custom data
 */
unsafe extern "system" fn ITypeInfo2_fnGetParamCustData(
    iface: *mut ITypeInfo2,
    indexFunc: UINT,
    indexParam: UINT,
    guid: REFGUID,
    pVarVal: *mut VARIANT,
) -> HRESULT {
    TRACE!();

    let pFDesc = match internal_tlb_funcdesc(iface as *mut ITypeInfo, indexFunc) {
        Ok(f) => f,
        Err(hr) => return hr,
    };

    if indexParam >= (*pFDesc).funcdesc.cParams as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    let pCData = TLB_get_custdata_by_guid(
        &raw mut (*(*pFDesc).pParamDesc.add(indexParam as usize)).custdata_list,
        guid,
    );
    if pCData.is_null() {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    VariantInit(pVarVal);
    VariantCopy(pVarVal, &raw const (*pCData).data);

    S_OK
}

/* ITypeInfo2::GetVarCustData
 *
 * Gets the custom data
 */
unsafe extern "system" fn ITypeInfo2_fnGetVarCustData(
    iface: *mut ITypeInfo2,
    index: UINT,
    guid: REFGUID,
    pVarVal: *mut VARIANT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let pVDesc = (*This).vardescs.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cVars as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    let pCData = TLB_get_custdata_by_guid(&raw mut (*pVDesc).custdata_list, guid);
    if pCData.is_null() {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    VariantInit(pVarVal);
    VariantCopy(pVarVal, &raw const (*pCData).data);

    S_OK
}

/* ITypeInfo2::GetImplCustData
 *
 * Gets the custom data
 */
unsafe extern "system" fn ITypeInfo2_fnGetImplTypeCustData(
    iface: *mut ITypeInfo2,
    index: UINT,
    guid: REFGUID,
    pVarVal: *mut VARIANT,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let pRDesc = (*This).impltypes.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cImplTypes as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    let pCData = TLB_get_custdata_by_guid(&raw mut (*pRDesc).custdata_list, guid);
    if pCData.is_null() {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    VariantInit(pVarVal);
    VariantCopy(pVarVal, &raw const (*pCData).data);

    S_OK
}

/* ITypeInfo2::GetDocumentation2
 *
 * Retrieves the documentation string, the complete Help file name and path,
 * the localization context to use, and the context ID for the library Help
 * topic in the Help file.
 */
unsafe extern "system" fn ITypeInfo2_fnGetDocumentation2(
    iface: *mut ITypeInfo2,
    memid: MEMBERID,
    lcid: LCID,
    pbstrHelpString: *mut BSTR,
    pdwHelpStringContext: *mut DWORD,
    pbstrHelpStringDll: *mut BSTR,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    TRACE!();
    /* the help string should be obtained from the helpstringdll, using the
     * _DLLGetDocumentation function, based on the supplied lcid. Nice to do
     * sometime... */
    if memid == MEMBERID_NIL {
        /* documentation for the typeinfo */
        if !pbstrHelpString.is_null() {
            *pbstrHelpString = SysAllocString(TLB_get_bstr((*This).Name));
        }
        if !pdwHelpStringContext.is_null() {
            *pdwHelpStringContext = (*This).dwHelpStringContext;
        }
        if !pbstrHelpStringDll.is_null() {
            *pbstrHelpStringDll =
                SysAllocString(TLB_get_bstr((*(*This).pTypeLib).HelpStringDll)); /* FIXME */
        }
        return S_OK;
    } else {
        /* for a member */
        let pFDesc = TLB_get_funcdesc_by_memberid(
            (*This).funcdescs,
            (*This).typeattr.cFuncs as UINT,
            memid,
        );
        if !pFDesc.is_null() {
            if !pbstrHelpString.is_null() {
                *pbstrHelpString = SysAllocString(TLB_get_bstr((*pFDesc).HelpString));
            }
            if !pdwHelpStringContext.is_null() {
                *pdwHelpStringContext = (*pFDesc).HelpStringContext as DWORD;
            }
            if !pbstrHelpStringDll.is_null() {
                *pbstrHelpStringDll =
                    SysAllocString(TLB_get_bstr((*(*This).pTypeLib).HelpStringDll)); /* FIXME */
            }
            return S_OK;
        }
        let pVDesc = TLB_get_vardesc_by_memberid(
            (*This).vardescs,
            (*This).typeattr.cVars as UINT,
            memid,
        );
        if !pVDesc.is_null() {
            if !pbstrHelpString.is_null() {
                *pbstrHelpString = SysAllocString(TLB_get_bstr((*pVDesc).HelpString));
            }
            if !pdwHelpStringContext.is_null() {
                *pdwHelpStringContext = (*pVDesc).HelpStringContext as DWORD;
            }
            if !pbstrHelpStringDll.is_null() {
                *pbstrHelpStringDll =
                    SysAllocString(TLB_get_bstr((*(*This).pTypeLib).HelpStringDll)); /* FIXME */
            }
            return S_OK;
        }
    }
    TYPE_E_ELEMENTNOTFOUND
}

/* ITypeInfo2::GetAllCustData
 *
 * Gets all custom data items for the Type info.
 */
unsafe extern "system" fn ITypeInfo2_fnGetAllCustData(
    iface: *mut ITypeInfo2,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);

    TRACE!();

    super::itypelib::TLB_copy_all_custdata((*This).pcustdata_list, pCustData)
}

/* ITypeInfo2::GetAllFuncCustData
 *
 * Gets all custom data items for the specified Function
 */
unsafe extern "system" fn ITypeInfo2_fnGetAllFuncCustData(
    iface: *mut ITypeInfo2,
    index: UINT,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    TRACE!();

    let pFDesc = match internal_tlb_funcdesc(iface as *mut ITypeInfo, index) {
        Ok(f) => f,
        Err(hr) => return hr,
    };

    super::itypelib::TLB_copy_all_custdata(&raw mut (*pFDesc).custdata_list, pCustData)
}

/* ITypeInfo2::GetAllParamCustData
 *
 * Gets all custom data items for the Functions
 */
unsafe extern "system" fn ITypeInfo2_fnGetAllParamCustData(
    iface: *mut ITypeInfo2,
    indexFunc: UINT,
    indexParam: UINT,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    TRACE!();

    let pFDesc = match internal_tlb_funcdesc(iface as *mut ITypeInfo, indexFunc) {
        Ok(f) => f,
        Err(hr) => return hr,
    };

    if indexParam >= (*pFDesc).funcdesc.cParams as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    super::itypelib::TLB_copy_all_custdata(
        &raw mut (*(*pFDesc).pParamDesc.add(indexParam as usize)).custdata_list,
        pCustData,
    )
}

/* ITypeInfo2::GetAllVarCustData
 *
 * Gets all custom data items for the specified Variable
 */
unsafe extern "system" fn ITypeInfo2_fnGetAllVarCustData(
    iface: *mut ITypeInfo2,
    index: UINT,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let pVDesc = (*This).vardescs.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cVars as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    super::itypelib::TLB_copy_all_custdata(&raw mut (*pVDesc).custdata_list, pCustData)
}

/* ITypeInfo2::GetAllImplCustData
 *
 * Gets all custom data items for the specified implementation type
 */
unsafe extern "system" fn ITypeInfo2_fnGetAllImplTypeCustData(
    iface: *mut ITypeInfo2,
    index: UINT,
    pCustData: *mut CUSTDATA,
) -> HRESULT {
    let This = impl_from_ITypeInfo2(iface);
    let pRDesc = (*This).impltypes.add(index as usize);

    TRACE!();

    if index >= (*This).typeattr.cImplTypes as UINT {
        return TYPE_E_ELEMENTNOTFOUND;
    }

    super::itypelib::TLB_copy_all_custdata(&raw mut (*pRDesc).custdata_list, pCustData)
}

pub static tinfvt: ITypeInfo2Vtbl = ITypeInfo2Vtbl {
    QueryInterface: ITypeInfo_fnQueryInterface,
    AddRef: ITypeInfo_fnAddRef,
    Release: ITypeInfo_fnRelease,

    GetTypeAttr: ITypeInfo_fnGetTypeAttr,
    GetTypeComp: ITypeInfo_fnGetTypeComp,
    GetFuncDesc: ITypeInfo_fnGetFuncDesc,
    GetVarDesc: ITypeInfo_fnGetVarDesc,
    GetNames: ITypeInfo_fnGetNames,
    GetRefTypeOfImplType: ITypeInfo_fnGetRefTypeOfImplType,
    GetImplTypeFlags: ITypeInfo_fnGetImplTypeFlags,
    GetIDsOfNames: ITypeInfo_fnGetIDsOfNames,
    Invoke: ITypeInfo_fnInvoke,
    GetDocumentation: ITypeInfo_fnGetDocumentation,
    GetDllEntry: ITypeInfo_fnGetDllEntry,
    GetRefTypeInfo: ITypeInfo_fnGetRefTypeInfo,
    AddressOfMember: ITypeInfo_fnAddressOfMember,
    CreateInstance: ITypeInfo_fnCreateInstance,
    GetMops: ITypeInfo_fnGetMops,
    GetContainingTypeLib: ITypeInfo_fnGetContainingTypeLib,
    ReleaseTypeAttr: ITypeInfo_fnReleaseTypeAttr,
    ReleaseFuncDesc: ITypeInfo_fnReleaseFuncDesc,
    ReleaseVarDesc: ITypeInfo_fnReleaseVarDesc,

    GetTypeKind: ITypeInfo2_fnGetTypeKind,
    GetTypeFlags: ITypeInfo2_fnGetTypeFlags,
    GetFuncIndexOfMemId: ITypeInfo2_fnGetFuncIndexOfMemId,
    GetVarIndexOfMemId: ITypeInfo2_fnGetVarIndexOfMemId,
    GetCustData: ITypeInfo2_fnGetCustData,
    GetFuncCustData: ITypeInfo2_fnGetFuncCustData,
    GetParamCustData: ITypeInfo2_fnGetParamCustData,
    GetVarCustData: ITypeInfo2_fnGetVarCustData,
    GetImplTypeCustData: ITypeInfo2_fnGetImplTypeCustData,
    GetDocumentation2: ITypeInfo2_fnGetDocumentation2,
    GetAllCustData: ITypeInfo2_fnGetAllCustData,
    GetAllFuncCustData: ITypeInfo2_fnGetAllFuncCustData,
    GetAllParamCustData: ITypeInfo2_fnGetAllParamCustData,
    GetAllVarCustData: ITypeInfo2_fnGetAllVarCustData,
    GetAllImplTypeCustData: ITypeInfo2_fnGetAllImplTypeCustData,
};
