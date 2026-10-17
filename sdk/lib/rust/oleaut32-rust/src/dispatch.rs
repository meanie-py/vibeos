//! dispatch.rs — Rust translation of dll/win32/oleaut32/dispatch.c
//! (Dispatch API functions: DispInvoke/DispGetIDsOfNames/DispGetParam +
//! CreateStdDispatch's StdDispatch IDispatch implementation).
//!
//! Faithful 1:1. Copyright (original C): 2000 Francois Jacques, Macadamian
//! Technologies Inc. LGPL 2.1+.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_unsafe, dead_code, unused_variables, unused_mut)]

use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::null_mut;

use super::olesys::*;

macro_rules! TRACE {
    ($($t:tt)*) => {{}};
}
macro_rules! FIXME {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::olesys::dbg_print(format_args!(concat!("fixme:ole:dispatch ", $fmt, "\n") $(, $e)*))
    };
}

// oleaut32's own exports consumed here (still-C or Phase-1/2 Rust; resolved
// at link): variant.c
extern "system" {
    pub fn VariantChangeType(
        pvargDest: *mut VARIANTARG,
        pvargSrc: *mut VARIANTARG,
        wFlags: USHORT,
        vt: VARTYPE,
    ) -> HRESULT;
}

/******************************************************************************
 *		DispInvoke (OLEAUT32.30)
 *
 * Call an object method using the information from its type library.
 */
#[no_mangle]
pub unsafe extern "system" fn DispInvoke(
    _this: *mut c_void,
    ptinfo: *mut ITypeInfo,
    dispidMember: DISPID,
    wFlags: USHORT,
    pparams: *mut DISPPARAMS,
    pvarResult: *mut VARIANT,
    pexcepinfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    TRACE!();

    ((*(*ptinfo).lpVtbl).Invoke)(
        ptinfo,
        _this,
        dispidMember,
        wFlags as WORD,
        pparams,
        pvarResult,
        pexcepinfo,
        puArgErr,
    )
}

/******************************************************************************
 *		DispGetIDsOfNames (OLEAUT32.29)
 *
 * Convert a set of parameter names to DISPIDs for DispInvoke().
 */
#[no_mangle]
pub unsafe extern "system" fn DispGetIDsOfNames(
    ptinfo: *mut ITypeInfo,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    rgdispid: *mut DISPID,
) -> HRESULT {
    ((*(*ptinfo).lpVtbl).GetIDsOfNames)(ptinfo, rgszNames, cNames, rgdispid)
}

/******************************************************************************
 *		DispGetParam (OLEAUT32.28)
 *
 * Retrieve a parameter from a DISPPARAMS structure and coerce it to the
 * specified variant type. Coercion is done using system (0) locale.
 */
#[no_mangle]
pub unsafe extern "system" fn DispGetParam(
    pdispparams: *mut DISPPARAMS,
    position: UINT,
    vtTarg: VARTYPE,
    pvarResult: *mut VARIANT,
    puArgErr: *mut UINT,
) -> HRESULT {
    /* position is counted backwards */
    let mut pos: UINT;
    let mut hr: HRESULT;

    TRACE!();

    if position < (*pdispparams).cArgs {
        /* positional arg? */
        pos = (*pdispparams).cArgs - position - 1;
    } else {
        /* FIXME: is this how to handle named args? */
        pos = 0;
        while pos < (*pdispparams).cNamedArgs {
            if *(*pdispparams).rgdispidNamedArgs.add(pos as usize) == position as DISPID {
                break;
            }
            pos += 1;
        }

        if pos == (*pdispparams).cNamedArgs {
            return DISP_E_PARAMNOTFOUND;
        }
    }

    'done: {
        if (*pdispparams).cArgs > 0 && (*pdispparams).rgvarg.is_null() {
            hr = E_INVALIDARG;
            break 'done;
        }

        if pvarResult.is_null() {
            hr = E_INVALIDARG;
            break 'done;
        }

        hr = VariantChangeType(pvarResult, (*pdispparams).rgvarg.add(pos as usize), 0, vtTarg);
    }

    // done:
    if FAILED(hr) {
        *puArgErr = pos;
    }

    hr
}

// ---------------------------------------------------------------------------
// StdDispatch — the standard IDispatch implementation (CreateStdDispatch)
// ---------------------------------------------------------------------------

#[repr(C)]
struct StdDispatch {
    IDispatch_iface: IDispatch,
    pvThis: *mut c_void,
    pTypeInfo: *mut ITypeInfo,
    ref_: LONG,
}

#[inline]
unsafe fn impl_from_IDispatch(iface: *mut IDispatch) -> *mut StdDispatch {
    (iface as *mut u8).sub(offset_of!(StdDispatch, IDispatch_iface)) as *mut StdDispatch
}

unsafe extern "system" fn StdDispatch_QueryInterface(
    iface: LPDISPATCH,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IDispatch(iface);
    TRACE!();

    *ppvObject = null_mut();

    if guid_eq(riid, &IID_IDispatch) || guid_eq(riid, &IID_IUnknown) {
        *ppvObject = iface as *mut c_void;
        ((*(*iface).lpVtbl).AddRef)(iface);
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn StdDispatch_AddRef(iface: LPDISPATCH) -> ULONG {
    let This = impl_from_IDispatch(iface);
    let refCount = InterlockedIncrement(&raw mut (*This).ref_) as ULONG;

    TRACE!();

    refCount
}

unsafe extern "system" fn StdDispatch_Release(iface: LPDISPATCH) -> ULONG {
    let This = impl_from_IDispatch(iface);
    let refCount = InterlockedDecrement(&raw mut (*This).ref_) as ULONG;

    TRACE!();

    if refCount == 0 {
        let ti = (*This).pTypeInfo;
        ((*(*ti).lpVtbl).Release)(ti);
        CoTaskMemFree(This as PVOID);
    }

    refCount
}

unsafe extern "system" fn StdDispatch_GetTypeInfoCount(
    iface: LPDISPATCH,
    pctinfo: *mut UINT,
) -> HRESULT {
    TRACE!();
    *pctinfo = 1;
    S_OK
}

unsafe extern "system" fn StdDispatch_GetTypeInfo(
    iface: LPDISPATCH,
    iTInfo: UINT,
    lcid: LCID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let This = impl_from_IDispatch(iface);
    TRACE!();

    *ppTInfo = null_mut();
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }

    *ppTInfo = (*This).pTypeInfo;
    ((*(**ppTInfo).lpVtbl).AddRef)(*ppTInfo);

    S_OK
}

unsafe extern "system" fn StdDispatch_GetIDsOfNames(
    iface: LPDISPATCH,
    riid: REFIID,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    lcid: LCID,
    rgDispId: *mut DISPID,
) -> HRESULT {
    let This = impl_from_IDispatch(iface);
    TRACE!();

    if !guid_eq(riid, &IID_NULL) {
        FIXME!(" expected riid == IID_NULL");
        return E_INVALIDARG;
    }
    DispGetIDsOfNames((*This).pTypeInfo, rgszNames, cNames, rgDispId)
}

unsafe extern "system" fn StdDispatch_Invoke(
    iface: LPDISPATCH,
    dispIdMember: DISPID,
    riid: REFIID,
    lcid: LCID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExcepInfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    let This = impl_from_IDispatch(iface);
    TRACE!();

    if !guid_eq(riid, &IID_NULL) {
        FIXME!(" expected riid == IID_NULL");
        return E_INVALIDARG;
    }
    DispInvoke(
        (*This).pvThis,
        (*This).pTypeInfo,
        dispIdMember,
        wFlags as USHORT,
        pDispParams,
        pVarResult,
        pExcepInfo,
        puArgErr,
    )
}

static StdDispatch_VTable: IDispatchVtbl = IDispatchVtbl {
    QueryInterface: StdDispatch_QueryInterface,
    AddRef: StdDispatch_AddRef,
    Release: StdDispatch_Release,
    GetTypeInfoCount: StdDispatch_GetTypeInfoCount,
    GetTypeInfo: StdDispatch_GetTypeInfo,
    GetIDsOfNames: StdDispatch_GetIDsOfNames,
    Invoke: StdDispatch_Invoke,
};

/******************************************************************************
 * CreateStdDispatch [OLEAUT32.32]
 *
 * Create and return a standard IDispatch object.
 *
 * NOTES
 *  Outer unknown appears to be completely ignored.
 */
#[no_mangle]
pub unsafe extern "system" fn CreateStdDispatch(
    punkOuter: *mut IUnknown,
    pvThis: *mut c_void,
    ptinfo: *mut ITypeInfo,
    stddisp: *mut *mut IUnknown,
) -> HRESULT {
    TRACE!();

    if pvThis.is_null() || ptinfo.is_null() || stddisp.is_null() {
        return E_INVALIDARG;
    }

    let pStdDispatch = CoTaskMemAlloc((size_of::<StdDispatch>()) as _) as *mut StdDispatch;
    if pStdDispatch.is_null() {
        return E_OUTOFMEMORY;
    }

    (*pStdDispatch).IDispatch_iface.lpVtbl = &StdDispatch_VTable;
    (*pStdDispatch).pvThis = pvThis;
    (*pStdDispatch).pTypeInfo = ptinfo;
    (*pStdDispatch).ref_ = 1;

    /* we keep a reference to the type info so prevent it from
     * being destroyed until we are done with it */
    ((*(*ptinfo).lpVtbl).AddRef)(ptinfo);
    *stddisp = &raw mut (*pStdDispatch).IDispatch_iface as *mut IUnknown;

    S_OK
}
