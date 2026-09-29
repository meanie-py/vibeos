//! `startup/onexit.cpp` -- the `_onexit` registries: `_initialize_onexit_table`,
//! `_register_onexit_function`, `_execute_onexit_table`, `_crt_atexit`,
//! `_crt_at_quick_exit`, and the two process tables `__acrt_atexit_table` /
//! `__acrt_at_quick_exit_table` that `crate::internal::init` (initialize_c)
//! and `super::exit` reach by those exact names.
//!
//! Details that are silent if wrong:
//! * `_onexit_table_t` is three `_PVFV*` -- `_first`, `_last`, `_end` -- and
//!   ALL THREE are stored encoded (`__crt_fast_encode_pointer`), as are the
//!   function pointers in the array. An initialized empty table therefore holds
//!   the cookie in all three fields, not zero.
//! * The process tables are VALUE-initialized (all-zero, `{}`), not
//!   encoded-null. `_initialize_onexit_table` treats "`_first == _end`" as
//!   uninitialized, so it converts a zero table and leaves any table with
//!   entries alone. Registering into a zero table WITHOUT initializing it first
//!   decodes zero into the cookie and hands that to `_recalloc` -- Microsoft's
//!   contract, kept.
//! * Growth: from empty to 32 slots; otherwise double, capped at +512 per
//!   step; if that allocation fails (or the count overflowed), retry with
//!   just +4. New slots are filled with the ENCODED null, not zero, because
//!   `_execute_onexit_table` skips encoded-null slots -- which is also why
//!   registering a NULL function succeeds and is silently skipped at exit.
//! * `_execute_onexit_table` runs LIFO, clearing each slot before calling it,
//!   and after EVERY call re-reads the table: if the callback registered
//!   another function (moving `_first` or `_last`), iteration restarts from
//!   the new end, so the newcomer runs next. It holds the exit lock (a
//!   recursive critical section) across the callbacks, so that works from
//!   inside a callback on the same thread. Afterwards it frees the array and
//!   resets the table to encoded null. A decoded `_first` of null or -1 means
//!   "nothing to do" and returns 0 without resetting anything.
//! * `_initialize_onexit_table(NULL)` / `_register_onexit_function(NULL, f)` /
//!   `_execute_onexit_table(NULL)` return -1 and set NO errno and call no
//!   invalid-parameter handler.

use core::ffi::{c_int, c_void};
use core::mem::size_of;

use super::{fast_code, with_exit_lock};

/// `_PVFV`: `void (__cdecl*)(void)`.
pub type Pvfv = Option<unsafe extern "C" fn()>;
/// `_onexit_t`: `int (__cdecl*)(void)`.
pub type OnexitT = Option<unsafe extern "C" fn() -> c_int>;

/// `_onexit_t _onexit_table_t` (`corecrt_startup.h:137`): `_first`, `_last`,
/// `_end`, each an ENCODED `_PVFV*` -- held here as their bit patterns, since
/// nothing may dereference them before decoding. Same layout as
/// `crate::internal::init::OnexitTable`.
#[repr(C)]
pub struct OnexitTable {
    pub first: usize,
    pub last: usize,
    pub end: usize,
}

const _: () = assert!(size_of::<OnexitTable>() == 3 * size_of::<*mut c_void>());

/// The global `atexit` registry (`onexit.cpp`: `_onexit_table_t
/// __acrt_atexit_table{}`).
#[no_mangle]
pub static mut __acrt_atexit_table: OnexitTable = OnexitTable { first: 0, last: 0, end: 0 };

/// The global `at_quick_exit` registry.
#[no_mangle]
pub static mut __acrt_at_quick_exit_table: OnexitTable = OnexitTable { first: 0, last: 0, end: 0 };

// onexit.cpp's anonymous enum.
const INITIAL_TABLE_COUNT: usize = 32;
const MINIMUM_TABLE_INCREMENT: usize = 4;
const MAXIMUM_TABLE_INCREMENT: usize = 512;

/// `sizeof(_PVFV)`.
const SLOT: usize = size_of::<usize>();

extern "C" {
    fn _recalloc(block: *mut c_void, count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

/// `_crt_atexit`.
#[no_mangle]
pub unsafe extern "C" fn _crt_atexit(function: Pvfv) -> c_int {
    _register_onexit_function(
        core::ptr::addr_of_mut!(__acrt_atexit_table),
        core::mem::transmute::<Pvfv, OnexitT>(function),
    )
}

/// `_crt_at_quick_exit`.
#[no_mangle]
pub unsafe extern "C" fn _crt_at_quick_exit(function: Pvfv) -> c_int {
    _register_onexit_function(
        core::ptr::addr_of_mut!(__acrt_at_quick_exit_table),
        core::mem::transmute::<Pvfv, OnexitT>(function),
    )
}

/// `_initialize_onexit_table`.
#[no_mangle]
pub unsafe extern "C" fn _initialize_onexit_table(table: *mut OnexitTable) -> c_int {
    if table.is_null() {
        return -1;
    }

    // Already initialized (either form): leave it.
    if (*table).first != (*table).end {
        return 0;
    }

    let encoded_nullptr = fast_code(0);
    (*table).first = encoded_nullptr;
    (*table).last = encoded_nullptr;
    (*table).end = encoded_nullptr;
    0
}

/// `_register_onexit_function`: append `function` to `table`, growing it as
/// needed. 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn _register_onexit_function(table: *mut OnexitTable, function: OnexitT) -> c_int {
    with_exit_lock(|| {
        if table.is_null() {
            return -1;
        }

        // Addresses, decoded. Kept as integers: the C compares and subtracts
        // them as pointers, and a zero (uninitialized) table decodes to the
        // cookie, which is not a pointer into anything.
        let mut first = fast_code((*table).first);
        let mut last = fast_code((*table).last);
        let mut end = fast_code((*table).end);

        if last == end {
            // `end - first`: ptrdiff_t, an exact (arithmetic) division.
            let old_count = ((end.wrapping_sub(first) as isize) / SLOT as isize) as usize;

            let increment = if old_count > MAXIMUM_TABLE_INCREMENT { MAXIMUM_TABLE_INCREMENT } else { old_count };

            // First, try to double the capacity of the table:
            let mut new_count = old_count.wrapping_add(increment);
            if new_count == 0 {
                new_count = INITIAL_TABLE_COUNT;
            }

            let mut new_first: *mut c_void = core::ptr::null_mut();
            if new_count >= old_count {
                new_first = _recalloc(first as *mut c_void, new_count, SLOT);
            }

            // If that didn't work, try a smaller increment:
            if new_first.is_null() {
                new_count = old_count.wrapping_add(MINIMUM_TABLE_INCREMENT);
                new_first = _recalloc(first as *mut c_void, new_count, SLOT);
            }

            if new_first.is_null() {
                return -1;
            }

            first = new_first as usize;
            last = first.wrapping_add(old_count.wrapping_mul(SLOT));
            end = first.wrapping_add(new_count.wrapping_mul(SLOT));

            // recalloc zero-fills the new storage, but the array holds ENCODED
            // pointers, so the spare slots get the encoded null.
            let encoded_nullptr = fast_code(0);
            let mut it = last;
            while it != end {
                (it as *mut usize).write(encoded_nullptr);
                it += SLOT;
            }
        }

        (last as *mut usize).write(fast_code(function.map_or(0, |f| f as usize)));
        last += SLOT;

        (*table).first = fast_code(first);
        (*table).last = fast_code(last);
        (*table).end = fast_code(end);
        0
    })
}

/// `_execute_onexit_table`: run the table LIFO, then free and reset it.
/// 0 on success (including "table uninitialized"), -1 for a NULL table.
#[no_mangle]
pub unsafe extern "C" fn _execute_onexit_table(table: *mut OnexitTable) -> c_int {
    with_exit_lock(|| {
        if table.is_null() {
            return -1;
        }

        let mut first = fast_code((*table).first);
        let mut last = fast_code((*table).last);
        if first == 0 || first == usize::MAX {
            return 0;
        }

        // (`scoped_global_state_reset` is an empty RAII type in this build:
        // no global state isolation.)
        {
            let encoded_nullptr = fast_code(0);

            let mut saved_first = first;
            let mut saved_last = last;
            loop {
                // Find the last valid function pointer to call. The C's
                // `--last >= first` is an unsigned pointer compare.
                loop {
                    last = last.wrapping_sub(SLOT);
                    if !(last >= first && (last as *const usize).read() == encoded_nullptr) {
                        break;
                    }
                }

                if last < first {
                    // No more valid entries: done.
                    break;
                }

                // Mark the slot visited BEFORE the call.
                let function = fast_code((last as *const usize).read());
                (last as *mut usize).write(encoded_nullptr);

                let function: unsafe extern "C" fn() = core::mem::transmute::<usize, unsafe extern "C" fn()>(function);
                function();

                let new_first = fast_code((*table).first);
                let new_last = fast_code((*table).last);

                // Reset iteration if either the begin or end pointer changed:
                if new_first != saved_first || new_last != saved_last {
                    saved_first = new_first;
                    first = new_first;
                    saved_last = new_last;
                    last = new_last;
                }
            }
        }

        if first != usize::MAX {
            free(first as *mut c_void);
        }

        let encoded_nullptr = fast_code(0);
        (*table).first = encoded_nullptr;
        (*table).last = encoded_nullptr;
        (*table).end = encoded_nullptr;
        0
    })
}
