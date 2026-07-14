//! Type aliases and constants the ring translations reference as bare names: the
//! COM `LPxxx`/`REFxxx` spellings the C headers use, ole32's own private typedefs
//! (dictionary.h / enumx.h), and the handful of Win32 constants the ring needs.
//!
//! HRESULTs live in hresults.rs (generated from winerror.h). Values here are cited
//! to their defining header — `verify_constants.py` re-checks them in the lint gate.
#![allow(clippy::all)]
use crate::prelude::*;
use core::ffi::c_void;

// ---- COM pointer/reference aliases (objbase.h / wtypes.h spellings) ----
// The C uses these constantly; com-sys emits the underlying structs but not the
// LP*/REF* typedef spellings.
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;

pub type LPUNKNOWN = *mut IUnknown;
pub type LPMALLOC = *mut IMalloc;
pub type LPMALLOCSPY = *mut IMallocSpy;
pub type LPCLASSFACTORY = *mut IClassFactory;
pub type LPMONIKER = *mut IMoniker;
pub type LPENUMMONIKER = *mut IEnumMoniker;
pub type LPRUNNINGOBJECTTABLE = *mut IRunningObjectTable;
pub type LPPERSIST = *mut IPersist;
pub type LPPERSISTSTREAM = *mut IPersistStream;
pub type LPSTORAGE = *mut IStorage;
pub type LPADVISESINK = *mut IAdviseSink;
pub type LPFORMATETC = *mut FORMATETC;
pub type LPOLECLIENTSITE = *mut IOleClientSite;
pub type LPOLEOBJECT = *mut IOleObject;
pub type LPENUMFORMATETC = *mut IEnumFORMATETC;
pub type LPENUMSTATDATA = *mut IEnumSTATDATA;
pub type LPENUMUNKNOWN = *mut IEnumUnknown;
pub type LPENUMSTRING = *mut IEnumString;
pub type LPSTGMEDIUM = *mut STGMEDIUM;
pub type LPERRORINFO = *mut IErrorInfo;

// ---- ole32 private typedefs ----

// These four are C function POINTERS, which are nullable — the sources test them
// (`if (d->destroy)`) and call them. The faithful Rust spelling is `Option<fn>`
// (same ABI, niche-optimized to a bare pointer), which is also what bindgen emits
// for a C fn pointer. A bare `fn` type would be non-nullable and make `.is_some()`
// / a NULL assignment impossible.

/// dictionary.h:35 — `int (*comparefunc)(const void *a, const void *b, void *extra)`
pub type comparefunc = Option<
    unsafe extern "C" fn(a: *const c_void, b: *const c_void, extra: *mut c_void) -> c_int,
>;

/// dictionary.h:40 — `void (*destroyfunc)(void *k, void *v, void *extra)`
pub type destroyfunc =
    Option<unsafe extern "C" fn(k: *mut c_void, v: *mut c_void, extra: *mut c_void)>;

/// dictionary.h:45 — `BOOL (*enumeratefunc)(const void *k, const void *v, void *extra, void *closure)`
pub type enumeratefunc = Option<
    unsafe extern "C" fn(
        k: *const c_void,
        v: *const c_void,
        extra: *mut c_void,
        closure: *mut c_void,
    ) -> BOOL,
>;

/// enumx.h:24 — `void (*enumx_copy_cb)(IUnknown *parent, void *orig, void *dest)`
pub type enumx_copy_cb =
    Option<unsafe extern "C" fn(parent: *mut IUnknown, orig: *mut c_void, dest: *mut c_void)>;

// ---- wingdi.h metafile / mapping-mode constants (the clipboard + ole2 paths) ----
/// wingdi.h: metafile record comment escape
pub const MFCOMMENT: c_int = 15;
/// wingdi.h mapping modes
pub const MM_TEXT: c_int = 1;
pub const MM_LOMETRIC: c_int = 2;
pub const MM_HIMETRIC: c_int = 3;
pub const MM_LOENGLISH: c_int = 4;
pub const MM_HIENGLISH: c_int = 5;
pub const MM_TWIPS: c_int = 6;
pub const MM_ISOTROPIC: c_int = 7;
pub const MM_ANISOTROPIC: c_int = 8;

/// winnls.h: CP_ACP — the system ANSI code page
pub const CP_ACP: u32 = 0;

// ---- objidl.h STREAM_SEEK / STGTY / LOCKTYPE ----
pub const STREAM_SEEK_SET: DWORD = 0;
pub const STREAM_SEEK_CUR: DWORD = 1;
pub const STREAM_SEEK_END: DWORD = 2;

pub const STGTY_STORAGE: DWORD = 1;
pub const STGTY_STREAM: DWORD = 2;
pub const STGTY_LOCKBYTES: DWORD = 3;
pub const STGTY_PROPERTY: DWORD = 4;

// ---- objbase.h ROT / bind flags ----
pub const ROTFLAGS_REGISTRATIONKEEPSALIVE: DWORD = 1;
pub const ROTFLAGS_ALLOWANYCLIENT: DWORD = 2;

// ---- objbase.h STGM_* (storage mode flags) ----
pub const STGM_READ: DWORD = 0x0000_0000;
pub const STGM_WRITE: DWORD = 0x0000_0001;
pub const STGM_READWRITE: DWORD = 0x0000_0002;
pub const STGM_SHARE_DENY_NONE: DWORD = 0x0000_0040;
pub const STGM_SHARE_EXCLUSIVE: DWORD = 0x0000_0010;
pub const STGM_CREATE: DWORD = 0x0000_1000;

// ---- wtypesbase.h CLSCTX_* ----
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;
pub const CLSCTX_INPROC_HANDLER: DWORD = 0x2;
pub const CLSCTX_LOCAL_SERVER: DWORD = 0x4;
pub const CLSCTX_REMOTE_SERVER: DWORD = 0x10;
/// objbase.h: CLSCTX_SERVER = INPROC_SERVER | LOCAL_SERVER | REMOTE_SERVER
pub const CLSCTX_SERVER: DWORD =
    CLSCTX_INPROC_SERVER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER;
/// objbase.h: CLSCTX_ALL = CLSCTX_SERVER | CLSCTX_INPROC_HANDLER
pub const CLSCTX_ALL: DWORD = CLSCTX_SERVER | CLSCTX_INPROC_HANDLER;

// ---- objidl.h MEMCTX ----
pub const MEMCTX_TASK: DWORD = 1;
pub const MEMCTX_SHARED: DWORD = 2;
