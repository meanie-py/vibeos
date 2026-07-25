//! Does the pinned MBCS fixture actually contain MBCS?
//!
//! Every one of mbcs.c's 160 gradeable functions answers its question out of one
//! `threadmbcinfo` — `mbctype[b+1]` for "is this a lead byte", `mbcasemap[b]` for case.
//! If that table comes back empty the whole family still RUNS, both sides take the
//! single-byte path, and the harness reports a confident green over precisely the DBCS
//! logic it exists to check.
//!
//! So this is the gate that has to pass before any verdict from the mbcs cases means
//! anything: the fixture is built by the reference's own `create_mbcinfo`, and these
//! assertions say it came back with real DBCS content rather than zeros.

#![allow(non_snake_case)]

use core::ffi::c_uint;

// mbctype[] flag bits, from the GENUINE dll/win32/msvcrt/include/mbctype.h — the same
// header the reference TU compiled against, forwarded from OUT_DIR by build.rs.
const _M1: u8 = 0x04; // lead byte
const _M2: u8 = 0x08; // trail byte
const _SBUP: u8 = 0x10; // single byte, uppercase
const _SBLOW: u8 = 0x20; // single byte, lowercase

#[repr(C)]
struct ThreadMbcInfo {
    refcount: i32,
    mbcodepage: i32,
    ismbcodepage: i32,
    mblcid: i32,
    mbulinfo: [u16; 6],
    mbctype: [u8; 257],
    mbcasemap: [u8; 256],
}

extern "C" {
    fn mbcs_fixture_install(cp: c_uint) -> i32;
    fn mbcs_fixture_check() -> i32;
    fn mbcs_ref_slot() -> *mut ThreadMbcInfo;
    fn mbcs_subj_slot() -> *mut ThreadMbcInfo;
    fn nls_codepage_count() -> i32;
    fn nls_codepage_at(i: i32) -> c_uint;
    fn nls_is_dbcs_codepage(cp: c_uint) -> i32;
}

fn count(t: &[u8; 257], bit: u8) -> usize {
    t.iter().filter(|b| *b & bit != 0).count()
}

fn main() {
    let mut failures: Vec<String> = Vec::new();
    println!("\nMBCS fixture — built by the reference's own create_mbcinfo\n");
    println!(
        "  {:>6}  {:>5} {:>5} {:>5} {:>5} {:>6}  {}",
        "cp", "lead", "trail", "upper", "lower", "case", "verdict"
    );

    for i in 0..unsafe { nls_codepage_count() } {
        let cp = unsafe { nls_codepage_at(i) };
        if unsafe { mbcs_fixture_install(cp) } != 0 {
            failures.push(format!("cp{cp}: create_mbcinfo returned NULL"));
            continue;
        }

        let r = unsafe { &*mbcs_ref_slot() };
        let s = unsafe { &*mbcs_subj_slot() };
        let dbcs = unsafe { nls_is_dbcs_codepage(cp) } != 0;

        let lead = count(&r.mbctype, _M1);
        let trail = count(&r.mbctype, _M2);
        let upper = count(&r.mbctype, _SBUP);
        let lower = count(&r.mbctype, _SBLOW);
        let cased = r.mbcasemap.iter().filter(|b| **b != 0).count();

        let mut bad = Vec::new();

        // The two slots must start identical — everything downstream compares them.
        if unsafe { mbcs_fixture_check() } != 0 {
            bad.push("slots differ at install".to_string());
        }
        if r.mbcodepage != s.mbcodepage {
            bad.push("codepage differs between slots".to_string());
        }

        // A DBCS codepage whose table has no lead bytes is the silent-green failure
        // this whole file exists to prevent.
        if dbcs {
            if r.ismbcodepage == 0 {
                bad.push("ismbcodepage=0 for a DBCS codepage".to_string());
            }
            if lead == 0 {
                bad.push("no lead bytes — every DBCS branch is unreachable".to_string());
            }
            if trail == 0 {
                bad.push("no trail bytes".to_string());
            }
        } else if r.ismbcodepage != 0 {
            bad.push("ismbcodepage set for a single-byte codepage".to_string());
        }

        // Case mapping drives _mbctolower/_mbctoupper and the _SBUP/_SBLOW predicates.
        // Empty means those grade as no-ops that agree.
        if cased == 0 {
            bad.push("mbcasemap empty — case paths grade nothing".to_string());
        }
        if upper == 0 || lower == 0 {
            bad.push("no cased single bytes".to_string());
        }

        let verdict = if bad.is_empty() {
            "ok".to_string()
        } else {
            failures.push(format!("cp{cp}: {}", bad.join("; ")));
            format!("FAIL: {}", bad.join("; "))
        };
        println!(
            "  {cp:>6}  {lead:>5} {trail:>5} {upper:>5} {lower:>5} {cased:>6}  {verdict}"
        );
    }

    if !failures.is_empty() {
        println!("\n  THE FIXTURE IS NOT MBCS.\n");
        for f in &failures {
            println!("    - {f}");
        }
        println!(
            "\n  Every mbcs case is pinned against this state. An empty table does not\n\
             \x20 fail those cases — it makes both sides take the single-byte path and\n\
             \x20 AGREE, so the run comes back green having graded nothing."
        );
        std::process::exit(1);
    }
    println!("\n  fixture is MBCS on all {} codepages\n", unsafe {
        nls_codepage_count()
    });
}
