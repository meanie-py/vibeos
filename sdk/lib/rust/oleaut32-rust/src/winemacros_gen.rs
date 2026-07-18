//! GENERATED V_* accessors (oleauto.h V_UNION map) + wine variant.h macro fns.
#![allow(clippy::all)]
use crate::prelude::*;

#[inline]
pub unsafe fn V_ARRAY_ref(v: *mut VARIANT) -> *mut *mut SAFEARRAY {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.parray)
}
#[inline]
pub unsafe fn V_ARRAYREF_ref(v: *mut VARIANT) -> *mut *mut *mut SAFEARRAY {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pparray)
}
#[inline]
pub unsafe fn V_BOOL_ref(v: *mut VARIANT) -> *mut VARIANT_BOOL {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.boolVal)
}
#[inline]
pub unsafe fn V_BOOLREF_ref(v: *mut VARIANT) -> *mut *mut VARIANT_BOOL {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pboolVal)
}
#[inline]
pub unsafe fn V_BSTR_ref(v: *mut VARIANT) -> *mut BSTR {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.bstrVal)
}
#[inline]
pub unsafe fn V_BSTRREF_ref(v: *mut VARIANT) -> *mut *mut BSTR {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pbstrVal)
}
#[inline]
pub unsafe fn V_BYREF_ref(v: *mut VARIANT) -> *mut PVOID {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.byref)
}
#[inline]
pub unsafe fn V_CY_ref(v: *mut VARIANT) -> *mut CY {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.cyVal)
}
#[inline]
pub unsafe fn V_CYREF_ref(v: *mut VARIANT) -> *mut *mut CY {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pcyVal)
}
#[inline]
pub unsafe fn V_DATE_ref(v: *mut VARIANT) -> *mut DATE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.date)
}
#[inline]
pub unsafe fn V_DATEREF_ref(v: *mut VARIANT) -> *mut *mut DATE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pdate)
}
#[inline]
pub unsafe fn V_DECIMALREF_ref(v: *mut VARIANT) -> *mut *mut DECIMAL {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pdecVal)
}
#[inline]
pub unsafe fn V_DISPATCH_ref(v: *mut VARIANT) -> *mut *mut IDispatch {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pdispVal)
}
#[inline]
pub unsafe fn V_DISPATCHREF_ref(v: *mut VARIANT) -> *mut *mut *mut IDispatch {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.ppdispVal)
}
#[inline]
pub unsafe fn V_ERROR_ref(v: *mut VARIANT) -> *mut SCODE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.scode)
}
#[inline]
pub unsafe fn V_ERRORREF_ref(v: *mut VARIANT) -> *mut *mut SCODE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pscode)
}
#[inline]
pub unsafe fn V_I1_ref(v: *mut VARIANT) -> *mut ::core::ffi::c_schar {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.cVal)
}
#[inline]
pub unsafe fn V_I1REF_ref(v: *mut VARIANT) -> *mut *mut ::core::ffi::c_schar {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pcVal)
}
#[inline]
pub unsafe fn V_I2_ref(v: *mut VARIANT) -> *mut SHORT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.iVal)
}
#[inline]
pub unsafe fn V_I2REF_ref(v: *mut VARIANT) -> *mut *mut SHORT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.piVal)
}
#[inline]
pub unsafe fn V_I4_ref(v: *mut VARIANT) -> *mut LONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.lVal)
}
#[inline]
pub unsafe fn V_I4REF_ref(v: *mut VARIANT) -> *mut *mut LONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.plVal)
}
#[inline]
pub unsafe fn V_I8_ref(v: *mut VARIANT) -> *mut LONGLONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.llVal)
}
#[inline]
pub unsafe fn V_I8REF_ref(v: *mut VARIANT) -> *mut *mut LONGLONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pllVal)
}
#[inline]
pub unsafe fn V_INT_ref(v: *mut VARIANT) -> *mut INT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.intVal)
}
#[inline]
pub unsafe fn V_INTREF_ref(v: *mut VARIANT) -> *mut *mut INT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pintVal)
}
#[inline]
pub unsafe fn V_R4_ref(v: *mut VARIANT) -> *mut FLOAT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.fltVal)
}
#[inline]
pub unsafe fn V_R4REF_ref(v: *mut VARIANT) -> *mut *mut FLOAT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pfltVal)
}
#[inline]
pub unsafe fn V_R8_ref(v: *mut VARIANT) -> *mut DOUBLE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.dblVal)
}
#[inline]
pub unsafe fn V_R8REF_ref(v: *mut VARIANT) -> *mut *mut DOUBLE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pdblVal)
}
#[inline]
pub unsafe fn V_UI1_ref(v: *mut VARIANT) -> *mut BYTE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.bVal)
}
#[inline]
pub unsafe fn V_UI1REF_ref(v: *mut VARIANT) -> *mut *mut BYTE {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pbVal)
}
#[inline]
pub unsafe fn V_UI2_ref(v: *mut VARIANT) -> *mut USHORT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.uiVal)
}
#[inline]
pub unsafe fn V_UI2REF_ref(v: *mut VARIANT) -> *mut *mut USHORT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.puiVal)
}
#[inline]
pub unsafe fn V_UI4_ref(v: *mut VARIANT) -> *mut ULONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.ulVal)
}
#[inline]
pub unsafe fn V_UI4REF_ref(v: *mut VARIANT) -> *mut *mut ULONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pulVal)
}
#[inline]
pub unsafe fn V_UI8_ref(v: *mut VARIANT) -> *mut ULONGLONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.ullVal)
}
#[inline]
pub unsafe fn V_UI8REF_ref(v: *mut VARIANT) -> *mut *mut ULONGLONG {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pullVal)
}
#[inline]
pub unsafe fn V_UINT_ref(v: *mut VARIANT) -> *mut UINT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.uintVal)
}
#[inline]
pub unsafe fn V_UINTREF_ref(v: *mut VARIANT) -> *mut *mut UINT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.puintVal)
}
#[inline]
pub unsafe fn V_UNKNOWN_ref(v: *mut VARIANT) -> *mut *mut IUnknown {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.punkVal)
}
#[inline]
pub unsafe fn V_UNKNOWNREF_ref(v: *mut VARIANT) -> *mut *mut *mut IUnknown {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.ppunkVal)
}
#[inline]
pub unsafe fn V_VARIANTREF_ref(v: *mut VARIANT) -> *mut *mut VARIANT {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.pvarVal)
}
#[inline]
pub unsafe fn V_VT_ref(v: *mut VARIANT) -> *mut VARTYPE {
    core::ptr::addr_of_mut!((*v).n1.n2.vt)
}
#[inline]
pub unsafe fn V_DECIMAL_ref(v: *mut VARIANT) -> *mut DECIMAL {
    core::ptr::addr_of_mut!((*v).n1.decVal)
}
