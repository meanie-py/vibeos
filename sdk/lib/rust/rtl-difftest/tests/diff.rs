//! Differential parity: rtl-strings `cabi` NTAPI shims vs the genuine
//! compiled ReactOS `sdk/lib/rtl/unicode.c` (build.rs). Wave RTL-1 =
//! the NLS-free counted-string ops. Pins the load-bearing rtl
//! misfeatures: the `Length` vs `MaximumLength` (+sizeof(NUL))
//! asymmetry, the `MAXUSHORT - sizeof(CHAR)` silent clamp,
//! `RtlInitUnicodeString`'s `(MAXUSHORT & ~1)` even-clamp, the
//! `*Ex` `STATUS_NAME_TOO_LONG`-vs-silent-clamp divergence (and that
//! `*Ex` leaves the dest UNTOUCHED on error), NULL ⇒ `{0,0}` while
//! `Buffer` is still set to the (NULL) source, `RtlCompareString`'s
//! signed-`char` diff + `(int)L1-(int)L2` length tiebreak,
//! `RtlPrefixString`'s NULL-Buffer ⇒ FALSE.

use rtl_strings::cabi::{self, STRING, UNICODE_STRING};

extern "C" {
    fn ref_RtlInitAnsiString(d: *mut STRING, s: *const u8);
    fn ref_RtlInitString(d: *mut STRING, s: *const u8);
    fn ref_RtlInitAnsiStringEx(d: *mut STRING, s: *const u8) -> i32;
    fn ref_RtlInitUnicodeString(d: *mut UNICODE_STRING, s: *const u16);
    fn ref_RtlInitUnicodeStringEx(d: *mut UNICODE_STRING, s: *const u16) -> i32;
    fn ref_RtlCompareString(a: *const STRING, b: *const STRING, ci: u8) -> i32;
    fn ref_RtlEqualString(a: *const STRING, b: *const STRING, ci: u8) -> u8;
    fn ref_RtlPrefixString(a: *const STRING, b: *const STRING, ci: u8) -> u8;
    // Wave RTL-2
    fn ref_RtlUpcaseUnicodeChar(c: u16) -> u16;
    fn ref_RtlUpperChar(c: u8) -> u8;
    fn ref_RtlCompareUnicodeString(
        a: *const UNICODE_STRING,
        b: *const UNICODE_STRING,
        ci: u8,
    ) -> i32;
    fn ref_RtlEqualUnicodeString(
        a: *const UNICODE_STRING,
        b: *const UNICODE_STRING,
        ci: u8,
    ) -> u8;
    fn ref_RtlPrefixUnicodeString(
        a: *const UNICODE_STRING,
        b: *const UNICODE_STRING,
        ci: u8,
    ) -> u8;
    fn ref_RtlUpperString(d: *mut STRING, s: *const STRING);
    fn ref_RtlHashUnicodeString(
        s: *const UNICODE_STRING,
        ci: u8,
        alg: u32,
        h: *mut u32,
    ) -> i32;
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn upto(&mut self, n: usize) -> usize {
        (self.next() as usize) % n
    }
    fn byte(&mut self) -> u8 {
        (self.next() & 0xff) as u8
    }
}

fn mk_string(buf: *mut u8, len: u16, max: u16) -> STRING {
    STRING {
        Length: len,
        MaximumLength: max,
        Buffer: buf,
    }
}

/// A STRING dest prefilled with a recognizable sentinel so we can
/// assert "untouched" on the `*Ex` error path.
fn sentinel_s() -> STRING {
    STRING {
        Length: 0xDEAD,
        MaximumLength: 0xBEEF,
        Buffer: 0x1234_5678_9abc_def0u64 as *mut u8,
    }
}
fn sentinel_u() -> UNICODE_STRING {
    UNICODE_STRING {
        Length: 0xDEAD,
        MaximumLength: 0xBEEF,
        Buffer: 0x1234_5678_9abc_def0u64 as *mut u16,
    }
}
fn seq(a: &STRING, b: &STRING) -> bool {
    a.Length == b.Length && a.MaximumLength == b.MaximumLength && a.Buffer == b.Buffer
}
fn ueq(a: &UNICODE_STRING, b: &UNICODE_STRING) -> bool {
    a.Length == b.Length && a.MaximumLength == b.MaximumLength && a.Buffer == b.Buffer
}

#[test]
fn diff_rtl_init_ansi() {
    let mut rng = Rng(0x1234_abcd);
    for _ in 0..20_000 {
        let n = rng.upto(40);
        let mut s: Vec<u8> = (0..n).map(|_| 1 + rng.byte() % 255).collect();
        s.push(0);
        for nul in [false, true] {
            let p = if nul {
                core::ptr::null()
            } else {
                s.as_ptr()
            };
            let (mut c, mut r) = (sentinel_s(), sentinel_s());
            unsafe {
                cabi::RtlInitAnsiString(&mut c, p);
                ref_RtlInitAnsiString(&mut r, p);
            }
            assert!(seq(&c, &r), "RtlInitAnsiString nul={nul} n={n}: {:?} vs {:?}",
                (c.Length, c.MaximumLength), (r.Length, r.MaximumLength));
            let (mut c, mut r) = (sentinel_s(), sentinel_s());
            unsafe {
                cabi::RtlInitString(&mut c, p);
                ref_RtlInitString(&mut r, p);
            }
            assert!(seq(&c, &r), "RtlInitString nul={nul} n={n}");
        }
    }
    // The MAXUSHORT - sizeof(CHAR) silent clamp (Init) vs the
    // STATUS_NAME_TOO_LONG hard error (InitEx), and that *Ex leaves
    // the dest UNTOUCHED on error.
    for len in [0usize, 1, 0xFFFD, 0xFFFE, 0xFFFF, 0x1_0000, 0x2_0000] {
        let mut s = vec![b'A'; len];
        s.push(0);
        let p = s.as_ptr();
        let (mut c, mut r) = (sentinel_s(), sentinel_s());
        unsafe {
            cabi::RtlInitAnsiString(&mut c, p);
            ref_RtlInitAnsiString(&mut r, p);
        }
        assert!(seq(&c, &r), "RtlInitAnsiString clamp len={len}");
        let (mut c, mut r) = (sentinel_s(), sentinel_s());
        let cs;
        let rs;
        unsafe {
            cs = cabi::RtlInitAnsiStringEx(&mut c, p);
            rs = ref_RtlInitAnsiStringEx(&mut r, p);
        }
        assert_eq!(cs, rs, "RtlInitAnsiStringEx ret len={len}");
        if cs == 0 {
            assert!(seq(&c, &r), "RtlInitAnsiStringEx ok fields len={len}");
        } else {
            // error ⇒ dest must be untouched (both stay sentinel).
            assert!(seq(&c, &sentinel_s()) && seq(&r, &sentinel_s()),
                "RtlInitAnsiStringEx error must not touch dest len={len}");
        }
    }
}

#[test]
fn diff_rtl_init_unicode() {
    let mut rng = Rng(0xfeed_face);
    for _ in 0..20_000 {
        let n = rng.upto(40);
        let mut s: Vec<u16> = (0..n).map(|_| 1 + (rng.next() as u16 % 0xFFFE)).collect();
        s.push(0);
        for nul in [false, true] {
            let p = if nul {
                core::ptr::null()
            } else {
                s.as_ptr()
            };
            let (mut c, mut r) = (sentinel_u(), sentinel_u());
            unsafe {
                cabi::RtlInitUnicodeString(&mut c, p);
                ref_RtlInitUnicodeString(&mut r, p);
            }
            assert!(ueq(&c, &r), "RtlInitUnicodeString nul={nul} n={n}: {:?} vs {:?}",
                (c.Length, c.MaximumLength), (r.Length, r.MaximumLength));
        }
    }
    for units in [0usize, 1, 0x7FFD, 0x7FFE, 0x7FFF, 0x8000, 0x1_0000] {
        let mut s = vec![b'A' as u16; units];
        s.push(0);
        let p = s.as_ptr();
        let (mut c, mut r) = (sentinel_u(), sentinel_u());
        unsafe {
            cabi::RtlInitUnicodeString(&mut c, p);
            ref_RtlInitUnicodeString(&mut r, p);
        }
        assert!(ueq(&c, &r), "RtlInitUnicodeString clamp units={units}");
        let (mut c, mut r) = (sentinel_u(), sentinel_u());
        let cs;
        let rs;
        unsafe {
            cs = cabi::RtlInitUnicodeStringEx(&mut c, p);
            rs = ref_RtlInitUnicodeStringEx(&mut r, p);
        }
        assert_eq!(cs, rs, "RtlInitUnicodeStringEx ret units={units}");
        if cs == 0 {
            assert!(ueq(&c, &r), "RtlInitUnicodeStringEx ok units={units}");
        } else {
            assert!(ueq(&c, &sentinel_u()) && ueq(&r, &sentinel_u()),
                "RtlInitUnicodeStringEx error must not touch dest units={units}");
        }
    }
}

#[test]
fn diff_rtl_compare_equal_prefix() {
    let mut rng = Rng(0x0bad_c0de);
    // small alphabet incl. high bytes (signed-char path) so
    // collisions / sign flips actually happen.
    let alpha: &[u8] = &[0, 1, b'A', b'a', b'z', 0x7f, 0x80, 0xff];
    for _ in 0..40_000 {
        let mk = |rng: &mut Rng| -> Vec<u8> {
            (0..rng.upto(10)).map(|_| alpha[rng.upto(alpha.len())]).collect()
        };
        let b1 = mk(&mut rng);
        let b2 = mk(&mut rng);
        // counted strings: Length is the explicit byte count (NOT
        // NUL-bounded). Buffer may be a dangling-but-unused ptr when
        // Length==0.
        let p1 = if b1.is_empty() {
            core::ptr::null_mut()
        } else {
            b1.as_ptr() as *mut u8
        };
        let p2 = if b2.is_empty() {
            core::ptr::null_mut()
        } else {
            b2.as_ptr() as *mut u8
        };
        let s1 = mk_string(p1, b1.len() as u16, b1.len() as u16);
        let s2 = mk_string(p2, b2.len() as u16, b2.len() as u16);

        unsafe {
            assert_eq!(
                cabi::RtlCompareString(&s1, &s2, 0),
                ref_RtlCompareString(&s1, &s2, 0),
                "RtlCompareString {b1:?} {b2:?}"
            );
            assert_eq!(
                cabi::RtlEqualString(&s1, &s2, 0),
                ref_RtlEqualString(&s1, &s2, 0),
                "RtlEqualString {b1:?} {b2:?}"
            );
            assert_eq!(
                cabi::RtlPrefixString(&s1, &s2, 0),
                ref_RtlPrefixString(&s1, &s2, 0),
                "RtlPrefixString {b1:?} {b2:?}"
            );
            // swapped (prefix is asymmetric; also exercises L2<L1)
            assert_eq!(
                cabi::RtlPrefixString(&s2, &s1, 0),
                ref_RtlPrefixString(&s2, &s1, 0),
                "RtlPrefixString swapped {b2:?} {b1:?}"
            );
        }
    }
    // explicit NULL-Buffer prefix (genuine: pc1&&pc2 else FALSE) and
    // the length tiebreak.
    let data = *b"abc";
    let s_ok = mk_string(data.as_ptr() as *mut u8, 3, 3);
    let s_nul = mk_string(core::ptr::null_mut(), 3, 3);
    let s_empty_nul = mk_string(core::ptr::null_mut(), 0, 0);
    unsafe {
        // RtlPrefixString is the only one that guards NULL buffers
        // (`if (pc1 && pc2)`); RtlCompareString/EqualString deref
        // unconditionally, so they're only exercised on the safe
        // Length==0 (no-deref) pair here.
        for (a, b) in [
            (&s_nul, &s_ok),
            (&s_ok, &s_nul),
            (&s_empty_nul, &s_ok),
            (&s_empty_nul, &s_empty_nul),
        ] {
            assert_eq!(
                cabi::RtlPrefixString(a, b, 0),
                ref_RtlPrefixString(a, b, 0),
                "RtlPrefixString NULL-buffer"
            );
        }
        // Length==0 ⇒ the compare loop never derefs the (NULL) Buffer.
        assert_eq!(
            cabi::RtlCompareString(&s_empty_nul, &s_empty_nul, 0),
            ref_RtlCompareString(&s_empty_nul, &s_empty_nul, 0),
            "RtlCompareString empty"
        );
        assert_eq!(
            cabi::RtlEqualString(&s_empty_nul, &s_empty_nul, 0),
            ref_RtlEqualString(&s_empty_nul, &s_empty_nul, 0),
            "RtlEqualString empty"
        );
    }
}

// ── Wave RTL-2: the shared-CP1252 case-aware surface ───────────────

fn mk_ustr(buf: *mut u16, len: u16, max: u16) -> UNICODE_STRING {
    UNICODE_STRING {
        Length: len,
        MaximumLength: max,
        Buffer: buf,
    }
}

/// The single primitive both string ops fold through — exhaustive
/// over the whole WCHAR domain pins the shared model itself.
#[test]
fn diff_rtl_upcase_char() {
    for w in 0u32..=0xFFFF {
        let w = w as u16;
        unsafe {
            assert_eq!(
                cabi::RtlUpcaseUnicodeChar(w),
                ref_RtlUpcaseUnicodeChar(w),
                "RtlUpcaseUnicodeChar {w:#06x}"
            );
        }
    }
    for b in 0u16..=0xFF {
        let b = b as u8;
        unsafe {
            assert_eq!(
                cabi::RtlUpperChar(b),
                ref_RtlUpperChar(b),
                "RtlUpperChar {b:#04x}"
            );
        }
    }
}

/// The RTL-1 ANSI ops, now with the case-INsensitive path live
/// (`RtlUpperChar` fold). Alphabet spans a–z / A–Z / high bytes so
/// the signed-`char` fold + tiebreak both fire.
#[test]
fn diff_rtl_ansi_ci() {
    let mut rng = Rng(0xa11ce_5eed);
    let alpha: &[u8] = &[0, 1, b'A', b'M', b'Z', b'a', b'm', b'z', 0x7f, 0x80, 0xC0, 0xff];
    for _ in 0..40_000 {
        let mk = |rng: &mut Rng| -> Vec<u8> {
            (0..rng.upto(12)).map(|_| alpha[rng.upto(alpha.len())]).collect()
        };
        let b1 = mk(&mut rng);
        let b2 = mk(&mut rng);
        let p1 = if b1.is_empty() { core::ptr::null_mut() } else { b1.as_ptr() as *mut u8 };
        let p2 = if b2.is_empty() { core::ptr::null_mut() } else { b2.as_ptr() as *mut u8 };
        let s1 = mk_string(p1, b1.len() as u16, b1.len() as u16);
        let s2 = mk_string(p2, b2.len() as u16, b2.len() as u16);
        unsafe {
            assert_eq!(
                cabi::RtlCompareString(&s1, &s2, 1),
                ref_RtlCompareString(&s1, &s2, 1),
                "RtlCompareString ci {b1:?} {b2:?}"
            );
            assert_eq!(
                cabi::RtlEqualString(&s1, &s2, 1),
                ref_RtlEqualString(&s1, &s2, 1),
                "RtlEqualString ci {b1:?} {b2:?}"
            );
            assert_eq!(
                cabi::RtlPrefixString(&s1, &s2, 1),
                ref_RtlPrefixString(&s1, &s2, 1),
                "RtlPrefixString ci {b1:?} {b2:?}"
            );
            assert_eq!(
                cabi::RtlPrefixString(&s2, &s1, 1),
                ref_RtlPrefixString(&s2, &s1, 1),
                "RtlPrefixString ci swapped {b2:?} {b1:?}"
            );
        }
    }
}

/// `RtlCompareUnicodeString` / `RtlEqualUnicodeString` /
/// `RtlPrefixUnicodeString`, both folds. The unit alphabet hits the
/// shared-model specials (a–z, à–þ, ÿ, µ, š, œ, ž) and an
/// out-of-model BMP point; byte `Length` is independently random so
/// the `min(L1,L2)/2` floor, the odd-trailing-byte drop and the
/// `(int)L1-(int)L2` tiebreak all fire.
#[test]
fn diff_rtl_unicode_cmp() {
    let mut rng = Rng(0xc0ffee_99);
    let alpha: &[u16] = &[
        0x0000, 0x0041, 0x0061, 0x007A, 0x00B5, 0x00C0, 0x00E0, 0x00FE,
        0x00FF, 0x0152, 0x0153, 0x017D, 0x017E, 0x0130, 0x20AC, 0xFFFF,
    ];
    for _ in 0..30_000 {
        let mk = |rng: &mut Rng| -> Vec<u16> {
            // a couple spare units so an odd byte-Length still has a
            // valid (read-past-floor-safe) backing unit.
            let n = rng.upto(10);
            let mut v: Vec<u16> = (0..n + 2).map(|_| alpha[rng.upto(alpha.len())]).collect();
            v.truncate(if n == 0 { 0 } else { n + 1 });
            v
        };
        let v1 = mk(&mut rng);
        let v2 = mk(&mut rng);
        // byte Length: usually 2*units, sometimes odd / shorter.
        let l1 = {
            let full = (v1.len().saturating_sub(1)) * 2;
            (match rng.upto(4) { 0 => full.saturating_sub(1), 1 => full / 2, _ => full }) as u16
        };
        let l2 = {
            let full = (v2.len().saturating_sub(1)) * 2;
            (match rng.upto(4) { 0 => full.saturating_sub(1), 1 => full / 2, _ => full }) as u16
        };
        let p1 = if v1.is_empty() { core::ptr::null_mut() } else { v1.as_ptr() as *mut u16 };
        let p2 = if v2.is_empty() { core::ptr::null_mut() } else { v2.as_ptr() as *mut u16 };
        // RtlCompare/Equal deref unconditionally → only feed NULL when
        // that side's Length is 0 (loop never runs).
        let l1 = if p1.is_null() { 0 } else { l1 };
        let l2 = if p2.is_null() { 0 } else { l2 };
        let s1 = mk_ustr(p1, l1, l1);
        let s2 = mk_ustr(p2, l2, l2);
        for ci in [0u8, 1] {
            unsafe {
                assert_eq!(
                    cabi::RtlCompareUnicodeString(&s1, &s2, ci),
                    ref_RtlCompareUnicodeString(&s1, &s2, ci),
                    "RtlCompareUnicodeString ci={ci} {v1:?}/{l1} {v2:?}/{l2}"
                );
                assert_eq!(
                    cabi::RtlEqualUnicodeString(&s1, &s2, ci),
                    ref_RtlEqualUnicodeString(&s1, &s2, ci),
                    "RtlEqualUnicodeString ci={ci}"
                );
                assert_eq!(
                    cabi::RtlPrefixUnicodeString(&s1, &s2, ci),
                    ref_RtlPrefixUnicodeString(&s1, &s2, ci),
                    "RtlPrefixUnicodeString ci={ci}"
                );
                assert_eq!(
                    cabi::RtlPrefixUnicodeString(&s2, &s1, ci),
                    ref_RtlPrefixUnicodeString(&s2, &s1, ci),
                    "RtlPrefixUnicodeString swapped ci={ci}"
                );
            }
        }
    }
    // explicit NULL-Buffer prefix (genuine `pc1 && pc2` else FALSE).
    let data = [0x0061u16, 0x0062, 0x0063];
    let s_ok = mk_ustr(data.as_ptr() as *mut u16, 6, 6);
    let s_nul = mk_ustr(core::ptr::null_mut(), 6, 6);
    let s_e = mk_ustr(core::ptr::null_mut(), 0, 0);
    unsafe {
        for (a, b) in [(&s_nul, &s_ok), (&s_ok, &s_nul), (&s_e, &s_ok), (&s_e, &s_e)] {
            for ci in [0u8, 1] {
                assert_eq!(
                    cabi::RtlPrefixUnicodeString(a, b, ci),
                    ref_RtlPrefixUnicodeString(a, b, ci),
                    "RtlPrefixUnicodeString NULL"
                );
            }
        }
    }
}

/// `RtlUpperString` — `Length = min(Src->Length, Dest->Max)`, that
/// many upcased bytes written (no NUL), `Dest->Length` set; the rest
/// of the dest buffer is left untouched.
#[test]
fn diff_rtl_upper_string() {
    let mut rng = Rng(0x5ca7_7e1d);
    for _ in 0..20_000 {
        let sn = rng.upto(20);
        let src: Vec<u8> = (0..sn).map(|_| rng.byte()).collect();
        let dmax = rng.upto(24);
        let sp = if src.is_empty() { core::ptr::null_mut() } else { src.as_ptr() as *mut u8 };
        let ss = mk_string(sp, src.len() as u16, src.len() as u16);
        let mut db_c = vec![0xCCu8; dmax.max(1)];
        let mut db_r = vec![0xCCu8; dmax.max(1)];
        let mut dc = mk_string(db_c.as_mut_ptr(), 0xAAAA, dmax as u16);
        let mut dr = mk_string(db_r.as_mut_ptr(), 0xAAAA, dmax as u16);
        unsafe {
            cabi::RtlUpperString(&mut dc, &ss);
            ref_RtlUpperString(&mut dr, &ss);
        }
        assert_eq!(dc.Length, dr.Length, "RtlUpperString Length sn={sn} dmax={dmax}");
        assert_eq!(db_c, db_r, "RtlUpperString buffer sn={sn} dmax={dmax}");
    }
}

/// `RtlHashUnicodeString` — X65599, ASCII-only CI fold, the
/// `String||HashValue` NULL guard and the invalid-algorithm path
/// (both ⇒ STATUS_INVALID_PARAMETER, `*HashValue` untouched).
#[test]
fn diff_rtl_hash() {
    let mut rng = Rng(0x4a51_0001);
    for _ in 0..30_000 {
        let n = rng.upto(24);
        let v: Vec<u16> = (0..n + 1)
            .map(|_| {
                let r = rng.upto(5);
                match r {
                    0 => b'a' as u16 + (rng.next() as u16 % 26),
                    1 => b'A' as u16 + (rng.next() as u16 % 26),
                    2 => 0x00E0 + (rng.next() as u16 % 0x20),
                    3 => rng.next() as u16,
                    _ => 0,
                }
            })
            .collect();
        let bytes = match rng.upto(4) {
            0 => (n * 2).saturating_sub(1),
            1 => n,
            _ => n * 2,
        } as u16;
        let s = mk_ustr(v.as_ptr() as *mut u16, bytes, bytes);
        for ci in [0u8, 1] {
            for alg in [0u32, 1, 2, 99] {
                let (mut hc, mut hr) = (0xDEAD_BEEFu32, 0xDEAD_BEEFu32);
                let cs;
                let rs;
                unsafe {
                    cs = cabi::RtlHashUnicodeString(&s, ci, alg, &mut hc);
                    rs = ref_RtlHashUnicodeString(&s, ci, alg, &mut hr);
                }
                assert_eq!(cs, rs, "RtlHashUnicodeString ret ci={ci} alg={alg}");
                if cs == 0 {
                    assert_eq!(hc, hr, "RtlHashUnicodeString value ci={ci} alg={alg} {v:?}");
                } else {
                    assert_eq!((hc, hr), (0xDEAD_BEEF, 0xDEAD_BEEF),
                        "RtlHashUnicodeString must not write on error");
                }
            }
        }
    }
    // NULL guards.
    unsafe {
        let mut h = 1u32;
        let dummy = mk_ustr(core::ptr::null_mut(), 0, 0);
        assert_eq!(
            cabi::RtlHashUnicodeString(core::ptr::null(), 0, 0, &mut h),
            ref_RtlHashUnicodeString(core::ptr::null(), 0, 0, &mut h),
            "RtlHashUnicodeString NULL string"
        );
        assert_eq!(
            cabi::RtlHashUnicodeString(&dummy, 0, 0, core::ptr::null_mut()),
            ref_RtlHashUnicodeString(&dummy, 0, 0, core::ptr::null_mut()),
            "RtlHashUnicodeString NULL hashvalue"
        );
    }
}
