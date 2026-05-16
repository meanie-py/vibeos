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
    // Wave RTL-3 converter core
    fn ref_RtlMultiByteToUnicodeN(u: *mut u16, us: u32, rs: *mut u32, mb: *const u8, mbs: u32) -> i32;
    fn ref_RtlOemToUnicodeN(u: *mut u16, us: u32, rs: *mut u32, mb: *const u8, mbs: u32) -> i32;
    fn ref_RtlUnicodeToMultiByteN(mb: *mut u8, mbs: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    fn ref_RtlUnicodeToOemN(o: *mut u8, os: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    fn ref_RtlUpcaseUnicodeToMultiByteN(mb: *mut u8, mbs: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    fn ref_RtlUpcaseUnicodeToOemN(o: *mut u8, os: u32, rs: *mut u32, u: *const u16, us: u32) -> i32;
    fn ref_RtlMultiByteToUnicodeSize(us: *mut u32, mb: *const u8, mbs: u32) -> i32;
    fn ref_RtlUnicodeToMultiByteSize(mbs: *mut u32, u: *const u16, us: u32) -> i32;
    // Wave RTL-3 slice 2 wrappers
    fn ref_RtlAnsiCharToUnicodeChar(p: *mut *mut u8) -> u16;
    fn ref_RtlxAnsiStringToUnicodeSize(s: *const STRING) -> u32;
    fn ref_RtlxUnicodeStringToAnsiSize(s: *const UNICODE_STRING) -> u32;
    fn ref_RtlxUnicodeStringToOemSize(s: *const UNICODE_STRING) -> u32;
    fn ref_RtlxOemStringToUnicodeSize(s: *const STRING) -> u32;
    fn ref_RtlAnsiStringToUnicodeString(d: *mut UNICODE_STRING, s: *const STRING, a: u8) -> i32;
    fn ref_RtlOemStringToUnicodeString(d: *mut UNICODE_STRING, s: *const STRING, a: u8) -> i32;
    fn ref_RtlOemStringToCountedUnicodeString(d: *mut UNICODE_STRING, s: *const STRING, a: u8) -> i32;
    fn ref_RtlUnicodeStringToAnsiString(d: *mut STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlUnicodeStringToOemString(d: *mut STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlUnicodeStringToCountedOemString(d: *mut STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlUpcaseUnicodeString(d: *mut UNICODE_STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlUpcaseUnicodeStringToAnsiString(d: *mut STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlUpcaseUnicodeStringToCountedOemString(d: *mut STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlUpcaseUnicodeStringToOemString(d: *mut STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    // Wave RTL-3b
    fn ref_RtlDowncaseUnicodeChar(c: u16) -> u16;
    fn ref_RtlDowncaseUnicodeString(d: *mut UNICODE_STRING, s: *const UNICODE_STRING, a: u8) -> i32;
    fn ref_RtlGetDefaultCodePage(a: *mut u16, o: *mut u16);
    // Wave RTL-4a
    fn ref_RtlCharToInteger(s: *const u8, base: u32, v: *mut u32) -> i32;
    fn ref_RtlUnicodeStringToInteger(s: *const UNICODE_STRING, base: u32, v: *mut u32) -> i32;
    fn ref_RtlIntegerToUnicode(v: u32, base: u32, len: u32, s: *mut u16) -> i32;
    fn ref_RtlCopyString(d: *mut STRING, s: *const STRING);
    fn ref_RtlCopyUnicodeString(d: *mut UNICODE_STRING, s: *const UNICODE_STRING);
    fn ref_RtlAppendStringToString(d: *mut STRING, s: *const STRING) -> i32;
    fn ref_RtlAppendUnicodeStringToString(d: *mut UNICODE_STRING, s: *const UNICODE_STRING) -> i32;
    fn ref_RtlAppendUnicodeToString(d: *mut UNICODE_STRING, s: *const u16) -> i32;
    fn ref_RtlAppendAsciizToString(d: *mut STRING, s: *const u8) -> i32;
    fn ref_RtlCompareUnicodeStrings(
        s1: *const u16,
        l1: usize,
        s2: *const u16,
        l2: usize,
        ci: u8,
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

// ── Wave RTL-3: single-byte CP1252 converter core ──────────────────

const SPECIAL_W: &[u16] = &[
    0x0000, 0x0041, 0x0061, 0x007A, 0x00B5, 0x00C0, 0x00E0, 0x00FE,
    0x00FF, 0x0152, 0x0153, 0x017D, 0x017E, 0x0130, 0x20AC, 0x0081,
    0x008D, 0xABCD, 0xFFFF,
];

/// `*ToUnicodeN` (src bytes → dst WCHAR). Fuzz dst capacity vs input,
/// odd `UnicodeSize`, NULL `ResultSize`. Compares ret + ResultSize +
/// the whole dst buffer (untouched tail must match too).
#[test]
fn diff_rtl_mb_to_uni() {
    let mut rng = Rng(0x3b_1252_01);
    type F = unsafe extern "system" fn(*mut u16, u32, *mut u32, *const u8, u32) -> i32;
    let pairs: [(&str, F, unsafe extern "C" fn(*mut u16, u32, *mut u32, *const u8, u32) -> i32); 2] = [
        ("RtlMultiByteToUnicodeN", cabi::RtlMultiByteToUnicodeN, ref_RtlMultiByteToUnicodeN),
        ("RtlOemToUnicodeN", cabi::RtlOemToUnicodeN, ref_RtlOemToUnicodeN),
    ];
    for _ in 0..15_000 {
        let m = rng.upto(24);
        let src: Vec<u8> = (0..m).map(|_| rng.byte()).collect();
        let mb_size = (match rng.upto(4) { 0 => 0, 1 => m / 2, 2 => m.saturating_sub(1), _ => m }) as u32;
        let dcap = rng.upto(20);
        let usize_bytes = (dcap * 2 + rng.upto(2)) as u32; // sometimes odd
        for (name, cf, rf) in pairs {
            let sp = if src.is_empty() { core::ptr::null() } else { src.as_ptr() };
            let mut dc = vec![0xBEEFu16; dcap.max(1)];
            let mut dr = vec![0xBEEFu16; dcap.max(1)];
            let null_rs = rng.upto(3) == 0;
            let (mut rc, mut rr) = (0u32, 0u32);
            let (pc, pr) = if null_rs {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut rc as *mut u32, &mut rr as *mut u32)
            };
            let (sc, sr);
            unsafe {
                sc = cf(dc.as_mut_ptr(), usize_bytes, pc, sp, mb_size);
                sr = rf(dr.as_mut_ptr(), usize_bytes, pr, sp, mb_size);
            }
            assert_eq!(sc, sr, "{name} ret");
            assert_eq!(rc, rr, "{name} ResultSize");
            assert_eq!(dc, dr, "{name} dst m={m} mbs={mb_size} cap={dcap} us={usize_bytes}");
        }
    }
}

/// `Unicode*To{MultiByte,Oem}N` (+ Upcase variants): src WCHAR → dst
/// bytes. Pins the `RtlUnicodeToOemN` BUFFER_OVERFLOW asymmetry and
/// the upcase fold.
#[test]
fn diff_rtl_uni_to_mb() {
    let mut rng = Rng(0x3b_1252_02);
    type F = unsafe extern "system" fn(*mut u8, u32, *mut u32, *const u16, u32) -> i32;
    type R = unsafe extern "C" fn(*mut u8, u32, *mut u32, *const u16, u32) -> i32;
    let pairs: [(&str, F, R); 4] = [
        ("RtlUnicodeToMultiByteN", cabi::RtlUnicodeToMultiByteN, ref_RtlUnicodeToMultiByteN),
        ("RtlUnicodeToOemN", cabi::RtlUnicodeToOemN, ref_RtlUnicodeToOemN),
        ("RtlUpcaseUnicodeToMultiByteN", cabi::RtlUpcaseUnicodeToMultiByteN, ref_RtlUpcaseUnicodeToMultiByteN),
        ("RtlUpcaseUnicodeToOemN", cabi::RtlUpcaseUnicodeToOemN, ref_RtlUpcaseUnicodeToOemN),
    ];
    for _ in 0..15_000 {
        let u = rng.upto(20);
        let src: Vec<u16> = (0..u + 1).map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())]).collect();
        let us_bytes = (match rng.upto(4) {
            0 => 0,
            1 => u, // truncated, maybe odd
            2 => (u * 2).saturating_sub(rng.upto(2)),
            _ => u * 2,
        }) as u32;
        let dcap = rng.upto(22);
        for (name, cf, rf) in pairs {
            let sp = src.as_ptr();
            let mut dc = vec![0xCCu8; dcap.max(1)];
            let mut dr = vec![0xCCu8; dcap.max(1)];
            let null_rs = rng.upto(3) == 0;
            let (mut rc, mut rr) = (0u32, 0u32);
            let (pc, pr) = if null_rs {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut rc as *mut u32, &mut rr as *mut u32)
            };
            let (sc, sr);
            unsafe {
                sc = cf(dc.as_mut_ptr(), dcap as u32, pc, sp, us_bytes);
                sr = rf(dr.as_mut_ptr(), dcap as u32, pr, sp, us_bytes);
            }
            assert_eq!(sc, sr, "{name} ret u={u} us={us_bytes} cap={dcap}");
            assert_eq!(rc, rr, "{name} ResultSize u={u} us={us_bytes} cap={dcap}");
            assert_eq!(dc, dr, "{name} dst {src:?} us={us_bytes} cap={dcap}");
        }
    }
}

/// The two `*Size` queries (single-byte: pure arithmetic, but pin the
/// `*2` / `/2` direction and that they ignore the string).
#[test]
fn diff_rtl_conv_size() {
    let mut rng = Rng(0x3b_1252_03);
    for _ in 0..5_000 {
        let n = rng.upto(0x10000) as u32;
        let (mut a, mut b) = (0u32, 0u32);
        unsafe {
            assert_eq!(
                cabi::RtlMultiByteToUnicodeSize(&mut a, core::ptr::null(), n),
                ref_RtlMultiByteToUnicodeSize(&mut b, core::ptr::null(), n),
                "RtlMultiByteToUnicodeSize ret"
            );
            assert_eq!(a, b, "RtlMultiByteToUnicodeSize n={n}");
            let (mut c, mut d) = (0u32, 0u32);
            assert_eq!(
                cabi::RtlUnicodeToMultiByteSize(&mut c, core::ptr::null(), n),
                ref_RtlUnicodeToMultiByteSize(&mut d, core::ptr::null(), n),
                "RtlUnicodeToMultiByteSize ret"
            );
            assert_eq!(c, d, "RtlUnicodeToMultiByteSize n={n}");
        }
    }
}

// ── Wave RTL-3 slice 2: unicode.c ANSI/OEM/Upcase wrappers ─────────

#[test]
fn diff_rtl_ansichar_to_unicode() {
    let mut rng = Rng(0x3b_5202_aa);
    for _ in 0..3000 {
        let n = 1 + rng.upto(8);
        let buf: Vec<u8> = (0..n).map(|_| rng.byte()).collect();
        let mut pc = buf.as_ptr() as *mut u8;
        let mut pr = buf.as_ptr() as *mut u8;
        unsafe {
            let rc = cabi::RtlAnsiCharToUnicodeChar(&mut pc);
            let rr = ref_RtlAnsiCharToUnicodeChar(&mut pr);
            assert_eq!(rc, rr, "RtlAnsiCharToUnicodeChar val b={:#x}", buf[0]);
            assert_eq!(pc.offset_from(buf.as_ptr() as *mut u8),
                       pr.offset_from(buf.as_ptr() as *mut u8), "advance");
        }
    }
}

#[test]
fn diff_rtl_size_helpers() {
    let mut rng = Rng(0x3b_5202_bb);
    for _ in 0..4000 {
        let l = (rng.upto(0x10000)) as u16;
        let s = STRING { Length: l, MaximumLength: l, Buffer: 1 as *mut u8 };
        let u = UNICODE_STRING { Length: l, MaximumLength: l, Buffer: 1 as *mut u16 };
        unsafe {
            assert_eq!(cabi::RtlxAnsiStringToUnicodeSize(&s), ref_RtlxAnsiStringToUnicodeSize(&s), "xAnsi l={l}");
            assert_eq!(cabi::RtlxOemStringToUnicodeSize(&s), ref_RtlxOemStringToUnicodeSize(&s), "xOem l={l}");
            assert_eq!(cabi::RtlxUnicodeStringToAnsiSize(&u), ref_RtlxUnicodeStringToAnsiSize(&u), "xUniAnsi l={l}");
            assert_eq!(cabi::RtlxUnicodeStringToOemSize(&u), ref_RtlxUnicodeStringToOemSize(&u), "xUniOem l={l}");
        }
    }
}

type FbU = unsafe extern "system" fn(*mut UNICODE_STRING, *const STRING, u8) -> i32;
type RbU = unsafe extern "C" fn(*mut UNICODE_STRING, *const STRING, u8) -> i32;
type FuB = unsafe extern "system" fn(*mut STRING, *const UNICODE_STRING, u8) -> i32;
type RuB = unsafe extern "C" fn(*mut STRING, *const UNICODE_STRING, u8) -> i32;

/// bytes(STRING) → UNICODE_STRING wrappers.
#[test]
fn diff_rtl_str_to_uni() {
    let mut rng = Rng(0x3b_5202_01);
    let fns: [(&str, FbU, RbU); 3] = [
        ("RtlAnsiStringToUnicodeString", cabi::RtlAnsiStringToUnicodeString, ref_RtlAnsiStringToUnicodeString),
        ("RtlOemStringToUnicodeString", cabi::RtlOemStringToUnicodeString, ref_RtlOemStringToUnicodeString),
        ("RtlOemStringToCountedUnicodeString", cabi::RtlOemStringToCountedUnicodeString, ref_RtlOemStringToCountedUnicodeString),
    ];
    for _ in 0..12_000 {
        let sl = rng.upto(16);
        let sb: Vec<u8> = (0..sl.max(1)).map(|_| 1 + rng.byte() % 200).collect();
        // Backing is fixed-large; MaximumLength varies independently
        // (incl. too-small) to exercise overflow + the genuine
        // ignores-MaximumLength fall-through quirk without OOB.
        let ml = (match rng.upto(4) { 0 => 0, 1 => sl, 2 => sl * 2 + 2, _ => 60 }) as u16;
        for (name, cf, rf) in fns {
            for a in [0u8, 1] {
                let ss = STRING { Length: sl as u16, MaximumLength: sl as u16, Buffer: sb.as_ptr() as *mut u8 };
                let mut bc = vec![0xCCCCu16; 64];
                let mut br = vec![0xCCCCu16; 64];
                let mut dc = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: bc.as_mut_ptr() };
                let mut dr = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: br.as_mut_ptr() };
                let (sc, sr);
                unsafe {
                    sc = cf(&mut dc, &ss, a);
                    sr = rf(&mut dr, &ss, a);
                }
                assert_eq!(sc, sr, "{name} ret sl={sl} ml={ml} a={a}");
                assert_eq!((dc.Length, dc.MaximumLength), (dr.Length, dr.MaximumLength),
                    "{name} fields sl={sl} ml={ml} a={a}");
                assert_eq!(bc, br, "{name} buf sl={sl} ml={ml} a={a}");
            }
        }
    }
}

/// UNICODE_STRING → bytes(STRING) wrappers (incl. the Upcase + Counted
/// + DidWork/UNMAPPABLE + partial-copy variants).
#[test]
fn diff_rtl_uni_to_str() {
    let mut rng = Rng(0x3b_5202_02);
    let fns: [(&str, FuB, RuB); 6] = [
        ("RtlUnicodeStringToAnsiString", cabi::RtlUnicodeStringToAnsiString, ref_RtlUnicodeStringToAnsiString),
        ("RtlUnicodeStringToOemString", cabi::RtlUnicodeStringToOemString, ref_RtlUnicodeStringToOemString),
        ("RtlUnicodeStringToCountedOemString", cabi::RtlUnicodeStringToCountedOemString, ref_RtlUnicodeStringToCountedOemString),
        ("RtlUpcaseUnicodeStringToAnsiString", cabi::RtlUpcaseUnicodeStringToAnsiString, ref_RtlUpcaseUnicodeStringToAnsiString),
        ("RtlUpcaseUnicodeStringToCountedOemString", cabi::RtlUpcaseUnicodeStringToCountedOemString, ref_RtlUpcaseUnicodeStringToCountedOemString),
        ("RtlUpcaseUnicodeStringToOemString", cabi::RtlUpcaseUnicodeStringToOemString, ref_RtlUpcaseUnicodeStringToOemString),
    ];
    for _ in 0..12_000 {
        let su = rng.upto(14);
        let sv: Vec<u16> = (0..su.max(1)).map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())]).collect();
        let slen = (su * 2) as u16;
        let ml = (match rng.upto(4) { 0 => 0, 1 => su, 2 => su + 1, _ => 60 }) as u16;
        for (name, cf, rf) in fns {
            for a in [0u8, 1] {
                let ss = UNICODE_STRING { Length: slen, MaximumLength: slen, Buffer: sv.as_ptr() as *mut u16 };
                let mut bc = vec![0xCCu8; 128];
                let mut br = vec![0xCCu8; 128];
                let mut dc = STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: bc.as_mut_ptr() };
                let mut dr = STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: br.as_mut_ptr() };
                let (sc, sr);
                unsafe {
                    sc = cf(&mut dc, &ss, a);
                    sr = rf(&mut dr, &ss, a);
                }
                assert_eq!(sc, sr, "{name} ret su={su} ml={ml} a={a} {sv:?}");
                assert_eq!((dc.Length, dc.MaximumLength), (dr.Length, dr.MaximumLength),
                    "{name} fields su={su} ml={ml} a={a}");
                assert_eq!(bc, br, "{name} buf su={su} ml={ml} a={a} {sv:?}");
            }
        }
    }
}

/// `RtlUpcaseUnicodeString` (UNICODE_STRING → UNICODE_STRING).
#[test]
fn diff_rtl_upcase_unicode_string() {
    let mut rng = Rng(0x3b_5202_03);
    for _ in 0..12_000 {
        let su = rng.upto(16);
        let sv: Vec<u16> = (0..su.max(1)).map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())]).collect();
        let slen = (su * 2) as u16;
        let ml = (match rng.upto(4) { 0 => 0, 1 => slen as usize, 2 => su, _ => 60 }) as u16;
        for a in [0u8, 1] {
            let ss = UNICODE_STRING { Length: slen, MaximumLength: slen, Buffer: sv.as_ptr() as *mut u16 };
            let mut bc = vec![0xCCCCu16; 64];
            let mut br = vec![0xCCCCu16; 64];
            let mut dc = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: bc.as_mut_ptr() };
            let mut dr = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: br.as_mut_ptr() };
            let (sc, sr);
            unsafe {
                sc = cabi::RtlUpcaseUnicodeString(&mut dc, &ss, a);
                sr = ref_RtlUpcaseUnicodeString(&mut dr, &ss, a);
            }
            assert_eq!(sc, sr, "RtlUpcaseUnicodeString ret su={su} ml={ml} a={a}");
            assert_eq!((dc.Length, dc.MaximumLength), (dr.Length, dr.MaximumLength),
                "RtlUpcaseUnicodeString fields su={su} ml={ml} a={a}");
            assert_eq!(bc, br, "RtlUpcaseUnicodeString buf su={su} ml={ml} a={a}");
        }
    }
}

// ── Wave RTL-3b: downcase family + default code page ──────────────

#[test]
fn diff_rtl_downcase_char() {
    for w in 0u32..=0xFFFF {
        let w = w as u16;
        unsafe {
            assert_eq!(
                cabi::RtlDowncaseUnicodeChar(w),
                ref_RtlDowncaseUnicodeChar(w),
                "RtlDowncaseUnicodeChar {w:#06x}"
            );
        }
    }
}

#[test]
fn diff_rtl_default_codepage() {
    let (mut a1, mut o1) = (0u16, 0u16);
    let (mut a2, mut o2) = (0u16, 0u16);
    unsafe {
        cabi::RtlGetDefaultCodePage(&mut a1, &mut o1);
        ref_RtlGetDefaultCodePage(&mut a2, &mut o2);
    }
    assert_eq!((a1, o1), (a2, o2), "RtlGetDefaultCodePage");
}

#[test]
fn diff_rtl_downcase_unicode_string() {
    let mut rng = Rng(0x3b_5203_01);
    for _ in 0..12_000 {
        let su = rng.upto(16);
        let sv: Vec<u16> = (0..su.max(1)).map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())]).collect();
        let slen = (su * 2) as u16;
        let ml = (match rng.upto(4) { 0 => 0, 1 => slen as usize, 2 => su, _ => 60 }) as u16;
        for a in [0u8, 1] {
            let ss = UNICODE_STRING { Length: slen, MaximumLength: slen, Buffer: sv.as_ptr() as *mut u16 };
            let mut bc = vec![0xCCCCu16; 64];
            let mut br = vec![0xCCCCu16; 64];
            let mut dc = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: bc.as_mut_ptr() };
            let mut dr = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: br.as_mut_ptr() };
            let (sc, sr);
            unsafe {
                sc = cabi::RtlDowncaseUnicodeString(&mut dc, &ss, a);
                sr = ref_RtlDowncaseUnicodeString(&mut dr, &ss, a);
            }
            assert_eq!(sc, sr, "RtlDowncaseUnicodeString ret su={su} ml={ml} a={a}");
            assert_eq!((dc.Length, dc.MaximumLength), (dr.Length, dr.MaximumLength),
                "RtlDowncaseUnicodeString fields su={su} ml={ml} a={a}");
            assert_eq!(bc, br, "RtlDowncaseUnicodeString buf su={su} ml={ml} a={a}");
        }
    }
}

// ── Wave RTL-4a: integer ⇄ string + copy/append ───────────────────

/// Alphabet for the string→integer parsers: digits, hex/base letters,
/// sign, base prefixes, whitespace + control + high bytes (the
/// signed-`CHAR` "everything ≥0x80 is whitespace" quirk).
const INT_ALPHA: &[u8] = &[
    b'0', b'1', b'7', b'9', b'A', b'F', b'Z', b'a', b'f', b'x', b'o',
    b'b', b'z', b'+', b'-', b' ', b'\t', 0x01, 0x1f, 0x7f, 0x80, 0xC0,
    0xff,
];
const BASES: &[u32] = &[0, 2, 8, 10, 16, 1, 3, 7, 9, 255, 0xFFFF_FFFF];

#[test]
fn diff_rtl_char_to_integer() {
    let mut rng = Rng(0x4a_0001_01);
    for _ in 0..40_000 {
        let n = rng.upto(14);
        let mut s: Vec<u8> =
            (0..n).map(|_| INT_ALPHA[rng.upto(INT_ALPHA.len())]).collect();
        s.push(0); // NUL terminator
        let base = BASES[rng.upto(BASES.len())];
        for null_v in [false, true] {
            let (mut vc, mut vr) = (0xDEAD_BEEFu32, 0xDEAD_BEEFu32);
            let (pc, pr) = if null_v {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut vc as *mut u32, &mut vr as *mut u32)
            };
            let (sc, sr);
            unsafe {
                sc = cabi::RtlCharToInteger(s.as_ptr(), base, pc);
                sr = ref_RtlCharToInteger(s.as_ptr(), base, pr);
            }
            assert_eq!(sc, sr, "RtlCharToInteger ret base={base} {s:?}");
            assert_eq!(vc, vr, "RtlCharToInteger *value base={base} {s:?}");
        }
    }
}

#[test]
fn diff_rtl_unicode_string_to_integer() {
    let mut rng = Rng(0x4a_0001_02);
    let walpha: Vec<u16> = {
        let mut v: Vec<u16> = INT_ALPHA.iter().map(|&b| b as u16).collect();
        v.extend_from_slice(&[0x0000, 0x00B2, 0x0660, 0xFF10]);
        v
    };
    for _ in 0..40_000 {
        let units = rng.upto(14);
        let buf: Vec<u16> =
            (0..units.max(1)).map(|_| walpha[rng.upto(walpha.len())]).collect();
        // byte Length: usually units*2, sometimes shorter / odd.
        let blen = (match rng.upto(4) {
            0 => 0,
            1 => units,
            2 => (units * 2).saturating_sub(1),
            _ => units * 2,
        }) as u16;
        let base = BASES[rng.upto(BASES.len())];
        let us = mk_ustr(buf.as_ptr() as *mut u16, blen, blen);
        for null_v in [false, true] {
            let (mut vc, mut vr) = (0xDEAD_BEEFu32, 0xDEAD_BEEFu32);
            let (pc, pr) = if null_v {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut vc as *mut u32, &mut vr as *mut u32)
            };
            let (sc, sr);
            unsafe {
                sc = cabi::RtlUnicodeStringToInteger(&us, base, pc);
                sr = ref_RtlUnicodeStringToInteger(&us, base, pr);
            }
            assert_eq!(sc, sr, "RtlUnicodeStringToInteger ret base={base}");
            assert_eq!(vc, vr, "RtlUnicodeStringToInteger *value base={base} {buf:?}");
        }
    }
}

#[test]
fn diff_rtl_integer_to_unicode() {
    let mut rng = Rng(0x4a_0001_03);
    let vals: &[u32] = &[0, 1, 7, 9, 10, 15, 16, 255, 1000, 0x7FFF_FFFF, 0xFFFF_FFFF];
    let lens: &[u32] = &[0, 2, 4, 8, 16, 20, 64, 200, 0xFFFF_FFFF];
    for _ in 0..30_000 {
        let value = if rng.upto(3) == 0 {
            vals[rng.upto(vals.len())]
        } else {
            rng.next() as u32
        };
        let base = BASES[rng.upto(BASES.len())];
        let length = lens[rng.upto(lens.len())];
        // 64-unit backing absorbs the genuine count+1 (digits+NUL)
        // write even when `length` lies (genuine UB excluded).
        let mut bc = vec![0xAAAAu16; 64];
        let mut br = vec![0xAAAAu16; 64];
        let (sc, sr);
        unsafe {
            sc = cabi::RtlIntegerToUnicode(value, base, length, bc.as_mut_ptr());
            sr = ref_RtlIntegerToUnicode(value, base, length, br.as_mut_ptr());
        }
        assert_eq!(sc, sr, "RtlIntegerToUnicode ret v={value} base={base} len={length}");
        assert_eq!(bc, br, "RtlIntegerToUnicode buf v={value} base={base} len={length}");
    }
}

#[test]
fn diff_rtl_copy_string() {
    let mut rng = Rng(0x4a_0001_04);
    for _ in 0..20_000 {
        let sl = rng.upto(20);
        let sb: Vec<u8> = (0..sl.max(1)).map(|_| rng.byte()).collect();
        let smax = rng.upto(24) as u16;
        for null_src in [false, true] {
            let ss = STRING {
                Length: sl as u16,
                MaximumLength: smax,
                Buffer: sb.as_ptr() as *mut u8,
            };
            let sp: *const STRING =
                if null_src { core::ptr::null() } else { &ss };
            let dmax = rng.upto(24);
            let mut db_c = vec![0xCCu8; dmax.max(1)];
            let mut db_r = vec![0xCCu8; dmax.max(1)];
            let mut dc = mk_string(db_c.as_mut_ptr(), 0xAAAA, dmax as u16);
            let mut dr = mk_string(db_r.as_mut_ptr(), 0xAAAA, dmax as u16);
            unsafe {
                cabi::RtlCopyString(&mut dc, sp);
                ref_RtlCopyString(&mut dr, sp);
            }
            assert_eq!(dc.Length, dr.Length, "RtlCopyString Length");
            assert_eq!(db_c, db_r, "RtlCopyString buf sl={sl} dmax={dmax}");
        }
    }
}

#[test]
fn diff_rtl_copy_unicode_string() {
    let mut rng = Rng(0x4a_0001_05);
    for _ in 0..20_000 {
        let su = rng.upto(16);
        let sv: Vec<u16> =
            (0..su.max(1)).map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())]).collect();
        let slen = (su * 2) as u16;
        let smax = (rng.upto(18) * 2) as u16;
        for null_src in [false, true] {
            let ss = UNICODE_STRING {
                Length: slen,
                MaximumLength: smax,
                Buffer: sv.as_ptr() as *mut u16,
            };
            let sp: *const UNICODE_STRING =
                if null_src { core::ptr::null() } else { &ss };
            let dmax_u = rng.upto(20);
            let mut db_c = vec![0xCCCCu16; dmax_u.max(1)];
            let mut db_r = vec![0xCCCCu16; dmax_u.max(1)];
            let mut dc = mk_ustr(db_c.as_mut_ptr(), 0xAAAA, (dmax_u * 2) as u16);
            let mut dr = mk_ustr(db_r.as_mut_ptr(), 0xAAAA, (dmax_u * 2) as u16);
            unsafe {
                cabi::RtlCopyUnicodeString(&mut dc, sp);
                ref_RtlCopyUnicodeString(&mut dr, sp);
            }
            assert_eq!(dc.Length, dr.Length, "RtlCopyUnicodeString Length");
            assert_eq!(db_c, db_r, "RtlCopyUnicodeString buf su={su} dmax={dmax_u}");
        }
    }
}

#[test]
fn diff_rtl_append_string() {
    let mut rng = Rng(0x4a_0001_06);
    for _ in 0..20_000 {
        let dl = rng.upto(12);
        let sl = rng.upto(12);
        let total = dl + sl + rng.upto(6);
        let dmax = total.min(0xFFFF);
        let sb: Vec<u8> = (0..sl.max(1)).map(|_| rng.byte() | 1).collect();
        let asz: Vec<u8> = {
            let mut v: Vec<u8> = (0..sl).map(|_| rng.byte() | 1).collect();
            v.push(0);
            v
        };
        // backing is generously sized; Length/Max drive the contract.
        let mut db_c = vec![0xCCu8; total.max(1) + 4];
        let mut db_r = vec![0xCCu8; total.max(1) + 4];
        for i in 0..dl {
            db_c[i] = 0x40 + (i as u8 & 0xf);
            db_r[i] = db_c[i];
        }
        let ss = mk_string(sb.as_ptr() as *mut u8, sl as u16, sl as u16);
        let mut dc = mk_string(db_c.as_mut_ptr(), dl as u16, dmax as u16);
        let mut dr = mk_string(db_r.as_mut_ptr(), dl as u16, dmax as u16);
        let (sc, sr);
        unsafe {
            sc = cabi::RtlAppendStringToString(&mut dc, &ss);
            sr = ref_RtlAppendStringToString(&mut dr, &ss);
        }
        assert_eq!(sc, sr, "RtlAppendStringToString ret dl={dl} sl={sl} dmax={dmax}");
        assert_eq!(dc.Length, dr.Length, "RtlAppendStringToString Length");
        assert_eq!(db_c, db_r, "RtlAppendStringToString buf");

        // RtlAppendAsciizToString reuses the same dest shape.
        let mut eb_c = vec![0xCCu8; total.max(1) + 4];
        let mut eb_r = vec![0xCCu8; total.max(1) + 4];
        for i in 0..dl {
            eb_c[i] = 0x40 + (i as u8 & 0xf);
            eb_r[i] = eb_c[i];
        }
        let mut ec = mk_string(eb_c.as_mut_ptr(), dl as u16, dmax as u16);
        let mut er = mk_string(eb_r.as_mut_ptr(), dl as u16, dmax as u16);
        for null_src in [false, true] {
            let asp = if null_src { core::ptr::null() } else { asz.as_ptr() };
            let (ac, ar);
            unsafe {
                ac = cabi::RtlAppendAsciizToString(&mut ec, asp);
                ar = ref_RtlAppendAsciizToString(&mut er, asp);
            }
            assert_eq!(ac, ar, "RtlAppendAsciizToString ret");
            assert_eq!(ec.Length, er.Length, "RtlAppendAsciizToString Length");
            assert_eq!(eb_c, eb_r, "RtlAppendAsciizToString buf");
        }
    }
}

#[test]
fn diff_rtl_append_unicode() {
    let mut rng = Rng(0x4a_0001_07);
    for _ in 0..20_000 {
        let du = rng.upto(10);
        let suu = rng.upto(10);
        let total_u = du + suu + rng.upto(4);
        let dmax = (total_u * 2).min(0xFFFE) as u16;
        let sv: Vec<u16> =
            (0..suu.max(1)).map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())]).collect();
        let wsz: Vec<u16> = {
            let mut v: Vec<u16> = (0..suu)
                .map(|_| 1 + (rng.next() as u16 % 0x300))
                .collect();
            v.push(0);
            v
        };
        let mut db_c = vec![0xCCCCu16; total_u.max(1) + 4];
        let mut db_r = vec![0xCCCCu16; total_u.max(1) + 4];
        for i in 0..du {
            db_c[i] = 0x41 + (i as u16 & 0xf);
            db_r[i] = db_c[i];
        }
        let ss = mk_ustr(sv.as_ptr() as *mut u16, (suu * 2) as u16, (suu * 2) as u16);
        let mut dc = mk_ustr(db_c.as_mut_ptr(), (du * 2) as u16, dmax);
        let mut dr = mk_ustr(db_r.as_mut_ptr(), (du * 2) as u16, dmax);
        let (sc, sr);
        unsafe {
            sc = cabi::RtlAppendUnicodeStringToString(&mut dc, &ss);
            sr = ref_RtlAppendUnicodeStringToString(&mut dr, &ss);
        }
        assert_eq!(sc, sr, "RtlAppendUnicodeStringToString ret");
        assert_eq!(dc.Length, dr.Length, "RtlAppendUnicodeStringToString Length");
        assert_eq!(db_c, db_r, "RtlAppendUnicodeStringToString buf");

        let mut eb_c = vec![0xCCCCu16; total_u.max(1) + 4];
        let mut eb_r = vec![0xCCCCu16; total_u.max(1) + 4];
        for i in 0..du {
            eb_c[i] = 0x41 + (i as u16 & 0xf);
            eb_r[i] = eb_c[i];
        }
        let mut ec = mk_ustr(eb_c.as_mut_ptr(), (du * 2) as u16, dmax);
        let mut er = mk_ustr(eb_r.as_mut_ptr(), (du * 2) as u16, dmax);
        for null_src in [false, true] {
            let wsp = if null_src { core::ptr::null() } else { wsz.as_ptr() };
            let (ac, ar);
            unsafe {
                ac = cabi::RtlAppendUnicodeToString(&mut ec, wsp);
                ar = ref_RtlAppendUnicodeToString(&mut er, wsp);
            }
            assert_eq!(ac, ar, "RtlAppendUnicodeToString ret");
            assert_eq!(ec.Length, er.Length, "RtlAppendUnicodeToString Length");
            assert_eq!(eb_c, eb_r, "RtlAppendUnicodeToString buf");
        }
    }
}

#[test]
fn diff_rtl_compare_unicode_strings() {
    let mut rng = Rng(0x4a_0001_08);
    for _ in 0..40_000 {
        let mk = |rng: &mut Rng| -> Vec<u16> {
            (0..rng.upto(12).max(1))
                .map(|_| SPECIAL_W[rng.upto(SPECIAL_W.len())])
                .collect()
        };
        let v1 = mk(&mut rng);
        let v2 = mk(&mut rng);
        // unit-count lengths, independently random but ≤ backing.
        let l1 = rng.upto(v1.len() + 1);
        let l2 = rng.upto(v2.len() + 1);
        let p1 = if l1 == 0 { core::ptr::null() } else { v1.as_ptr() };
        let p2 = if l2 == 0 { core::ptr::null() } else { v2.as_ptr() };
        for ci in [0u8, 1] {
            let (rc, rr);
            unsafe {
                rc = cabi::RtlCompareUnicodeStrings(p1, l1, p2, l2, ci);
                rr = ref_RtlCompareUnicodeStrings(p1, l1, p2, l2, ci);
            }
            assert_eq!(
                rc, rr,
                "RtlCompareUnicodeStrings ci={ci} {v1:?}/{l1} {v2:?}/{l2}"
            );
        }
    }
}
