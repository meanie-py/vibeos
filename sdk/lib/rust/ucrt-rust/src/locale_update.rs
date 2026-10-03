//! Resolving "which locale does this call use" — the Rust twin of UCRT's
//! `_LocaleUpdate` (`sdk/lib/ucrt/inc/corecrt_internal.h:1911`).
//!
//! ## Why this is the keystone
//!
//! Almost every remaining family needs it, and not only the `_l`-suffixed
//! spellings. A NULL `_locale_t` does not mean "no locale": it means "resolve
//! the current thread's", so the plain `_ismbb*`, `strtod`, `_mbs*` and the
//! rest all route through here too. Nothing locale-aware can be taken over
//! from the vendored C until this exists.
//!
//! ## Transitional by design: we read the C's state, we do not start our own
//!
//! The obvious move — give ucrt-rust its own thread-local block, the way
//! msvcrt-crt-rust has `msvcrt_get_thread_data()` over plain Win32 TLS — would
//! be WRONG here, and quietly so. While any of `sdk/lib/ucrt` is still linked,
//! its C reads and writes the real `__acrt_ptd`. A second, parallel Rust ptd
//! would mean a `setlocale()` serviced by C and an `_ismbbalpha()` serviced by
//! Rust disagreeing about the current locale, per thread, with nothing to
//! notice. So this calls the SAME `__acrt_getptd()` the C does and reads the
//! same block through the layout pinned in `ptd.rs` — which is exactly what
//! pinning that layout bought.
//!
//! `__acrt_getptd` is internal to ucrtbase.dll (it is not in ucrtbase.spec),
//! but it is link-visible within the module, which is all we need. When the C
//! finally goes, the provider is swapped for a Rust one honouring the same
//! contract and nothing above this line changes.
//!
//! ## Fidelity
//!
//! Three arms, transcribed from the C's constructor:
//!   1. an explicit `_locale_t` is used as-is;
//!   2. otherwise, if the locale has never been changed on any thread, the
//!      process-initial pointers are used — the fast path, side-effect free;
//!   3. otherwise the thread's ptd is consulted and resynchronised with the
//!      global locale.
//!
//! Arm 3 has a SIDE EFFECT and therefore a lifetime, which is why this is a
//! guard type rather than a function: when the thread is not already running a
//! per-thread locale it sets `_PER_THREAD_LOCALE_BIT` to stop the locale
//! changing halfway through the call, and clears it again on drop. Getting the
//! pairing wrong leaks a thread into permanent per-thread-locale mode. `Drop`
//! is a better fit for that than C++'s destructor was, since it cannot be
//! forgotten at an early return.

use crate::locale::{CrtLocaleData, CrtLocalePointers, CrtMultibyteData};
use crate::ptd::AcrtPtd;
use core::ffi::{c_int, c_long};

/// `corecrt_internal.h:409` — set on `_own_locale` while a per-thread locale is
/// in force, and used by arm 3 to suppress resynchronisation mid-call.
pub const PER_THREAD_LOCALE_BIT: c_int = 0x2;

extern "C" {
    /// The per-thread data block. Provided by the vendored C today; see the
    /// module docs for why we deliberately share it rather than allocate our own.
    pub(crate) fn __acrt_getptd() -> *mut AcrtPtd;

    /// Resynchronise the thread's locale/multibyte pointers with the global
    /// locale. Still C: replacing these is the locale subsystem's own slice,
    /// and reimplementing them here to avoid one extern would be reimplementing
    /// the thing this module exists to defer.
    pub(crate) fn __acrt_update_locale_info(ptd: *mut AcrtPtd, data: *mut *mut CrtLocaleData);
    pub(crate) fn __acrt_update_multibyte_info(ptd: *mut AcrtPtd, data: *mut *mut CrtMultibyteData);

    /// `corecrt_internal.h:739`. Read directly rather than through the C's
    /// `__acrt_locale_changed()`, which is an `__inline` over exactly this and
    /// so has no symbol to call. Its own comment notes the read needs no
    /// interlock: a naturally-aligned 4-byte load is atomic on every target we
    /// build.
    pub(crate) static __acrt_locale_changed_data: c_long;

    /// The process-initial locale, used by the never-changed fast path.
    pub(crate) static __acrt_initial_locale_pointers: CrtLocalePointers;
}

/// A resolved locale for the duration of one call.
///
/// Hold it for as long as the resolved pointers are in use, and let it drop
/// before returning — arm 3's suppression bit is released on drop.
pub struct LocaleUpdate {
    pointers: CrtLocalePointers,
    ptd: *mut AcrtPtd,
    /// Whether THIS guard set the suppression bit, and so owes clearing it.
    /// Not "did we take arm 3": a nested call on a thread already in
    /// per-thread-locale mode takes arm 3 and must NOT clear a bit it did not set.
    updated: bool,
}

impl LocaleUpdate {
    /// # Safety
    /// `locale`, if non-null, must be a valid `_locale_t`. Must be called with
    /// the CRT initialised, since arm 3 reaches per-thread data.
    pub unsafe fn new(locale: *mut CrtLocalePointers) -> Self {
        if !locale.is_null() {
            return Self {
                pointers: CrtLocalePointers {
                    locinfo: (*locale).locinfo,
                    mbcinfo: (*locale).mbcinfo,
                },
                ptd: core::ptr::null_mut(),
                updated: false,
            };
        }

        if __acrt_locale_changed_data == 0 {
            return Self {
                pointers: CrtLocalePointers {
                    locinfo: __acrt_initial_locale_pointers.locinfo,
                    mbcinfo: __acrt_initial_locale_pointers.mbcinfo,
                },
                ptd: core::ptr::null_mut(),
                updated: false,
            };
        }

        let ptd = __acrt_getptd();
        let mut pointers = CrtLocalePointers {
            locinfo: (*ptd)._locale_info_ptr(),
            mbcinfo: (*ptd)._multibyte_info_ptr(),
        };
        __acrt_update_locale_info(ptd, &mut pointers.locinfo);
        __acrt_update_multibyte_info(ptd, &mut pointers.mbcinfo);

        let mut updated = false;
        if (*ptd).own_locale & PER_THREAD_LOCALE_BIT == 0 {
            (*ptd).own_locale |= PER_THREAD_LOCALE_BIT;
            updated = true;
        }
        Self { pointers, ptd, updated }
    }

    /// The locale data — what the C spells `GetLocaleT()->locinfo`.
    pub fn locinfo(&self) -> *mut CrtLocaleData {
        self.pointers.locinfo
    }

    /// The multibyte data — the home of the 257-byte `mbctype` table that the
    /// whole `_ismbb*` family indexes at `1 + byte`.
    pub fn mbcinfo(&self) -> *mut CrtMultibyteData {
        self.pointers.mbcinfo
    }

    /// `GetLocaleT()`: the resolved pair as a `_locale_t`, for the UCRT internals
    /// (`__acrt_GetStringTypeA` and friends) that take one. Points into `self`, so it
    /// is valid exactly as long as this `LocaleUpdate` -- the same lifetime the C
    /// object gives it.
    pub fn locale_t(&mut self) -> *mut CrtLocalePointers {
        &mut self.pointers
    }
}

impl Drop for LocaleUpdate {
    fn drop(&mut self) {
        if self.updated && !self.ptd.is_null() {
            unsafe { (*self.ptd).own_locale &= !PER_THREAD_LOCALE_BIT };
        }
    }
}
