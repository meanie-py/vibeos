//! ole32.dll's OWN exports, as seen from inside the DLL.
//!
//! The ring's C TUs called each other through ole32's public headers. Some of those
//! callees are now Rust (in `ring`, reachable by path), but many are still C TUs we
//! have not flipped (storage32, compobj's remaining surface, ...). Both kinds resolve
//! at ole32.dll's own link, so the honest declaration is an `extern` block — the
//! symbol exists in the same image either way.
//!
//! ABI: these are all WINAPI/STDAPI, i.e. __stdcall on i386 (`_Foo@N`). `extern "system"`
//! is the only correct spelling; `extern "C"` here links on amd64/arm64 and breaks i386.
//!
//! NOTE: a function that is ALREADY defined by a Rust ring module (CoTaskMemAlloc in
//! ifs.rs, CreateBindCtx in bindctx.rs, GetHGlobalFromStream in hglobalstream.rs, ...)
//! must NOT be redeclared here — it is in scope via the prelude's ring globs, and a
//! duplicate extern decl would be a conflicting definition. Only functions whose
//! definition lives OUTSIDE this crate's ring belong in this file.
#![allow(clippy::all)]
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
pub use crate::compobj::compobj::CoCreateInstance;
pub use crate::marshal::marshal::CoMarshalInterface;
pub use crate::marshal::marshal::CoReleaseMarshalData;
pub use crate::marshal::marshal::CoUnmarshalInterface;
use core::ffi::c_void;

extern "system" {
    // --- compobj.c (Rust, but #[no_mangle] exports — reachable as link symbols) ---
    pub fn CoInitializeEx(pvReserved: LPVOID, dwCoInit: DWORD) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoFileTimeNow(lpFileTime: *mut FILETIME) -> HRESULT;
    pub fn CLSIDFromString(lpsz: LPCOLESTR, pclsid: *mut CLSID) -> HRESULT;
    pub fn CLSIDFromProgID(lpszProgID: LPCOLESTR, lpclsid: *mut CLSID) -> HRESULT;
    pub fn StringFromGUID2(rguid: REFGUID, lpsz: LPOLESTR, cchMax: c_int) -> c_int;
    pub fn CoLockObjectExternal(pUnk: *mut IUnknown, fLock: BOOL, fLastUnlockReleases: BOOL) -> HRESULT;

    // --- marshal.c / stubmanager.c (Rust) ---
    pub fn CoGetMarshalSizeMax(
        pulSize: *mut ULONG,
        riid: REFIID,
        pUnk: *mut IUnknown,
        dwDestContext: DWORD,
        pvDestContext: LPVOID,
        mshlflags: DWORD,
    ) -> HRESULT;

}

// --- REMOVED, and here is why (do not "restore" them) ---------------------------
//
// StgOpenStorage, StgOpenStorageOnILockBytes, StgIsStorageFile, StgIsStorageILockBytes,
// ReadClassStg, WriteFmtUserTypeStg, ReadFmtUserTypeStg, SetConvertStg (storage32.rs),
// CreateILockBytesOnHGlobal (memlockbytes.rs), ReleaseStgMedium + OleLoad (ole2.rs),
// GetClassFile (moniker.rs), BindMoniker (bindctx.rs), CreateAntiMoniker (antimoniker.rs),
// CreateFileMoniker (filemoniker.rs), CreateGenericComposite (compositemoniker.rs)
//
// all used to be declared here, back when their TU was still C. Every one of them is now
// a real Rust DEFINITION in a `ring` module, and the prelude globs both this module and
// every ring module — so keeping the decl puts two different items with the same name in
// scope through two globs, which is an E0659 ambiguity the moment any module calls the
// name unqualified. (Eight of these were already ring definitions before Tier-D; the
// ambiguity was simply latent because no ring module happened to call them. Tier-D's
// datacache/oleobj call ReleaseStgMedium and stg_prop calls the Stg* family, so it fires.)
//
// The rule: a symbol belongs in this file ONLY while its definition lives outside the
// prelude's globs (compobj/, marshal/, or another DLL). Flip a TU into `ring` → delete
// its decl here.
// --------------------------------------------------------------------------------

// oleaut32 exports the ring reaches for (SAFEARRAY in the NDR user-marshal path, BSTRs).
// Cross-DLL, but the same stdcall rule applies.
extern "system" {
    pub fn SafeArrayCopy(psa: *mut SAFEARRAY, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT;
    pub fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
    pub fn SysAllocString(psz: *const u16) -> BSTR;
    pub fn SysAllocStringLen(strIn: *const u16, ui: UINT) -> BSTR;
    pub fn SysFreeString(bstrString: BSTR);
    pub fn SysStringLen(bstrString: BSTR) -> UINT;
    pub fn SysStringByteLen(bstrString: BSTR) -> UINT;
}

/// `IsValidInterface` is ole32-internal (ifs.c) but is NOT one of the DLL's exports —
/// it is a plain C function other TUs call. It is now defined in `ring::ifs`, so it
/// reaches call sites through the prelude's ring glob; declaring it here too would
/// conflict. Left as a note so nobody "helpfully" adds it.
const _: () = ();

/// FIELD_OFFSET(type, field) — the C macro. Always use `core::mem::offset_of!` at call
/// sites instead; a hand-written numeric offset is the magic-offset bug class that
/// `tools/scan_magic_offsets.py` exists to catch.
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {
        core::mem::offset_of!($t, $f) as isize
    };
}

/// basetsd.h: PtrToUlong / HandleToULong — truncate a pointer to 32 bits.
#[inline]
pub fn PtrToUlong(p: *const c_void) -> ULONG {
    p as usize as ULONG
}
#[inline]
pub fn HandleToULong(h: HANDLE) -> ULONG {
    h as usize as ULONG
}

/// basetsd.h: `UlongToPtr` — widen a 32-bit value back to a pointer. Zero-extends, per
/// the C macro (`(void *)(ULONG_PTR)(ULONG)ul`); it must NOT sign-extend on amd64/arm64.
#[inline]
pub fn UlongToPtr(ul: ULONG) -> *mut c_void {
    ul as usize as *mut c_void
}
