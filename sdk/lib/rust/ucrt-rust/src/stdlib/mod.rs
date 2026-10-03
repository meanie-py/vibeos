//! `sdk/lib/ucrt/stdlib/` -- the 18 remaining files of `UCRT_STDLIB_SOURCES`
//! (`stdlib/stdlib.cmake`), replaced in full:
//!
//! | C file | Rust |
//! |---|---|
//! | abs.cpp (`abs`, `_abs64`), labs.cpp, llabs.cpp, imaxabs.cpp | [`abs`] |
//! | div.cpp, ldiv.cpp, lldiv.cpp, imaxdiv.cpp | [`div`] |
//! | rand.cpp (`srand`, `rand`), rand_s.cpp | [`rand`] |
//! | bsearch.cpp, bsearch_s.cpp, lfind.cpp, lfind_s.cpp, lsearch.cpp, lsearch_s.cpp | [`search`] |
//! | qsort.cpp, qsort_s.cpp | [`qsort`] |
//!
//! The `*_s.cpp` files are `#define __USE_CONTEXT` + `#include` of their base
//! file: the same body with a `void* context` passed first to the comparator
//! (and, for `_lfind_s`/`_lsearch_s`, a `size_t` width where the plain form
//! has `unsigned int`). Here each pair is one generic body over a comparator
//! closure. The string/ stragglers the batch was also given (`memcpy_s.cpp`,
//! `strnlen.cpp`) were already ported and deleted (`crate::string_ops`); only
//! the orphan `string/arm64/{strlen,wcslen}.s` remain in that directory.
//!
//! `__crt_state_management::scoped_global_state_reset` (taken around every
//! comparator callout in the C) is an empty struct in this tree
//! (`ucrt_reactos/inc/corecrt_internal_state_isolation.h`), so there is
//! nothing to port for it.
//!
//! ## Details that are silent if wrong
//!
//! * **qsort's comparator call SEQUENCE is observable** (callers log it, and
//!   inconsistent comparators make the result depend on it): median-of-three
//!   over (lo, mid, hi) in the order (lo,mid), (lo,hi), (mid,hi); the doubled
//!   loops that never call `comp(mid, mid)`; the partition follows `mid` when
//!   it is swapped; the SMALLER side recursed first (a SIGNED `ptrdiff_t`
//!   comparison -- `hi - loguy` is negative when `loguy == hi + 1`); an
//!   explicit stack of 8*sizeof(void*)-2 entries; the `CUTOFF` of 8 elements
//!   switching to a max-selection sort that compares `(p, max)`.
//! * **The division functions keep the hardware's behaviour on /0 and
//!   `MIN / -1`**: x86 traps (#DE), arm64 `sdiv` gives 0 / MIN. Rust's `/`
//!   would panic instead, so the divide is the same instruction the C
//!   compiled to (on i386 the 64-bit forms call `__divdi3`, as the C does).
//! * **`abs(INT_MIN) == INT_MIN`** (and the 64-bit forms likewise) -- two's
//!   complement wrap, what the compiled C does.
//! * **`div_t`/`ldiv_t` are returned in registers** (EDX:EAX on i386, RAX on
//!   amd64, X0 on arm64) and `lldiv_t`/`imaxdiv_t` through a hidden pointer on
//!   i386/amd64 but in X0:X1 on arm64 -- all from `#[repr(C)]` + `extern "C"`.
//! * **`_lfind`/`_lsearch` compute the end as `unsigned int * unsigned int`**
//!   (32-bit wrap even on 64-bit targets); the `_s` forms as `size_t`.
//!   `_lfind` accepts a NULL base when `*num == 0`, `_lsearch` never does.
//! * **`rand` state is per-thread** (`__acrt_ptd::_rand_state`, seeded 1 by
//!   the ptd initializer), LCG 214013/2531011, `(state >> 16) & 0x7fff`.
//! * **`bsearch` with `num == 0`** computes `hi = base + (0-1)*width` (wraps),
//!   so with a NULL base the loop is entered once and leaves without calling
//!   the comparator.

use core::ffi::c_int;

pub mod abs;
pub mod div;
pub mod qsort;
pub mod rand;
pub mod search;

/// errno.h `EINVAL`.
const EINVAL: c_int = 22;
/// errno.h `ENOMEM`.
const ENOMEM: c_int = 12;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

/// `_VALIDATE_RETURN*(expr, EINVAL, ...)`'s failure arm, before the return.
#[cold]
unsafe fn invalid_einval() {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
}
