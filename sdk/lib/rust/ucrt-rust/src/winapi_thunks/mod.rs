//! `internal/winapi_thunks.cpp` -- the `__acrt_*` wrappers over Windows APIs
//! that not every supported OS has -- plus, in `narrow.rs`, the six narrow
//! (`*A`) wrappers `internal/{CreateProcessA,GetModuleFileNameA,LoadLibraryExA,
//! OutputDebugStringA,SetCurrentDirectoryA,SetEnvironmentVariableA}.cpp`.
//! Replaces all seven files in full.
//!
//! Each late-bound API is looked up ONCE, by `GetProcAddress` on the first of
//! its candidate modules that loads (an api-set first, then the classic DLL),
//! and the answer is cached XOR-encoded with the GS cookie
//! (`__crt_fast_encode_pointer`) -- an encoded -1 meaning "looked, not there".
//! When the API is missing each thunk falls back exactly as the C chooses: the
//! Vista `*Ex` NLS calls go to their LCID forms through `__acrt_LocaleNameToLCID`,
//! the i386 `Fls*` go to `Tls*`, and the ones with no fallback `abort()`.
//!
//! What runs on ReactOS: kernel32 exports every Vista NLS `*Ex` API the table
//! names, `InitializeCriticalSectionEx`, `GetSystemTimePreciseAsFileTime`,
//! `SetThreadStackGuarantee` and the `Fls*` set, so those take the direct path;
//! `GetTempPath2W` and the three XState APIs are absent (GetTempPathW fallback;
//! `__acrt_can_use_xstate_apis` says no), and whether an api-set name loads at
//! all decides which module a lookup lands in, never whether the fallback runs
//! (the first module that LOADS is the only one asked).
//!
//! ## Details that are silent if wrong
//!
//! * **Only the first module that loads is asked for the function.** If an
//!   api-set DLL loads but lacks the export, the classic DLL behind it is NOT
//!   tried; the lookup fails and is cached as failed.
//! * **The cache starts as encoded NULL, set by `__acrt_initialize_winapi_thunks`**
//!   (the static zero is not "empty": decoded it is the cookie). A thunk called
//!   before initialization reads a bogus pointer, as the C does.
//! * **`LOAD_LIBRARY_SEARCH_SYSTEM32` first; the flag-less retry happens only
//!   for `ERROR_INVALID_PARAMETER` and never for an `api-ms-`/`ext-ms-` name**
//!   (DLL-hijack guard on pre-Win7 loaders).
//! * **Module handles are freed on uninitialize unless the process is
//!   terminating**; the function-pointer cache is NOT reset there.
//! * **The EnumSystemLocalesEx fallback holds the locale lock** and parks the
//!   caller's callback, encoded, in a static for the LCID-era enumerator to
//!   call back through with `(name, 0, 0)`.
//! * **i386 only resolves the `Fls*` functions at run time** (fibers api-set,
//!   then kernel32, else the `Tls*` fallback); every 64-bit target calls
//!   kernel32's `Fls*` directly (`FLS_ALWAYS_AVAILABLE`).
//! * **WINAPI vs cdecl**: every `__acrt_<Api>` thunk here is `WINAPI`
//!   (`extern "system"`, `@N`-decorated on i386) EXCEPT the `bool`/`HWND`
//!   helpers at the bottom and five of the six narrow wrappers, which are
//!   `__cdecl`. `__acrt_OutputDebugStringA` is the one narrow wrapper that is
//!   `WINAPI`. (Checked against the compiled i386 objects' symbol names.)

mod narrow;

pub use narrow::{
    __acrt_CreateProcessA, __acrt_GetModuleFileNameA, __acrt_LoadLibraryExA, __acrt_OutputDebugStringA,
    __acrt_SetCurrentDirectoryA, __acrt_SetEnvironmentVariableA,
};

use crate::locale_abi::{__acrt_DownlevelLCIDToLocaleName, __acrt_DownlevelLocaleNameToLCID};
use crate::stdio_layout::CriticalSection;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

type Handle = *mut c_void;
/// `LOCALE_ENUMPROCEX` / `LOCALE_ENUMPROCW` (as `qualified_locale.rs` spells them;
/// LPARAM is pointer-sized).
type LocaleEnumProcEx = unsafe extern "system" fn(*mut u16, u32, isize) -> c_int;
type LocaleEnumProcW = unsafe extern "system" fn(*mut u16) -> c_int;

const LOAD_LIBRARY_SEARCH_SYSTEM32: u32 = 0x0000_0800;
const ERROR_INVALID_PARAMETER: u32 = 87;
const LCID_INSTALLED: u32 = 1;
/// `STATUS_NOT_FOUND`, returned as a `LONG`.
const STATUS_NOT_FOUND: i32 = 0xC000_0225_u32 as i32;
const S_OK: i32 = 0;
const UOI_FLAGS: c_int = 1;
const WSF_VISIBLE: u32 = 1;
/// `__acrt_locale_lock`.
const LOCALE_LOCK: c_int = 4;
/// `windowing_model_policy_hwnd` (corecrt_internal.h).
const WINDOWING_MODEL_POLICY_HWND: c_int = 1;
/// `GetCurrentThreadEffectiveToken()` (processthreadsapi.h): the pseudo-handle -6.
const CURRENT_THREAD_EFFECTIVE_TOKEN: Handle = -6isize as Handle;
/// `INVALID_HANDLE_VALUE` / `invalid_function_sentinel()`: both -1.
const SENTINEL: usize = usize::MAX;

/// `USEROBJECTFLAGS` (winuser.h).
#[repr(C)]
struct UserObjectFlags {
    inherit: c_int,
    reserved: c_int,
    flags: u32,
}

extern "system" {
    fn LoadLibraryExW(file_name: *const u16, file: Handle, flags: u32) -> Handle;
    fn FreeLibrary(module: *mut c_void) -> c_int;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn GetLastError() -> u32;
    fn CompareStringW(
        locale: u32,
        flags: u32,
        string1: *const u16,
        count1: c_int,
        string2: *const u16,
        count2: c_int,
    ) -> c_int;
    fn EnumSystemLocalesW(enum_proc: LocaleEnumProcW, flags: u32) -> c_int;
    fn GetDateFormatW(
        locale: u32,
        flags: u32,
        date: *const c_void,
        format: *const u16,
        buffer: *mut u16,
        buffer_count: c_int,
    ) -> c_int;
    fn GetTempPathW(buffer_length: u32, buffer: *mut u16) -> u32;
    fn GetLocaleInfoW(locale: u32, lc_type: u32, data: *mut u16, cch_data: c_int) -> c_int;
    fn GetSystemTimeAsFileTime(ft: *mut u64);
    fn GetTimeFormatW(
        locale: u32,
        flags: u32,
        time: *const c_void,
        format: *const u16,
        buffer: *mut u16,
        buffer_count: c_int,
    ) -> c_int;
    fn GetUserDefaultLCID() -> u32;
    fn InitializeCriticalSectionAndSpinCount(cs: *mut CriticalSection, spin_count: u32) -> c_int;
    fn IsValidLocale(locale: u32, flags: u32) -> c_int;
    fn LCMapStringW(
        locale: u32,
        flags: u32,
        source: *const u16,
        source_count: c_int,
        destination: *mut u16,
        destination_count: c_int,
    ) -> c_int;
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn FlsAlloc(callback: Option<unsafe extern "system" fn(*mut c_void)>) -> u32;
    fn FlsFree(index: u32) -> c_int;
    fn FlsGetValue(index: u32) -> *mut c_void;
    fn FlsSetValue(index: u32, value: *mut c_void) -> c_int;
}

#[cfg(target_arch = "x86")]
extern "system" {
    fn TlsAlloc() -> u32;
    fn TlsFree(index: u32) -> c_int;
    fn TlsGetValue(index: u32) -> *mut c_void;
    fn TlsSetValue(index: u32, value: *mut c_void) -> c_int;
}

extern "C" {
    fn abort() -> !;
    fn wcsncmp(a: *const u16, b: *const u16, count: usize) -> c_int;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    /// `windowing_model_policy __cdecl __acrt_get_windowing_model_policy(void)`
    /// (win_policies.cpp); the enum is `int`-sized.
    fn __acrt_get_windowing_model_policy() -> c_int;
    /// The GS cookie `__crt_fast_encode_pointer` XORs with (crt-startup-rust's gs.rs).
    static mut __security_cookie: usize;
}

/// `__crt_fast_encode_pointer` == `__crt_fast_decode_pointer`: XOR with the
/// cookie.
#[inline]
unsafe fn encode(p: usize) -> usize {
    p ^ core::ptr::addr_of!(__security_cookie).read()
}
#[inline]
unsafe fn decode(p: usize) -> usize {
    p ^ core::ptr::addr_of!(__security_cookie).read()
}

/// Wide string literal, NUL-terminated, from an ASCII `&str`.
macro_rules! w {
    ($s:literal) => {{
        const S: &str = $s;
        const N: usize = S.len() + 1;
        const W: [u16; N] = {
            let b = S.as_bytes();
            let mut w = [0u16; N];
            let mut i = 0;
            while i < b.len() {
                w[i] = b[i] as u16;
                i += 1;
            }
            w
        };
        &W
    }};
}

// ── the module table (`_ACRT_APPLY_TO_LATE_BOUND_MODULES`) ──────────────────

const API_MS_WIN_CORE_DATETIME_L1_1_1: usize = 0;
const API_MS_WIN_CORE_FILE_L1_2_4: usize = 1;
#[allow(dead_code)] // in the table; its one user (AreFileApisANSI) is commented out in the C
const API_MS_WIN_CORE_FILE_L1_2_2: usize = 2;
const API_MS_WIN_CORE_LOCALIZATION_L1_2_1: usize = 3;
const API_MS_WIN_CORE_LOCALIZATION_OBSOLETE_L1_2_0: usize = 4;
const API_MS_WIN_CORE_PROCESSTHREADS_L1_1_2: usize = 5;
const API_MS_WIN_CORE_STRING_L1_1_0: usize = 6;
const API_MS_WIN_CORE_SYNCH_L1_2_0: usize = 7;
const API_MS_WIN_CORE_SYSINFO_L1_2_1: usize = 8;
const API_MS_WIN_CORE_WINRT_L1_1_0: usize = 9;
const API_MS_WIN_CORE_XSTATE_L2_1_0: usize = 10;
const API_MS_WIN_RTCORE_NTUSER_WINDOW_L1_1_0: usize = 11;
const API_MS_WIN_SECURITY_SYSTEMFUNCTIONS_L1_1_0: usize = 12;
const EXT_MS_WIN_NTUSER_DIALOGBOX_L1_1_0: usize = 13;
const EXT_MS_WIN_NTUSER_WINDOWSTATION_L1_1_0: usize = 14;
const ADVAPI32: usize = 15;
const KERNEL32: usize = 16;
const KERNELBASE: usize = 17;
#[allow(dead_code)] // in the C's table, named by no function
const NTDLL: usize = 18;
const API_MS_WIN_APPMODEL_RUNTIME_L1_1_2: usize = 19;
const USER32: usize = 20;
#[cfg(target_arch = "x86")]
const API_MS_WIN_CORE_FIBERS_L1_1_0: usize = 21;

#[cfg(not(target_arch = "x86"))]
const MODULE_ID_COUNT: usize = 21;
#[cfg(target_arch = "x86")]
const MODULE_ID_COUNT: usize = 22;

static MODULE_NAMES: [&[u16]; MODULE_ID_COUNT] = [
    w!("api-ms-win-core-datetime-l1-1-1"),
    w!("api-ms-win-core-file-l1-2-4"),
    w!("api-ms-win-core-file-l1-2-2"),
    w!("api-ms-win-core-localization-l1-2-1"),
    w!("api-ms-win-core-localization-obsolete-l1-2-0"),
    w!("api-ms-win-core-processthreads-l1-1-2"),
    w!("api-ms-win-core-string-l1-1-0"),
    w!("api-ms-win-core-synch-l1-2-0"),
    w!("api-ms-win-core-sysinfo-l1-2-1"),
    w!("api-ms-win-core-winrt-l1-1-0"),
    w!("api-ms-win-core-xstate-l2-1-0"),
    w!("api-ms-win-rtcore-ntuser-window-l1-1-0"),
    w!("api-ms-win-security-systemfunctions-l1-1-0"),
    w!("ext-ms-win-ntuser-dialogbox-l1-1-0"),
    w!("ext-ms-win-ntuser-windowstation-l1-1-0"),
    w!("advapi32"),
    w!("kernel32"),
    w!("kernelbase"),
    w!("ntdll"),
    w!("api-ms-win-appmodel-runtime-l1-1-2"),
    w!("user32"),
    #[cfg(target_arch = "x86")]
    w!("api-ms-win-core-fibers-l1-1-0"),
];

// ── the function table (`_ACRT_APPLY_TO_LATE_BOUND_FUNCTIONS`) ──────────────

struct LateBound {
    name: &'static [u8],
    modules: &'static [usize],
}

const ARE_FILE_APIS_ANSI: usize = 0;
const COMPARE_STRING_EX: usize = 1;
const ENUM_SYSTEM_LOCALES_EX: usize = 2;
const GET_ACTIVE_WINDOW: usize = 3;
const GET_DATE_FORMAT_EX: usize = 4;
const GET_TEMP_PATH2_W: usize = 5;
const GET_ENABLED_XSTATE_FEATURES: usize = 6;
const GET_LAST_ACTIVE_POPUP: usize = 7;
const GET_LOCALE_INFO_EX: usize = 8;
const GET_PROCESS_WINDOW_STATION: usize = 9;
const GET_SYSTEM_TIME_PRECISE_AS_FILE_TIME: usize = 10;
const GET_TIME_FORMAT_EX: usize = 11;
const GET_USER_DEFAULT_LOCALE_NAME: usize = 12;
const GET_USER_OBJECT_INFORMATION_W: usize = 13;
const GET_XSTATE_FEATURES_MASK: usize = 14;
const INITIALIZE_CRITICAL_SECTION_EX: usize = 15;
const IS_VALID_LOCALE_NAME: usize = 16;
const LC_MAP_STRING_EX: usize = 17;
const LCID_TO_LOCALE_NAME: usize = 18;
const LOCALE_NAME_TO_LCID: usize = 19;
const LOCATE_XSTATE_FEATURE: usize = 20;
const MESSAGE_BOX_A: usize = 21;
const MESSAGE_BOX_W: usize = 22;
const RO_INITIALIZE: usize = 23;
const RO_UNINITIALIZE: usize = 24;
const APP_POLICY_GET_PROCESS_TERMINATION_METHOD: usize = 25;
const APP_POLICY_GET_THREAD_INITIALIZATION_TYPE: usize = 26;
const APP_POLICY_GET_SHOW_DEVELOPER_DIAGNOSTIC: usize = 27;
const APP_POLICY_GET_WINDOWING_MODEL: usize = 28;
const SET_THREAD_STACK_GUARANTEE: usize = 29;
const SYSTEM_FUNCTION_036: usize = 30;
#[cfg(target_arch = "x86")]
const FLS_ALLOC: usize = 31;
#[cfg(target_arch = "x86")]
const FLS_FREE: usize = 32;
#[cfg(target_arch = "x86")]
const FLS_GET_VALUE: usize = 33;
#[cfg(target_arch = "x86")]
const FLS_SET_VALUE: usize = 34;

#[cfg(not(target_arch = "x86"))]
const FUNCTION_ID_COUNT: usize = 31;
#[cfg(target_arch = "x86")]
const FUNCTION_ID_COUNT: usize = 35;

const fn lb(name: &'static [u8], modules: &'static [usize]) -> LateBound {
    LateBound { name, modules }
}

static FUNCTIONS: [LateBound; FUNCTION_ID_COUNT] = [
    // `/* api_ms_win_core_file_l1_2_2, */ kernel32` -- the api-set is commented out in the C.
    lb(b"AreFileApisANSI\0", &[KERNEL32]),
    lb(b"CompareStringEx\0", &[API_MS_WIN_CORE_STRING_L1_1_0, KERNEL32]),
    lb(b"EnumSystemLocalesEx\0", &[API_MS_WIN_CORE_LOCALIZATION_L1_2_1, KERNEL32]),
    lb(b"GetActiveWindow\0", &[API_MS_WIN_RTCORE_NTUSER_WINDOW_L1_1_0, USER32]),
    lb(b"GetDateFormatEx\0", &[API_MS_WIN_CORE_DATETIME_L1_1_1, KERNEL32]),
    lb(b"GetTempPath2W\0", &[API_MS_WIN_CORE_FILE_L1_2_4, KERNELBASE]),
    lb(b"GetEnabledXStateFeatures\0", &[API_MS_WIN_CORE_XSTATE_L2_1_0, KERNEL32]),
    lb(b"GetLastActivePopup\0", &[EXT_MS_WIN_NTUSER_DIALOGBOX_L1_1_0, USER32]),
    lb(b"GetLocaleInfoEx\0", &[API_MS_WIN_CORE_LOCALIZATION_L1_2_1, KERNEL32]),
    lb(b"GetProcessWindowStation\0", &[EXT_MS_WIN_NTUSER_WINDOWSTATION_L1_1_0, USER32]),
    lb(b"GetSystemTimePreciseAsFileTime\0", &[API_MS_WIN_CORE_SYSINFO_L1_2_1]),
    lb(b"GetTimeFormatEx\0", &[API_MS_WIN_CORE_DATETIME_L1_1_1, KERNEL32]),
    lb(b"GetUserDefaultLocaleName\0", &[API_MS_WIN_CORE_LOCALIZATION_L1_2_1, KERNEL32]),
    lb(b"GetUserObjectInformationW\0", &[EXT_MS_WIN_NTUSER_WINDOWSTATION_L1_1_0, USER32]),
    lb(b"GetXStateFeaturesMask\0", &[API_MS_WIN_CORE_XSTATE_L2_1_0, KERNEL32]),
    lb(b"InitializeCriticalSectionEx\0", &[API_MS_WIN_CORE_SYNCH_L1_2_0, KERNEL32]),
    lb(b"IsValidLocaleName\0", &[API_MS_WIN_CORE_LOCALIZATION_L1_2_1, KERNEL32]),
    lb(b"LCMapStringEx\0", &[API_MS_WIN_CORE_LOCALIZATION_L1_2_1, KERNEL32]),
    lb(b"LCIDToLocaleName\0", &[API_MS_WIN_CORE_LOCALIZATION_OBSOLETE_L1_2_0, KERNEL32]),
    lb(b"LocaleNameToLCID\0", &[API_MS_WIN_CORE_LOCALIZATION_L1_2_1, KERNEL32]),
    lb(b"LocateXStateFeature\0", &[API_MS_WIN_CORE_XSTATE_L2_1_0, KERNEL32]),
    lb(b"MessageBoxA\0", &[EXT_MS_WIN_NTUSER_DIALOGBOX_L1_1_0, USER32]),
    lb(b"MessageBoxW\0", &[EXT_MS_WIN_NTUSER_DIALOGBOX_L1_1_0, USER32]),
    lb(b"RoInitialize\0", &[API_MS_WIN_CORE_WINRT_L1_1_0]),
    lb(b"RoUninitialize\0", &[API_MS_WIN_CORE_WINRT_L1_1_0]),
    lb(b"AppPolicyGetProcessTerminationMethod\0", &[API_MS_WIN_APPMODEL_RUNTIME_L1_1_2]),
    lb(b"AppPolicyGetThreadInitializationType\0", &[API_MS_WIN_APPMODEL_RUNTIME_L1_1_2]),
    lb(b"AppPolicyGetShowDeveloperDiagnostic\0", &[API_MS_WIN_APPMODEL_RUNTIME_L1_1_2]),
    lb(b"AppPolicyGetWindowingModel\0", &[API_MS_WIN_APPMODEL_RUNTIME_L1_1_2]),
    lb(b"SetThreadStackGuarantee\0", &[API_MS_WIN_CORE_PROCESSTHREADS_L1_1_2, KERNEL32]),
    lb(b"SystemFunction036\0", &[API_MS_WIN_SECURITY_SYSTEMFUNCTIONS_L1_1_0, ADVAPI32]),
    #[cfg(target_arch = "x86")]
    lb(b"FlsAlloc\0", &[API_MS_WIN_CORE_FIBERS_L1_1_0, KERNEL32]),
    #[cfg(target_arch = "x86")]
    lb(b"FlsFree\0", &[API_MS_WIN_CORE_FIBERS_L1_1_0, KERNEL32]),
    #[cfg(target_arch = "x86")]
    lb(b"FlsGetValue\0", &[API_MS_WIN_CORE_FIBERS_L1_1_0, KERNEL32]),
    #[cfg(target_arch = "x86")]
    lb(b"FlsSetValue\0", &[API_MS_WIN_CORE_FIBERS_L1_1_0, KERNEL32]),
];

/// `module_handles`: 0 = not tried yet, -1 = tried and failed.
static MODULE_HANDLES: [AtomicUsize; MODULE_ID_COUNT] = [const { AtomicUsize::new(0) }; MODULE_ID_COUNT];

/// `encoded_function_pointers`: encoded 0 = not tried yet, encoded -1 = tried
/// and failed.
static ENCODED_FUNCTION_POINTERS: [AtomicUsize; FUNCTION_ID_COUNT] =
    [const { AtomicUsize::new(0) }; FUNCTION_ID_COUNT];

/// `__acrt_initialize_winapi_thunks`: every cache slot to encoded NULL.
///
/// # Safety
/// CRT startup, before any thunk runs.
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_winapi_thunks() -> bool {
    let encoded_nullptr = encode(0);
    for p in ENCODED_FUNCTION_POINTERS.iter() {
        p.store(encoded_nullptr, Ordering::Relaxed);
    }
    true
}

/// `__acrt_uninitialize_winapi_thunks`: free the loaded modules, unless the
/// process is terminating anyway.
///
/// # Safety
/// CRT shutdown.
#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_winapi_thunks(terminating: bool) -> bool {
    if terminating {
        return true;
    }
    for module in MODULE_HANDLES.iter() {
        let m = module.load(Ordering::Relaxed);
        if m != 0 {
            if m != SENTINEL {
                FreeLibrary(m as Handle);
            }
            module.store(0, Ordering::Relaxed);
        }
    }
    true
}

/// `try_load_library_from_system_directory`.
unsafe fn try_load_library_from_system_directory(name: *const u16) -> Handle {
    let handle = LoadLibraryExW(name, null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32);
    if !handle.is_null() {
        return handle;
    }

    // Pre-Win7 loaders reject the flag; retry without it, but never for an
    // api-set name (all forwarders there; the caller falls back to the real
    // DLL, which is a known DLL).
    if GetLastError() == ERROR_INVALID_PARAMETER
        && wcsncmp(name, w!("api-ms-").as_ptr(), 7) != 0
        && wcsncmp(name, w!("ext-ms-").as_ptr(), 7) != 0
    {
        return LoadLibraryExW(name, null_mut(), 0);
    }

    null_mut()
}

/// `try_get_module`.
unsafe fn try_get_module(id: usize) -> Handle {
    let cached_handle = MODULE_HANDLES[id].load(Ordering::Relaxed);
    if cached_handle != 0 {
        if cached_handle == SENTINEL {
            return null_mut();
        }
        return cached_handle as Handle;
    }

    let new_handle = try_load_library_from_system_directory(MODULE_NAMES[id].as_ptr());
    if new_handle.is_null() {
        MODULE_HANDLES[id].swap(SENTINEL, Ordering::SeqCst);
        return null_mut();
    }

    // Another thread may have cached it meanwhile: drop our extra reference.
    if MODULE_HANDLES[id].swap(new_handle as usize, Ordering::SeqCst) != 0 {
        FreeLibrary(new_handle);
    }

    new_handle
}

/// `try_get_first_available_module`.
unsafe fn try_get_first_available_module(modules: &[usize]) -> Handle {
    for &id in modules {
        let handle = try_get_module(id);
        if !handle.is_null() {
            return handle;
        }
    }
    null_mut()
}

/// `try_get_function`: the cached pointer, or look it up and cache the answer.
unsafe fn try_get_function(id: usize) -> usize {
    let cached_fp = decode(ENCODED_FUNCTION_POINTERS[id].load(Ordering::Relaxed));
    if cached_fp == SENTINEL {
        return 0;
    }
    if cached_fp != 0 {
        return cached_fp;
    }

    // `try_get_proc_address_from_first_available_module`
    let f = &FUNCTIONS[id];
    let module_handle = try_get_first_available_module(f.modules);
    let new_fp =
        if module_handle.is_null() { 0 } else { GetProcAddress(module_handle, f.name.as_ptr().cast()) as usize };

    if new_fp == 0 {
        ENCODED_FUNCTION_POINTERS[id].swap(encode(SENTINEL), Ordering::SeqCst);
        return 0;
    }

    ENCODED_FUNCTION_POINTERS[id].swap(encode(new_fp), Ordering::SeqCst);
    new_fp
}

/// `try_get_<Function>()`, typed: `None` when the API is absent.
macro_rules! try_get {
    ($id:expr, $ty:ty) => {{
        let p = try_get_function($id);
        if p == 0 {
            None
        } else {
            Some(core::mem::transmute::<usize, $ty>(p))
        }
    }};
}

// ── the thunks ──────────────────────────────────────────────────────────────

/// `__acrt_AreFileApisANSI`: absent means the file APIs are ANSI.
#[no_mangle]
pub unsafe extern "system" fn __acrt_AreFileApisANSI() -> c_int {
    if let Some(are_file_apis_ansi) = try_get!(ARE_FILE_APIS_ANSI, unsafe extern "system" fn() -> c_int) {
        return are_file_apis_ansi();
    }
    1
}

/// `__acrt_CompareStringEx`. (The C's `scoped_global_state_reset` OS-mode guard
/// is empty without state isolation.)
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "system" fn __acrt_CompareStringEx(
    locale_name: *const u16,
    flags: u32,
    string1: *const u16,
    string1_count: c_int,
    string2: *const u16,
    string2_count: c_int,
    version: *mut c_void,
    reserved: *mut c_void,
    param: isize,
) -> c_int {
    type F = unsafe extern "system" fn(
        *const u16,
        u32,
        *const u16,
        c_int,
        *const u16,
        c_int,
        *mut c_void,
        *mut c_void,
        isize,
    ) -> c_int;
    if let Some(compare_string_ex) = try_get!(COMPARE_STRING_EX, F) {
        return compare_string_ex(locale_name, flags, string1, string1_count, string2, string2_count, version, reserved, param);
    }
    CompareStringW(__acrt_LocaleNameToLCID(locale_name, 0), flags, string1, string1_count, string2, string2_count)
}

/// `static_enum_proc`, encoded: the caller's callback while the LCID-era
/// enumeration runs.
static STATIC_ENUM_PROC: AtomicUsize = AtomicUsize::new(0);

/// `LocaleEnumProcW` (the C's `__clang__` branch): forward each name as
/// `(name, 0, 0)`.
unsafe extern "system" fn locale_enum_proc_w(locale_string: *mut u16) -> c_int {
    let p = decode(STATIC_ENUM_PROC.load(Ordering::Relaxed));
    let enum_proc = core::mem::transmute::<usize, LocaleEnumProcEx>(p);
    enum_proc(locale_string, 0, 0)
}

/// `enum_system_locales_ex_nolock`.
unsafe fn enum_system_locales_ex_nolock(enum_proc: LocaleEnumProcEx) -> c_int {
    STATIC_ENUM_PROC.store(encode(enum_proc as usize), Ordering::Relaxed);
    let result = EnumSystemLocalesW(locale_enum_proc_w, LCID_INSTALLED);
    STATIC_ENUM_PROC.store(encode(0), Ordering::Relaxed);
    result
}

/// `__acrt_EnumSystemLocalesEx`. The fallback ignores `flags`, `param` and
/// `reserved`, as the C does, and runs under the locale lock.
#[no_mangle]
pub unsafe extern "system" fn __acrt_EnumSystemLocalesEx(
    enum_proc: LocaleEnumProcEx,
    flags: u32,
    param: isize,
    reserved: *mut c_void,
) -> c_int {
    type F = unsafe extern "system" fn(LocaleEnumProcEx, u32, isize, *mut c_void) -> c_int;
    if let Some(enum_system_locales_ex) = try_get!(ENUM_SYSTEM_LOCALES_EX, F) {
        return enum_system_locales_ex(enum_proc, flags, param, reserved);
    }

    // `__acrt_lock_and_call(__acrt_locale_lock, ...)`
    __acrt_lock(LOCALE_LOCK);
    let result = enum_system_locales_ex_nolock(enum_proc);
    __acrt_unlock(LOCALE_LOCK);
    result
}

/// `__acrt_FlsAlloc`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_FlsAlloc(callback: Option<unsafe extern "system" fn(*mut c_void)>) -> u32 {
    #[cfg(not(target_arch = "x86"))]
    {
        FlsAlloc(callback)
    }
    #[cfg(target_arch = "x86")]
    {
        type F = unsafe extern "system" fn(Option<unsafe extern "system" fn(*mut c_void)>) -> u32;
        if let Some(fls_alloc) = try_get!(FLS_ALLOC, F) {
            return fls_alloc(callback);
        }
        TlsAlloc()
    }
}

/// `__acrt_FlsFree`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_FlsFree(fls_index: u32) -> c_int {
    #[cfg(not(target_arch = "x86"))]
    {
        FlsFree(fls_index)
    }
    #[cfg(target_arch = "x86")]
    {
        if let Some(fls_free) = try_get!(FLS_FREE, unsafe extern "system" fn(u32) -> c_int) {
            return fls_free(fls_index);
        }
        TlsFree(fls_index)
    }
}

/// `__acrt_FlsGetValue`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_FlsGetValue(fls_index: u32) -> *mut c_void {
    #[cfg(not(target_arch = "x86"))]
    {
        FlsGetValue(fls_index)
    }
    #[cfg(target_arch = "x86")]
    {
        if let Some(fls_get_value) = try_get!(FLS_GET_VALUE, unsafe extern "system" fn(u32) -> *mut c_void) {
            return fls_get_value(fls_index);
        }
        TlsGetValue(fls_index)
    }
}

/// `__acrt_FlsSetValue`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_FlsSetValue(fls_index: u32, fls_data: *mut c_void) -> c_int {
    #[cfg(not(target_arch = "x86"))]
    {
        FlsSetValue(fls_index, fls_data)
    }
    #[cfg(target_arch = "x86")]
    {
        if let Some(fls_set_value) = try_get!(FLS_SET_VALUE, unsafe extern "system" fn(u32, *mut c_void) -> c_int) {
            return fls_set_value(fls_index, fls_data);
        }
        TlsSetValue(fls_index, fls_data)
    }
}

/// `__acrt_GetDateFormatEx`. The fallback drops `calendar`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetDateFormatEx(
    locale_name: *const u16,
    flags: u32,
    date: *const c_void,
    format: *const u16,
    buffer: *mut u16,
    buffer_count: c_int,
    calendar: *const u16,
) -> c_int {
    type F = unsafe extern "system" fn(*const u16, u32, *const c_void, *const u16, *mut u16, c_int, *const u16) -> c_int;
    if let Some(get_date_format_ex) = try_get!(GET_DATE_FORMAT_EX, F) {
        return get_date_format_ex(locale_name, flags, date, format, buffer, buffer_count, calendar);
    }
    GetDateFormatW(__acrt_LocaleNameToLCID(locale_name, 0), flags, date, format, buffer, buffer_count)
}

/// `__acrt_GetTempPath2W` -- `int` in the C, `DWORD` from both APIs; `u32`
/// here to match `stdio/bufio/tmp.rs`'s declaration (same register).
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetTempPath2W(n_buffer_length: u32, lp_buffer: *mut u16) -> u32 {
    if let Some(get_temp_path2w) = try_get!(GET_TEMP_PATH2_W, unsafe extern "system" fn(u32, *mut u16) -> u32) {
        return get_temp_path2w(n_buffer_length, lp_buffer);
    }
    GetTempPathW(n_buffer_length, lp_buffer)
}

/// `__acrt_GetEnabledXStateFeatures`: no fallback -- callers check
/// `__acrt_can_use_xstate_apis` first.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetEnabledXStateFeatures() -> u64 {
    if let Some(get_enabled_xstate_features) =
        try_get!(GET_ENABLED_XSTATE_FEATURES, unsafe extern "system" fn() -> u64)
    {
        return get_enabled_xstate_features();
    }
    abort()
}

/// `__acrt_GetLocaleInfoEx`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetLocaleInfoEx(
    locale_name: *const u16,
    lc_type: u32,
    data: *mut u16,
    data_count: c_int,
) -> c_int {
    type F = unsafe extern "system" fn(*const u16, u32, *mut u16, c_int) -> c_int;
    if let Some(get_locale_info_ex) = try_get!(GET_LOCALE_INFO_EX, F) {
        return get_locale_info_ex(locale_name, lc_type, data, data_count);
    }
    GetLocaleInfoW(__acrt_LocaleNameToLCID(locale_name, 0), lc_type, data, data_count)
}

/// `__acrt_GetSystemTimePreciseAsFileTime`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetSystemTimePreciseAsFileTime(system_time: *mut u64) {
    if let Some(get_system_time_precise_as_file_time) =
        try_get!(GET_SYSTEM_TIME_PRECISE_AS_FILE_TIME, unsafe extern "system" fn(*mut u64))
    {
        return get_system_time_precise_as_file_time(system_time);
    }
    GetSystemTimeAsFileTime(system_time)
}

/// `__acrt_GetTimeFormatEx`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetTimeFormatEx(
    locale_name: *const u16,
    flags: u32,
    time: *const c_void,
    format: *const u16,
    buffer: *mut u16,
    buffer_count: c_int,
) -> c_int {
    type F = unsafe extern "system" fn(*const u16, u32, *const c_void, *const u16, *mut u16, c_int) -> c_int;
    if let Some(get_time_format_ex) = try_get!(GET_TIME_FORMAT_EX, F) {
        return get_time_format_ex(locale_name, flags, time, format, buffer, buffer_count);
    }
    GetTimeFormatW(__acrt_LocaleNameToLCID(locale_name, 0), flags, time, format, buffer, buffer_count)
}

/// `__acrt_GetUserDefaultLocaleName`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetUserDefaultLocaleName(locale_name: *mut u16, locale_name_count: c_int) -> c_int {
    type F = unsafe extern "system" fn(*mut u16, c_int) -> c_int;
    if let Some(get_user_default_locale_name) = try_get!(GET_USER_DEFAULT_LOCALE_NAME, F) {
        return get_user_default_locale_name(locale_name, locale_name_count);
    }
    __acrt_LCIDToLocaleName(GetUserDefaultLCID(), locale_name, locale_name_count, 0)
}

/// `__acrt_GetXStateFeaturesMask`: no fallback.
#[no_mangle]
pub unsafe extern "system" fn __acrt_GetXStateFeaturesMask(context: *mut c_void, feature_mask: *mut u64) -> c_int {
    type F = unsafe extern "system" fn(*mut c_void, *mut u64) -> c_int;
    if let Some(get_xstate_features_mask) = try_get!(GET_XSTATE_FEATURES_MASK, F) {
        return get_xstate_features_mask(context, feature_mask);
    }
    abort()
}

/// `__acrt_InitializeCriticalSectionEx`. The fallback drops `flags`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_InitializeCriticalSectionEx(
    critical_section: *mut CriticalSection,
    spin_count: u32,
    flags: u32,
) -> c_int {
    type F = unsafe extern "system" fn(*mut CriticalSection, u32, u32) -> c_int;
    if let Some(initialize_critical_section_ex) = try_get!(INITIALIZE_CRITICAL_SECTION_EX, F) {
        return initialize_critical_section_ex(critical_section, spin_count, flags);
    }
    InitializeCriticalSectionAndSpinCount(critical_section, spin_count)
}

/// `__acrt_IsValidLocaleName`.
#[no_mangle]
pub unsafe extern "system" fn __acrt_IsValidLocaleName(locale_name: *const u16) -> c_int {
    if let Some(is_valid_locale_name) = try_get!(IS_VALID_LOCALE_NAME, unsafe extern "system" fn(*const u16) -> c_int) {
        return is_valid_locale_name(locale_name);
    }
    IsValidLocale(__acrt_LocaleNameToLCID(locale_name, 0), LCID_INSTALLED)
}

/// `__acrt_LCMapStringEx`. The fallback drops version/reserved/sort handle.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "system" fn __acrt_LCMapStringEx(
    locale_name: *const u16,
    flags: u32,
    source: *const u16,
    source_count: c_int,
    destination: *mut u16,
    destination_count: c_int,
    version: *mut c_void,
    reserved: *mut c_void,
    sort_handle: isize,
) -> c_int {
    type F = unsafe extern "system" fn(
        *const u16,
        u32,
        *const u16,
        c_int,
        *mut u16,
        c_int,
        *mut c_void,
        *mut c_void,
        isize,
    ) -> c_int;
    if let Some(lc_map_string_ex) = try_get!(LC_MAP_STRING_EX, F) {
        return lc_map_string_ex(
            locale_name,
            flags,
            source,
            source_count,
            destination,
            destination_count,
            version,
            reserved,
            sort_handle,
        );
    }
    LCMapStringW(__acrt_LocaleNameToLCID(locale_name, 0), flags, source, source_count, destination, destination_count)
}

/// `__acrt_LCIDToLocaleName`: downlevel table fallback (drops `flags`).
#[no_mangle]
pub unsafe extern "system" fn __acrt_LCIDToLocaleName(locale: u32, name: *mut u16, name_count: c_int, flags: u32) -> c_int {
    type F = unsafe extern "system" fn(u32, *mut u16, c_int, u32) -> c_int;
    if let Some(lcid_to_locale_name) = try_get!(LCID_TO_LOCALE_NAME, F) {
        return lcid_to_locale_name(locale, name, name_count, flags);
    }
    __acrt_DownlevelLCIDToLocaleName(locale, name, name_count)
}

/// `__acrt_LocaleNameToLCID`: downlevel table fallback (drops `flags`).
#[no_mangle]
pub unsafe extern "system" fn __acrt_LocaleNameToLCID(name: *const u16, flags: u32) -> u32 {
    if let Some(locale_name_to_lcid) = try_get!(LOCALE_NAME_TO_LCID, unsafe extern "system" fn(*const u16, u32) -> u32) {
        return locale_name_to_lcid(name, flags);
    }
    __acrt_DownlevelLocaleNameToLCID(name)
}

/// `__acrt_LocateXStateFeature`: no fallback.
#[no_mangle]
pub unsafe extern "system" fn __acrt_LocateXStateFeature(content: *mut c_void, feature_id: u32, length: *mut u32) -> *mut c_void {
    type F = unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> *mut c_void;
    if let Some(locate_xstate_feature) = try_get!(LOCATE_XSTATE_FEATURE, F) {
        return locate_xstate_feature(content, feature_id, length);
    }
    abort()
}

/// `__acrt_MessageBoxA`: no fallback.
#[no_mangle]
pub unsafe extern "system" fn __acrt_MessageBoxA(hwnd: Handle, text: *const c_char, caption: *const c_char, type_: c_uint) -> c_int {
    type F = unsafe extern "system" fn(Handle, *const c_char, *const c_char, c_uint) -> c_int;
    if let Some(message_box_a) = try_get!(MESSAGE_BOX_A, F) {
        return message_box_a(hwnd, text, caption, type_);
    }
    abort()
}

/// `__acrt_MessageBoxW`: no fallback.
#[no_mangle]
pub unsafe extern "system" fn __acrt_MessageBoxW(hwnd: Handle, text: *const u16, caption: *const u16, type_: c_uint) -> c_int {
    type F = unsafe extern "system" fn(Handle, *const u16, *const u16, c_uint) -> c_int;
    if let Some(message_box_w) = try_get!(MESSAGE_BOX_W, F) {
        return message_box_w(hwnd, text, caption, type_);
    }
    abort()
}

/// `__acrt_RtlGenRandom` (`SystemFunction036`): no fallback -- it should exist.
#[no_mangle]
pub unsafe extern "system" fn __acrt_RtlGenRandom(buffer: *mut c_void, buffer_count: u32) -> u8 {
    if let Some(rtl_gen_random) = try_get!(SYSTEM_FUNCTION_036, unsafe extern "system" fn(*mut c_void, u32) -> u8) {
        return rtl_gen_random(buffer, buffer_count);
    }
    abort()
}

/// `__acrt_RoInitialize`: best effort, S_OK when WinRT is absent.
#[no_mangle]
pub unsafe extern "system" fn __acrt_RoInitialize(init_type: c_int) -> i32 {
    if let Some(ro_initialize) = try_get!(RO_INITIALIZE, unsafe extern "system" fn(c_int) -> i32) {
        return ro_initialize(init_type);
    }
    S_OK
}

/// `__acrt_RoUninitialize`: best effort.
#[no_mangle]
pub unsafe extern "system" fn __acrt_RoUninitialize() {
    if let Some(ro_uninitialize) = try_get!(RO_UNINITIALIZE, unsafe extern "system" fn()) {
        ro_uninitialize();
    }
}

/// The four `AppPolicyGet*` wrappers: ask for the current thread's effective
/// token, or report `STATUS_NOT_FOUND` when the app-model API is absent.
/// `policy` points at an `int`-sized enum.
macro_rules! app_policy {
    ($name:ident, $id:expr) => {
        #[no_mangle]
        pub unsafe extern "system" fn $name(policy: *mut c_int) -> i32 {
            type F = unsafe extern "system" fn(Handle, *mut c_int) -> i32;
            if let Some(app_policy_get) = try_get!($id, F) {
                return app_policy_get(CURRENT_THREAD_EFFECTIVE_TOKEN, policy);
            }
            STATUS_NOT_FOUND
        }
    };
}
app_policy!(__acrt_AppPolicyGetProcessTerminationMethodInternal, APP_POLICY_GET_PROCESS_TERMINATION_METHOD);
app_policy!(__acrt_AppPolicyGetThreadInitializationTypeInternal, APP_POLICY_GET_THREAD_INITIALIZATION_TYPE);
app_policy!(__acrt_AppPolicyGetShowDeveloperDiagnosticInternal, APP_POLICY_GET_SHOW_DEVELOPER_DIAGNOSTIC);
app_policy!(__acrt_AppPolicyGetWindowingModelInternal, APP_POLICY_GET_WINDOWING_MODEL);

/// `__acrt_SetThreadStackGuarantee`: FALSE when absent.
#[no_mangle]
pub unsafe extern "system" fn __acrt_SetThreadStackGuarantee(stack_size_in_bytes: *mut u32) -> c_int {
    if let Some(set_thread_stack_guarantee) =
        try_get!(SET_THREAD_STACK_GUARANTEE, unsafe extern "system" fn(*mut u32) -> c_int)
    {
        return set_thread_stack_guarantee(stack_size_in_bytes);
    }
    0
}

// ── the cdecl helpers ───────────────────────────────────────────────────────

/// `__acrt_can_show_message_box`: an HWND windowing model and both MessageBox
/// entry points. The policy is asked FIRST (it short-circuits the lookups).
#[no_mangle]
pub unsafe extern "C" fn __acrt_can_show_message_box() -> bool {
    __acrt_get_windowing_model_policy() == WINDOWING_MODEL_POLICY_HWND
        && try_get_function(MESSAGE_BOX_A) != 0
        && try_get_function(MESSAGE_BOX_W) != 0
}

/// `__acrt_can_use_vista_locale_apis`: CompareStringEx stands for the set.
#[no_mangle]
pub unsafe extern "C" fn __acrt_can_use_vista_locale_apis() -> bool {
    try_get_function(COMPARE_STRING_EX) != 0
}

/// `__acrt_eagerly_load_locale_apis`: resolve every locale API now, so
/// `_wsetlocale` never calls LoadLibrary under the locale lock.
#[no_mangle]
pub unsafe extern "C" fn __acrt_eagerly_load_locale_apis() {
    for id in [
        ARE_FILE_APIS_ANSI,
        COMPARE_STRING_EX,
        ENUM_SYSTEM_LOCALES_EX,
        GET_DATE_FORMAT_EX,
        GET_LOCALE_INFO_EX,
        GET_TIME_FORMAT_EX,
        GET_USER_DEFAULT_LOCALE_NAME,
        IS_VALID_LOCALE_NAME,
        LC_MAP_STRING_EX,
        LCID_TO_LOCALE_NAME,
        LOCALE_NAME_TO_LCID,
    ] {
        try_get_function(id);
    }
}

/// `__acrt_can_use_xstate_apis`: LocateXStateFeature stands for the set.
#[no_mangle]
pub unsafe extern "C" fn __acrt_can_use_xstate_apis() -> bool {
    try_get_function(LOCATE_XSTATE_FEATURE) != 0
}

/// `__acrt_get_parent_window`: the active window's last active popup (or the
/// active window itself when GetLastActivePopup is absent); NULL without
/// GetActiveWindow or an active window.
#[no_mangle]
pub unsafe extern "C" fn __acrt_get_parent_window() -> Handle {
    let Some(get_active_window) = try_get!(GET_ACTIVE_WINDOW, unsafe extern "system" fn() -> Handle) else {
        return null_mut();
    };

    let active_window = get_active_window();
    if active_window.is_null() {
        return null_mut();
    }

    let Some(get_last_active_popup) = try_get!(GET_LAST_ACTIVE_POPUP, unsafe extern "system" fn(Handle) -> Handle)
    else {
        return active_window;
    };

    get_last_active_popup(active_window)
}

/// `__acrt_is_interactive`: true unless the process's window station is known
/// and NOT visible. Absent APIs count as interactive; a failed query does not.
#[no_mangle]
pub unsafe extern "C" fn __acrt_is_interactive() -> bool {
    let Some(get_process_window_station) =
        try_get!(GET_PROCESS_WINDOW_STATION, unsafe extern "system" fn() -> Handle)
    else {
        return true;
    };

    type GetUserObjectInformationW = unsafe extern "system" fn(Handle, c_int, *mut c_void, u32, *mut u32) -> c_int;
    let Some(get_user_object_information) = try_get!(GET_USER_OBJECT_INFORMATION_W, GetUserObjectInformationW) else {
        return true;
    };

    let hwinsta = get_process_window_station();
    if hwinsta.is_null() {
        return false;
    }

    let mut uof = UserObjectFlags { inherit: 0, reserved: 0, flags: 0 };
    if get_user_object_information(
        hwinsta,
        UOI_FLAGS,
        &mut uof as *mut UserObjectFlags as *mut c_void,
        core::mem::size_of::<UserObjectFlags>() as u32,
        null_mut(),
    ) == 0
    {
        return false;
    }

    uof.flags & WSF_VISIBLE != 0
}
