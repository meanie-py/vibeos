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
