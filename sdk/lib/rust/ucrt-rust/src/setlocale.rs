//! `setlocale` and the locale lifetime machinery -- the spine of the locale
//! takeover. Replaces `sdk/lib/ucrt/locale/wsetlocale.cpp`, `setlocale.cpp`,
//! `locale_refcounting.cpp` and `locale_update.cpp` in full.
//!
//! Exports: `setlocale`, `_wsetlocale`, `_create_locale`, `_wcreate_locale`,
//! `_free_locale`, `_get_current_locale`, `_configthreadlocale`. Internals that
//! ~160 other ucrt objects call by C name (see `locale_abi.rs`): the reference
//! counting, `_updatetlocinfoEx_nolock`, `__acrt_update_thread_locale_data`,
//! the four `__acrt_update_*_info` hooks behind `_LocaleUpdate`,
//! `__acrt_copy_locale_name`, `__acrt_locale_changed_data`,
//! `__acrt_set_locale_changed`, `__acrt_uninitialize_locale`.
//!
//! ## Details that are silent if wrong
//!
//! * **Every category string carries its refcount in front of it.** A category
//!   locale is one `malloc` of `sizeof(int) + chars`: the `long` refcount at the
//!   start, the wide string after it, and `wrefcount` points at the block. The
//!   LC_ALL composite and `setlocale`'s narrow copy use the same trick.
//! * **`setlocale`'s narrow string borrows the whole struct's refcount** as its
//!   starting count, and its category refcount is decremented TWICE when the
//!   thread follows the global locale. Transcribed; the C's comment calls it a
//!   "risky trick" that only holds under the locale lock.
//! * **`_expandlocale` caches in the per-thread `_setloc_data`**: input,
//!   output, code page and the internal locale name. The name cache is saved
//!   and restored around a failed lookup by a scope guard.
//! * **The ".utf8" test has an operator-precedence hole**, kept: the C reads
//!   `u && t && f && (8 && NUL) || (- && 8 && NUL)`, so ANY code page spelled
//!   `xyz-8` is taken as UTF-8 for a BCP-47 tag.
//! * **A failed category init restores the old string but FREES the new
//!   name**, and the "refcount hit zero" cleanup in `_wsetlocale_set_cat` frees
//!   the NEW locale name. Both are `_ASSERT(0)`-guarded "cannot happen" paths
//!   in the C and are transcribed as written.
//! * **`__ascii_isalpha`/`__ascii_isdigit` in the BCP-47 parser are LOCALE
//!   reads** (`__pctype_func()`), not ASCII range checks: the header routes
//!   them through the live ctype table.
//! * **`_ERRCHECK_EINVAL_ERANGE` aborts on EINVAL/ERANGE and passes every other
//!   error through** -- so a conversion that fails with EILSEQ returns NULL
//!   quietly while a too-small buffer is a Watson crash.

use crate::locale::{
    CrtLocaleData, CrtLocalePointers, CrtMultibyteData, CtypeCompatibilityData, LcTimeData,
    LocaleStrings, CATNAMES_LEN, GLOBAL_LOCALE_BIT, CRT_LC_ALL, CRT_LC_COLLATE, CRT_LC_CTYPE, CRT_LC_MAX, CRT_LC_MIN,
    CRT_LC_TIME, LOCALE_NAME_MAX_LENGTH, MAX_CP_LEN, MAX_CTRY_LEN, MAX_LANG_LEN, MAX_LC_LEN,
    PER_THREAD_LOCALE_BIT, WSETLOCALE_AVOID_SYNC_LOCALE_BIT,
};
use crate::locale_abi::*;
use crate::ptd::AcrtPtd;
use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};
use core::sync::atomic::{AtomicI32, Ordering};

// wsetlocale.cpp carried `_CRT_LINKER_FORCE_INCLUDE(__acrt_locale_terminator)`; see force_include.rs.
crate::force_include!(__acrt_locale_terminator);

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;
const ENOMEM: c_int = 12;
const LONG_MAX: c_long = 0x7fff_ffff;
const INT_MAX: usize = 0x7fff_ffff;
const TRUNCATE: usize = usize::MAX;
/// `_COFFSET`: how far into the allocation `ctype1` points.
const COFFSET: usize = 127;
const CP_ACP: c_uint = 0;
const CP_UTF8: c_int = 65001;
const CT_CTYPE1: u32 = 0x1;
const LOCALE_IDEFAULTANSICODEPAGE: u32 = 0x1004;
const LOCALE_RETURN_NUMBER: u32 = 0x2000_0000;
/// `_ENABLE_PER_THREAD_LOCALE` / `_DISABLE_PER_THREAD_LOCALE` (locale.h).
const ENABLE_PER_THREAD_LOCALE: c_int = 0x1;
const DISABLE_PER_THREAD_LOCALE: c_int = 0x2;
/// `__acrt_lock_id`
const LOCALE_LOCK: c_int = 4;
const MULTIBYTE_CP_LOCK: c_int = 5;
/// `_UPPER|_LOWER|_DIGIT|_SPACE|_PUNCT|_CONTROL|_BLANK|_HEX|_ALPHA`
const CTYPE_STYLE_MASK: u16 = 0x1ff;
const CTYPE_ALPHA: u16 = 0x0103;
const CTYPE_DIGIT: u16 = 0x0004;

extern "C" {
    fn __acrt_getptd() -> *mut AcrtPtd;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    fn __acrt_eagerly_load_locale_apis();
    fn __acrt_can_use_vista_locale_apis() -> bool;
    fn __acrt_MultiByteToWideChar(
        cp: c_uint,
        flags: u32,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
    fn __pctype_func() -> *const u16;
    fn malloc(size: usize) -> *mut c_void;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn abort() -> !;
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: c_uint, r: usize) -> !;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    fn wcslen(s: *const u16) -> usize;
    fn wcsnlen(s: *const u16, n: usize) -> usize;
    fn wcspbrk(s: *const u16, set: *const u16) -> *const u16;
    fn wcscspn(s: *const u16, set: *const u16) -> usize;
    fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn wcsncpy_s(dst: *mut u16, size: usize, src: *const u16, count: usize) -> c_int;
    fn wcscat_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn wcsncat_s(dst: *mut u16, size: usize, src: *const u16, count: usize) -> c_int;
    fn mbstowcs_s(
        converted: *mut usize,
        dst: *mut u16,
        size: usize,
        src: *const c_char,
        count: usize,
    ) -> c_int;
    fn _wcstombs_s_l(
        converted: *mut usize,
        dst: *mut c_char,
        size: usize,
        src: *const u16,
        count: usize,
        locale: LocaleT,
    ) -> c_int;
}

extern "system" {
    fn __acrt_GetLocaleInfoEx(name: *const u16, lc_type: u32, data: *mut u16, count: c_int) -> c_int;
    fn __acrt_IsValidLocaleName(name: *const u16) -> c_int;
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

// ── small shared helpers ────────────────────────────────────────────────────

/// `_ERRCHECK(e)`
#[inline]
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(null(), null(), null(), 0, 0);
    }
}

/// `_ERRCHECK_EINVAL_ERANGE(e)`: Watson on EINVAL/ERANGE, else pass `e` through.
#[inline]
unsafe fn errcheck_einval_erange(e: c_int) -> c_int {
    if e == EINVAL || e == ERANGE {
        _invoke_watson(null(), null(), null(), 0, 0);
    }
    e
}

#[inline]
unsafe fn interlocked_increment(p: *mut c_long) -> c_long {
    AtomicI32::from_ptr(p).fetch_add(1, Ordering::SeqCst) + 1
}

#[inline]
unsafe fn interlocked_decrement(p: *mut c_long) -> c_long {
    AtomicI32::from_ptr(p).fetch_sub(1, Ordering::SeqCst) - 1
}

fn wide_c_locale_string() -> *mut u16 {
    addr_of_mut!(__acrt_wide_c_locale_string) as *mut u16
}

fn initial_locale_data() -> *mut CrtLocaleData {
    addr_of_mut!(__acrt_initial_locale_data)
}

/// `__acrt_should_sync_with_global_locale`
#[inline]
unsafe fn should_sync_with_global_locale(ptd: *mut AcrtPtd) -> bool {
    (*ptd).own_locale & __globallocalestatus == 0
}

/// A `__crt_locale_data` category slot, by `LC_*` index.
#[inline]
unsafe fn cat(ploci: *mut CrtLocaleData, category: c_int) -> *mut crate::locale::CrtLocaleRefcount {
    addr_of_mut!((*ploci).lc_category[category as usize])
}

// ── locale_refcounting.cpp ──────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn __acrt_add_locale_ref(ptloci: *mut CrtLocaleData) {
    interlocked_increment(addr_of_mut!((*ptloci).refcount));
    for rc in [
        (*ptloci).lconv_intl_refcount,
        (*ptloci).lconv_mon_refcount,
        (*ptloci).lconv_num_refcount,
        (*ptloci).ctype1_refcount,
    ] {
        if !rc.is_null() {
            interlocked_increment(rc);
        }
    }
    for category in CRT_LC_MIN..=CRT_LC_MAX {
        let c = cat(ptloci, category);
        if (*c).wlocale != wide_c_locale_string() && !(*c).wrefcount.is_null() {
            interlocked_increment((*c).wrefcount);
        }
        if !(*c).locale.is_null() && !(*c).refcount.is_null() {
            interlocked_increment((*c).refcount);
        }
    }
    __acrt_locale_add_lc_time_reference((*ptloci).lc_time_curr as *const LcTimeData);
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_release_locale_ref(ptloci: *mut CrtLocaleData) {
    if ptloci.is_null() {
        return;
    }
    interlocked_decrement(addr_of_mut!((*ptloci).refcount));
    for rc in [
        (*ptloci).lconv_intl_refcount,
        (*ptloci).lconv_mon_refcount,
        (*ptloci).lconv_num_refcount,
        (*ptloci).ctype1_refcount,
    ] {
        if !rc.is_null() {
            interlocked_decrement(rc);
        }
    }
    for category in CRT_LC_MIN..=CRT_LC_MAX {
        let c = cat(ptloci, category);
        if (*c).wlocale != wide_c_locale_string() && !(*c).wrefcount.is_null() {
            interlocked_decrement((*c).wrefcount);
        }
        if !(*c).locale.is_null() && !(*c).refcount.is_null() {
            interlocked_decrement((*c).refcount);
        }
    }
    __acrt_locale_release_lc_time_reference((*ptloci).lc_time_curr as *const LcTimeData);
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_free_locale(ptloci: *mut CrtLocaleData) {
    let lconv = (*ptloci).lconv as *mut crate::locale::Lconv;
    if !lconv.is_null()
        && lconv != addr_of_mut!(__acrt_lconv_c)
        && !(*ptloci).lconv_intl_refcount.is_null()
        && *(*ptloci).lconv_intl_refcount == 0
    {
        let mon = (*ptloci).lconv_mon_refcount;
        if !mon.is_null() && *mon == 0 {
            free(mon as *mut c_void);
            __acrt_locale_free_monetary(lconv);
        }
        let num = (*ptloci).lconv_num_refcount;
        if !num.is_null() && *num == 0 {
            free(num as *mut c_void);
            __acrt_locale_free_numeric(lconv);
        }
        free((*ptloci).lconv_intl_refcount as *mut c_void);
        free(lconv as *mut c_void);
    }

    let ctype_rc = (*ptloci).ctype1_refcount;
    if !ctype_rc.is_null() && *ctype_rc == 0 {
        free((*ptloci).ctype1.wrapping_sub(COFFSET) as *mut c_void);
        free((*ptloci).pclmap.wrapping_sub(COFFSET + 1) as *mut c_void);
        free((*ptloci).pcumap.wrapping_sub(COFFSET + 1) as *mut c_void);
        free(ctype_rc as *mut c_void);
    }

    __acrt_locale_free_lc_time_if_unreferenced((*ptloci).lc_time_curr as *const LcTimeData);

    for category in CRT_LC_MIN..=CRT_LC_MAX {
        let c = cat(ptloci, category);
        if (*c).wlocale != wide_c_locale_string() && !(*c).wrefcount.is_null() && *(*c).wrefcount == 0
        {
            free((*c).wrefcount as *mut c_void);
            free((*ptloci).locale_name[category as usize] as *mut c_void);
        }
        if !(*c).locale.is_null() && !(*c).refcount.is_null() && *(*c).refcount == 0 {
            free((*c).refcount as *mut c_void);
        }
    }

    free(ptloci as *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn _updatetlocinfoEx_nolock(
    pptlocid: *mut *mut CrtLocaleData,
    ptlocis: *mut CrtLocaleData,
) -> *mut CrtLocaleData {
    if ptlocis.is_null() || pptlocid.is_null() {
        return null_mut();
    }
    let ptloci = *pptlocid;
    if ptloci == ptlocis {
        return ptlocis;
    }
    *pptlocid = ptlocis;
    __acrt_add_locale_ref(ptlocis);
    if !ptloci.is_null() {
        __acrt_release_locale_ref(ptloci);
    }
    // Freed only after the new references are taken, in case the two share
    // refcounts.
    if !ptloci.is_null() && (*ptloci).refcount == 0 && ptloci != initial_locale_data() {
        __acrt_free_locale(ptloci);
    }
    ptlocis
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_update_thread_locale_data() -> *mut CrtLocaleData {
    let ptd = __acrt_getptd();
    let ptloci = if should_sync_with_global_locale(ptd) || (*ptd).locale_info.is_null() {
        __acrt_lock(LOCALE_LOCK);
        let p = _updatetlocinfoEx_nolock(addr_of_mut!((*ptd).locale_info), __acrt_current_locale_data);
        __acrt_unlock(LOCALE_LOCK);
        p
    } else {
        (*ptd).locale_info
    };
    if ptloci.is_null() {
        abort();
    }
    ptloci
}

/// `true` for NULL and for the immortal C-locale time data.
unsafe fn is_static_lc_time(lc_time: *const LcTimeData) -> bool {
    lc_time.is_null() || lc_time == addr_of!(__lc_time_c)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_locale_add_lc_time_reference(lc_time: *const LcTimeData) -> c_long {
    if is_static_lc_time(lc_time) {
        return LONG_MAX;
    }
    interlocked_increment(addr_of!((*lc_time).refcount) as *mut c_long)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_locale_release_lc_time_reference(lc_time: *const LcTimeData) -> c_long {
    if is_static_lc_time(lc_time) {
        return LONG_MAX;
    }
    interlocked_decrement(addr_of!((*lc_time).refcount) as *mut c_long)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_locale_free_lc_time_if_unreferenced(lc_time: *const LcTimeData) {
    if is_static_lc_time(lc_time) {
        return;
    }
    // `__crt_interlocked_read`
    if AtomicI32::from_ptr(addr_of!((*lc_time).refcount) as *mut c_long).load(Ordering::SeqCst) != 0 {
        return;
    }
    __acrt_locale_free_time(lc_time as *mut LcTimeData);
    free(lc_time as *mut c_void);
}

// ── locale_update.cpp ───────────────────────────────────────────────────────
//
// The `_explicit` forms take a global-state index; with state isolation
// compiled out (`state_index_count == 1`) `value_explicit(i)` reads
// `_value[i]`, which only index 0 can name. Callers pass 0.

#[no_mangle]
pub unsafe extern "C" fn __acrt_update_locale_info(ptd: *mut AcrtPtd, locale_info: *mut *mut CrtLocaleData) {
    if *locale_info != __acrt_current_locale_data && should_sync_with_global_locale(ptd) {
        *locale_info = __acrt_update_thread_locale_data();
    }
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_update_multibyte_info(
    ptd: *mut AcrtPtd,
    multibyte_info: *mut *mut CrtMultibyteData,
) {
    if *multibyte_info != __acrt_current_multibyte_data && should_sync_with_global_locale(ptd) {
        *multibyte_info = __acrt_update_thread_multibyte_data();
    }
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_update_locale_info_explicit(
    ptd: *mut AcrtPtd,
    locale_info: *mut *mut CrtLocaleData,
    current_global_state_index: usize,
) {
    let current = *addr_of!(__acrt_current_locale_data).add(current_global_state_index);
    if *locale_info != current && should_sync_with_global_locale(ptd) {
        *locale_info = __acrt_update_thread_locale_data();
    }
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_update_multibyte_info_explicit(
    ptd: *mut AcrtPtd,
    multibyte_info: *mut *mut CrtMultibyteData,
    current_global_state_index: usize,
) {
    let current = *addr_of!(__acrt_current_multibyte_data).add(current_global_state_index);
    if *multibyte_info != current && should_sync_with_global_locale(ptd) {
        *multibyte_info = __acrt_update_thread_multibyte_data();
    }
}

// ── wsetlocale.cpp: globals and the small exports ───────────────────────────

#[no_mangle]
pub static mut __acrt_locale_changed_data: c_long = 0;

/// `_first_127char`: bytes 1..=127, classified to decide `lc_clike`.
const FIRST_127CHAR: [c_char; 127] = {
    let mut a = [0 as c_char; 127];
    let mut i = 0;
    while i < 127 {
        a[i] = (i + 1) as c_char;
        i += 1;
    }
    a
};

type CategoryInit = unsafe extern "C" fn(*mut CrtLocaleData) -> c_int;

unsafe extern "C" fn no_op_initialize(_: *mut CrtLocaleData) -> c_int {
    0
}

/// `__lc_category`'s name and initializer columns. (Its `locale` column is
/// never read by anything; the C table is not referenced outside this file.)
struct LcCategory {
    catname: &'static [u16],
    init: CategoryInit,
}

static LC_CATEGORY: [LcCategory; 6] = [
    LcCategory { catname: w!("LC_ALL"), init: no_op_initialize },
    LcCategory { catname: w!("LC_COLLATE"), init: no_op_initialize },
    LcCategory { catname: w!("LC_CTYPE"), init: __acrt_locale_initialize_ctype },
    LcCategory { catname: w!("LC_MONETARY"), init: __acrt_locale_initialize_monetary },
    LcCategory { catname: w!("LC_NUMERIC"), init: __acrt_locale_initialize_numeric },
    LcCategory { catname: w!("LC_TIME"), init: __acrt_locale_initialize_time },
];

#[no_mangle]
pub unsafe extern "C" fn __acrt_set_locale_changed() {
    AtomicI32::from_ptr(addr_of_mut!(__acrt_locale_changed_data)).swap(1, Ordering::SeqCst);
}

#[no_mangle]
pub unsafe extern "C" fn _configthreadlocale(i: c_int) -> c_int {
    let ptd = __acrt_getptd();
    let retval = if (*ptd).own_locale & PER_THREAD_LOCALE_BIT == 0 {
        DISABLE_PER_THREAD_LOCALE
    } else {
        ENABLE_PER_THREAD_LOCALE
    };
    match i {
        ENABLE_PER_THREAD_LOCALE => (*ptd).own_locale |= PER_THREAD_LOCALE_BIT,
        DISABLE_PER_THREAD_LOCALE => (*ptd).own_locale &= !PER_THREAD_LOCALE_BIT,
        0 => {}
        // Used only during DLL startup for linkopt.
        -1 => __globallocalestatus = -1,
        _ => {
            *_errno() = EINVAL;
            _invalid_parameter_noinfo();
            return -1;
        }
    }
    retval
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_locale() {
    __acrt_lock(LOCALE_LOCK);
    let locale = addr_of_mut!(__acrt_current_locale_data);
    if *locale != initial_locale_data() {
        *locale = _updatetlocinfoEx_nolock(locale, initial_locale_data());
    }
    __acrt_unlock(LOCALE_LOCK);
}

#[no_mangle]
pub unsafe extern "C" fn _free_locale(plocinfo: LocaleT) {
    if plocinfo.is_null() {
        return;
    }
    __acrt_lock(MULTIBYTE_CP_LOCK);
    let mbc = (*plocinfo).mbcinfo;
    if !mbc.is_null()
        && interlocked_decrement(addr_of_mut!((*mbc).refcount)) == 0
        && mbc != addr_of_mut!(__acrt_initial_multibyte_data)
    {
        free(mbc as *mut c_void);
    }
    __acrt_unlock(MULTIBYTE_CP_LOCK);

    if !(*plocinfo).locinfo.is_null() {
        // Under the lock: another thread may hold this locale and be freeing it
        // too.
        __acrt_lock(LOCALE_LOCK);
        __acrt_release_locale_ref((*plocinfo).locinfo);
        let li = (*plocinfo).locinfo;
        if !li.is_null() && (*li).refcount == 0 && li != initial_locale_data() {
            __acrt_free_locale(li);
        }
        __acrt_unlock(LOCALE_LOCK);
    }
    free(plocinfo as *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn _get_current_locale() -> LocaleT {
    let ptd = __acrt_getptd();
    let copy = calloc(1, core::mem::size_of::<CrtLocalePointers>()) as LocaleT;
    if copy.is_null() {
        return null_mut(); // calloc set errno
    }
    __acrt_update_thread_locale_data();
    __acrt_update_thread_multibyte_data();

    (*copy).locinfo = (*ptd).locale_info;
    (*copy).mbcinfo = (*ptd).multibyte_info;
    __acrt_lock(LOCALE_LOCK);
    __acrt_add_locale_ref((*copy).locinfo);
    __acrt_unlock(LOCALE_LOCK);

    __acrt_lock(MULTIBYTE_CP_LOCK);
    interlocked_increment(addr_of_mut!((*(*copy).mbcinfo).refcount));
    __acrt_unlock(MULTIBYTE_CP_LOCK);
    copy
}

/// `_copytlocinfo_nolock`
unsafe fn copytlocinfo_nolock(ptlocid: *mut CrtLocaleData, ptlocis: *mut CrtLocaleData) {
    if !ptlocis.is_null() && !ptlocid.is_null() && ptlocid != ptlocis {
        core::ptr::copy_nonoverlapping(ptlocis, ptlocid, 1);
        (*ptlocid).refcount = 0;
        __acrt_add_locale_ref(ptlocid);
    }
}

/// `sync_legacy_variables_lk`
unsafe fn sync_legacy_variables_lk() {
    let cur = __acrt_current_locale_data;
    __acrt_lconv = (*cur).lconv as *mut crate::locale::Lconv;
    _pctype = (*cur).public.locale_pctype;
    __mb_cur_max = (*cur).public.locale_mb_cur_max;
}

#[no_mangle]
pub unsafe extern "C" fn _wcreate_locale(category: c_int, locale: *const u16) -> LocaleT {
    if !(CRT_LC_MIN..=CRT_LC_MAX).contains(&category) || locale.is_null() {
        return null_mut();
    }

    // Each `_VALIDATE_RETURN_NOEXC(..., ENOMEM, nullptr)`; the unique_ptrs
    // free whatever was already allocated.
    let result = calloc(1, core::mem::size_of::<CrtLocalePointers>()) as LocaleT;
    if result.is_null() {
        *_errno() = ENOMEM;
        return null_mut();
    }
    let locale_data = calloc(1, core::mem::size_of::<CrtLocaleData>()) as *mut CrtLocaleData;
    if locale_data.is_null() {
        *_errno() = ENOMEM;
        free(result as *mut c_void);
        return null_mut();
    }
    let multibyte_data = calloc(1, core::mem::size_of::<CrtMultibyteData>()) as *mut CrtMultibyteData;
    if multibyte_data.is_null() {
        *_errno() = ENOMEM;
        free(locale_data as *mut c_void);
        free(result as *mut c_void);
        return null_mut();
    }

    copytlocinfo_nolock(locale_data, initial_locale_data());

    if wsetlocale_nolock(locale_data, category, locale).is_null()
        || _setmbcp_nolock((*locale_data).public.locale_lc_codepage as c_int, multibyte_data) != 0
    {
        __acrt_release_locale_ref(locale_data);
        __acrt_free_locale(locale_data);
        free(multibyte_data as *mut c_void);
        free(result as *mut c_void);
        return null_mut();
    }

    (*multibyte_data).refcount = 1;
    (*result).locinfo = locale_data;
    (*result).mbcinfo = multibyte_data;
    result
}

#[no_mangle]
pub unsafe extern "C" fn _create_locale(category: c_int, locale: *const c_char) -> LocaleT {
    let mut wlocale = [0u16; MAX_LC_LEN];
    if !(CRT_LC_MIN..=CRT_LC_MAX).contains(&category) || locale.is_null() {
        return null_mut();
    }
    if __acrt_MultiByteToWideChar(CP_ACP, 0, locale, -1, wlocale.as_mut_ptr(), MAX_LC_LEN as c_int) == 0 {
        return null_mut();
    }
    _wcreate_locale(category, wlocale.as_ptr())
}

// ── _wsetlocale ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _wsetlocale(category: c_int, wlocale: *const u16) -> *mut u16 {
    if !(CRT_LC_MIN..=CRT_LC_MAX).contains(&category) {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return null_mut();
    }

    let ptd = __acrt_getptd();

    // Load the locale APIs BEFORE taking the locale lock: a LoadLibrary under
    // it inverts the loader-lock / locale-lock order (new threads take the
    // locale lock under the loader lock).
    __acrt_eagerly_load_locale_apis();
    __acrt_update_thread_locale_data();

    // No global resync while we call locale-less functions below.
    (*ptd).own_locale |= WSETLOCALE_AVOID_SYNC_LOCALE_BIT;

    let mut retval: *mut u16 = null_mut();
    let ptloci = calloc(1, core::mem::size_of::<CrtLocaleData>()) as *mut CrtLocaleData;
    if !ptloci.is_null() {
        __acrt_lock(LOCALE_LOCK);
        copytlocinfo_nolock(ptloci, (*ptd).locale_info);
        retval = wsetlocale_nolock(ptloci, category, wlocale);
        if !retval.is_null() {
            // Only a change AWAY from "C" marks the locale changed; nothing
            // ever marks it unchanged again.
            if !wlocale.is_null() && wcscmp(wlocale, wide_c_locale_string()) != 0 {
                __acrt_set_locale_changed();
            }
            _updatetlocinfoEx_nolock(addr_of_mut!((*ptd).locale_info), ptloci);
            __acrt_release_locale_ref(ptloci);
            // Deliberately not should_sync_with_global_locale(): the AVOID_SYNC
            // bit set above would answer for it.
            if (*ptd).own_locale & PER_THREAD_LOCALE_BIT == 0 && __globallocalestatus & GLOBAL_LOCALE_BIT == 0 {
                _updatetlocinfoEx_nolock(addr_of_mut!(__acrt_current_locale_data), (*ptd).locale_info);
                sync_legacy_variables_lk();
            }
        } else {
            __acrt_release_locale_ref(ptloci);
            __acrt_free_locale(ptloci);
        }
        __acrt_unlock(LOCALE_LOCK);
    }

    (*ptd).own_locale &= !WSETLOCALE_AVOID_SYNC_LOCALE_BIT;
    retval
}

/// `_wsetlocale_nolock`
unsafe fn wsetlocale_nolock(ploci: *mut CrtLocaleData, category: c_int, wlocale: *const u16) -> *mut u16 {
    if category != CRT_LC_ALL {
        return if wlocale.is_null() {
            (*cat(ploci, category)).wlocale
        } else {
            wsetlocale_set_cat(ploci, category, wlocale)
        };
    }

    if wlocale.is_null() {
        return wsetlocale_get_all(ploci);
    }

    let mut lctemp = [0u16; MAX_LC_LEN];
    let mut same = true;
    let mut locale_set = 0;

    if *wlocale == b'L' as u16 && *wlocale.add(1) == b'C' as u16 && *wlocale.add(2) == b'_' as u16 {
        // A compound "LC_COLLATE=x;LC_CTYPE=y;..." string. Unknown LC_ names
        // are skipped; it fails only if every category failed.
        let mut p = wlocale;
        loop {
            let s = wcspbrk(p, w!("=;").as_ptr());
            if s.is_null() {
                return null_mut();
            }
            let len = s.offset_from(p) as usize;
            if len == 0 || *s == b';' as u16 {
                return null_mut();
            }

            let mut i = CRT_LC_ALL + 1;
            while i <= CRT_LC_MAX {
                let name = LC_CATEGORY[i as usize].catname;
                if wcsncmp(name.as_ptr(), p, len) == 0 && len == name.len() - 1 {
                    break;
                }
                i += 1;
            }

            let s = s.add(1);
            let len = wcscspn(s, w!(";").as_ptr());
            if len == 0 && *s != b';' as u16 {
                return null_mut();
            }

            if i <= CRT_LC_MAX {
                errcheck(wcsncpy_s(lctemp.as_mut_ptr(), MAX_LC_LEN, s, len));
                lctemp[len] = 0;
                if !wsetlocale_set_cat(ploci, i, lctemp.as_ptr()).is_null() {
                    locale_set += 1;
                }
            }
            p = s.add(len);
            if *p != 0 {
                p = p.add(1);
            }
            if *p == 0 {
                break;
            }
        }
        return if locale_set != 0 { wsetlocale_get_all(ploci) } else { null_mut() };
    }

    // A simple LC_ALL locale string.
    let mut code_page: c_uint = 0;
    let mut locale_name_temp = [0u16; LOCALE_NAME_MAX_LENGTH];
    let retval = expandlocale(
        wlocale,
        lctemp.as_mut_ptr(),
        MAX_LC_LEN,
        locale_name_temp.as_mut_ptr(),
        LOCALE_NAME_MAX_LENGTH,
        &mut code_page,
    );
    if retval.is_null() {
        return null_mut();
    }
    for i in CRT_LC_MIN..=CRT_LC_MAX {
        if i == CRT_LC_ALL {
            continue;
        }
        if wcscmp(lctemp.as_ptr(), (*cat(ploci, i)).wlocale) != 0 {
            if !wsetlocale_set_cat(ploci, i, lctemp.as_ptr()).is_null() {
                locale_set += 1;
            } else {
                same = false;
            }
        } else {
            locale_set += 1; // trivial success
        }
    }
    if same || locale_set != 0 {
        wsetlocale_get_all(ploci)
    } else {
        null_mut()
    }
}

/// `_wsetlocale_set_cat`
unsafe fn wsetlocale_set_cat(ploci: *mut CrtLocaleData, category: c_int, wlocale: *const u16) -> *mut u16 {
    let mut cptemp: c_uint = 0;
    let mut lctemp = [0u16; MAX_LC_LEN];
    let mut locale_name_string = [0u16; LOCALE_NAME_MAX_LENGTH];
    let mut out = [0u16; 127];
    let ptd = __acrt_getptd();
    let loc_c: *mut CtypeCompatibilityData = addr_of_mut!((*ptd).setloc_data.loc_c) as *mut CtypeCompatibilityData;
    let loc_ccache = 5usize;

    if expandlocale(
        wlocale,
        lctemp.as_mut_ptr(),
        MAX_LC_LEN,
        locale_name_string.as_mut_ptr(),
        LOCALE_NAME_MAX_LENGTH,
        &mut cptemp,
    )
    .is_null()
    {
        return null_mut();
    }

    let slot = cat(ploci, category);
    if wcscmp(lctemp.as_ptr(), (*slot).wlocale) == 0 {
        return (*slot).wlocale;
    }

    let cch = wcslen(lctemp.as_ptr()) + 1;
    let pch = malloc(core::mem::size_of::<c_int>() + cch * 2) as *mut u16;
    if pch.is_null() {
        return null_mut();
    }
    let pch_cat_locale = pch.add(core::mem::size_of::<c_int>() / 2);

    // Saved for a possible restore.
    let oldlocale = (*slot).wlocale;
    let oldlocalename = (*ploci).locale_name[category as usize];
    let oldcodepage = (*ploci).public.locale_lc_codepage;

    errcheck(wcscpy_s(pch_cat_locale, cch, lctemp.as_ptr()));
    (*slot).wlocale = pch_cat_locale;

    (*ploci).locale_name[category as usize] = if lctemp[0] == b'C' as u16 && lctemp[1] == 0 {
        null_mut()
    } else {
        __acrt_copy_locale_name(locale_name_string.as_ptr())
    };

    if category == CRT_LC_CTYPE {
        (*ploci).public.locale_lc_codepage = cptemp;
        // A move-to-front cache of "does this code page classify 1..127 like
        // the C locale", in the thread's _setloc_data.
        let mut buf1 = *loc_c.add(loc_ccache - 1);
        let mut i = 0;
        while i < loc_ccache {
            let entry = loc_c.add(i);
            if (*ploci).public.locale_lc_codepage == (*entry).id {
                if i != 0 {
                    *loc_c = *entry;
                    *entry = buf1;
                }
                break;
            }
            let buf2 = *entry;
            *entry = buf1;
            buf1 = buf2;
            i += 1;
        }
        if i == loc_ccache {
            let got = __acrt_GetStringTypeA(
                null_mut(),
                CT_CTYPE1,
                FIRST_127CHAR.as_ptr(),
                127,
                out.as_mut_ptr(),
                (*ploci).public.locale_lc_codepage as c_int,
                1,
            );
            (*loc_c).is_clike = if got != 0 {
                for o in out.iter_mut() {
                    *o &= CTYPE_STYLE_MASK;
                }
                // `_ctype_loc_style = _wctype + 2`: the wide table's rows for 1..127.
                let style = addr_of!(_wctype) as *const u16;
                (memcmp(out.as_ptr() as *const c_void, style.add(2) as *const c_void, 127 * 2) == 0) as c_int
            } else {
                0
            };
            (*loc_c).id = (*ploci).public.locale_lc_codepage;
        }
        (*ploci).lc_clike = (*loc_c).is_clike;
    } else if category == CRT_LC_COLLATE {
        (*ploci).lc_collate_cp = cptemp;
    } else if category == CRT_LC_TIME {
        (*ploci).lc_time_cp = cptemp;
    }

    if (LC_CATEGORY[category as usize].init)(ploci) != 0 {
        // Restore -- and free the NEW name (the C does exactly this).
        (*slot).wlocale = oldlocale;
        free((*ploci).locale_name[category as usize] as *mut c_void);
        (*ploci).locale_name[category as usize] = oldlocalename;
        free(pch as *mut c_void);
        (*ploci).public.locale_lc_codepage = oldcodepage;
        return null_mut();
    }

    // `_ASSERT(0)` path in the C: the old string's last reference. Note it
    // frees the NEW locale name and nulls the new wlocale -- transcribed.
    if oldlocale != wide_c_locale_string() && interlocked_decrement((*slot).wrefcount) == 0 {
        free((*slot).wrefcount as *mut c_void);
        free((*slot).refcount as *mut c_void);
        free((*ploci).locale_name[category as usize] as *mut c_void);
        (*slot).wlocale = null_mut();
        (*ploci).locale_name[category as usize] = null_mut();
    }
    *(pch as *mut c_long) = 1;
    (*slot).wrefcount = pch as *mut c_long;
    (*slot).wlocale
}

/// `_wcscats`: append each string with `wcscat_s`, Watson on failure.
unsafe fn wcscats(out: *mut u16, size: usize, parts: &[*const u16]) {
    for &p in parts {
        errcheck(wcscat_s(out, size, p));
    }
}

/// `_wsetlocale_get_all`: the "LC_COLLATE=...;LC_CTYPE=...;..." string, or
/// just the LC_CTYPE name when every category agrees.
unsafe fn wsetlocale_get_all(ploci: *mut CrtLocaleData) -> *mut u16 {
    let refcount_size = core::mem::size_of::<c_int>()
        + 2 * (MAX_LC_LEN + 1) * (CRT_LC_MAX - CRT_LC_MIN + 1) as usize
        + 2 * CATNAMES_LEN;
    let refcount = malloc(refcount_size) as *mut c_long;
    if refcount.is_null() {
        return null_mut();
    }
    let pch = (refcount as *mut u16).add(core::mem::size_of::<c_int>() / 2);
    let cch = (refcount_size - core::mem::size_of::<c_int>()) / 2;
    *pch = 0;
    *refcount = 1;

    let mut same = true;
    let mut i = CRT_LC_MIN + 1;
    loop {
        wcscats(pch, cch, &[LC_CATEGORY[i as usize].catname.as_ptr(), w!("=").as_ptr(), (*cat(ploci, i)).wlocale]);
        if i < CRT_LC_MAX {
            errcheck(wcscat_s(pch, cch, w!(";").as_ptr()));
            if wcscmp((*cat(ploci, i)).wlocale, (*cat(ploci, i + 1)).wlocale) != 0 {
                same = false;
            }
            i += 1;
            continue;
        }

        let all = cat(ploci, CRT_LC_ALL);
        // Both arms drop LC_ALL's old composite first (`_ASSERT(0)` if either
        // count reaches zero).
        if !(*all).wrefcount.is_null() && interlocked_decrement((*all).wrefcount) == 0 {
            free((*all).wrefcount as *mut c_void);
        }
        if !(*all).refcount.is_null() && interlocked_decrement((*all).refcount) == 0 {
            free((*all).refcount as *mut c_void);
        }
        (*all).refcount = null_mut();
        (*all).locale = null_mut();
        if !same {
            (*all).wrefcount = refcount;
            (*all).wlocale = pch;
            return pch;
        }
        // The C frees the new block BEFORE the decrements; nothing reads it in
        // between, so the order is not observable.
        free(refcount as *mut c_void);
        (*all).wrefcount = null_mut();
        (*all).wlocale = null_mut();
        return (*cat(ploci, CRT_LC_CTYPE)).wlocale;
    }
}

// ── BCP-47 parsing for _expandlocale ────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Delim {
    Normal,
    EndOfString,
    CodePage,
}

#[derive(Clone, Copy)]
struct Section {
    ptr: *const u16,
    length: usize,
    delimiter: Delim,
}

fn categorize_delimiter(wc: u16) -> Delim {
    match wc {
        0x2d | 0x5f => Delim::Normal, // '-' '_'
        0x2e => Delim::CodePage,      // '.'
        _ => Delim::EndOfString,
    }
}

/// `__acrt_locale_get_ctype_array_value(__pctype_func(), c, mask)`
unsafe fn ctype_test(c: u16, mask: u16) -> bool {
    let c = c as c_int;
    (-1..=255).contains(&c) && *__pctype_func().offset(c as isize) & mask != 0
}

unsafe fn string_is(s: *const u16, len: usize, mask: u16) -> bool {
    (0..len).all(|i| ctype_test(*s.add(i), mask))
}

unsafe fn parse_bcp47_language(names: *mut LocaleStrings, s: &Section) -> bool {
    if s.delimiter != Delim::Normal || !(2..=3).contains(&s.length) || !string_is(s.ptr, s.length, CTYPE_ALPHA) {
        return false;
    }
    errcheck(wcsncpy_s((*names).language.as_mut_ptr(), MAX_LANG_LEN, s.ptr, s.length));
    errcheck(wcsncpy_s((*names).locale_name.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH, s.ptr, s.length));
    true
}

unsafe fn parse_bcp47_script(names: *mut LocaleStrings, s: &Section) -> bool {
    if s.delimiter != Delim::Normal || s.length != 4 || !string_is(s.ptr, s.length, CTYPE_ALPHA) {
        return false;
    }
    errcheck(wcsncat_s((*names).locale_name.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH, w!("-").as_ptr(), 1));
    errcheck(wcsncat_s((*names).locale_name.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH, s.ptr, s.length));
    true
}

unsafe fn parse_bcp47_region(names: *mut LocaleStrings, s: &Section) -> bool {
    if s.delimiter != Delim::Normal {
        return false;
    }
    if !(s.length == 2 && string_is(s.ptr, s.length, CTYPE_ALPHA))
        && !(s.length == 3 && string_is(s.ptr, s.length, CTYPE_DIGIT))
    {
        return false;
    }
    errcheck(wcsncpy_s((*names).country.as_mut_ptr(), MAX_CTRY_LEN, s.ptr, s.length));
    errcheck(wcsncat_s((*names).locale_name.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH, w!("-").as_ptr(), 1));
    errcheck(wcsncat_s((*names).locale_name.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH, s.ptr, s.length));
    true
}

unsafe fn parse_bcp47_code_page(names: *mut LocaleStrings, s: &Section) -> bool {
    if s.delimiter != Delim::CodePage {
        return false;
    }
    errcheck(wcsncpy_s((*names).code_page.as_mut_ptr(), MAX_CP_LEN, s.ptr, s.length));
    true
}

unsafe fn parse_bcp47(names: *mut LocaleStrings, expr: *const u16) -> bool {
    core::ptr::write_bytes(names, 0, 1);
    let mut sections = [Section { ptr: null(), length: 0, delimiter: Delim::Normal }; 4];
    let mut num = 0usize;
    let mut p = expr;
    let mut last = Delim::Normal;
    while last != Delim::EndOfString {
        if num >= 4 {
            return false; // ran out of sections before the end of the string
        }
        let len = if last != Delim::CodePage { wcscspn(p, w!("-_.").as_ptr()) } else { wcslen(p) };
        sections[num] = Section { ptr: p, length: len, delimiter: last };
        p = p.add(len);
        num += 1;
        last = categorize_delimiter(*p);
        p = p.add(1);
    }
    let s = &sections;
    match num {
        1 => parse_bcp47_language(names, &s[0]),
        2 => {
            parse_bcp47_language(names, &s[0])
                && (parse_bcp47_script(names, &s[1])
                    || parse_bcp47_region(names, &s[1])
                    || parse_bcp47_code_page(names, &s[1]))
        }
        3 => {
            parse_bcp47_language(names, &s[0])
                && ((parse_bcp47_script(names, &s[1])
                    && (parse_bcp47_region(names, &s[2]) || parse_bcp47_code_page(names, &s[2])))
                    || (parse_bcp47_region(names, &s[1]) && parse_bcp47_code_page(names, &s[2])))
        }
        4 => {
            parse_bcp47_language(names, &s[0])
                && parse_bcp47_script(names, &s[1])
                && parse_bcp47_region(names, &s[2])
                && parse_bcp47_code_page(names, &s[3])
        }
        _ => false,
    }
}

/// `get_default_code_page`: the locale's ANSI code page, or UTF-8 when it has
/// none.
unsafe fn get_default_code_page(name: *const u16) -> c_int {
    let mut code_page: c_int = 0;
    let written = __acrt_GetLocaleInfoEx(
        name,
        LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
        addr_of_mut!(code_page) as *mut u16,
        (core::mem::size_of::<c_int>() / 2) as c_int,
    );
    if written == 0 || code_page == 0 {
        CP_UTF8
    } else {
        code_page
    }
}

/// `_expandlocale_locale_name_cache`: `localeNameOutput` starts as the cached
/// name, and if the lookup is not committed the cache is rewritten from it on
/// the way out.
struct LocaleNameCache {
    output: *mut u16,
    output_size: usize,
    cache: *mut u16,
    committed: bool,
}

impl LocaleNameCache {
    unsafe fn new(output: *mut u16, output_size: usize, cache: *mut u16) -> Self {
        errcheck(wcsncpy_s(output, output_size, cache, LOCALE_NAME_MAX_LENGTH));
        LocaleNameCache { output, output_size, cache, committed: false }
    }

    unsafe fn commit_locale_name(&mut self, name: *const u16, len: usize) {
        errcheck(wcsncpy_s(self.cache, LOCALE_NAME_MAX_LENGTH, name, len));
        self.commit_locale_name_cache_already_updated(name, len);
    }

    unsafe fn commit_locale_name_cache_already_updated(&mut self, name: *const u16, len: usize) {
        errcheck(wcsncpy_s(self.output, self.output_size, name, len));
        self.committed = true;
    }
}

impl Drop for LocaleNameCache {
    fn drop(&mut self) {
        if !self.committed {
            unsafe { errcheck(wcsncpy_s(self.cache, LOCALE_NAME_MAX_LENGTH, self.output, self.output_size)) };
        }
    }
}

#[inline]
fn ascii_towlower(c: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&c) {
        c - b'A' as u16 + b'a' as u16
    } else {
        c
    }
}

/// `_expandlocale`: resolve a locale expression to the name handed back to
/// the user (`output`, and the returned pointer into the ptd cache), the name
/// used for Win32 calls (`locale_name_output`) and the code page.
unsafe fn expandlocale(
    expr: *const u16,
    output: *mut u16,
    size_in_chars: usize,
    locale_name_output: *mut u16,
    locale_name_size: usize,
    output_code_page: &mut c_uint,
) -> *mut u16 {
    if expr.is_null() {
        return null_mut();
    }

    // "C": callers know locale_name_output was not updated.
    if *expr == b'C' as u16 && *expr.add(1) == 0 {
        errcheck(wcscpy_s(output, size_in_chars, w!("C").as_ptr()));
        *output_code_page = CP_ACP;
        return output;
    }

    let setloc = addr_of_mut!((*__acrt_getptd()).setloc_data);
    let pcachecp = addr_of_mut!((*setloc).cachecp);
    let cachein = addr_of_mut!((*setloc).cachein) as *mut u16;
    let cacheout = addr_of_mut!((*setloc).cacheout) as *mut u16;

    let mut guard = LocaleNameCache::new(
        locale_name_output,
        locale_name_size,
        addr_of_mut!((*setloc).cache_locale_name) as *mut u16,
    );

    let chars = wcslen(expr);
    if chars >= MAX_LC_LEN || (wcscmp(cacheout, expr) != 0 && wcscmp(cachein, expr) != 0) {
        let mut names: LocaleStrings = core::mem::zeroed();
        let names_p = addr_of_mut!(names);
        let mut getqloc_results = 0;
        let is_downlevel = !__acrt_can_use_vista_locale_apis();

        if lc_wcstolc(names_p, expr) == 0 {
            getqloc_results = if is_downlevel {
                __acrt_get_qualified_locale_downlevel(names_p, pcachecp, names_p)
            } else {
                __acrt_get_qualified_locale(names_p, pcachecp, names_p)
            };
        }

        if getqloc_results != 0 {
            // Legacy/Windows name: both names normalized.
            lc_lctowcs(cacheout, MAX_LC_LEN, names_p);
            // get_qualified_locale already updated the name cache.
            guard.commit_locale_name_cache_already_updated(
                names.locale_name.as_ptr(),
                wcslen(names.locale_name.as_ptr()) + 1,
            );
        } else if __acrt_IsValidLocaleName(expr) != 0 {
            // Windows knows the whole string: only the code page is looked up.
            let code_page = get_default_code_page(expr);
            *pcachecp = code_page as u16 as c_uint;
            errcheck(wcsncpy_s(cacheout, MAX_LC_LEN, expr, chars + 1));
            guard.commit_locale_name(expr, chars + 1);
        } else if parse_bcp47(names_p, expr) && __acrt_IsValidLocaleName(names.locale_name.as_ptr()) != 0 {
            let normalized = names.locale_name.as_ptr();
            let code_page = if names.code_page[0] != 0 {
                let cp = &names.code_page;
                // Precedence kept from the C: `u && t && f && (8 NUL) || (- 8 NUL)`,
                // so the second alternative does not need the "utf" prefix.
                if (ascii_towlower(cp[0]) == b'u' as u16
                    && ascii_towlower(cp[1]) == b't' as u16
                    && ascii_towlower(cp[2]) == b'f' as u16
                    && (cp[3] == b'8' as u16 && cp[4] == 0))
                    || (cp[3] == b'-' as u16 && cp[4] == b'8' as u16 && cp[5] == 0)
                {
                    CP_UTF8
                } else {
                    // Other code pages are not supported for BCP-47 tags.
                    return null_mut();
                }
            } else {
                get_default_code_page(normalized)
            };
            *pcachecp = code_page as u16 as c_uint;
            errcheck(wcsncpy_s(cacheout, MAX_LC_LEN, expr, chars + 1));
            guard.commit_locale_name(normalized, wcslen(normalized) + 1);
        } else {
            return null_mut();
        }

        if *expr != 0 && chars < MAX_LC_LEN {
            errcheck(wcsncpy_s(cachein, MAX_LC_LEN, expr, chars + 1));
        } else {
            *cachein = 0;
        }
    }

    *output_code_page = *pcachecp;
    errcheck(wcscpy_s(output, size_in_chars, cacheout));
    cacheout
}

/// `__lc_wcstolc`: split "Language_Country.CodePage" (or ".CodePage") into
/// `names`. 0 on success, -1 on a syntax error.
unsafe fn lc_wcstolc(names: *mut LocaleStrings, wlocale: *const u16) -> c_int {
    core::ptr::write_bytes(names, 0, 1);
    if *wlocale == 0 {
        return 0;
    }
    if *wlocale == b'.' as u16 && *wlocale.add(1) != 0 {
        errcheck(wcsncpy_s((*names).code_page.as_mut_ptr(), MAX_CP_LEN, wlocale.add(1), MAX_CP_LEN - 1));
        (*names).code_page[MAX_CP_LEN - 1] = 0;
        return 0;
    }

    let mut w = wlocale;
    let mut i = 0;
    loop {
        let len = wcscspn(w, w!("_.,").as_ptr());
        if len == 0 {
            return -1; // cannot start with a separator
        }
        let wch = *w.add(len);

        if i == 0 && len < MAX_LANG_LEN {
            errcheck(wcsncpy_s((*names).language.as_mut_ptr(), MAX_LANG_LEN, w, len));
            if wch == b'.' as u16 {
                i += 1; // a code page follows: skip the country slot
            }
        } else if i == 1 && len < MAX_CTRY_LEN && wch != b'_' as u16 {
            errcheck(wcsncpy_s((*names).country.as_mut_ptr(), MAX_CTRY_LEN, w, len));
        } else if i == 2 && len < MAX_CP_LEN && (wch == 0 || wch == b',' as u16) {
            errcheck(wcsncpy_s((*names).code_page.as_mut_ptr(), MAX_CP_LEN, w, len));
        } else {
            return -1;
        }

        if wch == b',' as u16 {
            break; // a POSIX modifier: parsed past, not used
        }
        if wch == 0 {
            break;
        }
        w = w.add(len + 1);
        i += 1;
    }
    0
}

/// `__lc_lctowcs`: "Language_Country.CodePage" from its parts.
unsafe fn lc_lctowcs(locale: *mut u16, size: usize, names: *const LocaleStrings) {
    errcheck(wcscpy_s(locale, size, (*names).language.as_ptr()));
    if (*names).country[0] != 0 {
        wcscats(locale, size, &[w!("_").as_ptr(), (*names).country.as_ptr()]);
    }
    if (*names).code_page[0] != 0 {
        wcscats(locale, size, &[w!(".").as_ptr(), (*names).code_page.as_ptr()]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_copy_locale_name(locale_name: *const u16) -> *mut u16 {
    if locale_name.is_null() {
        return null_mut();
    }
    let cch = wcsnlen(locale_name, LOCALE_NAME_MAX_LENGTH);
    if cch >= LOCALE_NAME_MAX_LENGTH {
        return null_mut();
    }
    let copy = malloc((cch + 1) * 2) as *mut u16;
    if copy.is_null() {
        return null_mut();
    }
    errcheck(wcsncpy_s(copy, cch + 1, locale_name, cch + 1));
    copy
}

// ── setlocale.cpp ───────────────────────────────────────────────────────────

/// `call_wsetlocale`: widen, then `_wsetlocale`.
unsafe fn call_wsetlocale(category: c_int, narrow: *const c_char) -> *mut u16 {
    if narrow.is_null() {
        return _wsetlocale(category, null());
    }
    let mut size: usize = 0;
    // The result is deliberately not checked beyond the Watson cases.
    errcheck_einval_erange(mbstowcs_s(&mut size, null_mut(), 0, narrow, INT_MAX));

    let wide = calloc(size, 2) as *mut u16;
    if wide.is_null() {
        return null_mut();
    }
    let r = if errcheck_einval_erange(mbstowcs_s(null_mut(), wide, size, narrow, TRUNCATE)) != 0 {
        null_mut()
    } else {
        _wsetlocale(category, wide)
    };
    free(wide as *mut c_void);
    r
}

#[no_mangle]
pub unsafe extern "C" fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char {
    // See _wsetlocale on loading these before taking the lock.
    __acrt_eagerly_load_locale_apis();
    __acrt_lock(LOCALE_LOCK);
    let r = setlocale_locked(category, locale);
    __acrt_unlock(LOCALE_LOCK);
    r
}

unsafe fn setlocale_locked(category: c_int, locale: *const c_char) -> *mut c_char {
    let outwlocale = call_wsetlocale(category, locale);
    if outwlocale.is_null() {
        return null_mut();
    }

    let ptd = __acrt_getptd();
    let mut locale_ptrs = CrtLocalePointers { locinfo: (*ptd).locale_info, mbcinfo: (*ptd).multibyte_info };
    let lp: LocaleT = &mut locale_ptrs;

    // The explicit-locale conversions, so this thread's locale is not
    // refreshed (and the string freed) under us.
    let mut size: usize = 0;
    if errcheck_einval_erange(_wcstombs_s_l(&mut size, null_mut(), 0, outwlocale, 0, lp)) != 0 {
        return null_mut();
    }

    let refcount = malloc(size + core::mem::size_of::<c_long>()) as *mut c_long;
    if refcount.is_null() {
        return null_mut();
    }
    let outlocale = refcount.add(1) as *mut c_char;

    if errcheck_einval_erange(_wcstombs_s_l(null_mut(), outlocale, size, outwlocale, TRUNCATE, lp)) != 0 {
        free(refcount as *mut c_void);
        return null_mut();
    }

    let ptloci = locale_ptrs.locinfo;
    let slot = cat(ptloci, category);

    if !(*slot).refcount.is_null() && interlocked_decrement((*slot).refcount) == 0 {
        free((*slot).refcount as *mut c_void);
        (*slot).refcount = null_mut();
    }
    if should_sync_with_global_locale(ptd)
        && !(*slot).refcount.is_null()
        && interlocked_decrement((*slot).refcount) == 0
    {
        free((*slot).refcount as *mut c_void);
        (*slot).refcount = null_mut();
    }

    // The "risky trick": this category's narrow string starts with the WHOLE
    // struct's refcount. Only sound under the locale lock.
    *refcount = (*ptloci).refcount;
    (*slot).refcount = refcount;
    (*slot).locale = outlocale;
    outlocale
}
