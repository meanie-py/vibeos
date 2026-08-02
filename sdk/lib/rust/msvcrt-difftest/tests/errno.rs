#![no_builtins] // the #[no_mangle] libc-named shims must be CALLED, not folded to LLVM builtins.
//! Wave 18c — grade `msvcrt-crt-rust/src/msvcrt/errno.rs` against the GENUINE
//! `dll/win32/msvcrt/errno.c` it replaces.
//!
//! # What this suite is really for
//!
//! Most rows here grade an export. The FIRST one grades something else, and it is the
//! reason the file exists: `msvcrt_set_errno`, the Win32 -> errno mapping.
//!
//! Until errno.c became reference TU #7 the harness owned a hand-written copy of that
//! mapping (fileimpl.c's `msvcrt_difftest_win32_to_errno`, 42 ERROR_* cases) and BOTH
//! sides called it. `file.c` stores through it 54 times and `file.rs` does too, so the
//! single most exercised mapping in the whole file suite was the one thing no row could
//! grade: shared substrate is invisible to a differential test, both sides are wrong
//! identically, and every row reports MATCH. That is §9.73 in miniature.
//!
//! Both sides now own their mapping — the reference runs msvcrt's genuine switch, the
//! subject runs Rust's — so the row below is the first time the 42 cases have ever been
//! compared against anything. Note what that means for `file.rs`: its 32 rows were always
//! green on errno, and they were green partly because both sides consulted one table. They
//! are still green, and now for a reason.
//!
//! # The two slots, and why both are read
//!
//! `msvcrt_set_errno` writes TWO places: `*__doserrno() = err` (the raw Win32 code,
//! verbatim) and `*_errno() = <mapped>`. A row that only checks errno would pass over an
//! implementation that never wrote doserrno at all, and `_get_doserrno`/`__doserrno` are
//! exports in their own right. `ref_doserrno_get`/`subj_doserrno_get` are the windows;
//! the reference's is new with this wave, because before it the slot was a file-static in
//! fileimpl.c that nothing outside that file could see.
//!
//! # Coverage is asserted, not assumed
//!
//! The 42 mapped codes are enumerated rather than fuzzed — a fuzz over `u32` would reach
//! the `default: EINVAL` arm essentially always and essentially never reach a named case,
//! which is the "a fuzzer only falsifies what it VARIES" trap (§9.60) in its purest form.
//! The unmapped arm is then checked separately, on codes chosen to sit BETWEEN the mapped
//! ones. Both halves must fire or the row fails as UNEXERCISED.

extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_int, c_uint, c_void};

use msvcrt_difftest::{ref_doserrno_get, ref_errno_get, ref_errno_set};

// ── the REFERENCE: genuine errno.c, renamed by build.rs's errno prelude ──
extern "C" {
    fn msvcrt_ref_msvcrt_set_errno(err: c_int);
    fn ref_strerror(err: c_int) -> *mut c_char;
    fn ref__strerror(str_: *const c_char) -> *mut c_char;
    fn ref_strerror_s(buf: *mut c_char, n: usize, err: c_int) -> c_int;
    fn ref__get_errno(out: *mut c_int) -> c_int;
    fn ref__set_errno(v: c_int) -> c_int;
    fn ref__get_doserrno(out: *mut c_uint) -> c_int;
    fn ref__set_doserrno(v: c_uint) -> c_int;
    fn ref__seterrormode(mode: c_int);
    fn ref__wcserror(err: c_int) -> *mut u16;
    fn ref__wcserror_s(buf: *mut u16, nc: usize, err: c_int) -> c_int;
    fn ref___wcserror(str_: *const u16) -> *mut u16;
    static mut ref_MSVCRT__sys_nerr: c_uint;
    static mut ref_MSVCRT__sys_errlist: [*mut c_char; 44];
}

// ── the SUBJECT: msvcrt-crt-rust's errno.rs, under the plain names ──
extern "C" {
    fn msvcrt_set_errno(err: c_int);
    fn strerror(err: c_int) -> *mut c_char;
    fn _strerror(str_: *const c_char) -> *mut c_char;
    fn strerror_s(buf: *mut c_char, n: usize, err: c_int) -> c_int;
    fn _get_errno(out: *mut c_int) -> c_int;
    fn _set_errno(v: c_int) -> c_int;
    fn _get_doserrno(out: *mut c_uint) -> c_int;
    fn _set_doserrno(v: c_uint) -> c_int;
    fn _seterrormode(mode: c_int);
    fn _wcserror(err: c_int) -> *mut u16;
    fn _wcserror_s(buf: *mut u16, nc: usize, err: c_int) -> c_int;
    fn __wcserror(str_: *const u16) -> *mut u16;
    static mut MSVCRT__sys_nerr: c_uint;
    static mut MSVCRT__sys_errlist: [*mut c_char; 44];

    // subjectimpl.c's window onto the subject's own two slots.
    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_doserrno_get() -> c_uint;
    fn subj_doserrno_set(v: c_uint);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
}
extern "C" {
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();
}

// ── verdict plumbing, same shape as classify.rs / wave16.rs ──
#[derive(PartialEq)]
enum V {
    Match,
    Diverge,
}
static mut ROWS: Vec<(&'static str, V, String)> = Vec::new();

fn record(name: &'static str, v: V, detail: String) {
    unsafe { (*core::ptr::addr_of_mut!(ROWS)).push((name, v, detail)) }
}

/// Every observable of an errno.c call: what it returned, both slots, and whether the
/// invalid-parameter handler fired.
#[derive(PartialEq, Debug)]
struct Fx {
    ret: i64,
    errno: i32,
    doserrno: u32,
    pmt: i32,
}

fn ref_fx(ret: i64) -> Fx {
    unsafe {
        Fx { ret, errno: ref_errno_get(), doserrno: ref_doserrno_get(), pmt: ref_invalid_pmt_count() }
    }
}
fn subj_fx(ret: i64) -> Fx {
    unsafe {
        Fx { ret, errno: subj_errno_get(), doserrno: subj_doserrno_get(), pmt: subj_invalid_pmt_count() }
    }
}

/// Zero BOTH sides' slots and handler counters so a row starts from a known state and
/// cannot inherit the previous row's errno. (The two sides genuinely have separate
/// storage now — that separation is what this suite grades — so both must be cleared.)
fn reset_both() {
    unsafe {
        ref_errno_set(0);
        msvcrt_difftest::ref_doserrno_set(0);
        subj_errno_set(0);
        subj_doserrno_set(0);
        ref_invalid_pmt_reset();
        subj_invalid_pmt_reset();
    }
}

fn cstr(p: *const c_char) -> Vec<u8> {
    if p.is_null() {
        return b"<null>".to_vec();
    }
    let mut out = Vec::new();
    unsafe {
        let mut i = 0;
        while *p.add(i) != 0 {
            out.push(*p.add(i) as u8);
            i += 1;
        }
    }
    out
}

fn wstr(p: *const u16) -> Vec<u16> {
    if p.is_null() {
        return vec![u16::MAX];
    }
    let mut out = Vec::new();
    unsafe {
        let mut i = 0;
        while *p.add(i) != 0 {
            out.push(*p.add(i));
            i += 1;
        }
    }
    out
}

/// The 42 Win32 codes msvcrt's switch names, with the errno each maps to. Enumerated,
/// not fuzzed, and the VALUES are here only so the table itself is reviewable — the row
/// compares the two implementations against each other, never against this column, so a
/// typo here cannot manufacture a false MATCH. It can only make the table misleading to
/// read, which is why it is checked against the reference at the end of the row.
const MAPPED: &[(u32, i32)] = &[
    (5, 13),      // ERROR_ACCESS_DENIED         -> EACCES
    (65, 13),     // ERROR_NETWORK_ACCESS_DENIED -> EACCES
    (82, 13),     // ERROR_CANNOT_MAKE           -> EACCES
    (132, 13),    // ERROR_SEEK_ON_DEVICE        -> EACCES
    (167, 13),    // ERROR_LOCK_FAILED           -> EACCES
    (83, 13),     // ERROR_FAIL_I24              -> EACCES
    (16, 13),     // ERROR_CURRENT_DIRECTORY     -> EACCES
    (108, 13),    // ERROR_DRIVE_LOCKED          -> EACCES
    (158, 13),    // ERROR_NOT_LOCKED            -> EACCES
    (12, 13),     // ERROR_INVALID_ACCESS        -> EACCES
    (32, 13),     // ERROR_SHARING_VIOLATION     -> EACCES
    (33, 13),     // ERROR_LOCK_VIOLATION        -> EACCES
    (2, 2),       // ERROR_FILE_NOT_FOUND        -> ENOENT
    (18, 2),      // ERROR_NO_MORE_FILES         -> ENOENT
    (161, 2),     // ERROR_BAD_PATHNAME          -> ENOENT
    (53, 2),      // ERROR_BAD_NETPATH           -> ENOENT
    (15, 2),      // ERROR_INVALID_DRIVE         -> ENOENT
    (67, 2),      // ERROR_BAD_NET_NAME          -> ENOENT
    (206, 2),     // ERROR_FILENAME_EXCED_RANGE  -> ENOENT
    (3, 2),       // ERROR_PATH_NOT_FOUND        -> ENOENT
    (1117, 5),    // ERROR_IO_DEVICE             -> EIO
    (11, 8),      // ERROR_BAD_FORMAT            -> ENOEXEC
    (6, 9),       // ERROR_INVALID_HANDLE        -> EBADF
    (14, 12),     // ERROR_OUTOFMEMORY           -> ENOMEM
    (9, 12),      // ERROR_INVALID_BLOCK         -> ENOMEM
    (1816, 12),   // ERROR_NOT_ENOUGH_QUOTA      -> ENOMEM
    (7, 12),      // ERROR_ARENA_TRASHED         -> ENOMEM
    (170, 16),    // ERROR_BUSY                  -> EBUSY
    (183, 17),    // ERROR_ALREADY_EXISTS        -> EEXIST
    (80, 17),     // ERROR_FILE_EXISTS           -> EEXIST
    (1200, 19),   // ERROR_BAD_DEVICE            -> ENODEV
    (4, 24),      // ERROR_TOO_MANY_OPEN_FILES   -> EMFILE
    (112, 28),    // ERROR_DISK_FULL             -> ENOSPC
    (109, 32),    // ERROR_BROKEN_PIPE           -> EPIPE
    (1131, 36),   // ERROR_POSSIBLE_DEADLOCK     -> EDEADLK
    (145, 41),    // ERROR_DIR_NOT_EMPTY         -> ENOTEMPTY
    (10, 7),      // ERROR_BAD_ENVIRONMENT       -> E2BIG
    (128, 10),    // ERROR_WAIT_NO_CHILDREN      -> ECHILD
    (129, 10),    // ERROR_CHILD_NOT_COMPLETE    -> ECHILD
    (89, 11),     // ERROR_NO_PROC_SLOTS         -> EAGAIN
    (164, 11),    // ERROR_MAX_THRDS_REACHED     -> EAGAIN
    (215, 11),    // ERROR_NESTING_NOT_ALLOWED   -> EAGAIN
];

/// Codes deliberately chosen to sit BETWEEN the mapped ones, so the `default: EINVAL`
/// arm is reached on purpose rather than by luck.
const UNMAPPED: &[u32] = &[0, 1, 8, 13, 17, 19, 50, 100, 200, 500, 1000, 5000, 0x7FFF_FFFF, 0xFFFF_FFFF];

fn main_rows() {
    // ── THE row: the Win32 -> errno mapping, both slots, all 42 named codes ──
    {
        let mut seen_mapped = 0usize;
        let mut seen_default = 0usize;
        let mut bad: Option<String> = None;
        let codes: Vec<u32> = MAPPED.iter().map(|&(c, _)| c).chain(UNMAPPED.iter().copied()).collect();
        for &code in &codes {
            reset_both();
            let r = unsafe {
                msvcrt_ref_msvcrt_set_errno(code as c_int);
                ref_fx(0)
            };
            let s = unsafe {
                msvcrt_set_errno(code as c_int);
                subj_fx(0)
            };
            if r != s {
                bad = Some(format!("[code {code}]\n        msvcrt= {r:?}\n        rust  = {s:?}"));
                break;
            }
            if MAPPED.iter().any(|&(c, _)| c == code) {
                seen_mapped += 1;
            } else {
                seen_default += 1;
            }
            // doserrno must be the RAW code, on both sides — a mapping that forgets to
            // store it would otherwise pass every errno-only assertion.
            if r.doserrno != code {
                bad = Some(format!("[code {code}] doserrno not stored verbatim: {}", r.doserrno));
                break;
            }
        }
        match bad {
            Some(d) => record("msvcrt_set_errno (42 mapped + default)", V::Diverge, d),
            None if seen_mapped != MAPPED.len() || seen_default == 0 => record(
                "msvcrt_set_errno (42 mapped + default)",
                V::Diverge,
                format!(
                    "UNEXERCISED — mapped={seen_mapped}/{} default={seen_default}. \
                     A MATCH that never reached an arm proves nothing.",
                    MAPPED.len()
                ),
            ),
            None => record(
                "msvcrt_set_errno (42 mapped + default)",
                V::Match,
                format!("mapped={seen_mapped} default-EINVAL={seen_default}, errno+doserrno both compared"),
            ),
        }
    }

    // ── the mapping table this file documents, checked against the reference ──
    // Cheap, and it keeps the MAPPED column above honest: it is documentation, and
    // documentation that nothing checks is documentation that rots.
    {
        let mut wrong = Vec::new();
        for &(code, expect) in MAPPED {
            reset_both();
            unsafe { msvcrt_ref_msvcrt_set_errno(code as c_int) };
            let got = unsafe { ref_errno_get() };
            if got != expect {
                wrong.push(format!("code {code}: table says {expect}, msvcrt says {got}"));
            }
        }
        if wrong.is_empty() {
            record("MAPPED table vs msvcrt", V::Match, format!("{} codes", MAPPED.len()));
        } else {
            record("MAPPED table vs msvcrt", V::Diverge, wrong.join("\n        "));
        }
    }

    // ── _sys_nerr and the whole _sys_errlist table ──
    // The strings are data, not code, and they are exactly the kind of thing a
    // translation drops a NUL off (wave 18c shipped two such: str_ENOTTY and str_ENOSYS
    // lost their terminators, which is why they are generated now).
    unsafe {
        let rn = ref_MSVCRT__sys_nerr;
        let sn = MSVCRT__sys_nerr;
        if rn != sn {
            record("_sys_nerr", V::Diverge, format!("msvcrt={rn} rust={sn}"));
        } else {
            record("_sys_nerr", V::Match, format!("{rn}"));
        }
        let mut bad = Vec::new();
        for i in 0..=(rn.min(sn) as usize) {
            let r = cstr(ref_MSVCRT__sys_errlist[i]);
            let s = cstr(MSVCRT__sys_errlist[i]);
            if r != s {
                bad.push(format!(
                    "[{i}] msvcrt={:?} rust={:?}",
                    String::from_utf8_lossy(&r),
                    String::from_utf8_lossy(&s)
                ));
            }
        }
        if bad.is_empty() {
            record("_sys_errlist", V::Match, format!("{} entries byte-identical", rn + 1));
        } else {
            record("_sys_errlist", V::Diverge, bad.join("\n        "));
        }
    }

    // ── strerror / _wcserror: the clamps are the contract ──
    // `if (err < 0 || err > _sys_nerr) err = _sys_nerr` — a translation that writes `>=`,
    // or that clamps to 0, is string-identical for every ordinary code and wrong exactly
    // at the two boundaries. So the sweep runs from -3 to _sys_nerr + 3.
    unsafe {
        let n = ref_MSVCRT__sys_nerr as i32;
        let probes: Vec<i32> = (-3..=3).chain([n - 1, n, n + 1, n + 2, n + 3]).collect();
        let mut bad = Vec::new();
        let (mut low, mut high, mut inrange) = (0, 0, 0);
        for &e in &probes {
            reset_both();
            let r = cstr(ref_strerror(e));
            let s = cstr(strerror(e));
            if r != s {
                bad.push(format!("strerror({e}): msvcrt={:?} rust={:?}",
                    String::from_utf8_lossy(&r), String::from_utf8_lossy(&s)));
            }
            let rw = wstr(ref__wcserror(e));
            let sw = wstr(_wcserror(e));
            if rw != sw {
                bad.push(format!("_wcserror({e}): msvcrt={rw:?} rust={sw:?}"));
            }
            if e < 0 { low += 1 } else if e > n { high += 1 } else { inrange += 1 }
        }
        if !bad.is_empty() {
            record("strerror/_wcserror clamps", V::Diverge, bad.join("\n        "));
        } else if low == 0 || high == 0 || inrange == 0 {
            record("strerror/_wcserror clamps", V::Diverge,
                format!("UNEXERCISED — low={low} high={high} inrange={inrange}"));
        } else {
            record("strerror/_wcserror clamps", V::Match,
                format!("below={low} above={high} inrange={inrange}"));
        }
    }

    // ── the four accessors ──
    // Trivial bodies, and precisely the shape a translation gets subtly wrong: msvcrt's
    // `_get_errno` returns EINVAL and does NOT touch *out on a NULL argument, and
    // `_set_errno` returns 0 always. Both NULL and non-NULL are driven.
    unsafe {
        let mut bad = Vec::new();
        for &v in &[0i32, 1, 22, -1, i32::MAX, i32::MIN] {
            reset_both();
            ref__set_errno(v);
            _set_errno(v);
            let (mut ro, mut so) = (0i32, 0i32);
            let rr = ref__get_errno(&mut ro);
            let sr = _get_errno(&mut so);
            if (rr, ro, ref_errno_get()) != (sr, so, subj_errno_get()) {
                bad.push(format!("_set/_get_errno({v}): msvcrt=({rr},{ro},{}) rust=({sr},{so},{})",
                    ref_errno_get(), subj_errno_get()));
            }
            reset_both();
            ref__set_doserrno(v as u32);
            _set_doserrno(v as u32);
            let (mut rd, mut sd) = (0u32, 0u32);
            let rr = ref__get_doserrno(&mut rd);
            let sr = _get_doserrno(&mut sd);
            if (rr, rd) != (sr, sd) {
                bad.push(format!("_set/_get_doserrno({v}): msvcrt=({rr},{rd}) rust=({sr},{sd})"));
            }
        }
        // the NULL arm — an out-parameter API's most-forgotten branch
        reset_both();
        let rn = ref__get_errno(core::ptr::null_mut());
        let sn = _get_errno(core::ptr::null_mut());
        let rnd = ref__get_doserrno(core::ptr::null_mut());
        let snd = _get_doserrno(core::ptr::null_mut());
        if (rn, rnd) != (sn, snd) {
            bad.push(format!("NULL out: msvcrt=({rn},{rnd}) rust=({sn},{snd})"));
        }
        if (ref_invalid_pmt_count() > 0) != (subj_invalid_pmt_count() > 0) {
            bad.push(format!("NULL out handler: msvcrt={} rust={}",
                ref_invalid_pmt_count(), subj_invalid_pmt_count()));
        }
        if bad.is_empty() {
            record("_get/_set_errno + doserrno (incl. NULL)", V::Match, "6 values x 2 slots + NULL arm".into());
        } else {
            record("_get/_set_errno + doserrno (incl. NULL)", V::Diverge, bad.join("\n        "));
        }
    }

    // ── strerror_s / _wcserror_s / __wcserror_s: buffer + effects ──
    // The `_s` family's whole contract is the error arms, so tiny buffers are driven on
    // purpose: n = 0 and n = 1 are where a translation either writes a NUL it should not
    // or skips one it should.
    unsafe {
        let mut bad = Vec::new();
        for &n in &[0usize, 1, 2, 8, 64] {
            for &e in &[0i32, 2, 22, -1, 1000] {
                reset_both();
                let mut rb = vec![0x7Fu8; 80];
                let mut sb = vec![0x7Fu8; 80];
                let rr = ref_strerror_s(rb.as_mut_ptr() as *mut c_char, n, e);
                let sr = strerror_s(sb.as_mut_ptr() as *mut c_char, n, e);
                if (rr, &rb, ref_fx(0)) != (sr, &sb, subj_fx(0)) {
                    bad.push(format!(
                        "strerror_s(n={n}, err={e}):\n        msvcrt= {rr} {:?} {:?}\n        rust  = {sr} {:?} {:?}",
                        &rb[..16], ref_fx(0), &sb[..16], subj_fx(0)));
                }
                reset_both();
                let mut rw = vec![0x7F7Fu16; 80];
                let mut sw = vec![0x7F7Fu16; 80];
                let rr = ref__wcserror_s(rw.as_mut_ptr(), n, e);
                let sr = _wcserror_s(sw.as_mut_ptr(), n, e);
                let (rfx, sfx) = (ref_fx(rr as i64), subj_fx(sr as i64));
                if rfx != sfx || rw != sw {
                    let first_diff = rw.iter().zip(sw.iter()).position(|(a, b)| a != b);
                    bad.push(format!(
                        "_wcserror_s(n={n}, err={e}):\n        msvcrt= {rfx:?}\n        rust  = {sfx:?}\n        \
                         buf first-diff at {first_diff:?}"));
                }
            }
        }
        if bad.is_empty() {
            record("strerror_s / _wcserror_s", V::Match, "5 sizes x 5 codes, buffer+effects".into());
        } else {
            record("strerror_s / _wcserror_s", V::Diverge, bad.join("\n        "));
        }
    }

    // ── _strerror / __wcserror: the "prefix: message\n" builders ──
    // Both take a caller string and BOTH accept NULL, which is the arm that matters:
    // NULL means "no prefix" and the output loses the ": " as well as the string.
    unsafe {
        let mut bad = Vec::new();
        let prefixes: [&[u8]; 4] = [b"\0", b"x\0", b"a longer prefix string\0", b"\0"];
        for (i, p) in prefixes.iter().enumerate() {
            for &e in &[0i32, 2, 22] {
                reset_both();
                ref_errno_set(e);
                subj_errno_set(e);
                let arg = if i == 3 { core::ptr::null() } else { p.as_ptr() as *const c_char };
                let r = cstr(ref__strerror(arg));
                let s = cstr(_strerror(arg));
                if r != s {
                    bad.push(format!("_strerror({:?}, errno={e}): msvcrt={:?} rust={:?}",
                        if i == 3 { "NULL".into() } else { String::from_utf8_lossy(p).to_string() },
                        String::from_utf8_lossy(&r), String::from_utf8_lossy(&s)));
                }
                reset_both();
                ref_errno_set(e);
                subj_errno_set(e);
                let wp: Vec<u16> = p.iter().map(|&b| b as u16).collect();
                let warg = if i == 3 { core::ptr::null() } else { wp.as_ptr() };
                let rw = wstr(ref___wcserror(warg));
                let sw = wstr(__wcserror(warg));
                if rw != sw {
                    bad.push(format!("__wcserror(errno={e}): msvcrt={rw:?} rust={sw:?}"));
                }
            }
        }
        if bad.is_empty() {
            record("_strerror / __wcserror (incl. NULL prefix)", V::Match, "4 prefixes x 3 errno".into());
        } else {
            record("_strerror / __wcserror (incl. NULL prefix)", V::Diverge, bad.join("\n        "));
        }
    }

    // ── _seterrormode ──
    // A straight forwarder to Win32 SetErrorMode. fileimpl.c models that as a real
    // read-modify-write process-wide slot, so "did the store happen" is observable —
    // without a real store this row would pass over an implementation that dropped it.
    unsafe {
        let mut bad = Vec::new();
        for &m in &[0i32, 1, 2, 0x8000, 0x7FFF] {
            reset_both();
            ref__seterrormode(m);
            _seterrormode(m);
            if ref_fx(0) != subj_fx(0) {
                bad.push(format!("_seterrormode({m}): msvcrt={:?} rust={:?}", ref_fx(0), subj_fx(0)));
            }
        }
        if bad.is_empty() {
            record("_seterrormode", V::Match, "5 modes".into());
        } else {
            record("_seterrormode", V::Diverge, bad.join("\n        "));
        }
    }
}

#[test]
fn errno_rows() {
    // Prove the subject crate is actually in the link. Without this the shared names
    // (strerror, perror, …) could bind to GLIBC and the suite would grade glibc against
    // msvcrt and report a meaningless MATCH — the same guard wave16.rs carries.
    unsafe {
        let p = strerror(2) as *const c_void;
        assert!(!p.is_null(), "subject strerror returned NULL — is msvcrt-crt-rust linked?");
    }

    main_rows();

    let rows = unsafe { &*core::ptr::addr_of!(ROWS) };
    let (mut m, mut d) = (0, 0);
    println!();
    for (name, v, detail) in rows.iter() {
        match v {
            V::Match => {
                m += 1;
                println!("  {name:<42} MATCH       ✓");
                if !detail.is_empty() {
                    println!("        {detail}");
                }
            }
            V::Diverge => {
                d += 1;
                println!("  {name:<42} DIVERGE     ✗");
                println!("        {detail}");
            }
        }
    }
    println!("\n  {m} MATCH   {d} DIVERGE   (of {} rows)\n", rows.len());
    assert!(d == 0, "{d} errno row(s) diverge");
}
