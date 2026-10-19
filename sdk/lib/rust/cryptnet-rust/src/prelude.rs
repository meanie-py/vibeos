//! cryptnet-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME): wine's own flag test on the default
// channel, then ros_dbg_log -- win32macros-rust's WINE_*! macros, which expand HERE (the
// channel storage and the ros_dbg_log extern are this crate's; `port resolve` writes them).
// MACROS, never out-of-line fns: an out-of-line wrapper is an optimization barrier that
// uncloaks latent ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => { win32macros_rust::WINE_TRACE!($($arg)*) }; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { win32macros_rust::WINE_ERR!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => { win32macros_rust::WINE_WARN!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { win32macros_rust::WINE_FIXME!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const CERT_E_REVOKED: HRESULT = ((0x800B010Cu32) as HRESULT); // gen:52a09ed0c6b3
pub const CERT_STORE_PROV_COLLECTION: LPCSTR = 0xbusize as LPCSTR; // gen:ed4d0472e268
pub const CERT_STORE_PROV_LDAP_W: LPCSTR = 0x10usize as LPCSTR; // gen:70300818946e
pub const CERT_STORE_PROV_MEMORY: LPCSTR = 0x2usize as LPCSTR; // gen:af4d0ce92919
pub const CONTEXT_OID_CERTIFICATE: LPCSTR = 0x1usize as LPCSTR; // gen:2fd43d591506
pub const CONTEXT_OID_CRL: LPCSTR = 0x2usize as LPCSTR; // gen:019f84b47e0c
pub const CONTEXT_OID_CTL: LPCSTR = 0x3usize as LPCSTR; // gen:1d17c29a768b
pub const CRYPT_E_NOT_FOUND: HRESULT = ((0x80092004u32) as HRESULT); // gen:fcc8d64692de
pub const CRYPT_E_NO_MATCH: HRESULT = ((0x80092009u32) as HRESULT); // gen:7149557b6431
pub const CRYPT_E_NO_REVOCATION_CHECK: HRESULT = ((0x80092012u32) as HRESULT); // gen:64c80c941401
pub const CRYPT_E_REVOCATION_OFFLINE: HRESULT = ((0x80092013u32) as HRESULT); // gen:0d1b01647eb6
pub const CRYPT_E_REVOKED: HRESULT = ((0x80092010u32) as HRESULT); // gen:b3e9021df38a
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const OCSP_BASIC_RESPONSE: LPCSTR = 0x45usize as LPCSTR; // gen:bfa8ec2f1900
pub const OCSP_BASIC_SIGNED_RESPONSE: LPCSTR = 0x44usize as LPCSTR; // gen:ef57b29ec41c
pub const OCSP_REQUEST: LPCSTR = 0x42usize as LPCSTR; // gen:9cf14754889c
pub const OCSP_RESPONSE: LPCSTR = 0x43usize as LPCSTR; // gen:69e0a192ff5c
pub const OCSP_SIGNED_REQUEST: LPCSTR = 0x41usize as LPCSTR; // gen:39e3d6354562
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const X509_AUTHORITY_INFO_ACCESS: LPCSTR = 0x20usize as LPCSTR; // gen:3f92d7621674
pub const X509_BITS: LPCSTR = 0x1ausize as LPCSTR; // gen:12f233ac10b5
pub const X509_CRL_DIST_POINTS: LPCSTR = 0x23usize as LPCSTR; // gen:05a0e8660872
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/cryptnet/cryptnet_main.c:45: #define IS_INTOID(x) (((ULONG_PTR)(x) >> 16) == 0) // gen:de47301f4638
#[allow(unused_macros)]
macro_rules! IS_INTOID {
    ($x:expr $(,)?) => {
        (((($x) as usize) >> 16) == 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_INTOID;
// dll/win32/cryptnet/cryptnet_main.c:49: #define SHGetKnownFolderPath(a, b, c, out) CryptnetSHGetKnownFolderPath(out) // gen:bf34bbc5f0a6
#[allow(unused_macros)]
macro_rules! SHGetKnownFolderPath {
    ($a:expr, $b:expr, $c:expr, $out:expr $(,)?) => {
        (CryptnetSHGetKnownFolderPath($out))
    };
}
#[allow(unused_imports)]
pub(crate) use SHGetKnownFolderPath;
// ==== end port macros ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const INTERNET_SCHEME_DEFAULT: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_DEFAULT; // island enum
pub const INTERNET_SCHEME_FILE: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_FILE; // island enum
pub const INTERNET_SCHEME_FTP: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_FTP; // island enum
pub const INTERNET_SCHEME_HTTP: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_HTTP; // island enum
pub const INTERNET_SCHEME_HTTPS: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_HTTPS; // island enum
pub const SHGFP_TYPE_CURRENT: SHGFP_TYPE = SHGFP_TYPE_SHGFP_TYPE_CURRENT; // island enum
pub const _SH_DENYRW: u32 = 16; // msvcrt-sys/src/bindgen_amd64.rs
pub const _SH_DENYWR: u32 = 32; // msvcrt-sys/src/bindgen_amd64.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs
extern "C" {
    pub fn fclose(stream: *mut FILE) -> c_int; // cmd-rust/src/prelude.rs
    pub fn fread(ptr: *mut core::ffi::c_void, size: usize, nmemb: usize, file: *mut FILE,) -> usize; // comp-rust/src/prelude.rs
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs
    pub fn fwrite(buffer: *const c_void, size: usize, count: usize, stream: *mut c_void) -> usize; // regedit-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CertAddStoreToCollection( hCollectionStore: *mut ::core::ffi::c_void, hSiblingStore: *mut ::core::ffi::c_void, dwUpdateFlags: DWORD, dwPriority: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CertCloseStore(hCertStore: *mut ::core::ffi::c_void, dwFlags: DWORD) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CertFindCRLInStore( hCertStore: *mut ::core::ffi::c_void, dwCertEncodingType: DWORD, dwFindFlags: DWORD, dwFindType: DWORD, pvFindPara: *const ::core::ffi::c_void, pPrevCrlContext: PCCRL_CONTEXT, ) -> PCCRL_CONTEXT; // certmgr-rust/src/types_i386.rs
    pub fn CertFindCertificateInCRL( pCert: PCCERT_CONTEXT, pCrlContext: PCCRL_CONTEXT, dwFlags: DWORD, pvReserved: *mut ::core::ffi::c_void, ppCrlEntry: *mut PCRL_ENTRY, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CertFindExtension( pszObjId: LPCSTR, cExtensions: DWORD, rgExtensions: *mut CERT_EXTENSION, ) -> PCERT_EXTENSION; // certmgr-rust/src/types_i386.rs
    pub fn CertFreeCRLContext(pCrlContext: PCCRL_CONTEXT) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CertFreeCTLContext(pCtlContext: PCCTL_CONTEXT) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CertFreeCertificateContext(pCertContext: PCCERT_CONTEXT) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CertOIDToAlgId(pszObjId: LPCSTR) -> DWORD; // certmgr-rust/src/types_i386.rs
    pub fn CertOpenStore( lpszStoreProvider: LPCSTR, dwEncodingType: DWORD, hCryptProv: HCRYPTPROV_LEGACY, dwFlags: DWORD, pvPara: *const ::core::ffi::c_void, ) -> HCERTSTORE; // certmgr-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CommitUrlCacheEntryW( lpszUrlName: LPCWSTR, lpszLocalFileName: LPCWSTR, ExpireTime: FILETIME, LastModifiedTime: FILETIME, CacheEntryType: u32, lpszHeaderInfo: LPWSTR, cchHeaderInfo: u32, lpszFileExtension: LPCWSTR, lpszOriginalUrl: LPCWSTR, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn CompareFileTime(arg1: *const FILETIME, arg2: *const FILETIME) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateUrlCacheEntryW( lpszUrlName: LPCWSTR, dwExpectedFileSize: u32, lpszFileExtension: LPCWSTR, lpszFileName: LPWSTR, dwReserved: u32, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn CryptAcquireContextW( arg1: *mut HCRYPTPROV, arg2: LPCWSTR, arg3: LPCWSTR, arg4: DWORD, arg5: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptBinaryToStringW( pbBinary: *const BYTE, cbBinary: DWORD, dwFlags: DWORD, pszString: LPWSTR, pcchString: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptCreateHash( arg1: HCRYPTPROV, arg2: ::core::ffi::c_uint, arg3: HCRYPTKEY, arg4: DWORD, arg5: *mut HCRYPTHASH, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptDecodeObjectEx( dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pDecodePara: PCRYPT_DECODE_PARA, pvStructInfo: *mut ::core::ffi::c_void, pcbStructInfo: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptDestroyHash(arg1: HCRYPTHASH) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptDestroyKey(arg1: HCRYPTKEY) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptEncodeObjectEx( dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const ::core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pvEncoded: *mut ::core::ffi::c_void, pcbEncoded: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptFreeOIDFunctionAddress(hFuncAddr: *mut ::core::ffi::c_void, dwFlags: DWORD) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptGetHashParam( hHash: HCRYPTHASH, dwParam: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptGetOIDFunctionAddress( hFuncSet: *mut ::core::ffi::c_void, dwEncodingType: DWORD, pszOID: LPCSTR, dwFlags: DWORD, ppvFuncAddr: *mut *mut ::core::ffi::c_void, phFuncAddr: *mut HCRYPTOIDFUNCADDR, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptHashData( hHash: HCRYPTHASH, pbData: *const BYTE, dwDataLen: DWORD, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptImportPublicKeyInfoEx( hCryptProv: HCRYPTPROV, dwCertEncodingType: DWORD, pInfo: PCERT_PUBLIC_KEY_INFO, aiKeyAlg: ::core::ffi::c_uint, dwFlags: DWORD, pvAuxInfo: *mut ::core::ffi::c_void, phKey: *mut HCRYPTKEY, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptInitOIDFunctionSet(arg1: LPCSTR, arg2: DWORD) -> HCRYPTOIDFUNCSET; // certmgr-rust/src/types_i386.rs
    pub fn CryptMemAlloc(cbSize: ULONG) -> LPVOID; // certmgr-rust/src/types_i386.rs
    pub fn CryptMemFree(pv: LPVOID); // certmgr-rust/src/types_i386.rs
    pub fn CryptMemRealloc(pv: LPVOID, cbSize: ULONG) -> LPVOID; // certmgr-rust/src/types_i386.rs
    pub fn CryptQueryObject( dwObjectType: DWORD, pvObject: *const ::core::ffi::c_void, dwExpectedContentTypeFlags: DWORD, dwExpectedFormatTypeFlags: DWORD, dwFlags: DWORD, pdwMsgAndCertEncodingType: *mut DWORD, pdwContentType: *mut DWORD, pdwFormatType: *mut DWORD, phCertStore: *mut HCERTSTORE, phMsg: *mut HCRYPTMSG, ppvContext: *mut *const ::core::ffi::c_void, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptRegisterDefaultOIDFunction( arg1: DWORD, arg2: LPCSTR, arg3: DWORD, arg4: LPCWSTR, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptRegisterOIDFunction( arg1: DWORD, arg2: LPCSTR, arg3: LPCSTR, arg4: LPCWSTR, arg5: LPCSTR, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptReleaseContext(arg1: HCRYPTPROV, arg2: DWORD) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptStringToBinaryA( pszString: LPCSTR, cchString: DWORD, dwFlags: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptUnregisterDefaultOIDFunction(arg1: DWORD, arg2: LPCSTR, arg3: LPCWSTR) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptUnregisterOIDFunction(arg1: DWORD, arg2: LPCSTR, arg3: LPCSTR) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn CryptVerifySignatureW( hHash: HCRYPTHASH, pbSignature: *const BYTE, dwSigLen: DWORD, hPubKey: HCRYPTKEY, szDescription: LPCWSTR, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs
    pub fn DeleteUrlCacheEntryW(arg1: LPCWSTR) -> BOOL; // rapps-rust/src/types_i386.rs
    pub fn GetCurrentDirectoryW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileSizeEx(arg1: HANDLE, arg2: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSystemTimeAsFileTime(arg1: LPFILETIME); // aclui-rust/src/types_i386.rs
    pub fn GetUrlCacheEntryInfoW( lpszUrlName: LPCWSTR, lpCacheEntryInfo: LPINTERNET_CACHE_ENTRY_INFOW, lpcbCacheEntryInfo: LPDWORD, ) -> BOOL; // rapps-rust/src/types_i386.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn HttpEndRequestW( arg1: HINTERNET, arg2: LPINTERNET_BUFFERSW, arg3: u32, arg4: DWORD_PTR, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn HttpOpenRequestW( hConnect: HINTERNET, lpszVerb: LPCWSTR, lpszObjectName: LPCWSTR, lpszVersion: LPCWSTR, lpszReferrer: LPCWSTR, lplpszAcceptTypes: *mut LPCWSTR, dwFlags: u32, dwContext: DWORD_PTR, ) -> HINTERNET; // shell32-rust/src/island_i386.rs
    pub fn HttpQueryInfoW( hRequest: HINTERNET, dwInfoLevel: u32, lpBuffer: *mut c_void, lpdwBufferLength: LPDWORD, lpdwIndex: LPDWORD, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn HttpSendRequestExW( arg1: HINTERNET, arg2: LPINTERNET_BUFFERSW, arg3: LPINTERNET_BUFFERSW, arg4: u32, arg5: DWORD_PTR, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn HttpSendRequestW( hRequest: HINTERNET, lpszHeaders: LPCWSTR, dwHeadersLength: u32, lpOptional: *mut c_void, dwOptionalLength: u32, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn InternetCloseHandle(arg1: HINTERNET) -> BOOL; // rapps-rust/src/types_i386.rs
    pub fn InternetConnectW( arg1: HINTERNET, arg2: LPCWSTR, arg3: INTERNET_PORT, arg4: LPCWSTR, arg5: LPCWSTR, arg6: u32, arg7: u32, arg8: DWORD_PTR, ) -> HINTERNET; // shell32-rust/src/island_i386.rs
    pub fn InternetCrackUrlW( lpszUrl: LPCWSTR, dwUrlLength: u32, dwFlags: u32, lpUrlComponents: LPURL_COMPONENTSW, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn InternetOpenW( arg1: LPCWSTR, arg2: u32, arg3: LPCWSTR, arg4: LPCWSTR, arg5: u32, ) -> HINTERNET; // shell32-rust/src/island_i386.rs
    pub fn InternetQueryDataAvailable( hFile: HINTERNET, lpdwNumberOfBytesAvailable: LPDWORD, dwFlags: u32, dwContext: DWORD_PTR, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn InternetReadFile( hFile: HINTERNET, lpBuffer: *mut c_void, dwNumberOfBytesToRead: u32, lpdwNumberOfBytesRead: LPDWORD, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn InternetReadFileExA( hFile: HINTERNET, lpBuffersOut: LPINTERNET_BUFFERSA, dwFlags: u32, dwContext: DWORD_PTR, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn InternetSetOptionW(arg1: HINTERNET, arg2: u32, arg3: *mut c_void, arg4: u32) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn InternetSetStatusCallbackW( arg1: HINTERNET, arg2: INTERNET_STATUS_CALLBACK, ) -> INTERNET_STATUS_CALLBACK; // rapps-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RetrieveUrlCacheEntryFileW( lpszUrlName: LPCWSTR, lpCacheEntryInfo: LPINTERNET_CACHE_ENTRY_INFOW, lpcbCacheEntryInfo: LPDWORD, dwReserved: u32, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn SHCreateDirectoryExW( arg1: HWND, arg2: LPCWSTR, arg3: LPSECURITY_ATTRIBUTES, ) -> ::core::ffi::c_int; // acppage-rust/src/types_i386.rs
    pub fn SHGetFolderPathW( hwnd: HWND, nFolder: ::core::ffi::c_int, hToken: HANDLE, dwFlags: DWORD, pszPath: LPWSTR, ) -> HRESULT; // acppage-rust/src/types_i386.rs
    pub fn SHStrDupW(psz: LPCWSTR, ppwsz: *mut *mut WCHAR) -> HRESULT; // acppage-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SetUrlCacheEntryInfoW( arg1: LPCWSTR, arg2: LPINTERNET_CACHE_ENTRY_INFOW, arg3: u32, ) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnlockUrlCacheEntryFileW(arg1: LPCWSTR, arg2: u32) -> BOOL; // shell32-rust/src/island_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CERT_REVOCATION_PARA_NO_EXTRA_FIELDS: c2rust, cryptnet_main.c
pub type CERT_REVOCATION_PARA_NO_EXTRA_FIELDS = _CERT_REVOCATION_PARA_NO_EXTRA_FIELDS;
// _CERT_REVOCATION_PARA_NO_EXTRA_FIELDS: c2rust, cryptnet_main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CERT_REVOCATION_PARA_NO_EXTRA_FIELDS {
    pub cbSize: DWORD,
    pub pIssuerCert: PCCERT_CONTEXT,
    pub cCertStore: DWORD,
    pub rgCertStore: *mut HCERTSTORE,
    pub hCrlStore: HCERTSTORE,
    pub pftTimeToUse: LPFILETIME,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Count: size_t,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // cryptnet_main.c
    pub fn _wfsopen(
        _Filename: *const wchar_t,
        _Mode: *const wchar_t,
        _ShFlag: ::core::ffi::c_int,
    ) -> *mut FILE; // cryptnet_main.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // cryptnet_main.c
}
pub use crate::prelude::_snprintf as snprintf; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_cryptnet: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x63\x72\x79\x70\x74\x6e\x65\x74\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_cryptnet };
// ==== end wine debug channels ====
