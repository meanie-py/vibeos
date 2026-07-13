//! OLE/Win32 types the translations use that com-sys's bindgen does not export.
//!
//! com-sys is generated from the COM *interface* headers, so it has IStorage/IStream/
//! STATDATA/… but not the plain-C surrounding types: the DVASPECT enum, the OLE 1.0
//! `OLESTREAM` conversion callbacks, the serialized property-value header, STGOPTIONS,
//! and the bare `LPBYTE`/`LPCSTR`/`PCHAR` aliases. Each is transcribed from the header
//! cited above it.
#![allow(clippy::all)]
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Plain aliases (windef.h / winnt.h)
// ---------------------------------------------------------------------------
pub type LPBYTE = *mut BYTE;
pub type LPCSTR = *const c_char;
pub type PCHAR = *mut c_char;
pub type LPSTATDATA = *mut STATDATA;

/// wtypes.h `tagDVASPECT`. A C enum, so it is an `int` — NOT a DWORD. The DVASPECT_*
/// values themselves are harvested into `win_consts`.
pub type DVASPECT = c_int;

// ---------------------------------------------------------------------------
// ole2.h — the OLE 1.0 stream-conversion callbacks (OleConvertOLESTREAMToIStorage).
// ---------------------------------------------------------------------------
pub type LPOLESTREAM = *mut OLESTREAM;

/// ole2.h `_OLESTREAMVTBL`. `CALLBACK` is __stdcall on i386, so these are `extern "system"`
/// function pointers — `extern "C"` here would link on amd64/arm64 and take the wrong
/// stack discipline on i386.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct OLESTREAMVTBL {
    pub Get: Option<unsafe extern "system" fn(LPOLESTREAM, LPSTR, DWORD) -> DWORD>,
    pub Put: Option<unsafe extern "system" fn(LPOLESTREAM, LPSTR, DWORD) -> DWORD>,
}
pub type LPOLESTREAMVTBL = *mut OLESTREAMVTBL;

/// ole2.h `_OLESTREAM`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct OLESTREAM {
    pub lpstbl: LPOLESTREAMVTBL,
}

// ---------------------------------------------------------------------------
// propidl.h — the on-disk property value.
// ---------------------------------------------------------------------------
/// propidl.h `SERIALIZEDPROPERTYVALUE`. WIRE FORMAT: a type tag followed by the value's
/// bytes. `rgb: [BYTE; 1]` is the C's flexible-array idiom — real buffers are longer and
/// are indexed past the end, so never take `size_of` of this as the payload size.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SERIALIZEDPROPERTYVALUE {
    pub dwType: DWORD,
    pub rgb: [BYTE; 1],
}

// ---------------------------------------------------------------------------
// objbase.h — StgCreateStorageEx options.
// ---------------------------------------------------------------------------
/// objbase.h `tagSTGOPTIONS`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct STGOPTIONS {
    pub usVersion: USHORT,
    pub reserved: USHORT,
    pub ulSectorSize: ULONG,
    pub pwcsTemplateFile: *const WCHAR,
}
