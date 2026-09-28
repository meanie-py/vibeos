//! The CRT's own bootstrap: `internal/locks.cpp`, `internal/shared_initialization.cpp`,
//! `internal/per_thread_data.cpp` and `startup/initterm.cpp`, ported.
//!
//! Everything else in ucrtbase stands on these: the lock table every `__acrt_lock`
//! call indexes, the per-thread data every `errno` lives in, and the two loops
//! that run the initializer tables.
//!
//! Details that are silent if wrong:
//! * `__acrt_getptd_noexit` must leave `GetLastError()` as it found it (the C's
//!   `__crt_scoped_get_last_error_reset`). `FlsGetValue` clears it, and callers
//!   such as `_errno()` sit between a failing Win32 call and the caller's own
//!   `GetLastError()`.
//! * The per-thread block is created lazily, guarded against re-entrancy by a
//!   `SIZE_MAX` sentinel in the FLS slot: constructing it takes locks and
//!   touches locale data, which can call back into `__acrt_getptd_noexit`.
//! * No global state isolation in this build, so `state_index_count` is 1 and
//!   the current state index is always 0. `__acrt_getptd_noexit_explicit` is
//!   still exported (unmangled: the header declares it `extern "C"`) because
//!   the ptd-propagation header's inline code calls it.
//! * `_rand_state` starts at 1 and `_own_locale` at `_GLOBAL_LOCALE_BIT`; the
//!   signal action table pointer starts at the SHARED table, and is only freed
//!   when a thread has replaced it with its own copy.

pub mod init;

use crate::locale::{CrtLocaleData, CrtMultibyteData, GLOBAL_LOCALE_BIT};
use crate::ptd::AcrtPtd;
use crate::stdio_layout::CriticalSection;
use core::ffi::{c_int, c_long, c_uint, c_void};
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

/// `__acrt_lock_count` (corecrt_internal.h's `__acrt_lock_id`).
const ACRT_LOCK_COUNT: usize = 14;
const LOCALE_LOCK: c_int = 4;
const MULTIBYTE_CP_LOCK: c_int = 5;
const CORECRT_SPINCOUNT: u32 = 4000;
/// `FLS_OUT_OF_INDEXES`.
const FLS_OUT_OF_INDEXES: u32 = 0xFFFF_FFFF;
/// `__crt_state_management::state_index_count` without global state isolation.
const STATE_INDEX_COUNT: usize = 1;

extern "C" {
    fn _calloc_base(count: usize, size: usize) -> *mut c_void;
    fn _free_base(block: *mut c_void);
    fn abort() -> !;
    static __acrt_exception_action_table: [u8; 0];
}
use crate::locale_abi::{
    __acrt_add_locale_ref, __acrt_current_locale_data, __acrt_free_locale, __acrt_initial_locale_data,
    __acrt_initial_multibyte_data, __acrt_release_locale_ref,
};

extern "system" {
    fn __acrt_InitializeCriticalSectionEx(cs: *mut CriticalSection, spin_count: u32, flags: u32) -> c_int;
    fn EnterCriticalSection(cs: *mut CriticalSection);
    fn LeaveCriticalSection(cs: *mut CriticalSection);
    fn DeleteCriticalSection(cs: *mut CriticalSection);
    fn __acrt_FlsAlloc(callback: Option<unsafe extern "system" fn(*mut c_void)>) -> u32;
    fn __acrt_FlsFree(index: u32) -> c_int;
    fn __acrt_FlsGetValue(index: u32) -> *mut c_void;
    fn __acrt_FlsSetValue(index: u32, value: *mut c_void) -> c_int;
    fn GetLastError() -> u32;
    fn SetLastError(error: u32);
    fn GetCurrentThreadId() -> u32;
    fn GetCurrentThread() -> *mut c_void;
}

// ── locks.cpp ───────────────────────────────────────────────────────────────

static mut ACRT_LOCK_TABLE: [CriticalSection; ACRT_LOCK_COUNT] = [const { CriticalSection::ZERO }; ACRT_LOCK_COUNT];
static mut ACRT_LOCKS_INITIALIZED: c_uint = 0;

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_locks() -> bool {
    for i in 0..ACRT_LOCK_COUNT {
        if __acrt_InitializeCriticalSectionEx(addr_of_mut!(ACRT_LOCK_TABLE[i]), CORECRT_SPINCOUNT, 0) == 0 {
            __acrt_uninitialize_locks(false);
            return false;
        }
        ACRT_LOCKS_INITIALIZED += 1;
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_locks(_terminating: bool) -> bool {
    let mut i = ACRT_LOCKS_INITIALIZED;
    while i > 0 {
        DeleteCriticalSection(addr_of_mut!(ACRT_LOCK_TABLE[(i - 1) as usize]));
        ACRT_LOCKS_INITIALIZED -= 1;
        i -= 1;
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_lock(lock: c_int) {
    EnterCriticalSection(addr_of_mut!(ACRT_LOCK_TABLE[lock as usize]));
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_unlock(lock: c_int) {
    LeaveCriticalSection(addr_of_mut!(ACRT_LOCK_TABLE[lock as usize]));
}

#[no_mangle]
pub unsafe extern "C" fn _lock_locales() {
    extern "C" {
        fn __acrt_eagerly_load_locale_apis();
    }
    __acrt_eagerly_load_locale_apis();
    __acrt_lock(LOCALE_LOCK);
}

#[no_mangle]
pub unsafe extern "C" fn _unlock_locales() {
    __acrt_unlock(LOCALE_LOCK);
}

unsafe fn with_lock<R>(lock: c_int, action: impl FnOnce() -> R) -> R {
    __acrt_lock(lock);
    let r = action();
    __acrt_unlock(lock);
    r
}

// ── initterm.cpp ────────────────────────────────────────────────────────────

pub type Pvfv = Option<unsafe extern "C" fn()>;
pub type Pifv = Option<unsafe extern "C" fn() -> c_int>;

#[no_mangle]
pub unsafe extern "C" fn _initterm(first: *const Pvfv, last: *const Pvfv) {
    let mut it = first;
    while it != last {
        if let Some(f) = *it {
            f();
        }
        it = it.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _initterm_e(first: *const Pifv, last: *const Pifv) -> c_int {
    let mut it = first;
    while it != last {
        if let Some(f) = *it {
            let result = f();
            if result != 0 {
                return result;
            }
        }
        it = it.add(1);
    }
    0
}

// ── shared_initialization.cpp ───────────────────────────────────────────────

/// `__acrt_initializer`.
#[repr(C)]
pub struct AcrtInitializer {
    pub initialize: Option<unsafe extern "C" fn() -> bool>,
    pub uninitialize: Option<unsafe extern "C" fn(bool) -> bool>,
}

/// Runs the initializers in order; if one fails, runs the uninitializers of
/// the ones that succeeded, in reverse, and fails.
#[no_mangle]
pub unsafe extern "C" fn __acrt_execute_initializers(
    first: *const AcrtInitializer,
    last: *const AcrtInitializer,
) -> bool {
    if first == last {
        return true;
    }
    let mut it = first;
    while it != last {
        if let Some(init) = (*it).initialize {
            if !init() {
                break;
            }
        }
        it = it.add(1);
    }
    if it == last {
        return true;
    }
    // `it` is the one that failed; it is not uninitialized. Neither is an entry
    // with no initializer: there was nothing to undo.
    while it != first {
        let prev = it.sub(1);
        if let (Some(_), Some(uninit)) = ((*prev).initialize, (*prev).uninitialize) {
            uninit(false);
        }
        it = prev;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_execute_uninitializers(
    first: *const AcrtInitializer,
    last: *const AcrtInitializer,
) -> bool {
    if first == last {
        return true;
    }
    let mut it = last;
    while it != first {
        let prev = it.sub(1);
        if let Some(uninit) = (*prev).uninitialize {
            uninit(false);
        }
        it = prev;
    }
    true
}

// ── per_thread_data.cpp ─────────────────────────────────────────────────────

static ACRT_FLSINDEX: AtomicU32 = AtomicU32::new(FLS_OUT_OF_INDEXES);

fn flsindex() -> u32 {
    ACRT_FLSINDEX.load(Ordering::Relaxed)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_ptd() -> bool {
    let index = __acrt_FlsAlloc(Some(destroy_fls));
    ACRT_FLSINDEX.store(index, Ordering::Relaxed);
    if index == FLS_OUT_OF_INDEXES {
        return false;
    }
    if __acrt_getptd_noexit().is_null() {
        __acrt_uninitialize_ptd(false);
        return false;
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_ptd(_terminating: bool) -> bool {
    if flsindex() != FLS_OUT_OF_INDEXES {
        __acrt_FlsFree(flsindex());
        ACRT_FLSINDEX.store(FLS_OUT_OF_INDEXES, Ordering::Relaxed);
    }
    true
}

unsafe fn locale_refcount(data: *mut CrtLocaleData) -> c_long {
    (*(addr_of_mut!((*data).refcount) as *const AtomicI32)).load(Ordering::SeqCst)
}

unsafe fn replace_current_thread_locale_nolock(ptd: *mut AcrtPtd, new_locale_info: *mut CrtLocaleData) {
    let old = (*ptd).locale_info;
    if !old.is_null() {
        __acrt_release_locale_ref(old);
        // The global and the initial locale are owned elsewhere; anything else
        // whose count has just reached zero was this thread's to free.
        if old != __acrt_current_locale_data
            && old != addr_of_mut!(__acrt_initial_locale_data)
            && locale_refcount(old) == 0
        {
            __acrt_free_locale(old);
        }
    }
    (*ptd).locale_info = new_locale_info;
    if !new_locale_info.is_null() {
        __acrt_add_locale_ref(new_locale_info);
    }
}

unsafe fn multibyte_refcount(data: *mut CrtMultibyteData) -> &'static AtomicI32 {
    &*(addr_of_mut!((*data).refcount) as *const AtomicI32)
}

unsafe fn construct_ptd(ptd: *mut AcrtPtd, locale_data: *mut *mut CrtLocaleData) {
    (*ptd).rand_state = 1;
    (*ptd).pxcptacttab = core::ptr::addr_of!(__acrt_exception_action_table) as *mut c_void;
    (*ptd).own_locale = GLOBAL_LOCALE_BIT;
    (*ptd).multibyte_info = addr_of_mut!(__acrt_initial_multibyte_data);
    (*ptd).setloc_data.cachein[0] = b'C' as u16;
    (*ptd).setloc_data.cacheout[0] = b'C' as u16;
    (*ptd).setloc_downlevel_data = null_mut();

    with_lock(MULTIBYTE_CP_LOCK, || {
        multibyte_refcount((*ptd).multibyte_info).fetch_add(1, Ordering::SeqCst);
    });
    with_lock(LOCALE_LOCK, || replace_current_thread_locale_nolock(ptd, *locale_data));
}

unsafe fn destroy_ptd(ptd: *mut AcrtPtd) {
    if (*ptd).pxcptacttab != core::ptr::addr_of!(__acrt_exception_action_table) as *mut c_void {
        _free_base((*ptd).pxcptacttab);
    }
    _free_base((*ptd).cvtbuf.cast());
    _free_base((*ptd).asctime_buffer.cast());
    _free_base((*ptd).wasctime_buffer.cast());
    _free_base((*ptd).gmtime_buffer);
    _free_base((*ptd).tmpnam_narrow_buffer.cast());
    _free_base((*ptd).tmpnam_wide_buffer.cast());
    _free_base((*ptd).strerror_buffer.cast());
    _free_base((*ptd).wcserror_buffer.cast());
    _free_base((*ptd).beginthread_context);

    with_lock(MULTIBYTE_CP_LOCK, || {
        let multibyte_data = (*ptd).multibyte_info;
        if multibyte_data.is_null() {
            return;
        }
        // fetch_sub returns the old value: `_InterlockedDecrement(...) != 0`.
        if multibyte_refcount(multibyte_data).fetch_sub(1, Ordering::SeqCst) - 1 != 0 {
            return;
        }
        if multibyte_data == addr_of_mut!(__acrt_initial_multibyte_data) {
            return;
        }
        _free_base(multibyte_data.cast());
    });
    with_lock(LOCALE_LOCK, || replace_current_thread_locale_nolock(ptd, null_mut()));
}

unsafe extern "system" fn destroy_fls(pfd: *mut c_void) {
    if pfd.is_null() {
        return;
    }
    let ptd: *mut AcrtPtd = pfd.cast();
    for i in 0..STATE_INDEX_COUNT {
        destroy_ptd(ptd.add(i));
    }
    _free_base(pfd);
}

unsafe fn try_get_ptd_head() -> *mut AcrtPtd {
    if flsindex() == FLS_OUT_OF_INDEXES {
        return null_mut();
    }
    __acrt_FlsGetValue(flsindex()).cast()
}

/// The sentinel parked in the FLS slot while the block is being built.
const REENTRANCY_SENTINEL: *mut AcrtPtd = usize::MAX as *mut AcrtPtd;

unsafe fn internal_get_ptd_head() -> *mut AcrtPtd {
    let existing = try_get_ptd_head();
    if existing == REENTRANCY_SENTINEL {
        return null_mut();
    } else if !existing.is_null() {
        return existing;
    }
    if __acrt_FlsSetValue(flsindex(), REENTRANCY_SENTINEL.cast()) == 0 {
        return null_mut();
    }
    let new_head: *mut AcrtPtd = _calloc_base(STATE_INDEX_COUNT, core::mem::size_of::<AcrtPtd>()).cast();
    if new_head.is_null() {
        __acrt_FlsSetValue(flsindex(), null_mut());
        return null_mut();
    }
    if __acrt_FlsSetValue(flsindex(), new_head.cast()) == 0 {
        __acrt_FlsSetValue(flsindex(), null_mut());
        _free_base(new_head.cast());
        return null_mut();
    }
    for i in 0..STATE_INDEX_COUNT {
        construct_ptd(new_head.add(i), addr_of_mut!(__acrt_current_locale_data));
    }
    new_head
}

unsafe fn internal_getptd_noexit(global_state_index: usize) -> *mut AcrtPtd {
    let head = internal_get_ptd_head();
    if head.is_null() {
        return null_mut();
    }
    head.add(global_state_index)
}

/// The caller already holds a `__crt_scoped_get_last_error_reset`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_getptd_noexit_explicit(
    _last_error_reset: *const c_void,
    global_state_index: usize,
) -> *mut AcrtPtd {
    internal_getptd_noexit(global_state_index)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_getptd_noexit() -> *mut AcrtPtd {
    let last_error = GetLastError();
    let ptd = internal_getptd_noexit(0);
    SetLastError(last_error);
    ptd
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_getptd() -> *mut AcrtPtd {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        abort();
    }
    ptd
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_getptd_head() -> *mut AcrtPtd {
    let head = internal_get_ptd_head();
    if head.is_null() {
        abort();
    }
    head
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_freeptd() {
    let head = try_get_ptd_head();
    if head.is_null() {
        return;
    }
    __acrt_FlsSetValue(flsindex(), null_mut());
    destroy_fls(head.cast());
}

#[no_mangle]
pub unsafe extern "C" fn __threadid() -> u32 {
    GetCurrentThreadId()
}

#[no_mangle]
pub unsafe extern "C" fn __threadhandle() -> usize {
    GetCurrentThread() as usize
}
