//! `internal/win_policies.cpp`: which app-model policy applies to this process,
//! over the `__acrt_AppPolicyGet*Internal` thunks (winapi_thunks). A secure
//! process cannot load the app-model DLL, so it never asks and gets the
//! per-policy default. The getter's own return code is ignored: on failure the
//! default passed in stays.
//!
//! Only `__acrt_get_process_end_policy` asks every time; the other three cache
//! their first answer (`get_cached_win_policy`'s `state_cache`, 0 = unknown).

use super::peb_access::__acrt_is_secure_process;
use core::ffi::c_int;
use core::sync::atomic::{AtomicI32, Ordering};

// appmodel.h
const APP_POLICY_PROCESS_TERMINATION_METHOD_EXIT_PROCESS: c_int = 0;
const APP_POLICY_PROCESS_TERMINATION_METHOD_TERMINATE_PROCESS: c_int = 1;
const APP_POLICY_THREAD_INITIALIZATION_TYPE_NONE: c_int = 0;
const APP_POLICY_THREAD_INITIALIZATION_TYPE_INITIALIZE_WINRT: c_int = 1;
const APP_POLICY_SHOW_DEVELOPER_DIAGNOSTIC_SHOW_UI: c_int = 1;
const APP_POLICY_WINDOWING_MODEL_UNIVERSAL: c_int = 1;
const APP_POLICY_WINDOWING_MODEL_CLASSIC_DESKTOP: c_int = 2;
const APP_POLICY_WINDOWING_MODEL_CLASSIC_PHONE: c_int = 3;

// corecrt_internal.h
/// `process_end_policy`
pub(crate) const PROCESS_END_POLICY_TERMINATE_PROCESS: c_int = 0;
pub(crate) const PROCESS_END_POLICY_EXIT_PROCESS: c_int = 1;
/// `begin_thread_init_policy` (0 = unknown)
pub(crate) const BEGIN_THREAD_INIT_POLICY_NONE: c_int = 1;
pub(crate) const BEGIN_THREAD_INIT_POLICY_RO_INITIALIZE: c_int = 2;
/// `developer_information_policy` (0 = unknown)
pub(crate) const DEVELOPER_INFORMATION_POLICY_NONE: c_int = 1;
pub(crate) const DEVELOPER_INFORMATION_POLICY_UI: c_int = 2;
/// `windowing_model_policy` (0 = unknown)
pub(crate) const WINDOWING_MODEL_POLICY_HWND: c_int = 1;
pub(crate) const WINDOWING_MODEL_POLICY_COREWINDOW: c_int = 2;
pub(crate) const WINDOWING_MODEL_POLICY_LEGACYPHONE: c_int = 3;
pub(crate) const WINDOWING_MODEL_POLICY_NONE: c_int = 4;

extern "system" {
    fn __acrt_AppPolicyGetProcessTerminationMethodInternal(policy: *mut c_int) -> c_int;
    fn __acrt_AppPolicyGetThreadInitializationTypeInternal(policy: *mut c_int) -> c_int;
    fn __acrt_AppPolicyGetShowDeveloperDiagnosticInternal(policy: *mut c_int) -> c_int;
    fn __acrt_AppPolicyGetWindowingModelInternal(policy: *mut c_int) -> c_int;
}

/// `get_win_policy<TPolicy>`: uncached.
unsafe fn get_win_policy(
    default_value: c_int,
    appmodel_get_policy: unsafe extern "system" fn(*mut c_int) -> c_int,
    to_policy_type: fn(c_int) -> c_int,
) -> c_int {
    let mut appmodel_policy = default_value;

    // Secure processes cannot load the appmodel DLL.
    if !__acrt_is_secure_process() {
        appmodel_get_policy(&mut appmodel_policy);
    }

    to_policy_type(appmodel_policy)
}

/// `get_cached_win_policy<TPolicy>`: the first nonzero answer sticks.
unsafe fn get_cached_win_policy(
    state_cache: &AtomicI32,
    default_value: c_int,
    appmodel_get_policy: unsafe extern "system" fn(*mut c_int) -> c_int,
    to_policy_type: fn(c_int) -> c_int,
) -> c_int {
    // __crt_interlocked_read
    let cached_state = state_cache.load(Ordering::SeqCst);
    if cached_state != 0 {
        return cached_state;
    }

    let mut appmodel_policy = default_value;

    if !__acrt_is_secure_process() {
        appmodel_get_policy(&mut appmodel_policy);
    }

    let policy_value = to_policy_type(appmodel_policy);

    // _InterlockedExchange; the C only _ASSERTEs (debug) that a racing
    // writer stored the same value.
    state_cache.swap(policy_value, Ordering::SeqCst);

    policy_value
}

/// Determines whether ExitProcess() or TerminateProcess() ends the process.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_get_process_end_policy() -> c_int {
    fn to_policy_type(p: c_int) -> c_int {
        if p == APP_POLICY_PROCESS_TERMINATION_METHOD_TERMINATE_PROCESS {
            PROCESS_END_POLICY_TERMINATE_PROCESS
        } else {
            PROCESS_END_POLICY_EXIT_PROCESS
        }
    }
    get_win_policy(
        APP_POLICY_PROCESS_TERMINATION_METHOD_EXIT_PROCESS,
        __acrt_AppPolicyGetProcessTerminationMethodInternal,
        to_policy_type,
    )
}

static BEGIN_THREAD_INIT_STATE_CACHE: AtomicI32 = AtomicI32::new(0);
static DEVELOPER_INFORMATION_STATE_CACHE: AtomicI32 = AtomicI32::new(0);
static WINDOWING_MODEL_STATE_CACHE: AtomicI32 = AtomicI32::new(0);

/// Determines whether RoInitialize() is called when creating a thread.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_get_begin_thread_init_policy() -> c_int {
    fn to_policy_type(p: c_int) -> c_int {
        if p == APP_POLICY_THREAD_INITIALIZATION_TYPE_INITIALIZE_WINRT {
            BEGIN_THREAD_INIT_POLICY_RO_INITIALIZE
        } else {
            BEGIN_THREAD_INIT_POLICY_NONE
        }
    }
    get_cached_win_policy(
        &BEGIN_THREAD_INIT_STATE_CACHE,
        APP_POLICY_THREAD_INITIALIZATION_TYPE_NONE,
        __acrt_AppPolicyGetThreadInitializationTypeInternal,
        to_policy_type,
    )
}

/// Determines whether an assert dialog may be displayed. Note the DEFAULT is
/// ShowUI -- a secure process, or a failed query, shows UI.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_get_developer_information_policy() -> c_int {
    fn to_policy_type(p: c_int) -> c_int {
        if p == APP_POLICY_SHOW_DEVELOPER_DIAGNOSTIC_SHOW_UI {
            DEVELOPER_INFORMATION_POLICY_UI
        } else {
            DEVELOPER_INFORMATION_POLICY_NONE
        }
    }
    get_cached_win_policy(
        &DEVELOPER_INFORMATION_STATE_CACHE,
        APP_POLICY_SHOW_DEVELOPER_DIAGNOSTIC_SHOW_UI,
        __acrt_AppPolicyGetShowDeveloperDiagnosticInternal,
        to_policy_type,
    )
}

/// Determines which windowing model technology is available.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_get_windowing_model_policy() -> c_int {
    fn to_policy_type(p: c_int) -> c_int {
        match p {
            APP_POLICY_WINDOWING_MODEL_CLASSIC_DESKTOP => WINDOWING_MODEL_POLICY_HWND,
            APP_POLICY_WINDOWING_MODEL_UNIVERSAL => WINDOWING_MODEL_POLICY_COREWINDOW,
            APP_POLICY_WINDOWING_MODEL_CLASSIC_PHONE => WINDOWING_MODEL_POLICY_LEGACYPHONE,
            // APP_POLICY_WINDOWING_MODEL_NONE, and default:
            _ => WINDOWING_MODEL_POLICY_NONE,
        }
    }
    get_cached_win_policy(
        &WINDOWING_MODEL_STATE_CACHE,
        APP_POLICY_WINDOWING_MODEL_CLASSIC_DESKTOP,
        __acrt_AppPolicyGetWindowingModelInternal,
        to_policy_type,
    )
}
