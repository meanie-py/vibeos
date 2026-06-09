//! inflib shared type substrate.
//! Sourced from sdk/lib/inflib/infpriv.h and infcommon.h.
//! These types are NOT in ntkrnl-sys — inflib is a standalone SDK library.
//! All modules import via `use crate::types::*`.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use core::ffi::c_void;

// LANGID is in ntkrnl_sys::ddk (bindgen), not base.
// Re-export here so all inflib modules can use it without ddk.
/// LANGID — Language ID (USHORT), from WinNT / bindgen.
pub type LANGID = u16;

// Basic Win32/NT scalar aliases not re-exported by ntkrnl_sys::base.
pub type UINT   = u32;
pub type INT    = i32;
pub type BOOL   = i32;
pub type HANDLE = *mut c_void;
/// `_UNICODE_STRING` is the tag name; base exports it as `UNICODE_STRING`.
pub type _UNICODE_STRING = UNICODE_STRING;

// ---------- infcommon.h -------------------------------------------------

pub const MAX_INF_STRING_LENGTH: usize = 512;

/// HINF / PHINF (opaque handle, void * / void **)
// Already in ntkrnl_sys::base as PVOID/PVOID* but re-exported for clarity.
// HINF = void*, PHINF = void**  (infcommon.h: typedef void *HINF, **PHINF)
pub type HINF  = *mut c_void;
pub type PHINF = *mut *mut c_void;

// Forward declaration — PINFCONTEXT = pointer to _INFCONTEXT
// (infcommon.h: typedef struct _INFCONTEXT *PINFCONTEXT)
// Defined below after the struct.

// ---------- infpriv.h ---------------------------------------------------

/// INFSTATUS — signed int (typedef int INFSTATUS).
/// INF_SUCCESS(x) = (0 <= x), like NTSTATUS.
pub type INFSTATUS = i32;

// Status codes — kernel/setup (non-HOST) world:
// These map to NTSTATUS values via the !INFLIB_HOST branch of builddep.h.
pub const INF_STATUS_SUCCESS:                INFSTATUS = 0;
pub const INF_STATUS_NO_MEMORY:              INFSTATUS = 0xC0000017u32 as i32; // STATUS_NO_MEMORY
pub const INF_STATUS_INVALID_PARAMETER:      INFSTATUS = 0xC000000Du32 as i32; // STATUS_INVALID_PARAMETER
pub const INF_STATUS_NOT_FOUND:              INFSTATUS = 0xC0000225u32 as i32; // STATUS_NOT_FOUND
pub const INF_STATUS_BUFFER_OVERFLOW:        INFSTATUS = 0xC0000023u32 as i32; // STATUS_BUFFER_OVERFLOW
pub const INF_STATUS_INSUFFICIENT_RESOURCES: INFSTATUS = 0xC000009Au32 as i32; // STATUS_INSUFFICIENT_RESOURCES

// Private error codes (infpriv.h)
pub const INF_STATUS_BAD_SECTION_NAME_LINE:  INFSTATUS = 0xC0700001u32 as i32;
pub const INF_STATUS_SECTION_NAME_TOO_LONG:  INFSTATUS = 0xC0700002u32 as i32;
pub const INF_STATUS_WRONG_INF_STYLE:        INFSTATUS = 0xC0700003u32 as i32;
pub const INF_STATUS_NOT_ENOUGH_MEMORY:      INFSTATUS = 0xC0700004u32 as i32;

// ---------- struct definitions (infpriv.h) ------------------------------

#[repr(C)]
pub struct _INFCACHEFIELD {
    pub Next: *mut _INFCACHEFIELD,
    pub Prev: *mut _INFCACHEFIELD,
    pub Data: [u16; 1], // flexible array member (WCHAR Data[1])
}
pub type INFCACHEFIELD  = _INFCACHEFIELD;
pub type PINFCACHEFIELD = *mut _INFCACHEFIELD;

#[repr(C)]
pub struct _INFCACHELINE {
    pub Next:       *mut _INFCACHELINE,
    pub Prev:       *mut _INFCACHELINE,
    pub Id:         UINT,
    pub FieldCount: LONG,
    pub Key:        PWCHAR,
    pub FirstField: PINFCACHEFIELD,
    pub LastField:  PINFCACHEFIELD,
}
pub type INFCACHELINE  = _INFCACHELINE;
pub type PINFCACHELINE = *mut _INFCACHELINE;

#[repr(C)]
pub struct _INFCACHESECTION {
    pub Next:        *mut _INFCACHESECTION,
    pub Prev:        *mut _INFCACHESECTION,
    pub FirstLine:   PINFCACHELINE,
    pub LastLine:    PINFCACHELINE,
    pub Id:          UINT,
    pub LineCount:   LONG,
    pub NextLineId:  UINT,
    pub Name:        [u16; 1], // flexible array member (WCHAR Name[1])
}
pub type INFCACHESECTION  = _INFCACHESECTION;
pub type PINFCACHESECTION = *mut _INFCACHESECTION;

#[repr(C)]
pub struct _INFCACHE {
    pub LanguageId:    LANGID,
    pub FirstSection:  PINFCACHESECTION,
    pub LastSection:   PINFCACHESECTION,
    pub NextSectionId: UINT,
    pub StringsSection: PINFCACHESECTION,
}
pub type INFCACHE  = _INFCACHE;
pub type PINFCACHE = *mut _INFCACHE;

#[repr(C)]
pub struct _INFCONTEXT {
    pub Inf:        PINFCACHE,
    pub CurrentInf: PINFCACHE,
    pub Section:    UINT,
    pub Line:       UINT,
}
pub type INFCONTEXT  = _INFCONTEXT;
pub type PINFCONTEXT = *mut _INFCONTEXT;
