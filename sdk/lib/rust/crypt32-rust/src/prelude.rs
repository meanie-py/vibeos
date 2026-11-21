//! crypt32-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
// <stdint.h>/<stddef.h> names: no island allowlists those headers (ddraw used 26 of them).
// Fixed-width, so identical on every target; an island that does carry one shadows nothing
// (a glob import never conflicts with an item the prelude defines).
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type ptrdiff_t = isize;
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
// A Wine module's ReactOS-only TUs use reactos/debug.h too (crypt32's unixlib.c:
// `UNIMPLEMENTED;`), so the Wine prelude carries the ReactOS no-ops as well.
#[macro_export] macro_rules! DPRINT {
    ($($arg:tt)*) => { ::win32macros_rust::DBG_TYPECHECK!($($arg)*) };
}
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// <wchar.h> wmemchr: clang expands it as a builtin, so no CRT this tree links exports it
// (the C crypt32.dll imports none) -- chain.c's match_common_name calls it.
#[inline]
pub unsafe fn wmemchr(s: *const WCHAR, c: WCHAR, n: size_t) -> *mut WCHAR {
    let mut i = 0;
    while i < n {
        if *s.add(i) == c {
            return s.add(i) as *mut WCHAR;
        }
        i += 1;
    }
    core::ptr::null_mut()
}

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_TYPECHECK, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlSecureZeroMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, SecureZeroMemory, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_ERR_, WINE_FIXME, WINE_FIXME_, WINE_TRACE, WINE_TRACE_, WINE_WARN, WINE_WARN_, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_fourcc, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_fourcc, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_rect, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;
// winreg.h's predefined keys: `(HKEY)(ULONG_PTR)((LONG)0x8000000x)`, sign-extended on 64-bit.
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs // gen:2cf5bd2da631
pub const StoreTypeCollection: _CertStoreType = _CertStoreType_StoreTypeCollection; // island enum // gen:c7c0fbdc68ce
pub const StoreTypeEmpty: _CertStoreType = _CertStoreType_StoreTypeEmpty; // island enum // gen:ef3141915ccc
pub const StoreTypeMem: _CertStoreType = _CertStoreType_StoreTypeMem; // island enum // gen:704eb49d2e8e
pub const StoreTypeProvider: _CertStoreType = _CertStoreType_StoreTypeProvider; // island enum // gen:b5e9cd1148e8
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs // gen:845fcd2ce5ae
extern "C" {
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs // gen:0ad441c3c230
    pub fn isspace(c: i32) -> i32; // cnt-ctype/src/lib.rs // gen:bd320cdfba63
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:86a212c6fd14
    pub fn memchr(s: *const c_void, c: i32, n: usize) -> *mut c_void; // cmd-rust/src/prelude.rs // gen:40376f9ab53d
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // cmd-rust/src/prelude.rs // gen:038a9bc4bb3d
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs // gen:ce18447a16eb
    pub fn sscanf(s: *const i8, fmt: *const i8, ...) -> i32; // arp-rust/src/prelude.rs // gen:449a48721d25
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs // gen:1be472052f3d
    pub fn towlower(c: u16) -> u16; // cleanmgr-rust/src/prelude.rs // gen:fc55c20856b2
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs // gen:6f1e26b8a292
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs // gen:13391f57321c
    pub fn wcsicmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs // gen:d9102022f7d3
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs // gen:1246e81e5c62
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // apphelp-rust/src/prelude.rs // gen:12a747e973fa
    pub fn wcsnicmp(a: *const wchar_t, b: *const wchar_t, n: usize) -> c_int; // crt-shared-rust/src/prelude.rs // gen:1a5324f4d6fe
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // appwiz-rust/src/prelude.rs // gen:27e3f6f3f416
    pub fn wine_dbg_sprintf(format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char; // winegdi-rust/src/types_i386.rs // gen:64f2d13938b7
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:4bffac11aa02
}
extern "stdcall" {
    pub fn InitializeCriticalSectionEx( lpCriticalSection: LPCRITICAL_SECTION, dwSpinCount: u32, Flags: u32, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs // gen:90fdae6c8e87
}
extern "system" {
    pub fn CharNextA(arg1: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:37a4868db9f3
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4a43278b42fd
    pub fn CompareFileTime(arg1: *const FILETIME, arg2: *const FILETIME) -> LONG; // aclui-rust/src/types_i386.rs // gen:01018ee28ba8
    pub fn CreateFileA( arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:1313a31ff51f
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:90b2fdf62411
    pub fn CreateSemaphoreW( arg1: LPSECURITY_ATTRIBUTES, arg2: LONG, arg3: LONG, arg4: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:a8f0a29b02d6
    pub fn CryptAcquireContextA( arg1: *mut HCRYPTPROV, arg2: LPCSTR, arg3: LPCSTR, arg4: DWORD, arg5: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:e3ad1b548109
    pub fn CryptAcquireContextW( arg1: *mut HCRYPTPROV, arg2: LPCWSTR, arg3: LPCWSTR, arg4: DWORD, arg5: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:0ceba5272b81
    pub fn CryptContextAddRef(arg1: HCRYPTPROV, arg2: *mut DWORD, arg3: DWORD) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:d7fb2ab9a6e7
    pub fn CryptCreateHash( arg1: HCRYPTPROV, arg2: ::core::ffi::c_uint, arg3: HCRYPTKEY, arg4: DWORD, arg5: *mut HCRYPTHASH, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:cfdff8be8212
    pub fn CryptDecrypt( hKey: HCRYPTKEY, hHash: HCRYPTHASH, Final: BOOL, dwFlags: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:e24e7fd359ad
    pub fn CryptDeriveKey( arg1: HCRYPTPROV, arg2: ::core::ffi::c_uint, arg3: HCRYPTHASH, arg4: DWORD, arg5: *mut HCRYPTKEY, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:aff2c415b08d
    pub fn CryptDestroyHash(arg1: HCRYPTHASH) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:6bdc3e064151
    pub fn CryptDestroyKey(arg1: HCRYPTKEY) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:7145457187d9
    pub fn CryptEncrypt( hKey: HCRYPTKEY, hHash: HCRYPTHASH, Final: BOOL, dwFlags: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, dwBufLen: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:03fe367f59fa
    pub fn CryptEnumProvidersW( dwIndex: DWORD, pdwReserved: *mut DWORD, dwFlags: DWORD, pdwProvType: *mut DWORD, szProvName: LPWSTR, pcbProvName: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:b68bd69d38c6
    pub fn CryptExportKey( hKey: HCRYPTKEY, hExpKey: HCRYPTKEY, dwBlobType: DWORD, dwFlags: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:b2cb641cd4aa
    pub fn CryptGenKey(arg1: HCRYPTPROV, arg2: ::core::ffi::c_uint, arg3: DWORD, arg4: *mut HCRYPTKEY) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:d93fa348d6bf
    pub fn CryptGenRandom(hProv: HCRYPTPROV, dwLen: DWORD, pbBuffer: *mut BYTE) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:367b3972278c
    pub fn CryptGetHashParam( hHash: HCRYPTHASH, dwParam: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:0314c57f294a
    pub fn CryptGetKeyParam( hKey: HCRYPTKEY, dwParam: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:6b2b59359850
    pub fn CryptGetObjectUrl( pszUrlOid: LPCSTR, pvPara: LPVOID, dwFlags: DWORD, pUrlArray: PCRYPT_URL_ARRAY, pcbUrlArray: *mut DWORD, pUrlInfo: PCRYPT_URL_INFO, pcbUrlInfo: *mut DWORD, pvReserved: LPVOID, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:1f26e28bf124
    pub fn CryptGetProvParam( hProv: HCRYPTPROV, dwParam: DWORD, pbData: *mut BYTE, pdwDataLen: *mut DWORD, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:aea2d4c743b1
    pub fn CryptGetUserKey(arg1: HCRYPTPROV, arg2: DWORD, arg3: *mut HCRYPTKEY) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:bdfd318510dd
    pub fn CryptHashData( hHash: HCRYPTHASH, pbData: *const BYTE, dwDataLen: DWORD, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:15d2f5711a54
    pub fn CryptImportKey( hProv: HCRYPTPROV, pbData: *const BYTE, dwDataLen: DWORD, hPubKey: HCRYPTKEY, dwFlags: DWORD, phKey: *mut HCRYPTKEY, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:01292d513a1c
    pub fn CryptReleaseContext(arg1: HCRYPTPROV, arg2: DWORD) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:0aaa90eb3672
    pub fn CryptRetrieveObjectByUrlW( pszUrl: LPCWSTR, pszObjectOid: LPCSTR, dwRetrievalFlags: DWORD, dwTimeout: DWORD, ppvObject: *mut LPVOID, hAsyncRetrieve: HCRYPTASYNC, pCredentials: PCRYPT_CREDENTIALS, pvVerify: LPVOID, pAuxInfo: PCRYPT_RETRIEVE_AUX_INFO, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:fdc300439b36
    pub fn CryptSetProvParam(arg1: HCRYPTPROV, arg2: DWORD, arg3: *const BYTE, arg4: DWORD) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:c7a06ad85705
    pub fn CryptSignHashW( hHash: HCRYPTHASH, dwKeySpec: DWORD, szDescription: LPCWSTR, dwFlags: DWORD, pbSignature: *mut BYTE, pdwSigLen: *mut DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:14b2c4b83f7f
    pub fn CryptVerifySignatureW( hHash: HCRYPTHASH, pbSignature: *const BYTE, dwSigLen: DWORD, hPubKey: HCRYPTKEY, szDescription: LPCWSTR, dwFlags: DWORD, ) -> BOOL; // certmgr-rust/src/types_i386.rs // gen:d774b49322bb
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:7893eb6ab4e4
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:111127023c7f
    pub fn DuplicateHandle( arg1: HANDLE, arg2: HANDLE, arg3: HANDLE, arg4: PHANDLE, arg5: DWORD, arg6: BOOL, arg7: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:adcab531758b
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:b4d73724c7aa
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ad07598c9e6d
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:fcc80d36382e
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:0dfc645e02a4
    pub fn GetDateFormatA( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCSTR, arg5: LPSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:41bbf8ebbcbf
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:8fafee364f22
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs // gen:50e246193424
    pub fn GetLocaleInfoA( Locale: LCID, LCType: LCTYPE, lpLCData: LPSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:e50112f4c62a
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs // gen:2f213353c565
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:8b50fa1e44f3
    pub fn GetSystemTimeAsFileTime(arg1: LPFILETIME); // aclui-rust/src/types_i386.rs // gen:ee31e2d67b51
    pub fn GetTempFileNameW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: UINT, arg4: LPWSTR) -> UINT; // aclui-rust/src/types_i386.rs // gen:8301f5dedab4
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs // gen:f4e2a5f33faa
    pub fn GetUserNameA(lpBuffer: LPSTR, pcbBuffer: LPDWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5ee1774e092d
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:3aaa7317691b
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:cd40b76d9b6b
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:d32c25758174
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs // gen:5bdbdeebfcb7
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:cb1bdfb02278
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs // gen:68642d9620a9
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs // gen:7fcc067b765c
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:364345b52ec2
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:49bda6930c6d
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:d83c13224bef
    pub fn RegCreateKeyA(hKey: HKEY, lpSubKey: LPCSTR, phkResult: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:3d662427d941
    pub fn RegCreateKeyExA( hKey: HKEY, lpSubKey: LPCSTR, Reserved: DWORD, lpClass: LPSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:a9cd1c4e99f2
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:987a8cb30309
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:d7a8638de5e7
    pub fn RegDeleteKeyA(hKey: HKEY, lpSubKey: LPCSTR) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:f0125c1c036d
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:76786a34a2a4
    pub fn RegDeleteTreeW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // fontext-rust/src/types_i386.rs // gen:7e9cdd05f4e1
    pub fn RegEnumKeyA(hKey: HKEY, dwIndex: DWORD, lpName: LPSTR, cchName: DWORD) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:86f9323c9285
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:8762af9c2c00
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:28185a2f7567
    pub fn RegOpenKeyA(arg1: HKEY, arg2: LPCSTR, arg3: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:86e841a4c587
    pub fn RegOpenKeyExA( arg1: HKEY, arg2: LPCSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:e57575460c08
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:a5045db5aff0
    pub fn RegQueryValueExA( hKey: HKEY, lpValueName: LPCSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:6bcb7f4421b5
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:cef016468a4f
    pub fn RegSetValueExA( hKey: HKEY, lpValueName: LPCSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:dc4e7a64bf23
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:0e225f333c0f
    pub fn ReleaseSemaphore(arg1: HANDLE, arg2: LONG, arg3: LPLONG) -> BOOL; // aclui-rust/src/types_i386.rs // gen:1ea006080465
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:5bf26683f6db
    pub fn SetFilePointerEx( arg1: HANDLE, arg2: LARGE_INTEGER, arg3: PLARGE_INTEGER, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:714a68e003fa
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs // gen:08f4ac87288b
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs // gen:0739554715e5
    pub fn TlsAlloc() -> DWORD; // aclui-rust/src/types_i386.rs // gen:67961a36e037
    pub fn TlsFree(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c939accad583
    pub fn TlsGetValue(arg1: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs // gen:b6c80b186b22
    pub fn TlsSetValue(arg1: DWORD, arg2: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:3adc17cda88c
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:0d4f240a125e
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:0e129f7cfff2
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5af3a6d97065
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:09f7e4d3dd9f
    pub fn lstrcmpA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:6254df28289b
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:f3454dea79d1
    pub fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:035558f8c98c
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:eea6eb419b40
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:144411e62218
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:244f3df56f54
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CRYPT_E_NOT_IN_REVOCATION_DATABASE: HRESULT = 0x80092014_u32 as HRESULT; // clang, chain.c
pub const CRYPT_E_NO_REVOCATION_CHECK: HRESULT = 0x80092012_u32 as HRESULT; // clang, chain.c
pub const CRYPT_E_NO_REVOCATION_DLL: HRESULT = 0x80092011_u32 as HRESULT; // clang, cert.c
pub const CRYPT_E_REVOCATION_OFFLINE: HRESULT = 0x80092013_u32 as HRESULT; // clang, chain.c
pub const CRYPT_E_REVOKED: HRESULT = 0x80092010_u32 as HRESULT; // clang, chain.c
pub const SPC_FINANCIAL_CRITERIA_STRUCT: LPCSTR = 0x7D2usize as LPCSTR; // clang, object.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CERT_SYSTEM_STORE_LOCATION: c2rust, store.c
pub type CERT_SYSTEM_STORE_LOCATION = _CERT_SYSTEM_STORE_LOCATION;
// UUID: c2rust, cert.c
pub type UUID = GUID;
// _CERT_SYSTEM_STORE_LOCATION: c2rust, store.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CERT_SYSTEM_STORE_LOCATION {
    pub dwFlags: DWORD,
    pub pwszStoreLocation: PCWSTR,
}
// _CONTEXT_PROPERTY_LIST: c2rust, proplist.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CONTEXT_PROPERTY_LIST {
    pub cs: CRITICAL_SECTION,
    pub properties: list,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // chain.c
    pub fn _snprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Count: size_t,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // decode.c
    pub fn _strnicmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // cert.c
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // base64.c
    pub fn sprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // decode.c
    pub fn strchr(
        _Str: *const ::core::ffi::c_char,
        _Val: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // oid.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // cert.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // cert.c
    pub fn stricmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // oid.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // base64.c
    pub fn strncmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // cert.c
    pub fn strstr(
        _Str: *const ::core::ffi::c_char,
        _SubStr: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // base64.c
    pub fn strtol(
        _Str: *const ::core::ffi::c_char,
        _EndPtr: *mut *mut ::core::ffi::c_char,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long; // oid.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // chain.c
    pub fn wcslwr(_Str: *mut wchar_t) -> *mut wchar_t; // cert.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // filestore.c
}
extern "system" {
    pub fn BCryptCloseAlgorithmProvider(_: BCRYPT_ALG_HANDLE, _: ULONG) -> NTSTATUS; // cert.c
    pub fn BCryptCreateHash(
        _: BCRYPT_ALG_HANDLE,
        _: *mut BCRYPT_HASH_HANDLE,
        _: PUCHAR,
        _: ULONG,
        _: PUCHAR,
        _: ULONG,
        _: ULONG,
    ) -> NTSTATUS; // cert.c
    pub fn BCryptDestroyHash(_: BCRYPT_HASH_HANDLE) -> NTSTATUS; // cert.c
    pub fn BCryptDestroyKey(_: BCRYPT_KEY_HANDLE) -> NTSTATUS; // cert.c
    pub fn BCryptFinishHash(_: BCRYPT_HASH_HANDLE, _: PUCHAR, _: ULONG, _: ULONG) -> NTSTATUS; // cert.c
    pub fn BCryptGetProperty(
        _: BCRYPT_HANDLE,
        _: LPCWSTR,
        _: PUCHAR,
        _: ULONG,
        _: *mut ULONG,
        _: ULONG,
    ) -> NTSTATUS; // cert.c
    pub fn BCryptHashData(_: BCRYPT_HASH_HANDLE, _: PUCHAR, _: ULONG, _: ULONG) -> NTSTATUS; // cert.c
    pub fn BCryptImportKeyPair(
        _: BCRYPT_ALG_HANDLE,
        _: BCRYPT_KEY_HANDLE,
        _: LPCWSTR,
        _: *mut BCRYPT_KEY_HANDLE,
        _: *mut UCHAR,
        _: ULONG,
        _: ULONG,
    ) -> NTSTATUS; // cert.c
    pub fn BCryptOpenAlgorithmProvider(
        _: *mut BCRYPT_ALG_HANDLE,
        _: LPCWSTR,
        _: LPCWSTR,
        _: ULONG,
    ) -> NTSTATUS; // cert.c
    pub fn BCryptVerifySignature(
        _: BCRYPT_KEY_HANDLE,
        _: *mut ::core::ffi::c_void,
        _: *mut UCHAR,
        _: ULONG,
        _: *mut UCHAR,
        _: ULONG,
        _: ULONG,
    ) -> NTSTATUS; // cert.c
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
pub static mut __wine_dbch_crypt: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x63\x72\x79\x70\x74\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch_chain: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x63\x68\x61\x69\x6e\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_crypt };
// ==== end wine debug channels ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::crypt32::store::CRYPT_FreeStore;
pub use crate::crypt32::store::CRYPT_InitStore;
pub use crate::crypt32::store::CertAddCRLContextToStore;
pub use crate::crypt32::store::CertCloseStore;
pub use crate::crypt32::store::CertDeleteCRLFromStore;
pub use crate::crypt32::store::CertDeleteCertificateFromStore;
pub use crate::crypt32::store::CertDuplicateStore;
pub use crate::crypt32::store::CertEnumCRLsInStore;
pub use crate::crypt32::store::CertEnumCertificatesInStore;
pub use crate::crypt32::store::CertGetStoreProperty;
pub use crate::crypt32::store::CertOpenStore;
pub use crate::crypt32::store::CertOpenSystemStoreA;
pub use crate::crypt32::store::CertOpenSystemStoreW;
pub use crate::crypt32::store::I_CertUpdateStore;
pub use crate::crypt32::store::init_empty_store;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! GET_CERT_ENCODING_TYPE { ($p0:expr $(,)?) => { ($p0 & 0x0000ffff) }; } // gen:c2b1c03e4b47
#[macro_export] macro_rules! GET_CMSG_ENCODING_TYPE { ($p0:expr $(,)?) => { ($p0 & 0xffff0000u32) }; } // gen:52fba6704a1b
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
pub const BCRYPT_ECDSA_P256_ALG_HANDLE: BCRYPT_ALG_HANDLE = 0x2e1usize as BCRYPT_ALG_HANDLE; // gen:c40904877271
pub const BCRYPT_ECDSA_P384_ALG_HANDLE: BCRYPT_ALG_HANDLE = 0x2f1usize as BCRYPT_ALG_HANDLE; // gen:79a6def99aec
pub const BCRYPT_RSA_ALG_HANDLE: BCRYPT_ALG_HANDLE = 0xe1usize as BCRYPT_ALG_HANDLE; // gen:538687d0a1a7
pub const BCRYPT_RSA_SIGN_ALG_HANDLE: BCRYPT_ALG_HANDLE = 0x311usize as BCRYPT_ALG_HANDLE; // gen:b06659c4b893
pub const CERT_E_CHAINING: HRESULT = ((0x800B010Au32) as HRESULT); // gen:b16e0d3e3f13
pub const CERT_E_CN_NO_MATCH: HRESULT = ((0x800B010Fu32) as HRESULT); // gen:c9ae57fa321e
pub const CERT_E_CRITICAL: HRESULT = ((0x800B0105u32) as HRESULT); // gen:aa8539581ccd
pub const CERT_E_EXPIRED: HRESULT = ((0x800B0101u32) as HRESULT); // gen:76e6aef47278
pub const CERT_E_REVOCATION_FAILURE: HRESULT = ((0x800B010Eu32) as HRESULT); // gen:545e5e1bc5a9
pub const CERT_E_REVOKED: HRESULT = ((0x800B010Cu32) as HRESULT); // gen:52a09ed0c6b3
pub const CERT_E_UNTRUSTEDROOT: HRESULT = ((0x800B0109u32) as HRESULT); // gen:4dadb2c17f56
pub const CERT_E_UNTRUSTEDTESTROOT: HRESULT = ((0x800B010Du32) as HRESULT); // gen:170d4cff1c7e
pub const CERT_E_WRONG_USAGE: HRESULT = ((0x800B0110u32) as HRESULT); // gen:01850c659e66
pub const CERT_STORE_PROV_COLLECTION: LPCSTR = 0xbusize as LPCSTR; // gen:ed4d0472e268
pub const CERT_STORE_PROV_MEMORY: LPCSTR = 0x2usize as LPCSTR; // gen:af4d0ce92919
pub const CERT_STORE_PROV_MSG: LPCSTR = 0x1usize as LPCSTR; // gen:87e77ab0b966
pub const CERT_STORE_PROV_PKCS7: LPCSTR = 0x5usize as LPCSTR; // gen:fcc7e7285b5d
pub const CERT_STORE_PROV_SYSTEM_A: LPCSTR = 0x9usize as LPCSTR; // gen:1f6238aae6a4
pub const CERT_STORE_PROV_SYSTEM_REGISTRY_W: LPCSTR = 0xdusize as LPCSTR; // gen:c2112d52f53c
pub const CERT_STORE_PROV_SYSTEM_W: LPCSTR = 0xausize as LPCSTR; // gen:025ca5db595e
pub const CMS_SIGNER_INFO: LPCSTR = 0x1f5usize as LPCSTR; // gen:7290fd70afd5
pub const CONTEXT_OID_CERTIFICATE: LPCSTR = 0x1usize as LPCSTR; // gen:2fd43d591506
pub const CRYPT_E_ALREADY_DECRYPTED: HRESULT = ((0x80091009u32) as HRESULT); // gen:447c986a3f3f
pub const CRYPT_E_ASN1_BADTAG: HRESULT = ((0x8009310Bu32) as HRESULT); // gen:6e4b32cebd2e
pub const CRYPT_E_ASN1_CHOICE: HRESULT = ((0x8009310Cu32) as HRESULT); // gen:cac01040392a
pub const CRYPT_E_ASN1_CORRUPT: HRESULT = ((0x80093103u32) as HRESULT); // gen:631d74265117
pub const CRYPT_E_ASN1_EOD: HRESULT = ((0x80093102u32) as HRESULT); // gen:c95c846f1158
pub const CRYPT_E_ASN1_ERROR: HRESULT = ((0x80093100u32) as HRESULT); // gen:cf02fd026b9a
pub const CRYPT_E_ASN1_LARGE: HRESULT = ((0x80093104u32) as HRESULT); // gen:e07cc8897c85
pub const CRYPT_E_ASN1_RULE: HRESULT = ((0x8009310Du32) as HRESULT); // gen:7a479c9184a6
pub const CRYPT_E_BAD_ENCODE: HRESULT = ((0x80092002u32) as HRESULT); // gen:0a59da8c1e74
pub const CRYPT_E_CONTROL_TYPE: HRESULT = ((0x8009100Cu32) as HRESULT); // gen:81b527a0ee22
pub const CRYPT_E_EXISTS: HRESULT = ((0x80092005u32) as HRESULT); // gen:c21776cab670
pub const CRYPT_E_FILE_ERROR: HRESULT = ((0x80092003u32) as HRESULT); // gen:ae9481c09820
pub const CRYPT_E_HASH_VALUE: HRESULT = ((0x80091007u32) as HRESULT); // gen:6e4ebee6dd70
pub const CRYPT_E_INVALID_IA5_STRING: HRESULT = ((0x80092022u32) as HRESULT); // gen:5d10c62b8ec3
pub const CRYPT_E_INVALID_INDEX: HRESULT = ((0x80091008u32) as HRESULT); // gen:2f4ee6e4937d
pub const CRYPT_E_INVALID_MSG_TYPE: HRESULT = ((0x80091004u32) as HRESULT); // gen:d6d740d0f471
pub const CRYPT_E_INVALID_NUMERIC_STRING: HRESULT = ((0x80092020u32) as HRESULT); // gen:767e1e01582e
pub const CRYPT_E_INVALID_PRINTABLE_STRING: HRESULT = ((0x80092021u32) as HRESULT); // gen:b4a9d23d34ad
pub const CRYPT_E_INVALID_X500_STRING: HRESULT = ((0x80092023u32) as HRESULT); // gen:8816fec2d878
pub const CRYPT_E_MSG_ERROR: HRESULT = ((0x80091001u32) as HRESULT); // gen:c5ee5891c1f9
pub const CRYPT_E_NOT_CHAR_STRING: HRESULT = ((0x80092024u32) as HRESULT); // gen:979886ea562b
pub const CRYPT_E_NOT_FOUND: HRESULT = ((0x80092004u32) as HRESULT); // gen:fcc8d64692de
pub const CRYPT_E_NO_KEY_PROPERTY: HRESULT = ((0x8009200Bu32) as HRESULT); // gen:d3fff485c7c5
pub const CRYPT_E_NO_MATCH: HRESULT = ((0x80092009u32) as HRESULT); // gen:7149557b6431
pub const CRYPT_E_NO_TRUSTED_SIGNER: HRESULT = ((0x8009202Bu32) as HRESULT); // gen:4b5abac9a692
pub const CRYPT_E_PENDING_CLOSE: HRESULT = ((0x8009200Fu32) as HRESULT); // gen:846aa568f890
pub const CRYPT_E_SELF_SIGNED: HRESULT = ((0x80092007u32) as HRESULT); // gen:e8edebcbfd68
pub const CRYPT_E_SIGNER_NOT_FOUND: HRESULT = ((0x8009100Eu32) as HRESULT); // gen:ed95ffae53c8
pub const CRYPT_E_UNEXPECTED_MSG_TYPE: HRESULT = ((0x8009200Au32) as HRESULT); // gen:8437f0569087
pub const CRYPT_E_UNKNOWN_ALGO: HRESULT = ((0x80091002u32) as HRESULT); // gen:15f6f4106f61
pub const E_ACCESSDENIED: HRESULT = ((0x80070005u32) as HRESULT); // gen:97ace9e06777
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const HCCE_CURRENT_USER: HCERTCHAINENGINE = 0x0usize as HCERTCHAINENGINE; // gen:2524d5e9209e
pub const HCCE_LOCAL_MACHINE: HCERTCHAINENGINE = 0x1usize as HCERTCHAINENGINE; // gen:027108224e7d
pub const LOCALE_SYSTEM_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x02) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:2e6e0be784d6
pub const NOERROR: HRESULT = ((0) as HRESULT); // gen:e8f751a60ae6
pub const NTE_BAD_ALGID: HRESULT = ((0x80090008u32) as HRESULT); // gen:294cfca1f4f8
pub const NTE_BAD_FLAGS: HRESULT = ((0x80090009u32) as HRESULT); // gen:2efa8f945900
pub const NTE_BAD_KEYSET: HRESULT = ((0x80090016u32) as HRESULT); // gen:42e4382c654b
pub const NTE_BAD_SIGNATURE: HRESULT = ((0x80090006u32) as HRESULT); // gen:caae06333d02
pub const NTE_EXISTS: HRESULT = ((0x8009000Fu32) as HRESULT); // gen:51a7553449dc
pub const NTE_NO_KEY: HRESULT = ((0x8009000Du32) as HRESULT); // gen:91cd44810d90
pub const PKCS_ATTRIBUTES: LPCSTR = 0x30usize as LPCSTR; // gen:516e36a84d87
pub const PKCS_CONTENT_INFO: LPCSTR = 0x21usize as LPCSTR; // gen:b813dd8afd31
pub const PKCS_CTL: LPCSTR = 0x25usize as LPCSTR; // gen:8106b1dc6519
pub const RSA_CSP_PUBLICKEYBLOB: LPCSTR = 0x13usize as LPCSTR; // gen:5604d8b9eaf0
pub const SEC_E_BUFFER_TOO_SMALL: HRESULT = ((0x80090321u32) as HRESULT); // gen:ac819db8597f
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const TRUST_E_BASIC_CONSTRAINTS: HRESULT = ((0x80096019u32) as HRESULT); // gen:5257b3896104
pub const TRUST_E_CERT_SIGNATURE: HRESULT = ((0x80096004u32) as HRESULT); // gen:58f502325bed
pub const TRUST_E_SUBJECT_FORM_UNKNOWN: HRESULT = ((0x800B0003u32) as HRESULT); // gen:4ee6dc89c571
pub const URL_OID_CERTIFICATE_ISSUER: LPCSTR = 0x1usize as LPCSTR; // gen:fb785724f5c3
pub const X509_ALTERNATE_NAME: LPCSTR = 0xcusize as LPCSTR; // gen:8881dd219b94
pub const X509_AUTHORITY_INFO_ACCESS: LPCSTR = 0x20usize as LPCSTR; // gen:3f92d7621674
pub const X509_AUTHORITY_KEY_ID: LPCSTR = 0x9usize as LPCSTR; // gen:e58b2829a393
pub const X509_AUTHORITY_KEY_ID2: LPCSTR = 0x1fusize as LPCSTR; // gen:94b7c0be159c
pub const X509_BASIC_CONSTRAINTS2: LPCSTR = 0xfusize as LPCSTR; // gen:11777ba4b097
pub const X509_BITS: LPCSTR = 0x1ausize as LPCSTR; // gen:12f233ac10b5
pub const X509_CERT: LPCSTR = 0x1usize as LPCSTR; // gen:016a69896b73
pub const X509_CERT_CRL_TO_BE_SIGNED: LPCSTR = 0x3usize as LPCSTR; // gen:15cf6a0fe8b4
pub const X509_CERT_POLICIES: LPCSTR = 0x10usize as LPCSTR; // gen:5ca2e95ccdf9
pub const X509_CERT_TO_BE_SIGNED: LPCSTR = 0x2usize as LPCSTR; // gen:f125a92b18a5
pub const X509_CRL_DIST_POINTS: LPCSTR = 0x23usize as LPCSTR; // gen:05a0e8660872
pub const X509_ECC_SIGNATURE: LPCSTR = 0x2fusize as LPCSTR; // gen:b8263a60b932
pub const X509_ENHANCED_KEY_USAGE: LPCSTR = 0x24usize as LPCSTR; // gen:283453575be8
pub const X509_INTEGER: LPCSTR = 0x1busize as LPCSTR; // gen:7ca5dd53bed0
pub const X509_ISSUING_DIST_POINT: LPCSTR = 0x36usize as LPCSTR; // gen:91753b8ff03e
pub const X509_KEY_USAGE: LPCSTR = 0xeusize as LPCSTR; // gen:e0e5c5dca640
pub const X509_MULTI_BYTE_INTEGER: LPCSTR = 0x1cusize as LPCSTR; // gen:422cc8cac22c
pub const X509_MULTI_BYTE_UINT: LPCSTR = 0x26usize as LPCSTR; // gen:4b11d43d15cf
pub const X509_NAME: LPCSTR = 0x7usize as LPCSTR; // gen:0f21a9a3316e
pub const X509_NAME_CONSTRAINTS: LPCSTR = 0x37usize as LPCSTR; // gen:fa1caa142fee
pub const X509_OBJECT_IDENTIFIER: LPCSTR = 0x49usize as LPCSTR; // gen:43d05ac983b6
pub const X509_OCTET_STRING: LPCSTR = 0x19usize as LPCSTR; // gen:6e7cd4f0bc4b
pub const X509_PUBLIC_KEY_INFO: LPCSTR = 0x8usize as LPCSTR; // gen:3a2461c17cf8
pub const X509_UNICODE_ANY_STRING: LPCSTR = 0x18usize as LPCSTR; // gen:aea4bdfa8b12
pub const X509_UNICODE_NAME: LPCSTR = 0x14usize as LPCSTR; // gen:7214fadb8f55
pub const X509_UNICODE_NAME_VALUE: LPCSTR = 0x18usize as LPCSTR; // gen:4770c1f25ccd
pub use crate::{GET_CERT_ENCODING_TYPE, GET_CMSG_ENCODING_TYPE, max, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/crypt32/crypt32_private.h:459: #define ALIGN_DWORD_PTR(x) (((x) + sizeof(DWORD_PTR) - 1) & ~(sizeof(DWORD_PTR) - 1)) // gen:c58c304c32ce
#[allow(unused_macros)]
macro_rules! ALIGN_DWORD_PTR {
    ($x:expr $(,)?) => {
        ((($x) + core::mem::size_of::<DWORD_PTR>() - 1) & !(core::mem::size_of::<DWORD_PTR>() - 1))
    };
}
#[allow(unused_imports)]
pub(crate) use ALIGN_DWORD_PTR;
// dll/win32/crypt32/decode.c:301: #define FINALMEMBERSIZE(s, member) (sizeof(s) - offsetof(s, member)) // gen:e3dba372c5ab
#[allow(unused_macros)]
macro_rules! FINALMEMBERSIZE {
    // A field PATH, not just a name: a member inside a bindgen anonymous union is
    // `__bindgen_anon_1.FullName` (decode.c's CRL_DIST_POINT_NAME), and offset_of! takes one.
    ($s:ty, $($member:ident).+ $(,)?) => {
        (core::mem::size_of::<$s>() - core::mem::offset_of!($s, $($member).+))
    };
}
#[allow(unused_imports)]
pub(crate) use FINALMEMBERSIZE;
// dll/win32/crypt32/decode.c:116: #define GET_LEN_BYTES(b) ((b) <= 0x80 ? 1 : 1 + ((b) & 0x7f)) // gen:1713242772b5
#[allow(unused_macros)]
macro_rules! GET_LEN_BYTES {
    ($b:expr $(,)?) => {
        (if ($b) <= 0x80 { 1 } else { 1 + (($b) & 0x7f) })
    };
}
#[allow(unused_imports)]
pub(crate) use GET_LEN_BYTES;
// dll/win32/crypt32/crypt32_private.h:464: #define IS_INTOID(x) (((ULONG_PTR)(x) >> 16) == 0) // gen:532215abcb12
#[allow(unused_macros)]
macro_rules! IS_INTOID {
    ($x:expr $(,)?) => {
        (((($x) as ULONG_PTR) >> 16) == 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_INTOID;
// dll/win32/crypt32/chain.c:2521: #define IS_TRUST_ERROR_SET(TrustStatus, bits) (TrustStatus)->dwErrorStatus & (bits) // gen:534e22c7653d
#[allow(unused_macros)]
macro_rules! IS_TRUST_ERROR_SET {
    ($trust_status:expr, $bits:expr $(,)?) => {
        ((*($trust_status)).dwErrorStatus & ($bits))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_TRUST_ERROR_SET;
// dll/win32/crypt32/decode.c:302: #define MEMBERSIZE(s, member, nextmember) (offsetof(s, nextmember) - offsetof(s, member)) // gen:f5e45aa75497
#[allow(unused_macros)]
macro_rules! MEMBERSIZE {
    ($s:ty, $member:ident, $nextmember:ident $(,)?) => {
        (core::mem::offset_of!($s, $nextmember) - core::mem::offset_of!($s, $member))
    };
}
#[allow(unused_imports)]
pub(crate) use MEMBERSIZE;
// dll/win32/crypt32/crypt32_private.h:460: #define POINTER_ALIGN_DWORD_PTR(p) ((LPVOID)ALIGN_DWORD_PTR((DWORD_PTR)(p))) // gen:8d5d68b566da
#[allow(unused_macros)]
macro_rules! POINTER_ALIGN_DWORD_PTR {
    ($p:expr $(,)?) => {
        (ALIGN_DWORD_PTR!(($p) as usize) as LPVOID)
    };
}
#[allow(unused_imports)]
pub(crate) use POINTER_ALIGN_DWORD_PTR;
// dll/win32/crypt32/protectdata.c:135: #define TRACE_DATA_BLOB(blob) do { TRACE("%s cbData: %u\n", #blob ,(unsigned int)((blob)->cbData)); TRACE("%s pbData @ %p:%s\n", #blob ,(blob)->pbData, hex_str((blob)->pbData, (blob)->cbData)); } while (0) // gen:a6c60fba6ac8
#[allow(unused_macros)]
macro_rules! TRACE_DATA_BLOB {
    ($blob:expr $(,)?) => {{
        TRACE!("{} cbData: {}\n", stringify!($blob), ((*($blob)).cbData) as u32);
        TRACE!(
            "{} pbData @ {:p}:{}\n",
            stringify!($blob),
            (*($blob)).pbData,
            hex_str((*($blob)).pbData, (*($blob)).cbData as c_int)
        );
    }};
}
#[allow(unused_imports)]
pub(crate) use TRACE_DATA_BLOB;
// dll/win32/crypt32/wine/unixlib.h:15: header-inline `__reactos_call_unix_process_attach` // gen:2fab936d452b
#[allow(dead_code)]
#[inline]
pub unsafe fn __reactos_call_unix_process_attach(
    Args: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    let _ = Args;
    0
}
// dll/win32/crypt32/wine/unixlib.h:20: header-inline `__reactos_call_unix_process_detach` // gen:c613db9ab23c
#[allow(dead_code)]
#[inline]
pub unsafe fn __reactos_call_unix_process_detach(
    Args: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    let _ = Args;
    0
}
// dll/win32/crypt32/wine/unixlib.h:13: #define __wine_init_unix_call() 0 // gen:206b535e9310
#[allow(unused_macros)]
macro_rules! __wine_init_unix_call {
    ($(,)?) => {
        0
    };
}
#[allow(unused_imports)]
pub(crate) use __wine_init_unix_call;
// dll/win32/crypt32/crypt32_private.h:212: header-inline `cert_from_ptr` // gen:a15b80a9f165
#[allow(dead_code)]
#[inline]
pub unsafe fn cert_from_ptr(ptr: *const CERT_CONTEXT) -> *mut cert_t {
    CONTAINING_RECORD!(ptr, cert_t, ctx) as *mut cert_t
}
// dll/win32/crypt32/crypt32_private.h:197: header-inline `context_from_ptr` // gen:93fb613e3112
#[allow(dead_code)]
#[inline]
pub unsafe fn context_from_ptr(ptr: *const core::ffi::c_void) -> *mut context_t {
    (ptr as *const context_t).sub(1) as *mut context_t
}
// dll/win32/crypt32/crypt32_private.h:202: header-inline `context_ptr` // gen:50228d3d1cc3
#[allow(dead_code)]
#[inline]
pub unsafe fn context_ptr(context: *mut context_t) -> *mut core::ffi::c_void {
    context.add(1) as *mut core::ffi::c_void
}
// dll/win32/crypt32/crypt32_private.h:222: header-inline `crl_from_ptr` // gen:0a125d5972d1
#[allow(dead_code)]
#[inline]
pub unsafe fn crl_from_ptr(ptr: *const CRL_CONTEXT) -> *mut crl_t {
    CONTAINING_RECORD!(ptr, crl_t, ctx) as *mut crl_t
}
// dll/win32/crypt32/crypt32_private.h:232: header-inline `ctl_from_ptr` // gen:fe0614d53ca7
#[allow(dead_code)]
#[inline]
pub unsafe fn ctl_from_ptr(ptr: *const CTL_CONTEXT) -> *mut ctl_t {
    CONTAINING_RECORD!(ptr, ctl_t, ctx) as *mut ctl_t
}
// ==== end port macros ====
