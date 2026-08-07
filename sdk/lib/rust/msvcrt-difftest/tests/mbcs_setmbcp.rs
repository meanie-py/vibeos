//! Grades `_setmbcp` — the last ungraded function in mbcs.c, and the one that writes the
//! table `_mbctype` exports.
//!
//! WHY IT WAS REFUSED, AND WHAT CHANGED. `_setmbcp` is in the generator's transitive
//! `STATE_MUTATORS` set: it INSTALLS its `create_mbcinfo` result, either into the calling
//! thread's data or into `MSVCRT_locale->mbcinfo`, so a generated case would move the very
//! fixture all 173 other cases are pinned against. On top of that the harness defined
//! `MSVCRT_locale` as NULL — what msvcrt has before `setlocale()` — so the non-thread arm
//! dereferenced null. Both are answered here rather than worked around: this is its own
//! binary, hence its own process, so nothing it installs can reach the pinned fixture, and
//! `nlsimpl.c` now keeps a `_locale_tstruct` per side for this test to point the globals at.
//!
//! ONE PER SIDE IS THE WHOLE DESIGN. The non-thread arm ends in
//! `memcpy(MSVCRT_mbctype, MSVCRT_locale->mbcinfo->mbctype, …)`. With a shared
//! `MSVCRT_locale` both sides would install into the same slot and then each read back
//! whatever the other wrote last — a row comparing a table against itself, which passes
//! however wrong either side is. That is the same reasoning that renamed `MSVCRT_mbctype`
//! for the reference, and this is the row that needed the rename: the bug it would have
//! caught (a mangled Rust static leaving the export on the C's permanently-zero copy)
//! shipped, and nothing in the suite could see it.
//!
//! WHY THE TWO SIDES' THREAD DATA IS REACHED DIFFERENTLY. The subject's `thread_data_t` is
//! msvcrt-sys's — the same layout the Rust under test compiles against, and `ref_integrity`
//! already cross-checks it against subjectimpl.c — so its fields are read here directly.
//! The reference's is refstub/msvcrt.h's: a different struct, in a different order,
//! carrying only what the reference TUs touch. refimpl.c hands out accessors for it. One
//! `extern` declaration could not be right about both.

extern crate msvcrt_crt_rust;

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut};
use msvcrt_difftest::fixture::CODEPAGES;
use msvcrt_sys::thread_data_t;

/// `msvcrt.h:172-173`.
const LOCALE_FREE: i32 = 0x1;
const LOCALE_THREAD: i32 = 0x2;
/// `errno.h`. What `_setmbcp` reports for a codepage `create_mbcinfo` refuses.
const EINVAL: i32 = 22;

extern "C" {
    /// The shipping Rust, and the genuine C compiled by build.rs.
    fn _setmbcp(cp: i32) -> i32;
    fn ref__setmbcp(cp: i32) -> i32;

    // ── the subject's side of the world ──────────────────────────────────────────────
    fn msvcrt_get_thread_data() -> *mut thread_data_t;
    static mut MSVCRT_locale: *mut c_void;
    static mut MSVCRT_mbctype: [u8; 257];
    fn _errno() -> *mut i32;

    // ── the reference's ──────────────────────────────────────────────────────────────
    fn ref_set_locale_flags(flags: i32);
    fn ref_set_thread_mbcinfo(mbcinfo: *mut c_void);
    fn ref_get_thread_mbcinfo() -> *mut c_void;
    static mut msvcrt_ref_locale: *mut c_void;
    static mut ref_MSVCRT_mbctype: [u8; 257];
    fn msvcrt_ref__errno() -> *mut i32;

    // ── shared harness ───────────────────────────────────────────────────────────────
    fn mbcs_locale_slot(side: i32) -> *mut c_void;
    fn mbcs_last_freed() -> *mut c_void;
    fn mbcs_free_calls() -> i32;
    fn mbcs_reset_freed();
    fn mbcs_mbctype_len() -> usize;
    /// `&mbcinfo->mbctype[0]` — the offset of that field is the harness's C-side model of
    /// `threadmbcinfo`, and nothing else here needs to know it.
    fn mbcinfo_mbctype(mbcinfo: *const c_void) -> *const u8;
    fn mbcinfo_diff(a: *const c_void, b: *const c_void, out: *mut u8, n: usize) -> i32;
    fn nls_set_active_codepage(cp: u32);
    /// Each side's pinned-fixture slot — what its `get_mbcinfo()` returns, and so the
    /// `old_mbcinfo` argument `_setmbcp` hands to `create_mbcinfo`.
    fn mbcs_ref_slot() -> *mut c_void;
    fn mbcs_subj_slot() -> *mut c_void;
}

/// The two exported tables, as slices. Through `addr_of!` rather than by naming the static:
/// a `&` to a `static mut` is the `static_mut_refs` lint, and this file is where two of them
/// are read on every iteration.
unsafe fn subj_table(n: usize) -> &'static [u8] {
    core::slice::from_raw_parts(addr_of!(MSVCRT_mbctype) as *const u8, n)
}
unsafe fn ref_table(n: usize) -> &'static [u8] {
    core::slice::from_raw_parts(addr_of!(ref_MSVCRT_mbctype) as *const u8, n)
}
unsafe fn zero_tables(n: usize) {
    core::ptr::write_bytes(addr_of_mut!(MSVCRT_mbctype) as *mut u8, 0, n);
    core::ptr::write_bytes(addr_of_mut!(ref_MSVCRT_mbctype) as *mut u8, 0, n);
}

/// `_locale_tstruct` is `{ locinfo, mbcinfo }` — two pointers, the second of which is what
/// `_setmbcp` writes. An offset rather than a struct, because the harness's definition is
/// C-side (refstub/locale_layout.h) and this is the only field either side touches here.
unsafe fn locale_mbcinfo(slot: *mut c_void) -> *mut c_void {
    *(slot as *mut *mut c_void).add(1)
}
unsafe fn set_locale_mbcinfo(slot: *mut c_void, v: *mut c_void) {
    *(slot as *mut *mut c_void).add(1) = v;
}

fn diff_or_panic(what: &str, cp: u32, subject: *mut c_void, reference: *mut c_void) {
    assert!(
        !subject.is_null() && !reference.is_null(),
        "cp{cp}: {what} — one side installed nothing (subject {subject:?}, \
         reference {reference:?})"
    );
    let mut msg = [0u8; 160];
    let differs = unsafe { mbcinfo_diff(subject, reference, msg.as_mut_ptr(), msg.len()) } != 0;
    if differs {
        let end = msg.iter().position(|&b| b == 0).unwrap_or(msg.len());
        panic!(
            "cp{cp}: {what} — the installed mbcinfos differ: {}",
            String::from_utf8_lossy(&msg[..end])
        );
    }
}

fn main() {
    eprintln!("\n_setmbcp differential\n");
    let n = unsafe { mbcs_mbctype_len() };
    assert_eq!(
        n, 257,
        "mbctype is {n} bytes, not 257 — `_setmbcp` closes with a memcpy sized by \
         `sizeof(MSVCRT_mbctype)`, so a table of another length means this row compares \
         the wrong number of bytes"
    );
    let mut graded = 0usize;

    for &cp in CODEPAGES {
        unsafe { nls_set_active_codepage(cp) };

        // ── arm 1: the THREAD arm ────────────────────────────────────────────────────
        //
        // `locale_flags & LOCALE_THREAD` installs into the calling thread's data and must
        // touch neither `MSVCRT_locale` nor the exported table. Both are checked: "built
        // the right table" and "put it in the right place" are different claims, and only
        // the second one tells the two arms apart.
        unsafe {
            let subj = msvcrt_get_thread_data();
            (*subj).locale_flags = LOCALE_THREAD;
            (*subj).mbcinfo = core::ptr::null_mut();
            ref_set_locale_flags(LOCALE_THREAD);
            ref_set_thread_mbcinfo(core::ptr::null_mut());

            MSVCRT_locale = core::ptr::null_mut();
            msvcrt_ref_locale = core::ptr::null_mut();
            zero_tables(n);

            assert_eq!(_setmbcp(cp as i32), 0, "cp{cp}: subject _setmbcp refused");
            assert_eq!(ref__setmbcp(cp as i32), 0, "cp{cp}: reference _setmbcp refused");

            let subj_installed = (*subj).mbcinfo as *mut c_void;
            let ref_installed = ref_get_thread_mbcinfo();
            diff_or_panic("thread arm", cp, subj_installed, ref_installed);

            // The `old_mbcinfo` create_mbcinfo was handed is each side's fixture slot,
            // whose mbcodepage is 0 (nothing installed a fixture in this process). A side
            // that returned that pointer took the refcount shortcut instead of building a
            // table, and the diff above would have compared two zeroed structs and passed.
            assert_ne!(
                subj_installed,
                mbcs_subj_slot(),
                "cp{cp}: subject reused the old mbcinfo for a codepage that is not its own"
            );
            assert_ne!(
                ref_installed,
                mbcs_ref_slot(),
                "cp{cp}: reference reused the old mbcinfo"
            );
            assert!(
                subj_table(n).iter().all(|&b| b == 0) && ref_table(n).iter().all(|&b| b == 0),
                "cp{cp}: the thread arm wrote the GLOBAL mbctype — that copy belongs to the
                 non-thread arm alone, and a caller with a per-thread locale must not have
                 its table published process-wide"
            );
        }
        graded += 1;

        // ── arm 2: LOCALE_FREE, an arm whose only effect is a call ───────────────────
        //
        // With both bits set the mbcinfo being replaced is released first. Nothing
        // observable survives that except the call, so `free_mbcinfo` records its argument
        // and each side is read immediately after its own call — the same one-recorder,
        // read-it-now shape fileimpl.c uses for `_lock`.
        unsafe {
            let subj = msvcrt_get_thread_data();
            let subj_old = (*subj).mbcinfo as *mut c_void;
            let ref_old = ref_get_thread_mbcinfo();
            (*subj).locale_flags = LOCALE_THREAD | LOCALE_FREE;
            ref_set_locale_flags(LOCALE_THREAD | LOCALE_FREE);

            mbcs_reset_freed();
            assert_eq!(_setmbcp(cp as i32), 0);
            let (subj_freed, subj_calls) = (mbcs_last_freed(), mbcs_free_calls());

            mbcs_reset_freed();
            assert_eq!(ref__setmbcp(cp as i32), 0);
            let (ref_freed, ref_calls) = (mbcs_last_freed(), mbcs_free_calls());

            assert_eq!(
                (subj_calls, subj_freed),
                (1, subj_old),
                "cp{cp}: subject's LOCALE_FREE arm did not release exactly the mbcinfo it \
                 replaced — the reference released {ref_freed:?} in {ref_calls} call(s)"
            );
            assert_eq!(
                (ref_calls, ref_freed),
                (1, ref_old),
                "cp{cp}: reference's LOCALE_FREE arm disagrees with the C as written"
            );
        }
        graded += 1;

        // ── arm 3: the GLOBAL arm, and the exported table ────────────────────────────
        //
        // The arm that shipped a bug. Each side gets its OWN `_locale_tstruct`.
        unsafe {
            let subj = msvcrt_get_thread_data();
            (*subj).locale_flags = 0;
            ref_set_locale_flags(0);

            let (subj_locale, ref_locale) = (mbcs_locale_slot(0), mbcs_locale_slot(1));
            assert_ne!(subj_locale, ref_locale, "the two sides share a locale");
            MSVCRT_locale = subj_locale;
            msvcrt_ref_locale = ref_locale;
            set_locale_mbcinfo(subj_locale, core::ptr::null_mut());
            set_locale_mbcinfo(ref_locale, core::ptr::null_mut());
            zero_tables(n);

            assert_eq!(_setmbcp(cp as i32), 0);
            assert_eq!(ref__setmbcp(cp as i32), 0);

            let subj_installed = locale_mbcinfo(subj_locale);
            let ref_installed = locale_mbcinfo(ref_locale);
            diff_or_panic("global arm", cp, subj_installed, ref_installed);
            assert_eq!(
                subj_table(n),
                ref_table(n),
                "cp{cp}: the exported `_mbctype` tables differ after _setmbcp"
            );
            // Anchored: two all-zero tables compare equal and grade nothing, and all-zero
            // is exactly what the shipped bug looked like from outside.
            assert!(
                subj_table(n).iter().any(|&b| b != 0),
                "cp{cp}: both `_mbctype` copies are still zero after _setmbcp — the memcpy \
                 either did not run or wrote an array other than the one exported"
            );
            // The memcpy's contract stated directly, per side. Comparing only the two
            // sides' exported copies to each other cannot see a side that consistently
            // publishes the WRONG table — which is the shipped failure, one remove away.
            for (side, exported, installed) in [
                ("subject", subj_table(n), subj_installed),
                ("reference", ref_table(n), ref_installed),
            ] {
                assert_eq!(
                    exported,
                    core::slice::from_raw_parts(mbcinfo_mbctype(installed), n),
                    "cp{cp}: {side}'s exported `_mbctype` is not the table it just \
                     installed — they are two different arrays"
                );
            }
        }
        graded += 1;

        eprintln!("  cp{cp} ... ok  (thread / free / global)");
    }

    // ── the refusal path ────────────────────────────────────────────────────────────
    unsafe {
        let subj = msvcrt_get_thread_data();
        (*subj).locale_flags = LOCALE_THREAD;
        ref_set_locale_flags(LOCALE_THREAD);
        let before: Vec<u8> = subj_table(n).to_vec();
        let ref_before: Vec<u8> = ref_table(n).to_vec();

        *_errno() = 0;
        *msvcrt_ref__errno() = 0;
        // Not in nlsimpl.c's CP_TABLE, so `GetCPInfo` refuses and `create_mbcinfo` frees
        // its half-built struct and returns NULL. 20127 would NOT do — it is the one
        // codepage with a hand-rolled fallback past that same failure.
        let bad = 51932;
        assert_eq!(_setmbcp(bad), -1, "subject accepted a codepage that does not exist");
        assert_eq!(ref__setmbcp(bad), -1, "reference accepted it");
        assert_eq!(
            (*_errno(), *msvcrt_ref__errno()),
            (EINVAL, EINVAL),
            "a refused _setmbcp must leave EINVAL on both sides"
        );
        assert!(
            subj_table(n) == &before[..] && ref_table(n) == &ref_before[..],
            "a refused _setmbcp still moved the exported table"
        );
    }
    graded += 1;

    eprintln!("\n_setmbcp: {graded} arms graded across {} codepages\n", CODEPAGES.len());
}
