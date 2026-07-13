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
use core::ffi::c_void;

extern "system" {
    // --- compobj.c (Rust, but #[no_mangle] exports — reachable as link symbols) ---
    pub fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: *mut IUnknown,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
    pub fn CoInitializeEx(pvReserved: LPVOID, dwCoInit: DWORD) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoFileTimeNow(lpFileTime: *mut FILETIME) -> HRESULT;
    pub fn CLSIDFromString(lpsz: LPCOLESTR, pclsid: *mut CLSID) -> HRESULT;
    pub fn CLSIDFromProgID(lpszProgID: LPCOLESTR, lpclsid: *mut CLSID) -> HRESULT;
    pub fn StringFromGUID2(rguid: REFGUID, lpsz: LPOLESTR, cchMax: c_int) -> c_int;
    pub fn CoLockObjectExternal(pUnk: *mut IUnknown, fLock: BOOL, fLastUnlockReleases: BOOL) -> HRESULT;

    // --- marshal.c / stubmanager.c (Rust) ---
    pub fn CoMarshalInterface(
        pStm: *mut IStream,
        riid: REFIID,
        pUnk: *mut IUnknown,
        dwDestContext: DWORD,
        pvDestContext: LPVOID,
        mshlflags: DWORD,
    ) -> HRESULT;
    pub fn CoUnmarshalInterface(pStm: *mut IStream, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    pub fn CoReleaseMarshalData(pStm: *mut IStream) -> HRESULT;
    pub fn CoGetMarshalSizeMax(
        pulSize: *mut ULONG,
        riid: REFIID,
        pUnk: *mut IUnknown,
        dwDestContext: DWORD,
        pvDestContext: LPVOID,
        mshlflags: DWORD,
    ) -> HRESULT;

    // --- storage32.c / stg_stream.c (STILL C — not yet translated) ---
    pub fn StgOpenStorage(
        pwcsName: *const u16,
        pstgPriority: *mut IStorage,
        grfMode: DWORD,
        snbExclude: *mut *mut u16,
        reserved: DWORD,
        ppstgOpen: *mut *mut IStorage,
    ) -> HRESULT;
    pub fn StgOpenStorageOnILockBytes(
        plkbyt: *mut ILockBytes,
        pstgPriority: *mut IStorage,
        grfMode: DWORD,
        snbExclude: *mut *mut u16,
        reserved: DWORD,
        ppstgOpen: *mut *mut IStorage,
    ) -> HRESULT;
    pub fn StgIsStorageFile(fileName: *const u16) -> HRESULT;
    pub fn StgIsStorageILockBytes(plkbyt: *mut ILockBytes) -> HRESULT;
    pub fn ReadClassStg(pStg: *mut IStorage, pclsid: *mut CLSID) -> HRESULT;
    pub fn WriteFmtUserTypeStg(pstg: *mut IStorage, cf: CLIPFORMAT, lpszUserType: LPOLESTR) -> HRESULT;
    pub fn ReadFmtUserTypeStg(pstg: *mut IStorage, pcf: *mut CLIPFORMAT, lplpszUserType: *mut LPOLESTR) -> HRESULT;
    pub fn SetConvertStg(pStg: *mut IStorage, fConvert: BOOL) -> HRESULT;
    pub fn CreateILockBytesOnHGlobal(
        hGlobal: HGLOBAL,
        fDeleteOnRelease: BOOL,
        pplkbyt: *mut *mut ILockBytes,
    ) -> HRESULT;

    // --- ole2.c / datacache / defaulthandler (still C where not yet flipped) ---
    pub fn ReleaseStgMedium(pmedium: *mut STGMEDIUM);
    pub fn OleLoad(
        pStg: *mut IStorage,
        riid: REFIID,
        pClientSite: *mut IOleClientSite,
        ppvObj: *mut LPVOID,
    ) -> HRESULT;
    pub fn GetClassFile(filePathName: *const u16, pclsid: *mut CLSID) -> HRESULT;

    // --- moniker.c family (declared for the cross-moniker calls) ---
    pub fn BindMoniker(pmk: *mut IMoniker, grfOpt: DWORD, riid: REFIID, ppvResult: *mut LPVOID) -> HRESULT;
    pub fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT;
    pub fn CreateFileMoniker(lpszPathName: LPCOLESTR, ppmk: *mut *mut IMoniker) -> HRESULT;
    pub fn CreateGenericComposite(
        pmkFirst: *mut IMoniker,
        pmkRest: *mut IMoniker,
        ppmkComposite: *mut *mut IMoniker,
    ) -> HRESULT;
}

// oleaut32 exports the ring reaches for (SAFEARRAY in the NDR user-marshal path, BSTRs).
// Cross-DLL, but the same stdcall rule applies.
extern "system" {
    pub fn SafeArrayCopy(psa: *mut SAFEARRAY, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT;
    pub fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
    pub fn SysAllocString(psz: *const u16) -> BSTR;
    pub fn SysAllocStringLen(strIn: *const u16, ui: UINT) -> BSTR;
    pub fn SysFreeString(bstrString: BSTR);
    pub fn SysStringLen(bstrString: BSTR) -> UINT;
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
