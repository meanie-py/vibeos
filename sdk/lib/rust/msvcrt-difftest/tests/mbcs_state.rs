//! Grades `create_mbcinfo` — the function every other mbcs verdict rests on.
//!
//! WHY THIS IS SEPARATE. `mbcs.rs` pins one `threadmbcinfo` as a read-only fixture so that
//! a divergence is provably logic and not state, and `mbcs_fixture_install` builds that
//! fixture by calling the REFERENCE's `create_mbcinfo` and copying the result into BOTH
//! sides' slots. That is the right call for the 173 generated cases — they need identical
//! state or they are grading the fixture rather than the function — but it leaves one hole
//! that is invisible from inside: **the SUBJECT's `create_mbcinfo` is never run.** It
//! builds `mbctype`, the lead/trail-byte classification the entire DBCS surface reads, and
//! `mbcasemap`, the case table. A wrong one could not fail a single generated case,
//! because no generated case ever asks it anything.
//!
//! So: its own binary, which is its own process. `create_mbcinfo(cp, -1, NULL)` allocates
//! and fills a fresh struct and mutates nothing global, so it is safe to call directly —
//! the generator refuses it only because it is in the transitive `STATE_MUTATORS` set,
//! which is about the functions that INSTALL its result. Running it here cannot disturb
//! the pinned fixture the other binary depends on, because that fixture lives in a
//! different process.
//!
//! The returned structs are deliberately leaked. They are `malloc`'d by whichever side
//! built them and the two sides do not share an allocator view here; a handful of structs
//! per codepage is not worth the risk of freeing one with the wrong `free`.
//!
//! `_setmbcp` is the one function in mbcs.c still ungraded, and not for want of trying:
//! it installs its result into `MSVCRT_locale->mbcinfo`, and the harness defines
//! `MSVCRT_locale` as NULL (nlsimpl.c — "what msvcrt has before setlocale()"), so the
//! non-thread branch dereferences NULL. Grading it means giving both sides a modelled
//! `MSVCRT_locale` and thread-data `locale_flags`, which is a real extension of the
//! harness's locale model rather than another case. Left refused, deliberately and with
//! that reason, instead of being faked.

extern crate msvcrt_crt_rust;

use core::ffi::c_void;
use msvcrt_difftest::fixture::CODEPAGES;

extern "C" {
    /// The shipping Rust, and the genuine C compiled by build.rs.
    fn create_mbcinfo(cp: i32, lcid: u32, old: *mut c_void) -> *mut c_void;
    fn ref_create_mbcinfo(cp: i32, lcid: u32, old: *mut c_void) -> *mut c_void;
    /// Which codepage the NLS replay answers for. `create_mbcinfo` asks GetCPInfo and
    /// LCMapStringW, so this has to be set first or both sides build the wrong table —
    /// identically, and the case would pass having graded the wrong codepage.
    fn nls_set_active_codepage(cp: u32);
    fn mbcinfo_diff(a: *const c_void, b: *const c_void, out: *mut u8, n: usize) -> i32;
}

/// `-1` as msvcrt spells it: "whatever LCID goes with this codepage".
const LCID_ANY: u32 = u32::MAX;

fn main() {
    eprintln!("\ncreate_mbcinfo differential\n");
    let mut graded = 0usize;

    for &cp in CODEPAGES {
        unsafe { nls_set_active_codepage(cp) };

        let subject = unsafe { create_mbcinfo(cp as i32, LCID_ANY, core::ptr::null_mut()) };
        let reference = unsafe { ref_create_mbcinfo(cp as i32, LCID_ANY, core::ptr::null_mut()) };

        assert_eq!(
            subject.is_null(),
            reference.is_null(),
            "cp{cp}: subject returned {}, reference returned {} — one of them refused to \
             build an mbcinfo the other built",
            if subject.is_null() { "NULL" } else { "a struct" },
            if reference.is_null() { "NULL" } else { "a struct" },
        );

        // Both NULL is a legitimate match (an unsupported codepage), but it grades nothing,
        // and CODEPAGES exists precisely so that every entry builds. Say so rather than
        // counting it.
        assert!(
            !subject.is_null(),
            "cp{cp}: both sides failed to build an mbcinfo. Every codepage in CODEPAGES is \
             supposed to be buildable — a pair of NULLs here is agreement about nothing."
        );

        let mut msg = [0u8; 160];
        let differs =
            unsafe { mbcinfo_diff(subject, reference, msg.as_mut_ptr(), msg.len()) } != 0;
        if differs {
            let end = msg.iter().position(|&b| b == 0).unwrap_or(msg.len());
            panic!(
                "cp{cp}: create_mbcinfo built a different table — {}",
                String::from_utf8_lossy(&msg[..end])
            );
        }
        eprintln!("  cp{cp} ... ok");
        graded += 1;
    }

    println!("\n  {graded} create_mbcinfo codepages passed\n");
}
