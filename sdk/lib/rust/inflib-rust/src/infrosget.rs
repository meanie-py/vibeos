//! infrosget.rs — ReactOS public INF get/find API (thin BOOLEAN wrappers over Infp* interns).
//! Translated from sdk/lib/inflib/infrosget.c.
//!
//! Fixes applied:
//!  - `InfGetBinaryField` return: was `as i32` (incorrect for BOOLEAN) → `as BOOLEAN`.
//!  - `InfGetData` return type: C header says BOOLEAN; assembled had `-> i32` → BOOLEAN.
//!  - All `(0 <= fn()) as i32 as BOOLEAN` chains simplified to `(0 <= fn()) as BOOLEAN`.
//!  - No `extern {}` for sibling functions — all direct Rust calls.
//!  - `InfpFreeContext` called via `crate::infget::InfpFreeContext`.
//!  - `InfGetLineCount` / `InfGetFieldCount` return LONG (C header: LONG, not BOOLEAN).

use crate::types::*;
use crate::infget::{
    InfpFindFirstLine, InfpFindNextLine, InfpFindFirstMatchLine, InfpFindNextMatchLine,
    InfpGetLineCount, InfpGetFieldCount,
    InfpGetBinaryField, InfpGetIntField, InfpGetMultiSzField, InfpGetStringField,
    InfpGetData, InfpGetDataField,
    InfpFreeContext,
};
use ntkrnl_sys::base::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn InfFindFirstLine(
    InfHandle: HINF,
    Section:   PCWSTR,
    Key:       PCWSTR,
    Context:   *mut PINFCONTEXT,
) -> BOOLEAN {
    (InfpFindFirstLine(InfHandle as PINFCACHE, Section, Key, Context) >= 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfFindNextLine(
    ContextIn:  PINFCONTEXT,
    ContextOut: PINFCONTEXT,
) -> BOOLEAN {
    (0 <= InfpFindNextLine(ContextIn, ContextOut)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfFindFirstMatchLine(
    ContextIn:  PINFCONTEXT,
    Key:        PCWSTR,
    ContextOut: PINFCONTEXT,
) -> BOOLEAN {
    (0 <= InfpFindFirstMatchLine(ContextIn, Key, ContextOut)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfFindNextMatchLine(
    ContextIn:  PINFCONTEXT,
    Key:        PCWSTR,
    ContextOut: PINFCONTEXT,
) -> BOOLEAN {
    (0 <= InfpFindNextMatchLine(ContextIn, Key, ContextOut)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfGetLineCount(InfHandle: HINF, Section: PCWSTR) -> LONG {
    InfpGetLineCount(InfHandle, Section)
}

#[no_mangle]
pub unsafe extern "C" fn InfGetFieldCount(Context: PINFCONTEXT) -> LONG {
    InfpGetFieldCount(Context)
}

/// FIXED: return type is BOOLEAN; was returning `as i32` (compile error on BOOLEAN = u8).
#[no_mangle]
pub unsafe extern "C" fn InfGetBinaryField(
    Context:          PINFCONTEXT,
    FieldIndex:       ULONG,
    ReturnBuffer:     PUCHAR,
    ReturnBufferSize: ULONG,
    RequiredSize:     PULONG,
) -> BOOLEAN {
    (0 <= InfpGetBinaryField(Context, FieldIndex, ReturnBuffer, ReturnBufferSize, RequiredSize))
        as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfGetIntField(
    Context:      PINFCONTEXT,
    FieldIndex:   ULONG,
    IntegerValue: *mut INT,
) -> BOOLEAN {
    (0 <= InfpGetIntField(Context, FieldIndex, IntegerValue)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfGetMultiSzField(
    Context:          PINFCONTEXT,
    FieldIndex:       ULONG,
    ReturnBuffer:     PWSTR,
    ReturnBufferSize: ULONG,
    RequiredSize:     PULONG,
) -> BOOLEAN {
    (0 <= InfpGetMultiSzField(Context, FieldIndex, ReturnBuffer, ReturnBufferSize, RequiredSize))
        as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfGetStringField(
    Context:          PINFCONTEXT,
    FieldIndex:       ULONG,
    ReturnBuffer:     PWSTR,
    ReturnBufferSize: ULONG,
    RequiredSize:     PULONG,
) -> BOOLEAN {
    (0 <= InfpGetStringField(Context, FieldIndex, ReturnBuffer, ReturnBufferSize, RequiredSize))
        as BOOLEAN
}

/// FIXED: C header declares `BOOLEAN InfGetData(...)` — was `-> i32` in assembled, compile error.
#[no_mangle]
pub unsafe extern "C" fn InfGetData(
    Context: PINFCONTEXT,
    Key:     *mut PWCHAR,
    Data:    *mut PWCHAR,
) -> BOOLEAN {
    (0 <= InfpGetData(Context, Key, Data)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfGetDataField(
    Context:    PINFCONTEXT,
    FieldIndex: ULONG,
    Data:       *mut PWCHAR,
) -> BOOLEAN {
    (0 <= InfpGetDataField(Context, FieldIndex, Data)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn InfFreeContext(Context: PINFCONTEXT) {
    InfpFreeContext(Context as *mut c_void);
}
