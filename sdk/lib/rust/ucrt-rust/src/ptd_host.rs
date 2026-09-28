//! UCRT's `__crt_cached_ptd_host`, layout-identical, so Rust can hand one to
//! the C internals that take it by reference.
//!
//! ## What it is
//!
//! Every newer UCRT entry point constructs one of these on its stack and threads
//! it by reference through everything it calls (`corecrt_internal_ptd_propagation.h`).
//! It caches three things for the duration of the call:
//!
//!   * the resolved locale, so a call sees ONE locale even if another thread
//!     calls `setlocale` halfway through;
//!   * the per-thread data pointer, so `FlsGetValue` runs at most once;
//!   * errno and `_doserrno`, written into the ptd only when the host is
//!     destroyed.
//!
//! The third is observable, and it is why this exists rather than a plain
//! `_errno()` write. Inside the call, errno lives in the cache: the invalid
//! parameter handler runs BEFORE the ptd sees the new value, and an errno
//! "guard" restores the cache, not the ptd. printf's EILSEQ recovery, its width
//! parser's ERANGE check and `_vsnprintf_s`'s restore point are all written
//! against the cache, and cannot be reproduced over `_errno()` alone.
//!
//! ## Why the layout is pinned
//!
//! The C internals we still call -- `__acrt_fp_format`, `_wctomb_internal`,
//! `_fputwc_nolock_internal`, `_invalid_parameter_internal` -- inline the
//! class's accessors, so they read and write these fields directly. The offsets
//! below were measured with the build's own compiler flags on all three targets
//! (a probe TU emitting `offsetof` for each member, with `private` defined away),
//! and are asserted at compile time on every build.
//!
//! Same transitional stance as `locale_update.rs`: this reads the C's ptd, it
//! does not start a parallel one.

use core::ffi::{c_int, c_ulong};

use crate::locale::CrtLocalePointers;
use crate::locale_update::{
    __acrt_getptd, __acrt_initial_locale_pointers, __acrt_locale_changed_data,
    __acrt_update_locale_info, __acrt_update_multibyte_info, PER_THREAD_LOCALE_BIT,
};
use crate::ptd::AcrtPtd;

/// `locale_status` (corecrt_internal_ptd_propagation.h:80).
const UNINITIALIZED: u8 = 0;
const UPDATED_ON_CONSTRUCTION: u8 = 1;
const UPDATED_VIA_PTD: u8 = 2;

/// `__crt_cached_ptd_host::cached<T>`: a value and whether it has been set.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Cached<T: Copy> {
    value: T,
    valid: bool,
}

impl<T: Copy + PartialEq> Cached<T> {
    const fn new(zero: T) -> Self {
        Self { value: zero, valid: false }
    }
    pub fn valid(&self) -> bool {
        self.valid
    }
    pub fn set(&mut self, value: T) {
        self.valid = true;
        self.value = value;
    }
    pub fn check(&self, value: T) -> bool {
        self.valid && self.value == value
    }
}

#[repr(C)]
pub struct PtdHost {
    ptd: *mut AcrtPtd,
    current_global_state_index: usize,
    current_global_state_index_valid: bool,
    locale_pointers: CrtLocalePointers,
    locale_status: u8,
    current_errno: Cached<c_int>,
    current_doserrno: Cached<c_ulong>,
}

macro_rules! pin {
    ($field:ident, $want:expr) => {
        const _: () = assert!(
            core::mem::offset_of!(PtdHost, $field) == $want,
            concat!("__crt_cached_ptd_host.", stringify!($field), " is not where UCRT puts it"),
        );
    };
}

#[cfg(target_pointer_width = "32")]
mod pinned {
    use super::*;
    pin!(ptd, 0);
    pin!(current_global_state_index, 4);
    pin!(current_global_state_index_valid, 8);
    pin!(locale_pointers, 12);
    pin!(locale_status, 20);
    pin!(current_errno, 24);
    pin!(current_doserrno, 32);
    const _: () = assert!(core::mem::size_of::<PtdHost>() == 40);
}

#[cfg(target_pointer_width = "64")]
mod pinned {
    use super::*;
    pin!(ptd, 0);
    pin!(current_global_state_index, 8);
    pin!(current_global_state_index_valid, 16);
    pin!(locale_pointers, 24);
    pin!(locale_status, 40);
    pin!(current_errno, 44);
    pin!(current_doserrno, 52);
    const _: () = assert!(core::mem::size_of::<PtdHost>() == 64);
}

// cached<T> is { T _value; bool _valid; }: 8 bytes for both int and unsigned long.
const _: () = assert!(core::mem::size_of::<Cached<c_int>>() == 8);
const _: () = assert!(core::mem::size_of::<Cached<c_ulong>>() == 8);

extern "C" {
    /// `_UCRT_INVALID_PARAMETER` in a release build: nulls, zero, and the host.
    fn _invalid_parameter_internal(
        expression: *const u16,
        function_name: *const u16,
        file_name: *const u16,
        line_number: u32,
        reserved: usize,
        ptd: *mut PtdHost,
    );
}

impl PtdHost {
    /// The constructor: an explicit `_locale_t` is taken as-is; otherwise, if
    /// no thread has ever changed the locale, the process-initial pointers are;
    /// otherwise resolution waits for the first `get_locale()`.
    ///
    /// # Safety
    /// `locale`, if non-null, must be a valid `_locale_t`.
    pub unsafe fn new(locale: *mut CrtLocalePointers) -> Self {
        let mut host = Self {
            ptd: core::ptr::null_mut(),
            current_global_state_index: 0,
            current_global_state_index_valid: false,
            locale_pointers: CrtLocalePointers {
                locinfo: core::ptr::null_mut(),
                mbcinfo: core::ptr::null_mut(),
            },
            locale_status: UNINITIALIZED,
            current_errno: Cached::new(0),
            current_doserrno: Cached::new(0),
        };
        if !locale.is_null() {
            host.locale_pointers.locinfo = (*locale).locinfo;
            host.locale_pointers.mbcinfo = (*locale).mbcinfo;
            host.locale_status = UPDATED_ON_CONSTRUCTION;
        } else if __acrt_locale_changed_data == 0 {
            host.locale_pointers.locinfo = __acrt_initial_locale_pointers.locinfo;
            host.locale_pointers.mbcinfo = __acrt_initial_locale_pointers.mbcinfo;
            host.locale_status = UPDATED_ON_CONSTRUCTION;
        }
        host
    }

    /// `get_raw_ptd()`. `__acrt_getptd` aborts on failure, as
    /// `check_synchronize_per_thread_data` does.
    pub unsafe fn raw_ptd(&mut self) -> *mut AcrtPtd {
        if self.ptd.is_null() {
            self.ptd = __acrt_getptd();
        }
        self.ptd
    }

    /// `get_locale()`. Resolving through the ptd suppresses global-locale
    /// resynchronisation for the rest of the call, exactly as `LocaleUpdate`
    /// does; the destructor releases it.
    pub unsafe fn get_locale(&mut self) -> *mut CrtLocalePointers {
        if self.locale_status == UNINITIALIZED {
            let ptd = self.raw_ptd();
            self.locale_pointers.locinfo = (*ptd)._locale_info_ptr();
            self.locale_pointers.mbcinfo = (*ptd)._multibyte_info_ptr();
            __acrt_update_locale_info(ptd, &mut self.locale_pointers.locinfo);
            __acrt_update_multibyte_info(ptd, &mut self.locale_pointers.mbcinfo);
            // Only a thread NOT already in per-thread mode is marked, and so
            // owes the unmarking; the status is what records that, and a
            // thread that was already marked stays UNINITIALIZED here, as in
            // the C.
            if (*ptd).own_locale & PER_THREAD_LOCALE_BIT == 0 {
                (*ptd).own_locale |= PER_THREAD_LOCALE_BIT;
                self.locale_status = UPDATED_VIA_PTD;
            }
        }
        &mut self.locale_pointers
    }

    pub fn errno(&mut self) -> &mut Cached<c_int> {
        &mut self.current_errno
    }

    /// `get_doserrno()`: the cached `_doserrno`, as `errno()` is for errno.
    pub fn doserrno(&mut self) -> &mut Cached<c_ulong> {
        &mut self.current_doserrno
    }

    /// `_UCRT_VALIDATE_RETURN`'s failure arm: record `err` in the cache, then
    /// invoke the handler -- which therefore still sees the OLD errno.
    pub unsafe fn invalid_parameter(&mut self, err: c_int) {
        self.current_errno.set(err);
        _invalid_parameter_internal(
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            0,
            0,
            self,
        );
    }
}

impl Drop for PtdHost {
    fn drop(&mut self) {
        unsafe {
            if self.locale_status == UPDATED_VIA_PTD {
                (*self.ptd).own_locale &= !PER_THREAD_LOCALE_BIT;
            }
            if self.current_errno.valid {
                (*self.raw_ptd()).terrno = self.current_errno.value;
            }
            if self.current_doserrno.valid {
                (*self.raw_ptd()).tdoserrno = self.current_doserrno.value as _;
            }
        }
    }
}
