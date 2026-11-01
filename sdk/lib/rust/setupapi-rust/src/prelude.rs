//! setupapi-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

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

// wine/unicode.h, as THIS tree has it (sdk/include/wine/unicode.h, the one setupapi_private.h
// includes): not Wine's inline loops but `#define strlenW(s) wcslen((s))` and friends, so the
// C calls the CRT. Aliases, then, exactly as the #defines are -- no bodies of our own.
extern "C" {
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn wcstol(s: *const u16, end: *mut *mut u16, base: c_int) -> c_long;
}
pub use self::{
    wcscat as strcatW, wcscmp as strcmpW, wcscpy as strcpyW, wcslen as strlenW,
    wcsrchr as strrchrW, wcstol as strtolW,
};
// The rest of unicode.h's #defines the second batch calls. Bound by link_name to the CRT
// function the #define names, so they never collide with resolve's own CRT prototypes.
extern "C" {
    #[link_name = "_wcsicmp"]
    pub fn strcmpiW(s1: *const u16, s2: *const u16) -> c_int;
    #[link_name = "_wcsnicmp"]
    pub fn strncmpiW(s1: *const u16, s2: *const u16, n: usize) -> c_int;
    #[link_name = "wcschr"]
    pub fn strchrW(s: *const u16, c: u16) -> *mut u16;
    #[link_name = "wcstoul"]
    pub fn strtoulW(s: *const u16, end: *mut *mut u16, base: c_int) -> core::ffi::c_ulong;
    #[link_name = "_wtoi"]
    pub fn atoiW(s: *const u16) -> c_int;
    #[link_name = "towlower"]
    pub fn tolowerW(c: u16) -> u16;
}
pub use win32macros_rust::iswspace as isspaceW;
// stdlib.h `max(a,b)`: the shared transcription.
pub use win32macros_rust::{max, InterlockedCompareExchangePointer};
// wine/debug.h `enum __wine_debug_class` and guiddef.h `REFIID`. Hand-kept: resolve now
// declines to copy a type ("add its header to the island allowlist") and so dropped both.
pub type __wine_debug_class = c_int;
// wine/debug.h:51, the enum's values at the enum's type: cfgmgr's `u32` copy made every
// `let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;` of the second batch a type error.
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
pub type REFIID = *const GUID;
// guiddef.h `IsEqualIID(riid1, riid2)` is IsEqualGUID.
#[inline(always)]
pub unsafe fn IsEqualIID(a: *const GUID, b: *const GUID) -> bool {
    IsEqualGUID!(a, b)
}
// winerror.h. E_FAIL is an HRESULT; the C's SetLastError(E_FAIL) converts it to a DWORD.
pub const E_FAIL: HRESULT = 0x8000_4005_u32 as HRESULT;
#[inline(always)]
pub const fn HRESULT_CODE(hr: HRESULT) -> DWORD {
    (hr as DWORD) & 0xFFFF
}
#[inline(always)]
pub const fn HRESULT_FACILITY(hr: HRESULT) -> DWORD {
    ((hr as DWORD) >> 16) & 0x1FFF
}
// commctrl.h `#define ImageList_AddIcon(himl,hicon) ImageList_ReplaceIcon(himl,-1,hicon)`.
extern "system" {
    pub fn ImageList_ReplaceIcon(himl: HIMAGELIST, i: c_int, hicon: HICON) -> c_int;
}
#[inline(always)]
pub unsafe fn ImageList_AddIcon(himl: HIMAGELIST, hicon: HICON) -> c_int {
    ImageList_ReplaceIcon(himl, -1, hicon)
}
// DUMMY_PSEH: the _SEH2_EXCEPT arms are dead code in user mode here, so the code is never read.
#[inline(always)]
pub fn _SEH2_GetExceptionCode() -> LONG {
    0
}
// shobjidl.h / objidl.h COBJMACROS, through the island's vtables.
#[inline(always)]
pub unsafe fn IShellLinkW_QueryInterface(p: *mut IShellLinkW, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    ((*(*p).lpVtbl).QueryInterface.unwrap_unchecked())(p, riid, ppv)
}
#[inline(always)]
pub unsafe fn IShellLinkW_Release(p: *mut IShellLinkW) -> ULONG {
    ((*(*p).lpVtbl).Release.unwrap_unchecked())(p)
}
#[inline(always)]
pub unsafe fn IShellLinkW_SetPath(p: *mut IShellLinkW, s: LPCWSTR) -> HRESULT {
    ((*(*p).lpVtbl).SetPath.unwrap_unchecked())(p, s)
}
#[inline(always)]
pub unsafe fn IShellLinkW_SetArguments(p: *mut IShellLinkW, s: LPCWSTR) -> HRESULT {
    ((*(*p).lpVtbl).SetArguments.unwrap_unchecked())(p, s)
}
#[inline(always)]
pub unsafe fn IShellLinkW_SetWorkingDirectory(p: *mut IShellLinkW, s: LPCWSTR) -> HRESULT {
    ((*(*p).lpVtbl).SetWorkingDirectory.unwrap_unchecked())(p, s)
}
#[inline(always)]
pub unsafe fn IShellLinkW_SetIconLocation(p: *mut IShellLinkW, s: LPCWSTR, i: c_int) -> HRESULT {
    ((*(*p).lpVtbl).SetIconLocation.unwrap_unchecked())(p, s, i)
}
#[inline(always)]
pub unsafe fn IShellLinkW_SetDescription(p: *mut IShellLinkW, s: LPCWSTR) -> HRESULT {
    ((*(*p).lpVtbl).SetDescription.unwrap_unchecked())(p, s)
}
#[inline(always)]
pub unsafe fn IPersistFile_Save(p: *mut IPersistFile, s: LPCOLESTR, remember: BOOL) -> HRESULT {
    ((*(*p).lpVtbl).Save.unwrap_unchecked())(p, s, remember)
}
#[inline(always)]
pub unsafe fn IPersistFile_Release(p: *mut IPersistFile) -> ULONG {
    ((*(*p).lpVtbl).Release.unwrap_unchecked())(p)
}
// A NUL-terminated UTF-16 pointer from an ASCII literal, evaluated at compile time.
#[macro_export]
macro_rules! wz {
    ($s:literal) => {{
        const S: &[u8] = $s.as_bytes();
        const A: [u16; S.len() + 1] = {
            let mut a = [0u16; S.len() + 1];
            let mut i = 0;
            while i < S.len() {
                a[i] = S[i] as u16;
                i += 1;
            }
            a
        };
        &A as *const [u16; S.len() + 1] as *const u16
    }};
}
pub use crate::wz;
// regstr.h, UNICODE (values read from sdk/include/psdk/regstr.h).
pub const REGSTR_VAL_DRIVER: *const u16 = wz!("Driver");
pub const REGSTR_VAL_CLASS: *const u16 = wz!("Class");
pub const REGSTR_VAL_INSTALLER_32: *const u16 = wz!("Installer32");
pub const REGSTR_VAL_ENUMPROPPAGES_32: *const u16 = wz!("EnumPropPages32");
pub const REGSTR_PATH_CLASS_NT: *const u16 = wz!("System\\CurrentControlSet\\Control\\Class");
pub const REGSTR_VAL_CLASSGUID: *const u16 = wz!("ClassGUID");
pub const REGSTR_VAL_INSICON: *const u16 = wz!("Icon");
pub const REGSTR_VAL_INFSECTION: *const u16 = wz!("InfSection");
pub const REGSTR_VAL_INFPATH: *const u16 = wz!("InfPath");
pub const REGSTR_VAL_COINSTALLERS_32: *const u16 = wz!("CoInstallers32");
pub const REGSTR_VAL_PROVIDER_NAME: *const u16 = wz!("ProviderName");
pub const REGSTR_VAL_NOUSECLASS: *const u16 = wz!("NoUseClass");
pub const REGSTR_VAL_NOINSTALLCLASS: *const u16 = wz!("NoInstallClass");
pub const REGSTR_VAL_NODISPLAYCLASS: *const u16 = wz!("NoDisplayClass");
pub const REGSTR_VAL_MFG: *const u16 = wz!("Mfg");
pub const REGSTR_VAL_MATCHINGDEVID: *const u16 = wz!("MatchingDeviceId");
pub const REGSTR_VAL_INFSECTIONEXT: *const u16 = wz!("InfSectionExt");
pub const REGSTR_VAL_DRVDESC: *const u16 = wz!("DriverDesc");
pub const REGSTR_VAL_DEVDESC: *const u16 = wz!("DeviceDesc");
pub const REGSTR_VAL_CURRENTCONFIG: *const u16 = wz!("CurrentConfig");
pub const REGSTR_VAL_CSCONFIGFLAGS: *const u16 = wz!("CSConfigFlags");
pub const REGSTR_VAL_CONFIGFLAGS: *const u16 = wz!("ConfigFlags");
pub const REGSTR_PATH_SERVICES: *const u16 = wz!("System\\CurrentControlSet\\Services");
pub const REGSTR_PATH_IDCONFIGDB: *const u16 = wz!("System\\CurrentControlSet\\Control\\IDConfigDB");
pub const REGSTR_PATH_HWPROFILES: *const u16 = wz!("System\\CurrentControlSet\\Hardware Profiles");
pub const REGSTR_PATH_DEVICE_CLASSES: *const u16 = wz!("System\\CurrentControlSet\\Control\\DeviceClasses");
pub const REGSTR_PATH_CODEVICEINSTALLERS: *const u16 = wz!("System\\CurrentControlSet\\Control\\CoDeviceInstallers");
pub const REGSTR_KEY_CURRENT: *const u16 = wz!("Current");
// winreg.h `((HKEY)(LONG_PTR)(LONG)0x8000000N)`: sign-extended, like HKEY_LOCAL_MACHINE below.
pub const HKEY_CLASSES_ROOT: HKEY = (0x80000000u32 as LONG) as LONG_PTR as HKEY;
pub const HKEY_CURRENT_USER: HKEY = (0x80000001u32 as LONG) as LONG_PTR as HKEY;
pub const HKEY_USERS: HKEY = (0x80000003u32 as LONG) as LONG_PTR as HKEY;
// winerror.h `_HRESULT_TYPEDEF_(0)` / `(0x8007000E)`.
pub const NOERROR: HRESULT = 0;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000E_u32 as HRESULT;
// winnt.h: MAKELCID(MAKELANGID(LANG_NEUTRAL, SUBLANG_SYS_DEFAULT), SORT_DEFAULT) = 0x0800.
pub const LOCALE_SYSTEM_DEFAULT: LCID = 0x0800;
// devpropdef.h `((DEVPROP_BOOLEAN) 0)` / `((DEVPROP_BOOLEAN)-1)`.
pub const DEVPROP_FALSE: DEVPROP_BOOLEAN = 0;
pub const DEVPROP_TRUE: DEVPROP_BOOLEAN = -1i8 as DEVPROP_BOOLEAN;
// guiddef.h REFCLSID; setupapi.h:461 `#define LogSeverity DWORD`.
pub type REFCLSID = *const GUID;
pub type LogSeverity = DWORD;
// Cross-module calls the hand-ported SetupDiBuildDriverInfoList makes (integrate never saw it).
pub use crate::setupapi::devinst::SetupDiGetActualSectionToInstallExW;
pub use crate::setupapi::parser::{SetupGetInfFileListW, SetupGetLineCountW};
// winnt.h (xdk/winnt_old.h:225) `INVALID_FILE_ATTRIBUTES ((DWORD)-1)`, winbase.h:118
// `INVALID_SET_FILE_POINTER ((DWORD)-1)`;
// bindgen does not fold the casts.
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFF_FFFF;
pub const INVALID_SET_FILE_POINTER: DWORD = 0xFFFF_FFFF;
// regstr.h:483 `REGSTR_PATH_SYSTEMENUM TEXT("System\\CurrentControlSet\\Enum")`, UNICODE.
pub const REGSTR_PATH_SYSTEMENUM: *const u16 = (&[0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0043, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x0053, 0x0065, 0x0074, 0x005c, 0x0045, 0x006e, 0x0075, 0x006d, 0x0000u16]).as_ptr(); // L"System\\CurrentControlSet\\Enum"

// wine/debug.h:156 `int ros_dbg_log(enum __wine_debug_class cls, ...)`. Hand-kept: resolve's
// tree consensus picked winegdi's `cls: c_uint`, which no longer accepts the `c_int`
// __wine_debug_class every TRACE/ERR expansion here passes.
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        format: *const c_char,
        ...
    ) -> c_int;
}
// setupapi_private.h's static inline, translated once (port headers).
pub use crate::setupapi::setupapi_private_h::strdupAtoW;
// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CLASS_COINSTALLER: i32 = 1; // clang, devinst.c
pub const CLASS_INSTALLER: i32 = 4; // clang, devinst.c
pub const DEVICE_COINSTALLER: i32 = 2; // clang, devinst.c
pub const DIGCDP_FLAG_REMOTE_ADVANCED: i32 = 4; // clang, devclass.c
pub const DIGCDP_FLAG_REMOTE_BASIC: i32 = 3; // clang, devclass.c
pub const FLG_ADDREG_DELREG_BIT: i32 = 0x8000; // clang, install.c
pub const FLG_ADDREG_KEYONLY_COMMON: i32 = 0x2000; // clang, install.c
pub const FLG_DELREG_KEYONLY_COMMON: i32 = 0x2000; // clang, install.c
pub const FLG_DELREG_MULTI_SZ_DELSTRING: i32 = 0x18002; // clang, install.c
pub const SPINST_COPYINF: i32 = 0x200; // clang, devinst.c
pub const _O_RDONLY: i32 = 0; // clang, setupcab.c
pub const _O_RDWR: i32 = 2; // clang, setupcab.c
pub const _O_WRONLY: i32 = 1; // clang, setupcab.c
pub const _SH_COMPAT: i32 = 0; // clang, setupcab.c
pub const _SH_DENYNO: i32 = 0x40; // clang, setupcab.c
pub const _SH_DENYRD: i32 = 0x30; // clang, setupcab.c
pub const _SH_DENYRW: i32 = 0x10; // clang, setupcab.c
pub const _SH_DENYWR: i32 = 0x20; // clang, setupcab.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // acppage-rust/src/prelude.rs
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const CP_UTF8: u32 = 65001; // advapi32-sys/src/bindgen_amd64.rs
pub const DI_FLAGSEX_INSTALLEDDRIVER: i32 = 0x4000000; // dxdiag-rust/src/prelude.rs
pub const FILE_GENERIC_WRITE: u32 = 1179926; // apphelp-rust/src/prelude.rs
pub const ILC_COLOR16: u32 = 16; // access-rust/src/types_amd64.rs
pub const ILC_COLOR24: u32 = 24; // access-rust/src/types_amd64.rs
pub const ILC_COLOR32: u32 = 32; // access-rust/src/types_amd64.rs
pub const ILC_COLOR4: u32 = 4; // access-rust/src/types_amd64.rs
pub const ILC_COLOR8: u32 = 8; // access-rust/src/types_amd64.rs
pub const ILC_COLOR: u32 = 0; // access-rust/src/types_amd64.rs
pub const ILC_MASK: u32 = 1; // access-rust/src/types_amd64.rs
pub const LOCALE_ILANGUAGE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const PSH_PROPSHEETPAGE: u32 = 8; // access-rust/src/types_amd64.rs
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
pub const SECURITY_NT_AUTHORITY: i32 = 5; // msctfime-rust/src/prelude.rs
pub const SE_FILE_OBJECT: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_FILE_OBJECT; // island enum
pub const SPINST_REGISTERCALLBACKAWARE: DWORD = 0x00080000; // setupapi.h:610 (0x400 was no SPINST_ bit at all)
pub const ULONG_MAX: u32 = 4294967295; // calc-rust/src/prelude.rs
pub const _O_BINARY: u32 = 32768; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_CREAT: u32 = 256; // cabview-rust/src/prelude.rs
pub const _O_EXCL: u32 = 1024; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_NOINHERIT: u32 = 128; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_TRUNC: u32 = 512; // msvcrt-sys/src/bindgen_amd64.rs
pub const fdintCABINET_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCABINET_INFO; // island enum
pub const fdintCLOSE_FILE_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCLOSE_FILE_INFO; // island enum
pub const fdintCOPY_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCOPY_FILE; // island enum
pub const fdintNEXT_CABINET: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintNEXT_CABINET; // island enum
pub const fdintPARTIAL_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintPARTIAL_FILE; // island enum
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // winegdi-rust/src/types_i386.rs
    pub fn _errno() -> *mut c_int; // cmd-rust/src/prelude.rs
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub fn swscanf(_Src: *const wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // winsrv-sys/src/bindgen_i386.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs
    pub fn wcsicmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // charmap-rust/src/prelude.rs
    pub fn wcsnicmp(a: *const wchar_t, b: *const wchar_t, n: usize) -> c_int; // crt-shared-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // at-rust/src/prelude.rs
    pub fn wsprintfA(arg1: LPSTR, arg2: LPCSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub static GUID_NULL: IID; // evr-rust/src/types_amd64.rs
}
extern "system" {
    pub fn AdjustTokenPrivileges( arg1: HANDLE, arg2: BOOL, arg3: PTOKEN_PRIVILEGES, arg4: DWORD, arg5: PTOKEN_PRIVILEGES, arg6: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn AllocateAndInitializeSid( arg1: PSID_IDENTIFIER_AUTHORITY, arg2: BYTE, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: DWORD, arg9: DWORD, arg10: DWORD, arg11: *mut PSID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckTokenMembership(arg1: HANDLE, arg2: PSID, arg3: PBOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateDirectoryW( lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateFileA( arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DebugBreak(); // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn ExitWindowsEx(arg1: UINT, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FreeSid(arg1: PSID) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDiskFreeSpaceW( arg1: LPCWSTR, arg2: PDWORD, arg3: PDWORD, arg4: PDWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDriveTypeW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetEnvironmentVariableW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileSecurityW( arg1: LPCWSTR, arg2: SECURITY_INFORMATION, arg3: PSECURITY_DESCRIPTOR, arg4: DWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileSizeEx(arg1: HANDLE, arg2: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameA(arg1: LPCSTR, arg2: DWORD, arg3: LPSTR, arg4: *mut LPSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLogicalDriveStringsW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameA(hModule: HINSTANCE, lpFilename: LPSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSecurityDescriptorDacl( arg1: PSECURITY_DESCRIPTOR, arg2: LPBOOL, arg3: *mut PACL, arg4: LPBOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetSecurityDescriptorGroup( arg1: PSECURITY_DESCRIPTOR, arg2: *mut PSID, arg3: LPBOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetSecurityDescriptorOwner( arg1: PSECURITY_DESCRIPTOR, arg2: *mut PSID, arg3: LPBOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetSecurityDescriptorSacl( arg1: PSECURITY_DESCRIPTOR, arg2: LPBOOL, arg3: *mut PACL, arg4: LPBOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_i386.rs
    pub fn GetSystemWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetTempFileNameW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: UINT, arg4: LPWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTokenInformation( arg1: HANDLE, arg2: TOKEN_INFORMATION_CLASS, arg3: PVOID, arg4: DWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapSize(arg1: HANDLE, arg2: DWORD, arg3: LPCVOID) -> SIZE_T; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_SetBkColor(himl: HIMAGELIST, clrBk: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn ImageList_SetOverlayImage( himl: HIMAGELIST, iImage: ::core::ffi::c_int, iOverlay: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_i386.rs
    pub fn InitializeSecurityDescriptor(arg1: PSECURITY_DESCRIPTOR, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryExW(arg1: LPCWSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LookupPrivilegeValueW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: PLUID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxA( hWnd: HWND, lpText: LPCSTR, lpCaption: LPCSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MoveFileExW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MoveFileW(arg1: LPCWSTR, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn OpenEventW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ReleaseMutex(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RpcBindingFree(Binding: *mut RPC_BINDING_HANDLE) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcBindingFromStringBindingW( StringBinding: *mut ::core::ffi::c_ushort, Binding: *mut RPC_BINDING_HANDLE, ) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcStringBindingComposeW( ObjUuid: *mut ::core::ffi::c_ushort, Protseq: *mut ::core::ffi::c_ushort, NetworkAddr: *mut ::core::ffi::c_ushort, Endpoint: *mut ::core::ffi::c_ushort, Options: *mut ::core::ffi::c_ushort, StringBinding: *mut RPC_WSTR, ) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcStringFreeW(String: *mut RPC_WSTR) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetErrorMode(arg1: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetFileSecurityW( arg1: LPCWSTR, arg2: SECURITY_INFORMATION, arg3: PSECURITY_DESCRIPTOR, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFileTime( arg1: HANDLE, arg2: *const FILETIME, arg3: *const FILETIME, arg4: *const FILETIME, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SetSecurityDescriptorOwner(arg1: PSECURITY_DESCRIPTOR, arg2: PSID, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UuidFromStringW(StringUuid: *mut ::core::ffi::c_ushort, Uuid: *mut UUID) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WritePrivateProfileStringW( arg1: LPCWSTR, arg2: LPCWSTR, arg3: LPCWSTR, arg4: LPCWSTR, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, devclass.c
pub type HIMAGELIST = *mut _IMAGELIST;
// INSTALL_PARAMS_DATA: c2rust, devclass.c
pub type INSTALL_PARAMS_DATA = _INSTALL_PARAMS_DATA;
// PROPSHEETHEADERW: c2rust, devclass.c
pub type PROPSHEETHEADERW = _PROPSHEETHEADERW;
// RPC_WSTR: c2rust, misc.c
pub type RPC_WSTR = *mut ::core::ffi::c_ushort;
// VS_FIXEDFILEINFO: c2rust, misc.c
pub type VS_FIXEDFILEINFO = tagVS_FIXEDFILEINFO;
// _INSTALL_PARAMS_DATA: c2rust, devclass.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _INSTALL_PARAMS_DATA {
    pub Function: DI_FUNCTION,
    pub UpdateHandler: UPDATE_CLASS_PARAM_HANDLER,
    pub ParamsSize: ULONG,
    pub FieldOffset: LONG,
}
// field: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct field {
    pub text: *const WCHAR,
}
// inf_file: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct inf_file {
    pub next: *mut inf_file,
    pub strings: *mut WCHAR,
    pub string_pos: *mut WCHAR,
    pub nb_sections: ::core::ffi::c_uint,
    pub alloc_sections: ::core::ffi::c_uint,
    pub sections: *mut *mut section,
    pub nb_fields: ::core::ffi::c_uint,
    pub alloc_fields: ::core::ffi::c_uint,
    pub fields: *mut field,
    pub strings_section: ::core::ffi::c_int,
    pub filename: *mut WCHAR,
}
// line: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct line {
    pub first_field: ::core::ffi::c_int,
    pub nb_fields: ::core::ffi::c_int,
    pub key_field: ::core::ffi::c_int,
}
// section: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct section {
    pub name: *const WCHAR,
    pub nb_lines: ::core::ffi::c_uint,
    pub alloc_lines: ::core::ffi::c_uint,
    pub lines: [line; 16],
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snwprintf(
        _Dest: *mut wchar_t,
        _Count: size_t,
        _Format: *const wchar_t,
        ...
    ) -> ::core::ffi::c_int; // devinst.c
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // devprop.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // setupcab.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // setupcab.c
    pub fn strrchr(
        _Str: *const ::core::ffi::c_char,
        _Ch: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // misc.c
    pub fn strtol(
        _Str: *const ::core::ffi::c_char,
        _EndPtr: *mut *mut ::core::ffi::c_char,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long; // parser.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // cfgmgr.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // devinst.c
}
extern "system" {
    pub fn CM_Create_DevNode_ExW(
        pdnDevInst: PDEVINST,
        pDeviceID: DEVINSTID_W,
        dnParent: DEVINST,
        ulFlags: ULONG,
        hMachine: HANDLE,
    ) -> CONFIGRET; // devinst.c
    pub fn CM_Get_DevNode_Custom_Property_ExA(
        dnDevInst: DEVINST,
        pszCustomPropertyName: PCSTR,
        pulRegDataType: PULONG,
        Buffer: PVOID,
        pulLength: PULONG,
        ulFlags: ULONG,
        hMachine: HMACHINE,
    ) -> CONFIGRET; // devinst.c
    pub fn CM_Get_DevNode_Custom_Property_ExW(
        dnDevInst: DEVINST,
        pszCustomPropertyName: PCWSTR,
        pulRegDataType: PULONG,
        Buffer: PVOID,
        pulLength: PULONG,
        ulFlags: ULONG,
        hMachine: HMACHINE,
    ) -> CONFIGRET; // devinst.c
    pub fn CM_Get_Device_ID_ExW(
        dnDevInst: DEVINST,
        Buffer: PWCHAR,
        BufferLen: ULONG,
        ulFlags: ULONG,
        hMachine: HMACHINE,
    ) -> CONFIGRET; // devprop.c
    pub fn CM_Locate_DevNode_ExW(
        pdnDevInst: PDEVINST,
        pDeviceID: DEVINSTID_W,
        ulFlags: ULONG,
        hMachine: HMACHINE,
    ) -> CONFIGRET; // devclass.c
    pub fn CreateMutexW(_: LPSECURITY_ATTRIBUTES, _: BOOL, _: LPCWSTR) -> HANDLE; // cfgmgr.c
    pub fn GetLocaleInfoW(
        Locale: LCID,
        LCType: LCTYPE,
        lpLCData: LPWSTR,
        cchData: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // parser.c
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // devclass.c
    pub fn SetupDiGetClassInstallParamsW( DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA, ClassInstallParams: PSP_CLASSINSTALL_HEADER, ClassInstallParamsSize: DWORD, RequiredSize: PDWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn SetupDiSetClassInstallParamsW( DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA, ClassInstallParams: PSP_CLASSINSTALL_HEADER, ClassInstallParamsSize: DWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn UuidToStringW(Uuid: *mut UUID, StringUuid: *mut RPC_WSTR) -> RPC_STATUS; // cfgmgr.c
}
pub use crate::prelude::CM_Create_DevNode_ExW as CM_Create_DevInst_Ex; // sdk alias
pub use crate::prelude::CM_Create_DevNode_ExW as CM_Create_DevInst_ExW; // sdk alias
pub use crate::prelude::CM_Get_DevNode_Custom_Property_ExA as CM_Get_DevInst_Custom_Property_ExA; // sdk alias
pub use crate::prelude::CM_Get_DevNode_Custom_Property_ExW as CM_Get_DevInst_Custom_Property_ExW; // sdk alias
pub use crate::prelude::CM_Get_Device_ID_ExW as CM_Get_Device_ID_Ex; // sdk alias
pub use crate::prelude::CM_Locate_DevNode_ExW as CM_Locate_DevInst_ExW; // sdk alias
pub use crate::prelude::CM_Locate_DevNode_ExW as CM_Locate_DevNode_Ex; // sdk alias
pub use crate::prelude::CreateMutexW as CreateMutex; // sdk alias
pub use win32macros_rust::WINE_ERR as ERR; // sdk alias
pub use win32macros_rust::WINE_FIXME as FIXME; // sdk alias
pub use crate::prelude::GetLocaleInfoW as GetLocaleInfo; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::SetupDiGetClassInstallParamsW as SetupDiGetClassInstallParams; // sdk alias
pub use crate::prelude::SetupDiSetClassInstallParamsW as SetupDiSetClassInstallParams; // sdk alias
pub use win32macros_rust::WINE_TRACE as TRACE; // sdk alias
pub use win32macros_rust::WINE_WARN as WARN; // sdk alias
pub use crate::prelude::_snwprintf as snprintfW; // sdk alias
pub use crate::prelude::_swprintf as sprintfW; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub static mut __wine_dbch_setupapi: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x73\x65\x74\x75\x70\x61\x70\x69\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_setupapi };
// ==== end wine debug channels ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
// ==== end UNICODE TCHAR family ====

// winreg.h: `((HKEY)(ULONG_PTR)((LONG)0x80000002))` -- the LONG cast sign-extends, so on
// 64-bit the handle is 0xFFFFFFFF80000002, not 0x80000002.
pub const HKEY_LOCAL_MACHINE: HKEY = (0x80000002u32 as LONG) as LONG_PTR as HKEY;
// winbase.h: `((DWORD)0xFFFFFFFF)`.
pub const WAIT_FAILED: DWORD = 0xFFFFFFFF;

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::setupapi::devinst::CreateDeviceInfo;
pub use crate::setupapi::driver::CreateInfFileDetails;
pub use crate::setupapi::dirid::DIRID_get_string;
pub use crate::setupapi::driver::DereferenceInfFile;
pub use crate::setupapi::interface::DestroyDeviceInterface;
pub use crate::setupapi::driver::DestroyDriverInfoElement;
pub use crate::setupapi::parser::EnumerateSectionsStartingWith;
pub use crate::setupapi::misc::FreeFunctionPointer;
pub use crate::setupapi::devinst::GetErrorCodeFromCrCode;
pub use crate::setupapi::misc::GetFunctionPointer;
pub use crate::setupapi::install::GetStringField;
pub use crate::setupapi::misc::MyFree;
pub use crate::setupapi::misc::MyMalloc;
pub use crate::setupapi::parser::PARSER_get_dest_dir;
pub use crate::setupapi::parser::PARSER_get_inf_filename;
pub use crate::setupapi::parser::PARSER_get_src_root;
pub use crate::setupapi::rpc::PnpBindRpc;
pub use crate::setupapi::rpc::PnpGetLocalHandles;
pub use crate::setupapi::rpc::PnpUnbindRpc;
pub use crate::setupapi::queue::QUEUE_callback_WtoA;
pub use crate::setupapi::misc::QueryRegistryValue;
pub use crate::setupapi::devinst::SETUPDI_OpenDrvKey;
pub use crate::setupapi::devinst::SETUP_CreateClassKey;
pub use crate::setupapi::devclass::SETUP_CreateDevicesList;
pub use crate::setupapi::interface::SETUP_CreateInterfaceList;
pub use crate::setupapi::queue::SetupCloseFileQueue;
pub use crate::setupapi::parser::SetupCloseInfFile;
pub use crate::setupapi::misc::SetupCloseLog;
pub use crate::setupapi::queue::SetupCommitFileQueueW;
pub use crate::setupapi::install::SetupCopyOEMInfW;
pub use crate::setupapi::queue::SetupDefaultQueueCallbackW;
pub use crate::setupapi::devinst::SetupDiBuildClassInfoListExW;
pub use crate::setupapi::devinst::SetupDiCallClassInstaller;
pub use crate::setupapi::devinst::SetupDiClassNameFromGuidW;
pub use crate::setupapi::devinst::SetupDiCreateDeviceInfoList;
pub use crate::setupapi::devinst::SetupDiCreateDeviceInterfaceRegKeyW;
pub use crate::setupapi::devinst::SetupDiDestroyDeviceInfoList;
pub use crate::setupapi::devinst::SetupDiGetActualSectionToInstallW;
pub use crate::setupapi::devinst::SetupDiGetDeviceInstallParamsW;
pub use crate::setupapi::devinst::SetupDiGetDeviceRegistryPropertyW;
pub use crate::setupapi::parser::SetupDiGetINFClassW;
pub use crate::setupapi::devclass::SetupDiInstallClassExW;
pub use crate::setupapi::interface::SetupDiInstallDeviceInterfaces;
pub use crate::setupapi::driver::SetupDiInstallDriverFiles;
pub use crate::setupapi::devinst::SetupDiOpenClassRegKey;
pub use crate::setupapi::devinst::SetupDiOpenClassRegKeyExW;
pub use crate::setupapi::stubs::SetupDiRemoveDevice;
pub use crate::setupapi::driver::SetupDiSelectBestCompatDrv;
pub use crate::setupapi::devinst::SetupDiSetDeviceInstallParamsW;
pub use crate::setupapi::devinst::SetupDiSetDeviceRegistryPropertyW;
pub use crate::setupapi::stubs::SetupDiUnremoveDevice;
pub use crate::setupapi::parser::SetupFindFirstLineW;
pub use crate::setupapi::parser::SetupFindNextLine;
pub use crate::setupapi::parser::SetupFindNextMatchLineW;
pub use crate::setupapi::parser::SetupGetBinaryField;
pub use crate::setupapi::parser::SetupGetFieldCount;
pub use crate::setupapi::parser::SetupGetIntField;
pub use crate::setupapi::parser::SetupGetLineTextW;
pub use crate::setupapi::parser::SetupGetMultiSzFieldW;
pub use crate::setupapi::parser::SetupGetStringFieldW;
pub use crate::setupapi::queue::SetupInitDefaultQueueCallback;
pub use crate::setupapi::install::SetupInstallFromInfSectionW;
pub use crate::setupapi::install::SetupInstallServicesFromInfSectionExW;
pub use crate::setupapi::setupcab::SetupIterateCabinetW;
pub use crate::setupapi::parser::SetupOpenAppendInfFileW;
pub use crate::setupapi::queue::SetupOpenFileQueue;
pub use crate::setupapi::parser::SetupOpenInfFileW;
pub use crate::setupapi::stubs::SetupPromptReboot;
pub use crate::setupapi::queue::SetupQueueCopySectionW;
pub use crate::setupapi::queue::SetupQueueDefaultCopyW;
pub use crate::setupapi::queue::SetupQueueDeleteSectionW;
pub use crate::setupapi::queue::SetupQueueRenameSectionW;
pub use crate::setupapi::queue::SetupTermDefaultQueueCallback;
pub use crate::setupapi::devclass::UPDATE_CLASS_PARAM_HANDLER;
pub use crate::setupapi::misc::pSetupCaptureAndConvertAnsiArg;
pub use crate::setupapi::misc::pSetupDuplicateString;
pub use crate::setupapi::misc::pSetupIsUserAdmin;
pub use crate::setupapi::misc::pSetupMultiByteToUnicode;
pub use crate::setupapi::misc::pSetupStringFromGuid;
pub use crate::setupapi::stringtable::pSetupStringTableAddString;
pub use crate::setupapi::stringtable::pSetupStringTableDestroy;
pub use crate::setupapi::stringtable::pSetupStringTableInitialize;
pub use crate::setupapi::stringtable::pSetupStringTableStringFromId;
pub use crate::setupapi::stringtable::pSetupStringTableStringFromIdEx;
// ==== end cross-TU definitions ====
