//! Hand-written WDF convenience helpers — the few `<wdf.h>` client APIs that are
//! *macros* or *globals-accessors* rather than table dispatchers, so
//! gen_kmdf_dispatch.py (which only emits the FORCEINLINE table-dispatch APIs)
//! cannot generate them. Each is a faithful translation of the macro expansion in
//! wdfobject.h / wdfdriver.h. Keep this list minimal — anything with a
//! `WdfXxxTableIndex` belongs in the generated dispatch.rs instead.
#![allow(non_snake_case, dead_code, unused_imports)]

use crate::dispatch::{
    WdfObjectDereferenceActual, WdfObjectGetTypedContextWorker, WdfObjectReferenceActual,
};
use crate::types::*;
use ntkrnl_sys::ddk::*;

/// `WdfGetDriver()` → `WdfDriverGlobals->Driver` (wdfdriver.h FORCEINLINE; reads the
/// per-driver globals the bind stub populated — not a table dispatch).
#[inline]
pub unsafe fn WdfGetDriver() -> WDFDRIVER {
    (*WdfDriverGlobals).Driver
}

/// `WdfObjectReference(Handle)` → `WdfObjectReferenceWithTag(Handle, NULL)` →
/// `WdfObjectReferenceActual(Handle, NULL, __LINE__, __FILE__)` (wdfobject.h).
/// Tag/line/file are checked-build leak-tracking diagnostics only; null/0 is
/// functionally correct on free builds.
#[inline]
pub unsafe fn WdfObjectReference(Handle: WDFOBJECT) {
    WdfObjectReferenceActual(Handle, core::ptr::null_mut(), 0, core::ptr::null_mut());
}

/// `WdfObjectDereference(Handle)` → `WdfObjectDereferenceActual(Handle, NULL, __LINE__, __FILE__)`.
#[inline]
pub unsafe fn WdfObjectDereference(Handle: WDFOBJECT) {
    WdfObjectDereferenceActual(Handle, core::ptr::null_mut(), 0, core::ptr::null_mut());
}

/// `WdfObjectGetTypedContext(handle, type)` →
/// `WdfObjectGetTypedContextWorker(handle, WDF_GET_CONTEXT_TYPE_INFO(type)->UniqueType)`
/// (wdfobject.h). The caller passes the context type-info pointer; we forward its
/// `UniqueType` to the worker and the caller casts the returned `PVOID`.
#[inline]
pub unsafe fn WdfObjectGetTypedContext(
    Handle: WDFOBJECT,
    TypeInfo: PCWDF_OBJECT_CONTEXT_TYPE_INFO,
) -> PVOID {
    WdfObjectGetTypedContextWorker(Handle, (*TypeInfo).UniqueType)
}
