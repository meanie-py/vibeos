//! Grades misc.c — reference TU #9. `qsort`, `bsearch`, the `_l*` searches, and `rand`.
//!
//! WHAT MAKES THIS FILE DIFFERENT FROM THE OTHERS: four of its exports take a CALLBACK, and
//! the callback is the harness's. So a row can watch the algorithm work rather than only
//! check its answer — and for `qsort` that is not a refinement, it is the contract. misc.c
//! says so itself, above `qsort_s`:
//!
//!     This function is trying to sort data doing identical comparisons as native does.
//!     There are still cases where it behaves differently.
//!
//! A sort that produces the right order through a different sequence of comparisons is a
//! different function to any caller whose comparator has side effects — counts, logging, a
//! cache, a tie-break on external state. Comparing only the sorted array would grade the
//! easy half and report green.
//!
//! POINTERS ARE NEVER COMPARED ACROSS SIDES. `_lfind`, `_lsearch` and `bsearch` return
//! INTERIOR pointers into two different buffers, so the addresses are meaningless; each is
//! reduced to `(ret - base) / elem_size`, with NULL kept distinct. Comparing the raw values
//! would fail every row for a reason that says nothing about the code.

#![allow(non_snake_case)]

extern crate msvcrt_crt_rust;

use core::ffi::{c_int, c_uint, c_void};

/// An element with a payload the key does not determine, so the ORDER OF EQUALS is visible.
/// msvcrt's qsort is not stable and does not claim to be — which is exactly why the tag has
/// to be compared: "not stable" is a specific permutation here, not a licence to differ.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Elem {
    key: i32,
    tag: u32,
}

/// One comparison log per side. Shared storage would let one side's calls satisfy the
/// other's expectations — the §9.73 trap, one level up from data.
mod log {
    pub static mut SUBJ: Vec<(i32, i32)> = Vec::new();
    pub static mut REF: Vec<(i32, i32)> = Vec::new();

    pub fn reset() {
        unsafe {
            (*core::ptr::addr_of_mut!(SUBJ)).clear();
            (*core::ptr::addr_of_mut!(REF)).clear();
        }
    }
    pub fn push(side: usize, a: i32, b: i32) {
        unsafe {
            let s = if side == 0 {
                core::ptr::addr_of_mut!(SUBJ)
            } else {
                core::ptr::addr_of_mut!(REF)
            };
            (*s).push((a, b));
        }
    }
    pub fn get(side: usize) -> Vec<(i32, i32)> {
        unsafe {
            let s = if side == 0 {
                core::ptr::addr_of!(SUBJ)
            } else {
                core::ptr::addr_of!(REF)
            };
            (*s).clone()
        }
    }
}

unsafe extern "C" fn cmp_subj(a: *const c_void, b: *const c_void) -> c_int {
    let (x, y) = (*(a as *const Elem), *(b as *const Elem));
    log::push(0, x.key, y.key);
    x.key.cmp(&y.key) as c_int
}

unsafe extern "C" fn cmp_ref(a: *const c_void, b: *const c_void) -> c_int {
    let (x, y) = (*(a as *const Elem), *(b as *const Elem));
    log::push(1, x.key, y.key);
    x.key.cmp(&y.key) as c_int
}

extern "C" {
    fn qsort(base: *mut c_void, nmemb: usize, size: usize,
             compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int);
    fn ref_qsort(base: *mut c_void, nmemb: usize, size: usize,
                 compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int);
    fn bsearch(key: *const c_void, base: *const c_void, nmemb: usize, size: usize,
               compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int) -> *mut c_void;
    fn ref_bsearch(key: *const c_void, base: *const c_void, nmemb: usize, size: usize,
                   compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int) -> *mut c_void;
    fn _lfind(match_: *const c_void, start: *const c_void, array_size: *mut c_uint,
              elem_size: c_uint,
              cf: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int) -> *mut c_void;
    fn ref__lfind(match_: *const c_void, start: *const c_void, array_size: *mut c_uint,
                  elem_size: c_uint,
                  cf: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int) -> *mut c_void;
    fn _lsearch(match_: *const c_void, start: *mut c_void, array_size: *mut c_uint,
                elem_size: c_uint,
                cf: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int) -> *mut c_void;
    fn ref__lsearch(match_: *const c_void, start: *mut c_void, array_size: *mut c_uint,
                    elem_size: c_uint,
                    cf: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int) -> *mut c_void;
    fn rand() -> c_int;
    fn ref_rand() -> c_int;
    fn srand(seed: c_uint);
    fn ref_srand(seed: c_uint);
    fn _get_output_format() -> c_uint;
    fn ref__get_output_format() -> c_uint;
    fn _set_output_format(f: c_uint) -> c_uint;
    fn ref__set_output_format(f: c_uint) -> c_uint;
}

/// An interior pointer reduced to an ELEMENT INDEX, which is the only part of it that means
/// the same thing on both sides.
unsafe fn slot(ret: *const c_void, base: *const c_void, size: usize) -> Option<usize> {
    if ret.is_null() {
        None
    } else {
        Some((ret as usize - base as usize) / size)
    }
}

/// The datasets. Chosen so every branch of a quicksort is reached: already sorted and
/// reverse sorted (the classic pivot degeneracies), all-equal (every comparison returns 0),
/// the sizes either side of whatever cutoff an insertion-sort fallback uses, and 0/1 which
/// `qsort_s` returns from before touching the data.
fn datasets() -> Vec<Vec<Elem>> {
    let mut out = vec![vec![], vec![Elem { key: 7, tag: 0 }]];
    for n in [2usize, 3, 4, 5, 7, 8, 9, 16, 17, 33, 64] {
        let mk = |f: &dyn Fn(usize) -> i32| {
            (0..n).map(|i| Elem { key: f(i), tag: i as u32 }).collect::<Vec<_>>()
        };
        out.push(mk(&|i| i as i32)); // sorted
        out.push(mk(&|i| (n - i) as i32)); // reverse
        out.push(mk(&|_| 42)); // all equal
        out.push(mk(&|i| (i % 3) as i32)); // heavy duplicates
        out.push(mk(&|i| ((i * 2654435761usize) % 1000) as i32)); // scattered
    }
    out
}

extern "C" {
    fn _lfind_s(match_: *const c_void, start: *const c_void, array_size: *mut c_uint,
                elem_size: c_uint,
                cf: unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void) -> c_int,
                ctx: *mut c_void) -> *mut c_void;
    fn ref__lfind_s(match_: *const c_void, start: *const c_void, array_size: *mut c_uint,
                    elem_size: c_uint,
                    cf: unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void) -> c_int,
                    ctx: *mut c_void) -> *mut c_void;
    fn qsort_s(base: *mut c_void, nmemb: usize, size: usize,
               compar: unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void) -> c_int,
               ctx: *mut c_void);
    fn ref_qsort_s(base: *mut c_void, nmemb: usize, size: usize,
                   compar: unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void) -> c_int,
                   ctx: *mut c_void);
    fn rand_s(pval: *mut c_uint) -> c_int;
    fn ref_rand_s(pval: *mut c_uint) -> c_int;
    fn _beep(freq: c_uint, duration: c_uint);
    fn ref__beep(freq: c_uint, duration: c_uint);
    fn _sleep(timeout: c_uint);
    fn ref__sleep(timeout: c_uint);
    fn _resetstkoflw() -> c_int;
    fn ref__resetstkoflw() -> c_int;

    // Harness accessors — one recorder, read immediately after each side's call.
    fn ref_last_sleep() -> c_uint;
    fn ref_last_beep_f() -> c_uint;
    fn ref_last_beep_d() -> c_uint;
    fn ref_misc_reset();
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
}

/// The `_s` comparators, which take the context first. The context carries the SIDE, so one
/// pair of functions serves both and the log still separates them.
unsafe extern "C" fn cmp_s(ctx: *mut c_void, a: *const c_void, b: *const c_void) -> c_int {
    let side = ctx as usize;
    let (x, y) = (*(a as *const Elem), *(b as *const Elem));
    log::push(side, x.key, y.key);
    x.key.cmp(&y.key) as c_int
}

const SZ: usize = core::mem::size_of::<Elem>();

fn both(what: &str, n: usize, s: &[Elem], r: &[Elem]) {
    assert_eq!(s, r, "{what} [n={n}]: results differ");
    let (ls, lr) = (log::get(0), log::get(1));
    assert_eq!(
        ls, lr,
        "{what} [n={n}]: same result, DIFFERENT comparison sequence ({} vs {} calls). \
         misc.c's own comment above qsort_s says it is trying to do identical comparisons \
         to native, so this is a divergence, not a detail.",
        ls.len(),
        lr.len()
    );
}

fn main() {
    eprintln!("\nmisc.c differential\n");
    let mut rows = 0usize;

    for data in datasets() {
        let n = data.len();
        unsafe {
            // ── qsort: result AND comparison sequence ────────────────────────────────
            log::reset();
            let (mut s, mut r) = (data.clone(), data.clone());
            qsort(s.as_mut_ptr() as *mut c_void, n, SZ, cmp_subj);
            ref_qsort(r.as_mut_ptr() as *mut c_void, n, SZ, cmp_ref);
            both("qsort", n, &s, &r);
            rows += 1;

            // ── qsort_s: the same engine reached through the context form ────────────
            log::reset();
            let (mut s, mut r) = (data.clone(), data.clone());
            qsort_s(s.as_mut_ptr() as *mut c_void, n, SZ, cmp_s, 0usize as *mut c_void);
            ref_qsort_s(r.as_mut_ptr() as *mut c_void, n, SZ, cmp_s, 1usize as *mut c_void);
            both("qsort_s", n, &s, &r);
            rows += 1;

            // ── bsearch over the sorted array, for every key present plus two absent ──
            let sorted = s.clone();
            let mut keys: Vec<i32> = sorted.iter().map(|e| e.key).collect();
            keys.push(i32::MIN);
            keys.push(i32::MAX);
            for k in keys {
                let probe = Elem { key: k, tag: 0 };
                log::reset();
                let sp = bsearch(&probe as *const _ as *const c_void,
                                 sorted.as_ptr() as *const c_void, n, SZ, cmp_subj);
                let rp = ref_bsearch(&probe as *const _ as *const c_void,
                                     sorted.as_ptr() as *const c_void, n, SZ, cmp_ref);
                assert_eq!(
                    slot(sp, sorted.as_ptr() as *const c_void, SZ),
                    slot(rp, sorted.as_ptr() as *const c_void, SZ),
                    "bsearch [n={n}] key {k}: different element found"
                );
                assert_eq!(log::get(0), log::get(1),
                           "bsearch [n={n}] key {k}: different probe sequence");
                rows += 1;
            }

            // ── _lfind / _lfind_s / _lsearch ──────────────────────────────────────────
            for k in [data.first().map(|e| e.key).unwrap_or(0), 999_999] {
                let probe = Elem { key: k, tag: 0 };
                let (mut sa, mut ra) = (data.clone(), data.clone());
                let (mut sn, mut rn) = (n as c_uint, n as c_uint);

                log::reset();
                let sp = _lfind(&probe as *const _ as *const c_void,
                                sa.as_ptr() as *const c_void, &mut sn, SZ as c_uint, cmp_subj);
                let rp = ref__lfind(&probe as *const _ as *const c_void,
                                    ra.as_ptr() as *const c_void, &mut rn, SZ as c_uint, cmp_ref);
                assert_eq!(slot(sp, sa.as_ptr() as *const c_void, SZ),
                           slot(rp, ra.as_ptr() as *const c_void, SZ),
                           "_lfind [n={n}] key {k}");
                assert_eq!(log::get(0), log::get(1), "_lfind [n={n}] key {k}: scan differs");

                log::reset();
                let sp = _lfind_s(&probe as *const _ as *const c_void,
                                  sa.as_ptr() as *const c_void, &mut sn, SZ as c_uint,
                                  cmp_s, 0usize as *mut c_void);
                let rp = ref__lfind_s(&probe as *const _ as *const c_void,
                                      ra.as_ptr() as *const c_void, &mut rn, SZ as c_uint,
                                      cmp_s, 1usize as *mut c_void);
                assert_eq!(slot(sp, sa.as_ptr() as *const c_void, SZ),
                           slot(rp, ra.as_ptr() as *const c_void, SZ),
                           "_lfind_s [n={n}] key {k}");
                assert_eq!(log::get(0), log::get(1), "_lfind_s [n={n}] key {k}: scan differs");

                // `_lsearch` APPENDS on a miss, so the arrays need a spare slot and the
                // COUNT is part of the answer. This is the only one of the three that
                // mutates, and a translation that forgot the `array_size[0]++` would pass
                // every other check here.
                sa.push(Elem { key: 0, tag: 0 });
                ra.push(Elem { key: 0, tag: 0 });
                log::reset();
                let sp = _lsearch(&probe as *const _ as *const c_void,
                                  sa.as_mut_ptr() as *mut c_void, &mut sn, SZ as c_uint, cmp_subj);
                let rp = ref__lsearch(&probe as *const _ as *const c_void,
                                      ra.as_mut_ptr() as *mut c_void, &mut rn, SZ as c_uint, cmp_ref);
                assert_eq!(slot(sp, sa.as_ptr() as *const c_void, SZ),
                           slot(rp, ra.as_ptr() as *const c_void, SZ),
                           "_lsearch [n={n}] key {k}");
                assert_eq!((sn, &sa), (rn, &ra), "_lsearch [n={n}] key {k}: array or count differs");
                assert_eq!(log::get(0), log::get(1), "_lsearch [n={n}] key {k}: scan differs");
                rows += 3;
            }
        }
    }

    // ── rand / srand: a fixed LCG over the thread's seed ─────────────────────────────
    unsafe {
        for seed in [0u32, 1, 42, 0x7fff_ffff, 0xffff_ffff, 214013, 2531011] {
            srand(seed);
            ref_srand(seed);
            let s: Vec<c_int> = (0..64).map(|_| rand()).collect();
            let r: Vec<c_int> = (0..64).map(|_| ref_rand()).collect();
            assert_eq!(s, r, "rand after srand({seed})");
            rows += 1;
        }
    }

    // ── the output-format global ─────────────────────────────────────────────────────
    unsafe {
        for f in [0u32, 1, 0xffff_ffff] {
            let (sp, rp) = (_set_output_format(f), ref__set_output_format(f));
            assert_eq!(sp, rp, "_set_output_format({f}) returned different previous values");
            assert_eq!(_get_output_format(), ref__get_output_format(), "_get_output_format");
            rows += 1;
        }
        // Leave it as msvcrt starts: the printf rows in the other binaries read this.
        _set_output_format(0);
        ref__set_output_format(0);
    }

    // ── the thin wrappers, whose whole body IS the kernel32 call ─────────────────────
    unsafe {
        for (freq, dur) in [(0u32, 0u32), (440, 250), (0xffff_ffff, 1)] {
            ref_misc_reset();
            _beep(freq, dur);
            let subj = (ref_last_beep_f(), ref_last_beep_d());
            ref_misc_reset();
            ref__beep(freq, dur);
            assert_eq!(subj, (ref_last_beep_f(), ref_last_beep_d()),
                       "_beep({freq},{dur}) forwarded different arguments");
            rows += 1;
        }
        // 0 is the interesting one: `_sleep` turns it into 1, and that adjustment is the
        // function's only observable behaviour.
        for t in [0u32, 1, 1000] {
            ref_misc_reset();
            _sleep(t);
            let subj = ref_last_sleep();
            ref_misc_reset();
            ref__sleep(t);
            assert_eq!(subj, ref_last_sleep(), "_sleep({t}) asked for a different duration");
            rows += 1;
        }
        assert_eq!(_resetstkoflw(), ref__resetstkoflw(), "_resetstkoflw");
        rows += 1;
    }

    // ── rand_s's refusal path ────────────────────────────────────────────────────────
    unsafe {
        ref_errno_set(0);
        let (s, r) = (rand_s(core::ptr::null_mut()), ref_rand_s(core::ptr::null_mut()));
        assert_eq!(s, r, "rand_s(NULL) returned differently");
        assert_eq!(ref_errno_get(), 22, "rand_s(NULL) must leave EINVAL");
        let (mut sv, mut rv) = (0u32, 0u32);
        assert_eq!(rand_s(&mut sv), ref_rand_s(&mut rv), "rand_s return");
        assert_eq!(sv, rv, "rand_s filled differently from the same entropy");
        rows += 2;
    }

    eprintln!("\nmisc: {rows} rows\n");
}
