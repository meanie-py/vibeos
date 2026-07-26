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

use rtl_strings::cabi::{
    self, GenerateNameContext, GUID, RTL_BUFFER, STRING, UNICODE_STRING,
};

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
    // Wave RTL-4b slice 1
    fn ref_RtlValidateUnicodeString(flags: u32, s: *const UNICODE_STRING) -> i32;
    fn ref_RtlFindCharInUnicodeString(
        flags: u32,
        search: *const UNICODE_STRING,
        m: *const UNICODE_STRING,
        pos: *mut u16,
    ) -> i32;
    fn ref_RtlEraseUnicodeString(s: *mut UNICODE_STRING);
    fn ref_RtlFreeAnsiString(s: *mut STRING);
    fn ref_RtlFreeOemString(s: *mut STRING);
    fn ref_RtlFreeUnicodeString(s: *mut UNICODE_STRING);
    fn ref_RtlGUIDFromString(s: *const UNICODE_STRING, g: *mut GUID) -> i32;
    // Wave RTL-4b slice 2
    fn ref_RtlIsTextUnicode(buf: *const u8, len: i32, pf: *mut i32) -> u8;
    // Wave RTL-4b slice 3
    fn ref_RtlCreateUnicodeString(d: *mut UNICODE_STRING, s: *const u16) -> u8;
    fn ref_RtlCreateUnicodeStringFromAsciiz(d: *mut UNICODE_STRING, s: *const u8) -> u8;
    fn ref_RtlDuplicateUnicodeString(
        f: u32,
        s: *const UNICODE_STRING,
        d: *mut UNICODE_STRING,
    ) -> i32;
    fn ref_RtlStringFromGUID(g: *const GUID, s: *mut UNICODE_STRING) -> i32;
    fn ref_RtlpEnsureBufferSize(f: u32, b: *mut RTL_BUFFER, r: usize) -> i32;
    fn ref_RtlMultiAppendUnicodeStringBuffer(
        sb: *mut u8,
        n: u32,
        a: *const UNICODE_STRING,
    ) -> i32;
    // Wave RTL-4b slice 4
    fn ref_RtlEqualComputerName(a: *mut UNICODE_STRING, b: *mut UNICODE_STRING) -> u8;
    fn ref_RtlEqualDomainName(a: *mut UNICODE_STRING, b: *mut UNICODE_STRING) -> u8;
    fn ref_RtlDnsHostNameToComputerName(
        cn: *mut UNICODE_STRING,
        dns: *mut UNICODE_STRING,
        alloc: u8,
    ) -> i32;
    // Wave RTL-4c
    fn ref_RtlIntegerToChar(v: u32, base: u32, len: u32, s: *mut u8) -> i32;
    fn ref_RtlLargeIntegerToChar(v: *const i64, base: u32, len: u32, s: *mut u8) -> i32;
    fn ref_RtlIntegerToUnicodeString(v: u32, base: u32, s: *mut UNICODE_STRING) -> i32;
    fn ref_RtlInt64ToUnicodeString(v: u64, base: u32, s: *mut UNICODE_STRING) -> i32;
    // Wave RTL-6: dos8dot3.c
    fn ref_RtlIsValidOemCharacter(c: *mut u16) -> u8;
    fn ref_RtlGenerate8dot3Name(
        name: *const UNICODE_STRING,
        ax: u8,
        ctx: *mut GenerateNameContext,
        out: *mut UNICODE_STRING,
    );
    fn ref_RtlIsNameLegalDOS8Dot3(
        name: *const UNICODE_STRING,
        oem: *mut STRING,
        sp: *mut u8,
    ) -> u8;
    // Wave RTL-5: utf8.c
    fn ref_RtlUnicodeToUTF8N(
        d: *mut u8,
        dmax: u32,
        w: *mut u32,
        s: *const u16,
        sb: u32,
    ) -> i32;
    fn ref_RtlUTF8ToUnicodeN(
        d: *mut u16,
        dmax: u32,
        w: *mut u32,
        s: *const u8,
        sb: u32,
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
                // Buffer.is_null() is part of the contract, not decoration: on the
                // alloc-failure edge the C stores NULL into the out-parameter, and a
                // port that returns before that store leaves the caller's stale
                // pointer beside a freshly-written Length. Comparing only the two
                // lengths cannot see it — that is how it hid in 10 of 11 converters.
                assert_eq!((dc.Length, dc.MaximumLength, dc.Buffer.is_null()),
                           (dr.Length, dr.MaximumLength, dr.Buffer.is_null()),
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
                assert_eq!((dc.Length, dc.MaximumLength, dc.Buffer.is_null()),
                           (dr.Length, dr.MaximumLength, dr.Buffer.is_null()),
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
            assert_eq!((dc.Length, dc.MaximumLength, dc.Buffer.is_null()),
                       (dr.Length, dr.MaximumLength, dr.Buffer.is_null()),
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
            assert_eq!((dc.Length, dc.MaximumLength, dc.Buffer.is_null()),
                       (dr.Length, dr.MaximumLength, dr.Buffer.is_null()),
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

// ── Wave RTL-4b slice 1: validate / find / erase / free / GUID ────

#[test]
fn diff_rtl_validate_unicode_string() {
    let mut rng = Rng(0x4b_0001_01);
    let backing = vec![0x41u16; 8];
    for _ in 0..40_000 {
        let flags = if rng.upto(4) == 0 { rng.next() as u32 } else { 0 };
        let length = (rng.upto(0x12) as u16).wrapping_sub(rng.upto(2) as u16);
        let maxlen = rng.upto(0x14) as u16;
        let null_buf = rng.upto(3) == 0;
        let buf = if null_buf {
            core::ptr::null_mut()
        } else {
            backing.as_ptr() as *mut u16
        };
        let us = UNICODE_STRING { Length: length, MaximumLength: maxlen, Buffer: buf };
        for null_str in [false, true] {
            let p: *const UNICODE_STRING =
                if null_str { core::ptr::null() } else { &us };
            let (rc, rr);
            unsafe {
                rc = cabi::RtlValidateUnicodeString(flags, p);
                rr = ref_RtlValidateUnicodeString(flags, p);
            }
            assert_eq!(
                rc, rr,
                "RtlValidateUnicodeString f={flags} l={length} m={maxlen} nb={null_buf} ns={null_str}"
            );
        }
    }
}

#[test]
fn diff_rtl_find_char() {
    let mut rng = Rng(0x4b_0001_02);
    let alpha: &[u16] = &[
        0x0041, 0x0061, 0x007A, 0x005A, 0x00C0, 0x00E0, 0x0152, 0x0153,
        0x0030, 0x002E, 0x0020, 0x00FF, 0x0178, 0xABCD,
    ];
    for _ in 0..30_000 {
        let mk = |rng: &mut Rng| -> Vec<u16> {
            (0..rng.upto(12)).map(|_| alpha[rng.upto(alpha.len())]).collect()
        };
        let sv = mk(&mut rng);
        let mv = mk(&mut rng);
        let sp = if sv.is_empty() { core::ptr::null_mut() } else { sv.as_ptr() as *mut u16 };
        let mp = if mv.is_empty() { core::ptr::null_mut() } else { mv.as_ptr() as *mut u16 };
        let ss = mk_ustr(sp, (sv.len() * 2) as u16, (sv.len() * 2) as u16);
        let ms = mk_ustr(mp, (mv.len() * 2) as u16, (mv.len() * 2) as u16);
        // valid bits 0..7 plus an out-of-mask bit (0x8) to fire the
        // INVALID_PARAMETER flag guard.
        let flags = (rng.upto(16)) as u32;
        for null_pos in [false, true] {
            let (mut pc, mut pr) = (0xBEEFu16, 0xBEEFu16);
            let (ppc, ppr) = if null_pos {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut pc as *mut u16, &mut pr as *mut u16)
            };
            let (rc, rr);
            unsafe {
                rc = cabi::RtlFindCharInUnicodeString(flags, &ss, &ms, ppc);
                rr = ref_RtlFindCharInUnicodeString(flags, &ss, &ms, ppr);
            }
            assert_eq!(rc, rr, "RtlFindCharInUnicodeString ret f={flags} {sv:?} {mv:?}");
            assert_eq!(pc, pr, "RtlFindCharInUnicodeString pos f={flags}");
        }
    }
}

#[test]
fn diff_rtl_erase_unicode_string() {
    let mut rng = Rng(0x4b_0001_03);
    for _ in 0..20_000 {
        let cap = rng.upto(20).max(1);
        let mut bc = vec![0xABu8; cap * 2];
        let mut br = vec![0xABu8; cap * 2];
        let maxlen = (match rng.upto(4) { 0 => 0, 1 => 2, 2 => cap, _ => cap * 2 }) as u16;
        let length = rng.upto(cap * 2 + 1) as u16;
        let null_buf = rng.upto(4) == 0;
        let pc = if null_buf { core::ptr::null_mut() } else { bc.as_mut_ptr() as *mut u16 };
        let pr = if null_buf { core::ptr::null_mut() } else { br.as_mut_ptr() as *mut u16 };
        let mut dc = mk_ustr(pc, length, maxlen);
        let mut dr = mk_ustr(pr, length, maxlen);
        unsafe {
            cabi::RtlEraseUnicodeString(&mut dc);
            ref_RtlEraseUnicodeString(&mut dr);
        }
        assert_eq!(dc.Length, dr.Length, "RtlEraseUnicodeString Length");
        assert_eq!(bc, br, "RtlEraseUnicodeString buf cap={cap} ml={maxlen}");
    }
}

#[test]
fn diff_rtl_free_strings() {
    let mut rng = Rng(0x4b_0001_04);
    let abacking = vec![0x55u8; 8];
    let ubacking = vec![0x55u16; 8];
    for _ in 0..10_000 {
        let l = rng.upto(16) as u16;
        let m = rng.upto(20) as u16;
        for null_buf in [false, true] {
            let ap = if null_buf { core::ptr::null_mut() } else { abacking.as_ptr() as *mut u8 };
            let up = if null_buf { core::ptr::null_mut() } else { ubacking.as_ptr() as *mut u16 };

            let mut ac = mk_string(ap, l, m);
            let mut ar = mk_string(ap, l, m);
            unsafe {
                cabi::RtlFreeAnsiString(&mut ac);
                ref_RtlFreeAnsiString(&mut ar);
            }
            assert_eq!(
                (ac.Length, ac.MaximumLength, ac.Buffer.is_null()),
                (ar.Length, ar.MaximumLength, ar.Buffer.is_null()),
                "RtlFreeAnsiString nb={null_buf}"
            );

            let mut oc = mk_string(ap, l, m);
            let mut or = mk_string(ap, l, m);
            unsafe {
                cabi::RtlFreeOemString(&mut oc);
                ref_RtlFreeOemString(&mut or);
            }
            assert_eq!(
                (oc.Length, oc.MaximumLength, oc.Buffer.is_null()),
                (or.Length, or.MaximumLength, or.Buffer.is_null()),
                "RtlFreeOemString nb={null_buf}"
            );

            let mut uc = mk_ustr(up, l, m);
            let mut ur = mk_ustr(up, l, m);
            unsafe {
                cabi::RtlFreeUnicodeString(&mut uc);
                ref_RtlFreeUnicodeString(&mut ur);
            }
            assert_eq!(
                (uc.Length, uc.MaximumLength, uc.Buffer.is_null()),
                (ur.Length, ur.MaximumLength, ur.Buffer.is_null()),
                "RtlFreeUnicodeString nb={null_buf}"
            );
        }
    }
}

#[test]
fn diff_rtl_guid_from_string() {
    let mut rng = Rng(0x4b_0001_05);
    let hex = b"0123456789abcdefABCDEF";
    for _ in 0..30_000 {
        // canonical {8-4-4-4-12} skeleton, 38 chars + slack.
        let mut s: Vec<u16> = Vec::with_capacity(40);
        s.push(b'{' as u16);
        let layout = [8usize, 4, 4, 4, 12];
        for (gi, &grp) in layout.iter().enumerate() {
            if gi != 0 {
                s.push(b'-' as u16);
            }
            for _ in 0..grp {
                s.push(hex[rng.upto(hex.len())] as u16);
            }
        }
        s.push(b'}' as u16);
        // corrupt a random position sometimes (wrong char / bad hex).
        if rng.upto(2) == 0 {
            let pos = rng.upto(s.len());
            s[pos] = match rng.upto(3) {
                0 => b'Z' as u16,
                1 => b' ' as u16,
                _ => 0x00C0,
            };
        }
        while s.len() < 40 {
            s.push(0);
        }
        let us = mk_ustr(s.as_ptr() as *mut u16, 76, 78);
        // identical sentinel so partial-write-on-failure compares equal.
        let mut gc = GUID { Data1: 0xDEAD_BEEF, Data2: 0x1234, Data3: 0x5678, Data4: [0x9A; 8] };
        let mut gr = GUID { Data1: 0xDEAD_BEEF, Data2: 0x1234, Data3: 0x5678, Data4: [0x9A; 8] };
        let (rc, rr);
        unsafe {
            rc = cabi::RtlGUIDFromString(&us, &mut gc);
            rr = ref_RtlGUIDFromString(&us, &mut gr);
        }
        assert_eq!(rc, rr, "RtlGUIDFromString ret {s:?}");
        assert_eq!(
            (gc.Data1, gc.Data2, gc.Data3, gc.Data4),
            (gr.Data1, gr.Data2, gr.Data3, gr.Data4),
            "RtlGUIDFromString guid {s:?}"
        );
    }
}

#[test]
fn diff_rtl_is_text_unicode() {
    let mut rng = Rng(0x4b_0002_01);
    for _ in 0..40_000 {
        let units = rng.upto(40);
        let kind = rng.upto(6);
        let mut b: Vec<u8> = Vec::with_capacity(units * 2 + 2);
        match kind {
            0 => {
                // ASCII as UTF-16LE.
                for _ in 0..units {
                    b.push(0x20 + (rng.byte() % 0x5f));
                    b.push(0);
                }
            }
            1 => {
                // byte-reversed (UTF-16BE-looking).
                for _ in 0..units {
                    b.push(0);
                    b.push(0x20 + (rng.byte() % 0x5f));
                }
            }
            2 => {
                // controls / CRLF / NUL units.
                let pool: &[u16] = &[
                    0x000D, 0x000A, 0x0009, 0x0020, 0x3000, 0x0000, 0x0A0D,
                    0x0D00, 0x2000, 0x0041,
                ];
                for _ in 0..units {
                    let w = pool[rng.upto(pool.len())];
                    b.extend_from_slice(&w.to_le_bytes());
                }
            }
            3 => {
                // BOM-prefixed.
                let bom = if rng.upto(2) == 0 { 0xFEFFu16 } else { 0xFFFEu16 };
                b.extend_from_slice(&bom.to_le_bytes());
                for _ in 0..units {
                    b.push(rng.byte());
                    b.push(rng.byte());
                }
            }
            _ => {
                // fully random bytes.
                for _ in 0..(units * 2) {
                    b.push(rng.byte());
                }
            }
        }
        // sometimes append a trailing NUL byte (exercises the len--).
        if rng.upto(2) == 0 {
            b.push(0);
        }
        if b.len() < 2 {
            b.push(0);
            b.push(0);
        }
        // len variants: 0/1 (early-return boundary), -1 (odd), exact,
        // capped-huge. Negative len is genuine UB (the `len <
        // sizeof(WCHAR)` size_t comparison wraps it past the guard
        // into an OOB read) — excluded, like the other documented
        // OOB quirks.
        let len = match rng.upto(5) {
            0 => 0i32,
            1 => 1,
            2 => (b.len() as i32) - 1,
            3 => (b.len() as i32).min(600),
            _ => b.len() as i32,
        };
        let bp = b.as_ptr();
        for pf_variant in 0..4 {
            let pf0: Option<i32> = match pf_variant {
                0 => None,
                1 => Some(-1),               // all filters on
                2 => Some(0),                // all filters off
                _ => Some(rng.next() as i32), // random filter
            };
            let (mut fc, mut fr) = (pf0.unwrap_or(0), pf0.unwrap_or(0));
            let (cp, rp) = if pf0.is_none() {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut fc as *mut i32, &mut fr as *mut i32)
            };
            let (rc, rr);
            unsafe {
                rc = cabi::RtlIsTextUnicode(bp, len, cp);
                rr = ref_RtlIsTextUnicode(bp, len, rp);
            }
            assert_eq!(rc, rr, "RtlIsTextUnicode ret kind={kind} len={len} pf={pf_variant}");
            assert_eq!(fc, fr, "RtlIsTextUnicode pf kind={kind} len={len} pf={pf_variant}");
        }
    }
}

// ── Wave RTL-4b slice 3: alloc-edge group ─────────────────────────

fn usentinel() -> UNICODE_STRING {
    UNICODE_STRING {
        Length: 0xDEAD,
        MaximumLength: 0xBEEF,
        Buffer: 0x1234_5678_9abc_def0u64 as *mut u16,
    }
}
fn uflds(u: &UNICODE_STRING) -> (u16, u16, bool) {
    (u.Length, u.MaximumLength, u.Buffer.is_null())
}

#[test]
fn diff_rtl_create_unicode_string() {
    let mut rng = Rng(0x4b_0003_01);
    for _ in 0..20_000 {
        let n = rng.upto(40);
        let mut s: Vec<u16> = (0..n).map(|_| 1 + (rng.next() as u16 % 0xFFFE)).collect();
        s.push(0);
        let (mut c, mut r) = (usentinel(), usentinel());
        let (rc, rr);
        unsafe {
            rc = cabi::RtlCreateUnicodeString(&mut c, s.as_ptr());
            rr = ref_RtlCreateUnicodeString(&mut r, s.as_ptr());
        }
        assert_eq!(rc, rr, "RtlCreateUnicodeString ret n={n}");
        assert_eq!(uflds(&c), uflds(&r), "RtlCreateUnicodeString fields n={n}");
    }
    // >MAXUSHORT boundary: (wcslen+1)*2 > 0xFFFF ⇔ wcslen >= 32767.
    for units in [32765usize, 32766, 32767, 33000] {
        let mut s = vec![b'A' as u16; units];
        s.push(0);
        let (mut c, mut r) = (usentinel(), usentinel());
        let (rc, rr);
        unsafe {
            rc = cabi::RtlCreateUnicodeString(&mut c, s.as_ptr());
            rr = ref_RtlCreateUnicodeString(&mut r, s.as_ptr());
        }
        assert_eq!(rc, rr, "RtlCreateUnicodeString ret units={units}");
        assert_eq!(uflds(&c), uflds(&r), "RtlCreateUnicodeString fields units={units}");
    }
}

#[test]
fn diff_rtl_create_from_asciiz() {
    let mut rng = Rng(0x4b_0003_02);
    for _ in 0..20_000 {
        let n = rng.upto(40);
        let mut s: Vec<u8> = (0..n).map(|_| rng.byte() | 1).collect();
        s.push(0);
        for null_src in [false, true] {
            let p = if null_src { core::ptr::null() } else { s.as_ptr() };
            let (mut c, mut r) = (usentinel(), usentinel());
            let (rc, rr);
            unsafe {
                rc = cabi::RtlCreateUnicodeStringFromAsciiz(&mut c, p);
                rr = ref_RtlCreateUnicodeStringFromAsciiz(&mut r, p);
            }
            assert_eq!(rc, rr, "RtlCreateUnicodeStringFromAsciiz ret n={n} ns={null_src}");
            assert_eq!(uflds(&c), uflds(&r),
                "RtlCreateUnicodeStringFromAsciiz fields n={n} ns={null_src}");
        }
    }
}

#[test]
fn diff_rtl_duplicate_unicode_string() {
    let mut rng = Rng(0x4b_0003_03);
    let backing = vec![0x41u16; 8];
    for _ in 0..30_000 {
        let flags = (rng.upto(7)) as u32;
        let kind = rng.upto(6);
        let (len, max, buf): (u16, u16, *mut u16) = match kind {
            0 => (4, 8, backing.as_ptr() as *mut u16),       // normal
            1 => (0, 0, core::ptr::null_mut()),               // empty
            2 => (0, 4, core::ptr::null_mut()),               // L0 M>0 buf NULL (invalid)
            3 => (0, 4, backing.as_ptr() as *mut u16),        // L0 M>0 buf OK
            4 => (10, 4, backing.as_ptr() as *mut u16),       // L>M (invalid)
            _ => (8, 16, backing.as_ptr() as *mut u16),       // normal2
        };
        let null_src = rng.upto(8) == 0;
        let ss = UNICODE_STRING { Length: len, MaximumLength: max, Buffer: buf };
        let sp: *const UNICODE_STRING =
            if null_src { core::ptr::null() } else { &ss };
        let (mut c, mut r) = (usentinel(), usentinel());
        let (rc, rr);
        unsafe {
            rc = cabi::RtlDuplicateUnicodeString(flags, sp, &mut c);
            rr = ref_RtlDuplicateUnicodeString(flags, sp, &mut r);
        }
        assert_eq!(rc, rr, "RtlDuplicateUnicodeString ret f={flags} k={kind} ns={null_src}");
        if rc == 0 {
            assert_eq!(uflds(&c), uflds(&r),
                "RtlDuplicateUnicodeString fields f={flags} k={kind}");
        }
    }
}

#[test]
fn diff_rtl_string_from_guid() {
    let mut rng = Rng(0x4b_0003_04);
    for _ in 0..5_000 {
        let g = GUID {
            Data1: rng.next() as u32,
            Data2: rng.next() as u16,
            Data3: rng.next() as u16,
            Data4: [rng.byte(); 8],
        };
        let (mut c, mut r) = (usentinel(), usentinel());
        let (rc, rr);
        unsafe {
            rc = cabi::RtlStringFromGUID(&g, &mut c);
            rr = ref_RtlStringFromGUID(&g, &mut r);
        }
        assert_eq!(rc, rr, "RtlStringFromGUID ret");
        assert_eq!(uflds(&c), uflds(&r), "RtlStringFromGUID fields");
    }
}

#[test]
fn diff_rtl_ensure_buffer_size() {
    let mut rng = Rng(0x4b_0003_05);
    for _ in 0..30_000 {
        let flags = (rng.upto(4)) as u32;
        // pointers are only compared for equality (no deref).
        let staticp = (0x1000 + rng.upto(4) * 8) as *mut u8;
        let same = rng.upto(2) == 0;
        let bufp = if same { staticp } else { (0x9000usize) as *mut u8 };
        let size = rng.upto(64);
        let staticsize = rng.upto(64);
        let req = rng.upto(64);
        let mk = || RTL_BUFFER {
            Buffer: bufp,
            StaticBuffer: staticp,
            Size: size,
            StaticSize: staticsize,
            ReservedForAllocatedSize: 0,
            ReservedForIMalloc: core::ptr::null_mut(),
        };
        let null_buf = rng.upto(6) == 0;
        let (mut bc, mut br) = (mk(), mk());
        let (pc, pr): (*mut RTL_BUFFER, *mut RTL_BUFFER) = if null_buf {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (&mut bc, &mut br)
        };
        let (rc, rr);
        unsafe {
            rc = cabi::RtlpEnsureBufferSize(flags, pc, req);
            rr = ref_RtlpEnsureBufferSize(flags, pr, req);
        }
        assert_eq!(rc, rr,
            "RtlpEnsureBufferSize ret f={flags} same={same} sz={size} ss={staticsize} req={req}");
        assert_eq!(bc.Size, br.Size, "RtlpEnsureBufferSize Size");
    }
}

// ── Wave RTL-4c: integer→char + *ToUnicodeString ──────────────────

#[test]
fn diff_rtl_integer_to_char() {
    let mut rng = Rng(0x4c_0001_01);
    let vals: &[u32] = &[0, 1, 9, 10, 15, 16, 255, 1000, 0x7FFF_FFFF, 0xFFFF_FFFF];
    let lens: &[u32] = &[0, 1, 2, 5, 8, 10, 32, 33, 40, 0xFFFF_FFFF];
    for _ in 0..30_000 {
        let value = if rng.upto(3) == 0 { vals[rng.upto(vals.len())] } else { rng.next() as u32 };
        let base = BASES[rng.upto(BASES.len())];
        let length = lens[rng.upto(lens.len())];
        for null_str in [false, true] {
            let mut bc = vec![0xAAu8; 48];
            let mut br = vec![0xAAu8; 48];
            let (pc, pr) = if null_str {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (bc.as_mut_ptr(), br.as_mut_ptr())
            };
            let (sc, sr);
            unsafe {
                sc = cabi::RtlIntegerToChar(value, base, length, pc);
                sr = ref_RtlIntegerToChar(value, base, length, pr);
            }
            assert_eq!(sc, sr, "RtlIntegerToChar ret v={value} b={base} len={length} ns={null_str}");
            if !null_str {
                assert_eq!(bc, br, "RtlIntegerToChar buf v={value} b={base} len={length}");
            }
        }
    }
}

#[test]
fn diff_rtl_large_integer_to_char() {
    let mut rng = Rng(0x4c_0001_02);
    let lens: &[u32] = &[0, 1, 2, 8, 16, 32, 64, 65, 80, 0xFFFF_FFFF];
    for _ in 0..30_000 {
        let value = if rng.upto(3) == 0 {
            (rng.next() as u64) & ((1u64 << (1 + rng.upto(63))) - 1)
        } else {
            rng.next() as u64
        };
        let base = BASES[rng.upto(BASES.len())];
        let length = lens[rng.upto(lens.len())];
        let v = value as i64;
        let mut bc = vec![0xAAu8; 96];
        let mut br = vec![0xAAu8; 96];
        let (sc, sr);
        unsafe {
            sc = cabi::RtlLargeIntegerToChar(&v, base, length, bc.as_mut_ptr());
            sr = ref_RtlLargeIntegerToChar(&v, base, length, br.as_mut_ptr());
        }
        assert_eq!(sc, sr, "RtlLargeIntegerToChar ret v={value:#x} b={base} len={length}");
        assert_eq!(bc, br, "RtlLargeIntegerToChar buf v={value:#x} b={base} len={length}");
    }
}

#[test]
fn diff_rtl_integer_to_unicode_string() {
    let mut rng = Rng(0x4c_0001_03);
    let mls: &[u16] = &[0, 4, 8, 20, 60];
    for _ in 0..20_000 {
        let value = rng.next() as u32;
        let base = BASES[rng.upto(BASES.len())];
        let ml = mls[rng.upto(mls.len())];
        let mut bc = vec![0xCCCCu16; 64];
        let mut br = vec![0xCCCCu16; 64];
        let mut sc_u = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: bc.as_mut_ptr() };
        let mut sr_u = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: br.as_mut_ptr() };
        let (sc, sr);
        unsafe {
            sc = cabi::RtlIntegerToUnicodeString(value, base, &mut sc_u);
            sr = ref_RtlIntegerToUnicodeString(value, base, &mut sr_u);
        }
        assert_eq!(sc, sr, "RtlIntegerToUnicodeString ret v={value} b={base} ml={ml}");
        assert_eq!((sc_u.Length, sc_u.MaximumLength), (sr_u.Length, sr_u.MaximumLength),
            "RtlIntegerToUnicodeString fields v={value} b={base} ml={ml}");
        assert_eq!(bc, br, "RtlIntegerToUnicodeString buf v={value} b={base} ml={ml}");
    }
}

#[test]
fn diff_rtl_int64_to_unicode_string() {
    let mut rng = Rng(0x4c_0001_04);
    let mls: &[u16] = &[0, 4, 8, 20, 60, 130];
    for _ in 0..20_000 {
        let value = rng.next() as u64;
        let base = BASES[rng.upto(BASES.len())];
        let ml = mls[rng.upto(mls.len())];
        let mut bc = vec![0xCCCCu16; 80];
        let mut br = vec![0xCCCCu16; 80];
        let mut sc_u = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: bc.as_mut_ptr() };
        let mut sr_u = UNICODE_STRING { Length: 0xAAAA, MaximumLength: ml, Buffer: br.as_mut_ptr() };
        let (sc, sr);
        unsafe {
            sc = cabi::RtlInt64ToUnicodeString(value, base, &mut sc_u);
            sr = ref_RtlInt64ToUnicodeString(value, base, &mut sr_u);
        }
        assert_eq!(sc, sr, "RtlInt64ToUnicodeString ret v={value:#x} b={base} ml={ml}");
        assert_eq!((sc_u.Length, sc_u.MaximumLength), (sr_u.Length, sr_u.MaximumLength),
            "RtlInt64ToUnicodeString fields v={value:#x} b={base} ml={ml}");
        assert_eq!(bc, br, "RtlInt64ToUnicodeString buf v={value:#x} b={base} ml={ml}");
    }
}

#[test]
fn diff_rtl_equal_computer_name() {
    let mut rng = Rng(0x4b_0004_01);
    let backing = vec![0x41u16; 8];
    for _ in 0..10_000 {
        let l1 = (rng.upto(9) * 2) as u16;
        let l2 = (rng.upto(9) * 2) as u16;
        let mut a = mk_ustr(backing.as_ptr() as *mut u16, l1, l1);
        let mut b = mk_ustr(backing.as_ptr() as *mut u16, l2, l2);
        unsafe {
            assert_eq!(
                cabi::RtlEqualComputerName(&mut a, &mut b),
                ref_RtlEqualComputerName(&mut a, &mut b),
                "RtlEqualComputerName"
            );
            assert_eq!(
                cabi::RtlEqualDomainName(&mut a, &mut b),
                ref_RtlEqualDomainName(&mut a, &mut b),
                "RtlEqualDomainName"
            );
        }
    }
}

#[test]
fn diff_rtl_dns_host_name_to_computer_name() {
    let mut rng = Rng(0x4b_0004_02);
    // ascii letters/digits/'.' plus unmappable units (→ '?' via enc,
    // exercising the RtlpDidUnicodeToOemWork FALSE branch) and
    // shared-model specials.
    let alpha: &[u16] = &[
        0x0041, 0x0061, 0x007A, 0x0030, 0x0039, 0x002E, 0x002D, 0x00C0,
        0x00E0, 0x0152, 0x0153, 0x0178, 0x20AC, 0x0000,
    ];
    for _ in 0..20_000 {
        let units = rng.upto(24);
        let dv: Vec<u16> =
            (0..units.max(1)).map(|_| alpha[rng.upto(alpha.len())]).collect();
        let dlen = (units * 2) as u16;
        let dp = dv.as_ptr() as *mut u16;
        let cmax = (match rng.upto(4) { 0 => 0, 1 => 8, 2 => 32, _ => 60 }) as u16;
        for alloc in [0u8, 1] {
            let mut dns = mk_ustr(dp, dlen, dlen);
            let mut bc = vec![0xCCCCu16; 64];
            let mut br = vec![0xCCCCu16; 64];
            let mut cc = UNICODE_STRING { Length: 0xAAAA, MaximumLength: cmax, Buffer: bc.as_mut_ptr() };
            let mut cr = UNICODE_STRING { Length: 0xAAAA, MaximumLength: cmax, Buffer: br.as_mut_ptr() };
            let (sc, sr);
            unsafe {
                sc = cabi::RtlDnsHostNameToComputerName(&mut cc, &mut dns, alloc);
                sr = ref_RtlDnsHostNameToComputerName(&mut cr, &mut dns, alloc);
            }
            assert_eq!(sc, sr, "RtlDnsHostNameToComputerName ret u={units} cm={cmax} a={alloc} {dv:?}");
            assert_eq!(
                (cc.Length, cc.MaximumLength),
                (cr.Length, cr.MaximumLength),
                "RtlDnsHostNameToComputerName fields u={units} cm={cmax} a={alloc}"
            );
            assert_eq!(bc, br, "RtlDnsHostNameToComputerName buf u={units} cm={cmax} a={alloc} {dv:?}");
        }
    }
}

// ── Wave RTL-6: dos8dot3.c ────────────────────────────────────────

#[test]
fn diff_rtl_is_valid_oem_character() {
    for w in 0u32..=0xFFFF {
        let w = w as u16;
        let (mut a, mut b) = (w, w);
        let (rc, rr);
        unsafe {
            rc = cabi::RtlIsValidOemCharacter(&mut a);
            rr = ref_RtlIsValidOemCharacter(&mut b);
        }
        assert_eq!(rc, rr, "RtlIsValidOemCharacter ret {w:#06x}");
        assert_eq!(a, b, "RtlIsValidOemCharacter mutated *Char {w:#06x}");
    }
}

fn ctx_zero() -> GenerateNameContext {
    GenerateNameContext {
        checksum: 0,
        checksum_inserted: 0,
        name_length: 0,
        name_buffer: [0; 8],
        extension_length: 0,
        extension_buffer: [0; 4],
        last_index_value: 0,
    }
}
fn ctx_tuple(
    c: &GenerateNameContext,
) -> (u16, u8, u8, [u16; 8], u32, [u16; 4], u32) {
    (
        c.checksum,
        c.checksum_inserted,
        c.name_length,
        c.name_buffer,
        c.extension_length,
        c.extension_buffer,
        c.last_index_value,
    )
}

#[test]
fn diff_rtl_generate_8dot3_name() {
    let mut rng = Rng(0x6_0001_01);
    let pool: &[u16] = &[
        0x0041, 0x0061, 0x005A, 0x007A, 0x0030, 0x0039, 0x002E, 0x0020,
        0x005F, 0x002A, 0x003F, 0x00C0, 0x00E9, 0x0152, 0x20AC, 0x00FF,
        0x0080, 0xABCD,
    ];
    for _ in 0..8_000 {
        let units = 1 + rng.upto(20);
        let nv: Vec<u16> =
            (0..units).map(|_| pool[rng.upto(pool.len())]).collect();
        let nlen = (units * 2) as u16;
        let ax = (rng.upto(2)) as u8;
        let ns = UNICODE_STRING {
            Length: nlen,
            MaximumLength: nlen,
            Buffer: nv.as_ptr() as *mut u16,
        };
        let mut cc = ctx_zero();
        let mut cr = ctx_zero();
        // call repeatedly with the SAME context (real caller pattern):
        // exercises name_length<=2 checksum + LastIndexValue>4 paths.
        for call in 0..7 {
            let mut bc = vec![0xCCCCu16; 32];
            let mut br = vec![0xCCCCu16; 32];
            let mut oc = UNICODE_STRING {
                Length: 0xAAAA,
                MaximumLength: 64,
                Buffer: bc.as_mut_ptr(),
            };
            let mut or = UNICODE_STRING {
                Length: 0xAAAA,
                MaximumLength: 64,
                Buffer: br.as_mut_ptr(),
            };
            unsafe {
                cabi::RtlGenerate8dot3Name(&ns, ax, &mut cc, &mut oc);
                ref_RtlGenerate8dot3Name(&ns, ax, &mut cr, &mut or);
            }
            assert_eq!(
                ctx_tuple(&cc),
                ctx_tuple(&cr),
                "Generate8dot3 ctx call={call} ax={ax} {nv:?}"
            );
            assert_eq!(
                oc.Length, or.Length,
                "Generate8dot3 out.Length call={call} {nv:?}"
            );
            assert_eq!(bc, br, "Generate8dot3 out.buf call={call} {nv:?}");
        }
    }
}

#[test]
fn diff_rtl_is_name_legal_dos8dot3() {
    let mut rng = Rng(0x6_0001_02);
    let pool: &[u16] = &[
        0x0041, 0x0061, 0x007A, 0x0030, 0x002E, 0x0020, 0x005F, 0x002A,
        0x003C, 0x002B, 0x003A, 0x005C, 0x00E5, 0x00C0, 0x0152, 0x00FF,
        0x0080,
    ];
    for _ in 0..20_000 {
        let units = rng.upto(16);
        let nv: Vec<u16> =
            (0..units.max(1)).map(|_| pool[rng.upto(pool.len())]).collect();
        let nlen = (units * 2) as u16;
        let ns = UNICODE_STRING {
            Length: nlen,
            MaximumLength: nlen,
            Buffer: nv.as_ptr() as *mut u16,
        };
        let provide_oem = rng.upto(2) == 0;
        let provide_sp = rng.upto(2) == 0;
        let mut obc = vec![0u8; 16];
        let mut obr = vec![0u8; 16];
        let mut osc = STRING {
            Length: 16,
            MaximumLength: 16,
            Buffer: obc.as_mut_ptr(),
        };
        let mut osr = STRING {
            Length: 16,
            MaximumLength: 16,
            Buffer: obr.as_mut_ptr(),
        };
        let (opc, opr): (*mut STRING, *mut STRING) = if provide_oem {
            (&mut osc, &mut osr)
        } else {
            (core::ptr::null_mut(), core::ptr::null_mut())
        };
        let (mut spc, mut spr) = (0xEEu8, 0xEEu8);
        let (sppc, sppr) = if provide_sp {
            (&mut spc as *mut u8, &mut spr as *mut u8)
        } else {
            (core::ptr::null_mut(), core::ptr::null_mut())
        };
        let (rc, rr);
        unsafe {
            rc = cabi::RtlIsNameLegalDOS8Dot3(&ns, opc, sppc);
            rr = ref_RtlIsNameLegalDOS8Dot3(&ns, opr, sppr);
        }
        assert_eq!(rc, rr, "RtlIsNameLegalDOS8Dot3 ret {nv:?}");
        assert_eq!(spc, spr, "RtlIsNameLegalDOS8Dot3 *spaces {nv:?}");
        if provide_oem {
            assert_eq!(
                (osc.Length, osc.MaximumLength),
                (osr.Length, osr.MaximumLength),
                "RtlIsNameLegalDOS8Dot3 oem fields {nv:?}"
            );
            assert_eq!(obc, obr, "RtlIsNameLegalDOS8Dot3 oem buf {nv:?}");
        }
    }
}

#[test]
fn diff_rtl_multi_append() {
    unsafe {
        let rc = cabi::RtlMultiAppendUnicodeStringBuffer(
            core::ptr::null_mut(), 0, core::ptr::null());
        let rr = ref_RtlMultiAppendUnicodeStringBuffer(
            core::ptr::null_mut(), 0, core::ptr::null());
        assert_eq!(rc, rr, "RtlMultiAppendUnicodeStringBuffer");
    }
}

// ── Wave RTL-5: utf8.c ────────────────────────────────────────────

#[test]
fn diff_rtl_unicode_to_utf8() {
    let mut rng = Rng(0x5_0001_01);
    // ascii, BMP, high/low surrogates (lone + paired), U+FFFD.
    let pool: &[u16] = &[
        0x0041, 0x0024, 0x00A3, 0x07FF, 0x0800, 0x20AC, 0xFFFD, 0xFFFF,
        0xD800, 0xDBFF, 0xDC00, 0xDFFF, 0x0000, 0x0080,
    ];
    for _ in 0..30_000 {
        let units = rng.upto(20);
        let sv: Vec<u16> =
            (0..units).map(|_| pool[rng.upto(pool.len())]).collect();
        let null_src = rng.upto(12) == 0;
        let null_w = rng.upto(12) == 0;
        let null_dest = rng.upto(2) == 0;
        // uni_bytes: usually units*2, sometimes odd (PARAM_5 when dest).
        let uni_bytes = (match rng.upto(5) {
            0 => (units * 2).saturating_sub(1),
            1 => units,
            _ => units * 2,
        }) as u32;
        let dmax = (match rng.upto(4) { 0 => 0, 1 => units, 2 => units * 2, _ => units * 4 + 4 }) as u32;
        let sp = if null_src { core::ptr::null() } else { sv.as_ptr() };
        let mut bc = vec![0xCCu8; (dmax as usize).max(1)];
        let mut br = vec![0xCCu8; (dmax as usize).max(1)];
        let dp_c = if null_dest { core::ptr::null_mut() } else { bc.as_mut_ptr() };
        let dp_r = if null_dest { core::ptr::null_mut() } else { br.as_mut_ptr() };
        let (mut wc, mut wr) = (0xDEAD_BEEFu32, 0xDEAD_BEEFu32);
        let (wpc, wpr) = if null_w {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (&mut wc as *mut u32, &mut wr as *mut u32)
        };
        let (sc, sr);
        unsafe {
            sc = cabi::RtlUnicodeToUTF8N(dp_c, dmax, wpc, sp, uni_bytes);
            sr = ref_RtlUnicodeToUTF8N(dp_r, dmax, wpr, sp, uni_bytes);
        }
        assert_eq!(sc, sr, "RtlUnicodeToUTF8N ret u={units} ub={uni_bytes} dm={dmax} ns={null_src} nw={null_w} nd={null_dest}");
        assert_eq!(wc, wr, "RtlUnicodeToUTF8N written {sv:?}");
        if !null_dest {
            assert_eq!(bc, br, "RtlUnicodeToUTF8N dst {sv:?} dm={dmax}");
        }
    }
}

#[test]
fn diff_rtl_utf8_to_unicode() {
    let mut rng = Rng(0x5_0001_02);
    // valid ascii/2-3-4-byte leads, trail bytes, invalid leads,
    // overlong/truncation triggers.
    let pool: &[u8] = &[
        0x41, 0x24, 0xC2, 0xA3, 0xE0, 0xA0, 0x80, 0xE2, 0x82, 0xAC,
        0xF0, 0x90, 0x8D, 0x88, 0xF5, 0xFF, 0x80, 0xBF, 0xC0, 0xC1,
        0x00, 0xED, 0xA0,
    ];
    for _ in 0..30_000 {
        let n = rng.upto(24);
        let sv: Vec<u8> = (0..n).map(|_| pool[rng.upto(pool.len())]).collect();
        let null_src = rng.upto(12) == 0;
        let null_w = rng.upto(12) == 0;
        let null_dest = rng.upto(2) == 0;
        let utf8_bytes = (match rng.upto(4) { 0 => 0, 1 => n / 2, _ => n }) as u32;
        // backing big enough for the worst case (≤3 units/byte); dmax
        // (bytes) varied independently but ≤ backing*2.
        let cap_units = n * 3 + 4;
        let dmax = (match rng.upto(4) {
            0 => 0,
            1 => n,
            2 => n * 2,
            _ => cap_units * 2,
        }) as u32;
        let sp = if null_src { core::ptr::null() } else { sv.as_ptr() };
        let mut bc = vec![0xCCCCu16; cap_units.max(1)];
        let mut br = vec![0xCCCCu16; cap_units.max(1)];
        let dp_c = if null_dest { core::ptr::null_mut() } else { bc.as_mut_ptr() };
        let dp_r = if null_dest { core::ptr::null_mut() } else { br.as_mut_ptr() };
        let (mut wc, mut wr) = (0xDEAD_BEEFu32, 0xDEAD_BEEFu32);
        let (wpc, wpr) = if null_w {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (&mut wc as *mut u32, &mut wr as *mut u32)
        };
        let (sc, sr);
        unsafe {
            sc = cabi::RtlUTF8ToUnicodeN(dp_c, dmax, wpc, sp, utf8_bytes);
            sr = ref_RtlUTF8ToUnicodeN(dp_r, dmax, wpr, sp, utf8_bytes);
        }
        assert_eq!(sc, sr, "RtlUTF8ToUnicodeN ret n={n} ub={utf8_bytes} dm={dmax} ns={null_src} nw={null_w} nd={null_dest} {sv:?}");
        assert_eq!(wc, wr, "RtlUTF8ToUnicodeN written {sv:?}");
        if !null_dest {
            assert_eq!(bc, br, "RtlUTF8ToUnicodeN dst {sv:?} dm={dmax}");
        }
    }
}

// ── Wave SID-1: sdk/lib/rtl/sid.c (byte + alloc-edge surface) ─────
// RtlConvertSidToUnicodeString (swprintf) is Wave SID-2 — not here.

use rtl_strings::cabi::SidAndAttributes;

extern "C" {
    fn ref_RtlValidSid(s: *const u8) -> u8;
    fn ref_RtlLengthRequiredSid(c: u32) -> u32;
    fn ref_RtlInitializeSid(s: *mut u8, a: *const u8, c: u8) -> i32;
    fn ref_RtlSubAuthoritySid(s: *mut u8, n: u32) -> *mut u32;
    fn ref_RtlSubAuthorityCountSid(s: *mut u8) -> *mut u8;
    fn ref_RtlIdentifierAuthoritySid(s: *mut u8) -> *mut u8;
    fn ref_RtlEqualSid(a: *const u8, b: *const u8) -> u8;
    fn ref_RtlLengthSid(s: *const u8) -> u32;
    fn ref_RtlCopySid(bl: u32, d: *mut u8, s: *const u8) -> i32;
    fn ref_RtlFreeSid(s: *mut u8) -> *mut u8;
    fn ref_RtlEqualPrefixSid(a: *const u8, b: *const u8) -> u8;
    fn ref_RtlCopySidAndAttributesArray(
        n: u32,
        src: *const SidAndAttributes,
        sz: u32,
        dst: *mut SidAndAttributes,
        area: *mut u8,
        ra: *mut *mut u8,
        rs: *mut u32,
    ) -> i32;
    fn ref_RtlAllocateAndInitializeSid(
        a: *const u8,
        c: u8,
        s0: u32,
        s1: u32,
        s2: u32,
        s3: u32,
        s4: u32,
        s5: u32,
        s6: u32,
        s7: u32,
        out: *mut *mut u8,
    ) -> i32;
    fn ref_RtlCreateServiceSid(n: *const UNICODE_STRING, s: *mut u8, l: *mut u32) -> i32;
    fn ref_RtlConvertSidToUnicodeString(
        s: *mut UNICODE_STRING,
        sid: *const u8,
        alloc: u8,
    ) -> i32;
    // Wave ACL-1
    fn ref_RtlCreateAcl(a: *mut u8, sz: u32, rev: u32) -> i32;
    fn ref_RtlValidAcl(a: *const u8) -> u8;
    fn ref_RtlFirstFreeAce(a: *mut u8, ff: *mut *mut u8) -> u8;
    fn ref_RtlGetAce(a: *mut u8, idx: u32, ace: *mut *mut u8) -> i32;
    fn ref_RtlQueryInformationAcl(a: *mut u8, i: *mut u8, l: u32, c: u32) -> i32;
    fn ref_RtlSetInformationAcl(a: *mut u8, i: *mut u8, l: u32, c: u32) -> i32;
    // Wave ACL-2
    fn ref_RtlpAddData(al: *const u8, all: u32, ace: *mut u8, off: u32);
    fn ref_RtlpDeleteData(ace: *mut u8, asz: u32, off: u32);
    fn ref_RtlpAddKnownAce(
        a: *mut u8,
        rev: u32,
        fl: u32,
        m: u32,
        s: *const u8,
        ty: u8,
    ) -> i32;
    fn ref_RtlpAddKnownObjectAce(
        a: *mut u8,
        rev: u32,
        fl: u32,
        m: u32,
        otg: *const u8,
        iotg: *const u8,
        s: *const u8,
        ty: u8,
    ) -> i32;
    fn ref_RtlAddAce(a: *mut u8, rev: u32, idx: u32, l: *mut u8, ll: u32) -> i32;
    fn ref_RtlDeleteAce(a: *mut u8, idx: u32) -> i32;
    fn ref_RtlAddAccessAllowedAce(a: *mut u8, rev: u32, m: u32, s: *const u8) -> i32;
    fn ref_RtlAddAccessAllowedAceEx(
        a: *mut u8,
        rev: u32,
        fl: u32,
        m: u32,
        s: *const u8,
    ) -> i32;
    fn ref_RtlAddAccessAllowedObjectAce(
        a: *mut u8,
        rev: u32,
        fl: u32,
        m: u32,
        otg: *const u8,
        iotg: *const u8,
        s: *const u8,
    ) -> i32;
    fn ref_RtlAddAccessDeniedAce(a: *mut u8, rev: u32, m: u32, s: *const u8) -> i32;
    fn ref_RtlAddAuditAccessAce(
        a: *mut u8,
        rev: u32,
        m: u32,
        s: *const u8,
        su: u8,
        fa: u8,
    ) -> i32;
}

/// Build a well-formed ACL: `[rev,0,AclSize(le16),AceCount(le16),0,0]`
/// then `AceCount` ACCESS_ALLOWED ACEs (`[0,flags,size(le16),
/// mask(le32), SID...]`, AceSize = 8 + LengthSid, 4-aligned). `slack`
/// extra free bytes after the ACEs (still inside AclSize).
fn mk_acl(rng: &mut Rng, rev: u8, n: u8, slack: u16) -> Vec<u8> {
    let mut aces: Vec<Vec<u8>> = Vec::new();
    for _ in 0..n {
        let sub = rng.byte() % 4;
        let sid = mk_sid(rng, 1, sub);
        let sidlen = 8 + 4 * sub as usize;
        let asz = 8 + sidlen; // 4-aligned (sidlen %4==0)
        let mut ace = vec![0u8; asz];
        ace[0] = 0; // ACCESS_ALLOWED_ACE_TYPE
        ace[1] = (rng.byte() & 0x1F) as u8; // inherit flags
        ace[2..4].copy_from_slice(&(asz as u16).to_le_bytes());
        let mask = rng.next() as u32;
        ace[4..8].copy_from_slice(&mask.to_le_bytes());
        ace[8..8 + sidlen].copy_from_slice(&sid[..sidlen]);
        aces.push(ace);
    }
    let body: usize = aces.iter().map(|a| a.len()).sum();
    let mut acl_size = 8 + body + slack as usize;
    acl_size = (acl_size + 1) & !1; // USHORT-align
    let mut v = vec![0u8; acl_size + 16];
    v[0] = rev;
    v[2..4].copy_from_slice(&(acl_size as u16).to_le_bytes());
    v[4..6].copy_from_slice(&(n as u16).to_le_bytes());
    let mut off = 8;
    for a in &aces {
        v[off..off + a.len()].copy_from_slice(a);
        off += a.len();
    }
    v
}

#[test]
fn diff_rtl_create_acl() {
    let mut rng = Rng(0xAC10_0001);
    for _ in 0..20_000 {
        let sz = (rng.next() % 0x1_2000) as u32;
        let rev = (rng.byte() % 7) as u32;
        let mut bc = vec![0xCDu8; 0x12000 + 16];
        let mut br = bc.clone();
        let sc = unsafe { cabi::RtlCreateAcl(bc.as_mut_ptr(), sz, rev) };
        let sr = unsafe { ref_RtlCreateAcl(br.as_mut_ptr(), sz, rev) };
        assert_eq!(sc, sr, "RtlCreateAcl status sz={sz} rev={rev}");
        if sc == 0 {
            assert_eq!(bc[..8], br[..8], "ACL header sz={sz} rev={rev}");
        }
    }
}

#[test]
fn diff_rtl_valid_acl() {
    let mut rng = Rng(0xAC10_0002);
    for _ in 0..20_000 {
        let rev = match rng.byte() % 5 {
            0 => 1,
            1 => 5,
            _ => 2 + rng.byte() % 3,
        };
        let n = rng.byte() % 5;
        let slack = (rng.byte() as u16) & 0x1E;
        let mut v = mk_acl(&mut rng, rev, n, slack);
        // Perturbations.
        match rng.byte() % 6 {
            0 => v[2] ^= 1,                       // misalign AclSize
            1 => v[4] = v[4].wrapping_add(1),     // AceCount lie
            2 if v.len() > 9 => v[9] = 0,         // corrupt a SID rev
            3 if v.len() > 8 => v[8] = 0xFE,      // bad ACE type
            _ => {}
        }
        let c = unsafe { cabi::RtlValidAcl(v.as_ptr()) };
        let r = unsafe { ref_RtlValidAcl(v.as_ptr()) };
        assert_eq!(c, r, "RtlValidAcl rev={rev} n={n}");
    }
}

#[test]
fn diff_rtl_first_free_ace_and_get_ace() {
    let mut rng = Rng(0xAC10_0003);
    for _ in 0..10_000 {
        let n = rng.byte() % 6;
        let rev = 2 + rng.byte() % 3;
        let slack = (rng.byte() as u16) & 0x3E;
        let mut v = mk_acl(&mut rng, rev, n, slack);
        let p = v.as_mut_ptr();
        let mut fc: *mut u8 = core::ptr::null_mut();
        let mut fr: *mut u8 = core::ptr::null_mut();
        let bc = unsafe { cabi::RtlFirstFreeAce(p, &mut fc) };
        let br = unsafe { ref_RtlFirstFreeAce(p, &mut fr) };
        assert_eq!(bc, br, "RtlFirstFreeAce ret n={n}");
        assert_eq!(fc, fr, "RtlFirstFreeAce *out n={n}");

        for idx in 0..(n as u32 + 2) {
            let mut ac: *mut u8 = core::ptr::null_mut();
            let mut ar: *mut u8 = core::ptr::null_mut();
            let sc = unsafe { cabi::RtlGetAce(p, idx, &mut ac) };
            let sr = unsafe { ref_RtlGetAce(p, idx, &mut ar) };
            assert_eq!(sc, sr, "RtlGetAce status n={n} idx={idx}");
            if sc == 0 {
                assert_eq!(ac, ar, "RtlGetAce *out n={n} idx={idx}");
            }
        }
    }
}

#[test]
fn diff_rtl_query_set_information_acl() {
    let mut rng = Rng(0xAC10_0004);
    for _ in 0..10_000 {
        let rev = match rng.byte() % 4 {
            0 => 1,
            _ => 2 + rng.byte() % 3,
        };
        let n = rng.byte() % 4;
        let slack = (rng.byte() as u16) & 0x1E;
        let mut v = mk_acl(&mut rng, rev, n, slack);
        let p = v.as_mut_ptr();
        let class = (rng.byte() % 4) as u32;
        let len = match rng.byte() % 3 {
            0 => 0,
            1 => 4,
            _ => 16,
        };
        let mut ic = [0u8; 16];
        let mut ir = [0u8; 16];
        let sc = unsafe {
            cabi::RtlQueryInformationAcl(p, ic.as_mut_ptr(), len, class)
        };
        let sr = unsafe {
            ref_RtlQueryInformationAcl(p, ir.as_mut_ptr(), len, class)
        };
        assert_eq!(sc, sr, "Query status rev={rev} n={n} class={class} len={len}");
        if sc == 0 {
            assert_eq!(ic, ir, "Query info rev={rev} n={n} class={class}");
        }

        // SetInformationAcl: only AclRevisionInformation; new rev
        // must exceed current.
        let new_rev = (rng.byte() % 7) as u32;
        let mut sb_c = new_rev.to_le_bytes().to_vec();
        sb_c.resize(8, 0);
        let mut sb_r = sb_c.clone();
        let mut wc = v.clone();
        let mut wr = v.clone();
        let slen = match rng.byte() % 2 {
            0 => 0,
            _ => 4,
        };
        let setc = unsafe {
            cabi::RtlSetInformationAcl(wc.as_mut_ptr(), sb_c.as_mut_ptr(), slen, class)
        };
        let setr = unsafe {
            ref_RtlSetInformationAcl(wr.as_mut_ptr(), sb_r.as_mut_ptr(), slen, class)
        };
        assert_eq!(setc, setr, "Set status rev={rev} class={class} nr={new_rev}");
        if setc == 0 {
            assert_eq!(wc[0], wr[0], "Set wrote AclRevision");
        }
    }
}

/// A SID buffer `[rev, count, auth[6], subauth[count] LE]` with 16B
/// trailing slack (RtlEqualSid over-reads Sid2 up to len(Sid1)).
fn mk_sid(rng: &mut Rng, rev: u8, count: u8) -> Vec<u8> {
    let mut v = vec![0u8; 8 + 4 * count as usize + 16];
    v[0] = rev;
    v[1] = count;
    for i in 0..6 {
        v[2 + i] = rng.byte();
    }
    for i in 0..count as usize {
        let sa = (rng.next() as u32) & if rng.byte() & 1 == 0 { 0xF } else { !0 };
        v[8 + 4 * i..8 + 4 * i + 4].copy_from_slice(&sa.to_le_bytes());
    }
    v
}

#[test]
fn diff_rtl_valid_sid() {
    let mut rng = Rng(0x51D0_0001);
    for _ in 0..20_000 {
        let rev = if rng.byte() & 3 == 0 { rng.byte() } else { 1 };
        let count = if rng.byte() & 3 == 0 { rng.byte() } else { rng.byte() % 16 };
        let v = mk_sid(&mut rng, rev, count.min(15));
        let (c, r) = unsafe {
            (cabi::RtlValidSid(v.as_ptr()), ref_RtlValidSid(v.as_ptr()))
        };
        assert_eq!(c, r, "RtlValidSid rev={rev} count={count}");
    }
    // NULL ⇒ TRUE (the `&&` short-circuit quirk).
    let (c, r) = unsafe {
        (
            cabi::RtlValidSid(core::ptr::null()),
            ref_RtlValidSid(core::ptr::null()),
        )
    };
    assert_eq!(c, r);
    assert_eq!(c, 1, "RtlValidSid(NULL) must be TRUE");
}

#[test]
fn diff_rtl_length_sids() {
    let mut rng = Rng(0x51D0_0002);
    for c in 0..=40u32 {
        let cc = unsafe { cabi::RtlLengthRequiredSid(c) };
        let rr = unsafe { ref_RtlLengthRequiredSid(c) };
        assert_eq!(cc, rr, "RtlLengthRequiredSid {c}");
    }
    for _ in 0..5_000 {
        let count = rng.byte();
        let v = mk_sid(&mut rng, 1, count.min(15));
        let cc = unsafe { cabi::RtlLengthSid(v.as_ptr()) };
        let rr = unsafe { ref_RtlLengthSid(v.as_ptr()) };
        assert_eq!(cc, rr, "RtlLengthSid count={count}");
    }
}

#[test]
fn diff_rtl_initialize_sid() {
    let mut rng = Rng(0x51D0_0003);
    for _ in 0..10_000 {
        let count = rng.byte() % 16;
        let auth: [u8; 6] = core::array::from_fn(|_| rng.byte());
        let mut bc = vec![0xCDu8; 8 + 4 * count as usize + 8];
        let mut br = bc.clone();
        let sc = unsafe { cabi::RtlInitializeSid(bc.as_mut_ptr(), auth.as_ptr(), count) };
        let sr = unsafe { ref_RtlInitializeSid(br.as_mut_ptr(), auth.as_ptr(), count) };
        assert_eq!(sc, sr);
        assert_eq!(bc, br, "RtlInitializeSid count={count}");
    }
}

#[test]
fn diff_rtl_sid_accessors() {
    let mut rng = Rng(0x51D0_0004);
    for _ in 0..5_000 {
        let count = (rng.byte() % 15) + 1;
        let mut v = mk_sid(&mut rng, 1, count);
        let p = v.as_mut_ptr();
        let n = rng.upto(count as usize) as u32;
        unsafe {
            assert_eq!(
                cabi::RtlSubAuthoritySid(p, n),
                ref_RtlSubAuthoritySid(p, n),
                "RtlSubAuthoritySid n={n}"
            );
            assert_eq!(
                cabi::RtlSubAuthorityCountSid(p),
                ref_RtlSubAuthorityCountSid(p)
            );
            assert_eq!(
                cabi::RtlIdentifierAuthoritySid(p),
                ref_RtlIdentifierAuthoritySid(p)
            );
        }
    }
}

#[test]
fn diff_rtl_equal_sid() {
    let mut rng = Rng(0x51D0_0005);
    for _ in 0..20_000 {
        let count = rng.byte() % 16;
        let a = mk_sid(&mut rng, 1, count);
        let b = if rng.byte() & 1 == 0 {
            a.clone()
        } else {
            mk_sid(&mut rng, 1, count)
        };
        let cc = unsafe { cabi::RtlEqualSid(a.as_ptr(), b.as_ptr()) };
        let rr = unsafe { ref_RtlEqualSid(a.as_ptr(), b.as_ptr()) };
        assert_eq!(cc, rr, "RtlEqualSid count={count}");
    }
}

#[test]
fn diff_rtl_copy_sid() {
    let mut rng = Rng(0x51D0_0006);
    for _ in 0..10_000 {
        let count = rng.byte() % 16;
        let src = mk_sid(&mut rng, 1, count);
        let need = 8 + 4 * count as u32;
        let bl = need.wrapping_add((rng.byte() as u32).wrapping_sub(4));
        let mut dc = vec![0xAAu8; need as usize + 16];
        let mut dr = dc.clone();
        let sc = unsafe { cabi::RtlCopySid(bl, dc.as_mut_ptr(), src.as_ptr()) };
        let sr = unsafe { ref_RtlCopySid(bl, dr.as_mut_ptr(), src.as_ptr()) };
        assert_eq!(sc, sr, "RtlCopySid status bl={bl} need={need}");
        assert_eq!(dc, dr, "RtlCopySid dest bl={bl} need={need}");
    }
}

#[test]
fn diff_rtl_free_sid() {
    let mut buf = [0u8; 16];
    let c = unsafe { cabi::RtlFreeSid(buf.as_mut_ptr()) };
    let r = unsafe { ref_RtlFreeSid(buf.as_mut_ptr()) };
    assert_eq!(c, r);
    assert!(c.is_null(), "RtlFreeSid must return NULL");
}

#[test]
fn diff_rtl_equal_prefix_sid() {
    let mut rng = Rng(0x51D0_0007);
    for _ in 0..20_000 {
        let count = rng.byte() % 8;
        let a = mk_sid(&mut rng, 1, count);
        // Often share a prefix with b.
        let mut b = a.clone();
        if rng.byte() & 1 == 0 {
            // perturb the LAST subauthority only (prefix still equal)
            if count > 0 {
                let last = 8 + 4 * (count as usize - 1);
                b[last] ^= rng.byte();
            }
        } else {
            let brev = if rng.byte() & 1 == 0 { 1 } else { rng.byte() };
            b = mk_sid(&mut rng, brev, count);
        }
        let cc = unsafe { cabi::RtlEqualPrefixSid(a.as_ptr(), b.as_ptr()) };
        let rr = unsafe { ref_RtlEqualPrefixSid(a.as_ptr(), b.as_ptr()) };
        assert_eq!(cc, rr, "RtlEqualPrefixSid count={count}");
    }
}

#[test]
fn diff_rtl_copy_sid_and_attributes_array() {
    let mut rng = Rng(0x51D0_0008);
    for _ in 0..3_000 {
        let n = (rng.byte() % 5) as u32;
        let counts: Vec<u8> = (0..n).map(|_| rng.byte() % 6).collect();
        let sids: Vec<Vec<u8>> =
            counts.iter().map(|&c| mk_sid(&mut rng, 1, c)).collect();
        let total: u32 = sids.iter().map(|s| 8 + 4 * s[1] as u32).sum();
        let area_sz = total.wrapping_add((rng.byte() as u32).wrapping_sub(8));

        let mk_src = |sids: &Vec<Vec<u8>>| -> Vec<SidAndAttributes> {
            sids.iter()
                .map(|s| SidAndAttributes {
                    sid: s.as_ptr() as *mut u8,
                    attributes: 0xA77,
                })
                .collect()
        };
        let src_c = mk_src(&sids);
        let src_r = mk_src(&sids);
        let mut dst_c = vec![
            SidAndAttributes { sid: core::ptr::null_mut(), attributes: 0 };
            n as usize
        ];
        let mut dst_r = dst_c.iter().map(|_| SidAndAttributes {
            sid: core::ptr::null_mut(),
            attributes: 0,
        }).collect::<Vec<_>>();
        let mut ac = vec![0u8; total as usize + 32];
        let mut ar = ac.clone();
        let (mut rac, mut racs) = (core::ptr::null_mut(), 0u32);
        let (mut rar, mut rars) = (core::ptr::null_mut(), 0u32);
        let sc = unsafe {
            cabi::RtlCopySidAndAttributesArray(
                n, src_c.as_ptr(), area_sz, dst_c.as_mut_ptr(),
                ac.as_mut_ptr(), &mut rac, &mut racs,
            )
        };
        let sr = unsafe {
            ref_RtlCopySidAndAttributesArray(
                n, src_r.as_ptr(), area_sz, dst_r.as_mut_ptr(),
                ar.as_mut_ptr(), &mut rar, &mut rars,
            )
        };
        assert_eq!(sc, sr, "RtlCopySidAndAttributesArray status n={n}");
        if sc == 0 {
            assert_eq!(ac, ar, "copied SID area n={n}");
            assert_eq!(racs, rars, "remaining size n={n}");
            // remaining-area offset within the (distinct) buffers
            let oc = rac as usize - ac.as_ptr() as usize;
            let or = rar as usize - ar.as_ptr() as usize;
            assert_eq!(oc, or, "remaining-area offset n={n}");
            for i in 0..n as usize {
                assert_eq!(dst_c[i].attributes, dst_r[i].attributes);
                let dc = dst_c[i].sid as usize - ac.as_ptr() as usize;
                let dr = dst_r[i].sid as usize - ar.as_ptr() as usize;
                assert_eq!(dc, dr, "dest[{i}].Sid offset");
            }
        }
    }
}

#[test]
fn diff_rtl_allocate_and_initialize_sid() {
    let mut rng = Rng(0x51D0_0009);
    for _ in 0..2_000 {
        let count = rng.byte() % 12;
        let auth: [u8; 6] = core::array::from_fn(|_| rng.byte());
        let mut oc: *mut u8 = core::ptr::null_mut();
        let mut or: *mut u8 = core::ptr::null_mut();
        let sc = unsafe {
            cabi::RtlAllocateAndInitializeSid(
                auth.as_ptr(), count, 1, 2, 3, 4, 5, 6, 7, 8, &mut oc,
            )
        };
        let sr = unsafe {
            ref_RtlAllocateAndInitializeSid(
                auth.as_ptr(), count, 1, 2, 3, 4, 5, 6, 7, 8, &mut or,
            )
        };
        assert_eq!(sc, sr, "RtlAllocateAndInitializeSid count={count}");
    }
}

#[test]
fn diff_rtl_convert_sid_to_unicode_string() {
    let mut rng = Rng(0x51D0_000A);
    for _ in 0..20_000 {
        // Mostly-valid SIDs; sometimes invalid (bad rev / count>15).
        let bad = rng.byte() & 7 == 0;
        let rev = if bad && rng.byte() & 1 == 0 { rng.byte() } else { 1 };
        let count = if bad && rng.byte() & 1 == 0 {
            16 + rng.byte() % 8
        } else {
            rng.byte() % 16
        };
        let mut v = mk_sid(&mut rng, rev, count.min(40));
        // ~1/3: force the zero-IdentifierAuthority "%lu" branch.
        if v.len() > 8 && rng.byte() % 3 == 0 {
            v[2] = 0;
            v[3] = 0;
        }
        let alloc = rng.byte() & 3 == 0;

        // Caller buffer with a fuzzed MaximumLength (force the
        // too-small / exact / larger boundaries).
        let mut bc = [0u16; 300];
        let mut br = [0u16; 300];
        let max: u16 = match rng.byte() % 4 {
            0 => 0,
            1 => (rng.byte() as u16) & 0x7E, // small, even
            2 => 400,                        // ample
            _ => (rng.byte() as u16) * 2,
        };
        let mut sc = UNICODE_STRING {
            Length: 0xAAAA,
            MaximumLength: max,
            Buffer: bc.as_mut_ptr(),
        };
        let mut sr = UNICODE_STRING {
            Length: 0xAAAA,
            MaximumLength: max,
            Buffer: br.as_mut_ptr(),
        };
        let rc = unsafe {
            cabi::RtlConvertSidToUnicodeString(&mut sc, v.as_ptr(), alloc as u8)
        };
        let rr = unsafe {
            ref_RtlConvertSidToUnicodeString(&mut sr, v.as_ptr(), alloc as u8)
        };
        assert_eq!(
            rc, rr,
            "RtlConvertSidToUnicodeString status rev={rev} count={count} alloc={alloc} max={max}"
        );
        if rc == 0 && !alloc {
            assert_eq!(sc.Length, sr.Length, "Length");
            let nc = (sc.Length / 2) as usize;
            assert_eq!(&bc[..nc], &br[..nc], "Buffer text");
            // NUL only when Length < MaximumLength
            if (sc.Length as u16) < max {
                assert_eq!(bc[nc], 0);
                assert_eq!(br[nc], 0);
            }
        }
    }
}

#[test]
fn diff_rtl_create_service_sid() {
    let us = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut len = 0u32;
    let sc = unsafe {
        cabi::RtlCreateServiceSid(&us, core::ptr::null_mut(), &mut len)
    };
    let sr = unsafe {
        ref_RtlCreateServiceSid(&us, core::ptr::null_mut(), &mut len)
    };
    assert_eq!(sc, sr);
}

// ── Wave ACL-2: ACE add/delete + workers + typed wrappers ─────────

/// An empty ACL `[rev,0,cap(le16),0,0,0]` of `cap` bytes (+16 slack).
fn mk_empty_acl(rev: u8, cap: u16) -> Vec<u8> {
    let cap = (cap | 1) ^ 1; // USHORT-align (even)
    let mut v = vec![0u8; cap as usize + 16];
    v[0] = rev;
    v[2..4].copy_from_slice(&cap.to_le_bytes());
    v
}

#[test]
fn diff_rtl_add_known_ace_via_wrappers() {
    let mut rng = Rng(0xAC20_0001);
    for _ in 0..15_000 {
        let acl_rev = 2 + (rng.byte() % 4) as u32; // incl 5 (bad)
        let cap = 8 + ((rng.byte() as u16) & 0xFE) + 8;
        let mut bc = mk_empty_acl(acl_rev as u8, cap);
        let mut br = bc.clone();
        // Up to a few Add* operations on the same evolving ACL.
        for _ in 0..(1 + rng.byte() % 4) {
            let sub = rng.byte() % 4;
            let sid = mk_sid(&mut rng, 1, sub);
            let rev = (rng.byte() % 6) as u32;
            let flags = rng.next() as u32 & 0xFF;
            let mask = rng.next() as u32;
            let kind = rng.byte() % 5;
            let (sc, sr) = unsafe {
                match kind {
                    0 => (
                        cabi::RtlAddAccessAllowedAce(bc.as_mut_ptr(), rev, mask, sid.as_ptr()),
                        ref_RtlAddAccessAllowedAce(br.as_mut_ptr(), rev, mask, sid.as_ptr()),
                    ),
                    1 => (
                        cabi::RtlAddAccessAllowedAceEx(bc.as_mut_ptr(), rev, flags, mask, sid.as_ptr()),
                        ref_RtlAddAccessAllowedAceEx(br.as_mut_ptr(), rev, flags, mask, sid.as_ptr()),
                    ),
                    2 => (
                        cabi::RtlAddAccessDeniedAce(bc.as_mut_ptr(), rev, mask, sid.as_ptr()),
                        ref_RtlAddAccessDeniedAce(br.as_mut_ptr(), rev, mask, sid.as_ptr()),
                    ),
                    3 => {
                        let su = rng.byte() & 1;
                        let fa = rng.byte() & 1;
                        (
                            cabi::RtlAddAuditAccessAce(bc.as_mut_ptr(), rev, mask, sid.as_ptr(), su, fa),
                            ref_RtlAddAuditAccessAce(br.as_mut_ptr(), rev, mask, sid.as_ptr(), su, fa),
                        )
                    }
                    _ => (
                        cabi::RtlpAddKnownAce(bc.as_mut_ptr(), rev, flags, mask, sid.as_ptr(), 2),
                        ref_RtlpAddKnownAce(br.as_mut_ptr(), rev, flags, mask, sid.as_ptr(), 2),
                    ),
                }
            };
            assert_eq!(sc, sr, "AddKnownAce status kind={kind} rev={rev}");
            assert_eq!(bc, br, "AddKnownAce buffer kind={kind} rev={rev}");
        }
    }
}

#[test]
fn diff_rtl_add_known_object_ace() {
    let mut rng = Rng(0xAC20_0002);
    for _ in 0..10_000 {
        let acl_rev = 2 + (rng.byte() % 3) as u8;
        let cap = 8 + ((rng.byte() as u16) & 0xFE) + 64;
        let mut bc = mk_empty_acl(acl_rev, cap);
        let mut br = bc.clone();
        let sub = rng.byte() % 3;
        let sid = mk_sid(&mut rng, 1, sub);
        let rev = if rng.byte() & 1 == 0 { 4 } else { (rng.byte() % 6) as u32 };
        let flags = rng.next() as u32 & 0xFF;
        let mask = rng.next() as u32;
        let g0: [u8; 16] = core::array::from_fn(|_| rng.byte());
        let g1: [u8; 16] = core::array::from_fn(|_| rng.byte());
        let otg = if rng.byte() & 1 == 0 { g0.as_ptr() } else { core::ptr::null() };
        let iotg = if rng.byte() & 1 == 0 { g1.as_ptr() } else { core::ptr::null() };
        let sc = unsafe {
            cabi::RtlAddAccessAllowedObjectAce(bc.as_mut_ptr(), rev, flags, mask, otg, iotg, sid.as_ptr())
        };
        let sr = unsafe {
            ref_RtlAddAccessAllowedObjectAce(br.as_mut_ptr(), rev, flags, mask, otg, iotg, sid.as_ptr())
        };
        assert_eq!(sc, sr, "AddObjectAce status rev={rev}");
        assert_eq!(bc, br, "AddObjectAce buffer rev={rev}");
    }
}

#[test]
fn diff_rtl_p_add_delete_data() {
    let mut rng = Rng(0xAC20_0003);
    for _ in 0..20_000 {
        let n = 32 + rng.upto(64);
        let base: Vec<u8> = (0..n).map(|_| rng.byte()).collect();
        let ace_off = rng.upto(n / 2);
        let max_tail = n - ace_off;
        // RtlpAddData — non-overlapping domain (all==0 or all>=off).
        {
            let off = rng.upto(max_tail.max(1));
            let all = if rng.byte() & 1 == 0 { 0 } else { off + rng.upto(8) };
            if ace_off + all + off + 8 <= n {
                let payload: Vec<u8> = (0..all).map(|_| rng.byte()).collect();
                let mut bc = base.clone();
                let mut br = base.clone();
                unsafe {
                    cabi::RtlpAddData(payload.as_ptr(), all as u32, bc.as_mut_ptr().add(ace_off), off as u32);
                    ref_RtlpAddData(payload.as_ptr(), all as u32, br.as_mut_ptr().add(ace_off), off as u32);
                }
                assert_eq!(bc, br, "RtlpAddData all={all} off={off}");
            }
        }
        // RtlpDeleteData.
        {
            let off = rng.upto(max_tail.max(1));
            let asz = rng.upto(off + 1);
            let mut bc = base.clone();
            let mut br = base.clone();
            unsafe {
                cabi::RtlpDeleteData(bc.as_mut_ptr().add(ace_off), asz as u32, off as u32);
                ref_RtlpDeleteData(br.as_mut_ptr().add(ace_off), asz as u32, off as u32);
            }
            assert_eq!(bc, br, "RtlpDeleteData asz={asz} off={off}");
        }
    }
}

#[test]
fn diff_rtl_add_and_delete_ace() {
    let mut rng = Rng(0xAC20_0004);
    for _ in 0..8_000 {
        // Build a populated ACL with free space.
        let n = rng.byte() % 4;
        let rev = 2 + rng.byte() % 3;
        let slack = 64u16;
        let mut bc = mk_acl(&mut rng, rev, n, slack);
        let mut br = bc.clone();

        // RtlAddAce: a small ACE list (one ACCESS_ALLOWED ACE).
        let sub = rng.byte() % 3;
        let sid = mk_sid(&mut rng, 1, sub);
        let sidlen = 8 + 4 * sub as usize;
        let asz = 8 + sidlen;
        let mut list = vec![0u8; asz];
        list[0] = 0;
        list[1] = (rng.byte() & 0x1F) as u8;
        list[2..4].copy_from_slice(&(asz as u16).to_le_bytes());
        let m = rng.next() as u32;
        list[4..8].copy_from_slice(&m.to_le_bytes());
        list[8..8 + sidlen].copy_from_slice(&sid[..sidlen]);
        let start_idx = (rng.byte() % (n + 2)) as u32;
        let arev = 2 + (rng.byte() % 4) as u32;
        let ac = unsafe {
            cabi::RtlAddAce(bc.as_mut_ptr(), arev, start_idx, list.as_mut_ptr(), asz as u32)
        };
        let ar = unsafe {
            ref_RtlAddAce(br.as_mut_ptr(), arev, start_idx, list.as_mut_ptr(), asz as u32)
        };
        assert_eq!(ac, ar, "RtlAddAce status n={n} idx={start_idx}");
        assert_eq!(bc, br, "RtlAddAce buffer n={n} idx={start_idx}");

        // RtlDeleteAce an index.
        let di = (rng.byte() % (n + 2)) as u32;
        let dc = unsafe { cabi::RtlDeleteAce(bc.as_mut_ptr(), di) };
        let dr = unsafe { ref_RtlDeleteAce(br.as_mut_ptr(), di) };
        assert_eq!(dc, dr, "RtlDeleteAce status di={di}");
        assert_eq!(bc, br, "RtlDeleteAce buffer di={di}");
    }
}

// ============================================================================
// Wave SD-1 — sdk/lib/rtl/sd.c structural surface.
//
// SECURITY_DESCRIPTOR (absolute) on host x86_64: 1+1+2 header + 4 pad
// + 4*8 pointers = 40 bytes. SECURITY_DESCRIPTOR_RELATIVE: flat 20
// bytes everywhere. All tests build per-side byte buffers, cast to the
// cabi struct on the Rust side and the ref TU struct on the C side,
// and compare both the returned NTSTATUS/values AND the buffer bytes.
// ============================================================================

use rtl_strings::cabi::{SECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_RELATIVE};

extern "C" {
    fn ref_RtlCreateSecurityDescriptor(sd: *mut u8, rev: u32) -> i32;
    fn ref_RtlCreateSecurityDescriptorRelative(sd: *mut u8, rev: u32) -> i32;
    fn ref_RtlLengthSecurityDescriptor(sd: *const u8) -> u32;
    fn ref_RtlGetDaclSecurityDescriptor(
        sd: *const u8,
        present: *mut u8,
        dacl: *mut *mut u8,
        defaulted: *mut u8,
    ) -> i32;
    fn ref_RtlGetSaclSecurityDescriptor(
        sd: *const u8,
        present: *mut u8,
        sacl: *mut *mut u8,
        defaulted: *mut u8,
    ) -> i32;
    fn ref_RtlGetOwnerSecurityDescriptor(
        sd: *const u8,
        owner: *mut *mut u8,
        defaulted: *mut u8,
    ) -> i32;
    fn ref_RtlGetGroupSecurityDescriptor(
        sd: *const u8,
        group: *mut *mut u8,
        defaulted: *mut u8,
    ) -> i32;
    fn ref_RtlSetDaclSecurityDescriptor(
        sd: *mut u8,
        present: u8,
        dacl: *mut u8,
        defaulted: u8,
    ) -> i32;
    fn ref_RtlSetSaclSecurityDescriptor(
        sd: *mut u8,
        present: u8,
        sacl: *mut u8,
        defaulted: u8,
    ) -> i32;
    fn ref_RtlSetOwnerSecurityDescriptor(sd: *mut u8, owner: *mut u8, defaulted: u8) -> i32;
    fn ref_RtlSetGroupSecurityDescriptor(sd: *mut u8, group: *mut u8, defaulted: u8) -> i32;
    fn ref_RtlGetControlSecurityDescriptor(sd: *const u8, ctl: *mut u16, rev: *mut u32) -> i32;
    fn ref_RtlSetControlSecurityDescriptor(sd: *mut u8, of_interest: u16, to_set: u16) -> i32;
    fn ref_RtlGetSecurityDescriptorRMControl(sd: *const u8, rm: *mut u8) -> u8;
    fn ref_RtlSetSecurityDescriptorRMControl(sd: *mut u8, rm: *const u8);
    fn ref_RtlSetAttributesSecurityDescriptor(sd: *mut u8, ctl: u16, rev: *mut u32) -> i32;
    fn ref_RtlValidSecurityDescriptor(sd: *const u8) -> u8;
    fn ref_RtlValidRelativeSecurityDescriptor(sd: *const u8, len: u32, info: u32) -> u8;
}

/// `sizeof(SECURITY_DESCRIPTOR)` host: 40 (1+1+2+4 pad + 4*8 ptr).
const SD_SIZE: usize = core::mem::size_of::<SECURITY_DESCRIPTOR>();
const SDREL_SIZE: usize = core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>();

#[test]
fn diff_rtl_create_security_descriptor() {
    let mut rng = Rng(0x5D10_0001);
    for _ in 0..2_000 {
        // Pre-fill with junk so the "zero rest of struct" effect shows.
        let mut bc = vec![0xCDu8; SD_SIZE];
        let mut br = bc.clone();
        for b in bc.iter_mut() {
            *b = rng.byte();
        }
        br.copy_from_slice(&bc);
        let rev = if rng.byte() & 1 == 0 { 1 } else { rng.byte() as u32 };
        let sc = unsafe { cabi::RtlCreateSecurityDescriptor(bc.as_mut_ptr() as _, rev) };
        let sr = unsafe { ref_RtlCreateSecurityDescriptor(br.as_mut_ptr(), rev) };
        assert_eq!(sc, sr, "RtlCreateSecurityDescriptor rev={rev:#x}");
        assert_eq!(bc, br, "RtlCreateSecurityDescriptor buf rev={rev:#x}");
    }
}

#[test]
fn diff_rtl_create_security_descriptor_relative() {
    let mut rng = Rng(0x5D10_0002);
    for _ in 0..2_000 {
        let mut bc = vec![0xABu8; SDREL_SIZE];
        for b in bc.iter_mut() {
            *b = rng.byte();
        }
        let mut br = bc.clone();
        let rev = if rng.byte() & 1 == 0 { 1 } else { rng.byte() as u32 };
        let sc = unsafe { cabi::RtlCreateSecurityDescriptorRelative(bc.as_mut_ptr() as _, rev) };
        let sr = unsafe { ref_RtlCreateSecurityDescriptorRelative(br.as_mut_ptr(), rev) };
        assert_eq!(sc, sr, "RtlCreateSDRel rev={rev:#x}");
        assert_eq!(bc, br, "RtlCreateSDRel buf rev={rev:#x}");
    }
}

/// Build an absolute SD with random combinations of owner/group/dacl/
/// sacl present + the matching presence flags. Returns the SD buffer
/// (40 bytes) plus owned backing vectors keyed alive for the test
/// scope. The DACL/SACL bytes are well-formed (use mk_acl); SIDs use
/// mk_sid. Identical byte sequences on Rust + C sides.
fn mk_absolute_sd(
    rng: &mut Rng,
) -> (Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>) {
    let mut sd = vec![0u8; SD_SIZE];
    sd[0] = 1; // Revision
    sd[1] = 0; // Sbz1

    let owner_sub = rng.byte() % 4;
    let group_sub = rng.byte() % 4;
    let n_dacl = rng.byte() % 3;
    let n_sacl = rng.byte() % 3;

    let owner = if rng.byte() & 1 != 0 {
        Some(mk_sid(rng, 1, owner_sub))
    } else {
        None
    };
    let group = if rng.byte() & 1 != 0 {
        Some(mk_sid(rng, 1, group_sub))
    } else {
        None
    };
    let dacl_present = rng.byte() & 1 != 0;
    let dacl = if dacl_present {
        Some(mk_acl(rng, 2, n_dacl, 0))
    } else {
        None
    };
    let sacl_present = rng.byte() & 1 != 0;
    let sacl = if sacl_present {
        Some(mk_acl(rng, 2, n_sacl, 0))
    } else {
        None
    };

    let mut control: u16 = 0;
    if rng.byte() & 1 != 0 && owner.is_some() {
        control |= 0x0001;
    } // OWNER_DEFAULTED
    if rng.byte() & 1 != 0 && group.is_some() {
        control |= 0x0002;
    } // GROUP_DEFAULTED
    if dacl_present {
        control |= 0x0004;
    } // DACL_PRESENT
    if dacl_present && rng.byte() & 1 != 0 {
        control |= 0x0008;
    } // DACL_DEFAULTED
    if sacl_present {
        control |= 0x0010;
    } // SACL_PRESENT
    if sacl_present && rng.byte() & 1 != 0 {
        control |= 0x0020;
    } // SACL_DEFAULTED
    sd[2..4].copy_from_slice(&control.to_le_bytes());

    // Pointers at offsets 8 / 16 / 24 / 32 (after 4-byte pad).
    let owner_ptr = owner.as_ref().map(|v| v.as_ptr() as usize).unwrap_or(0);
    let group_ptr = group.as_ref().map(|v| v.as_ptr() as usize).unwrap_or(0);
    let sacl_ptr = sacl.as_ref().map(|v| v.as_ptr() as usize).unwrap_or(0);
    let dacl_ptr = dacl.as_ref().map(|v| v.as_ptr() as usize).unwrap_or(0);
    sd[8..16].copy_from_slice(&(owner_ptr as u64).to_le_bytes());
    sd[16..24].copy_from_slice(&(group_ptr as u64).to_le_bytes());
    sd[24..32].copy_from_slice(&(sacl_ptr as u64).to_le_bytes());
    sd[32..40].copy_from_slice(&(dacl_ptr as u64).to_le_bytes());

    (sd, owner, group, dacl, sacl)
}

#[test]
fn diff_rtl_length_security_descriptor() {
    let mut rng = Rng(0x5D10_0003);
    for _ in 0..2_000 {
        let (sd, _o, _g, _d, _s) = mk_absolute_sd(&mut rng);
        let lc = unsafe { cabi::RtlLengthSecurityDescriptor(sd.as_ptr() as _) };
        let lr = unsafe { ref_RtlLengthSecurityDescriptor(sd.as_ptr()) };
        assert_eq!(lc, lr, "RtlLengthSecurityDescriptor");
    }
}

#[test]
fn diff_rtl_get_security_descriptor_fields() {
    let mut rng = Rng(0x5D10_0004);
    for _ in 0..2_000 {
        let (sd, _o, _g, _d, _s) = mk_absolute_sd(&mut rng);

        // RtlGetDacl
        let (mut pc, mut pr) = (0u8, 0u8);
        let (mut dc_ptr, mut dr_ptr) = (core::ptr::null_mut(), core::ptr::null_mut());
        let (mut fc, mut fr) = (0u8, 0u8);
        let sc = unsafe {
            cabi::RtlGetDaclSecurityDescriptor(sd.as_ptr() as _, &mut pc, &mut dc_ptr, &mut fc)
        };
        let sr = unsafe {
            ref_RtlGetDaclSecurityDescriptor(sd.as_ptr(), &mut pr, &mut dr_ptr, &mut fr)
        };
        assert_eq!((sc, pc, dc_ptr as usize, fc), (sr, pr, dr_ptr as usize, fr), "GetDacl");

        // RtlGetSacl
        let (mut pc, mut pr) = (0u8, 0u8);
        let (mut dc_ptr, mut dr_ptr) = (core::ptr::null_mut(), core::ptr::null_mut());
        let (mut fc, mut fr) = (0u8, 0u8);
        let sc = unsafe {
            cabi::RtlGetSaclSecurityDescriptor(sd.as_ptr() as _, &mut pc, &mut dc_ptr, &mut fc)
        };
        let sr = unsafe {
            ref_RtlGetSaclSecurityDescriptor(sd.as_ptr(), &mut pr, &mut dr_ptr, &mut fr)
        };
        assert_eq!((sc, pc, dc_ptr as usize, fc), (sr, pr, dr_ptr as usize, fr), "GetSacl");

        // RtlGetOwner
        let (mut dc_ptr, mut dr_ptr) = (core::ptr::null_mut(), core::ptr::null_mut());
        let (mut fc, mut fr) = (0u8, 0u8);
        let sc = unsafe {
            cabi::RtlGetOwnerSecurityDescriptor(sd.as_ptr() as _, &mut dc_ptr, &mut fc)
        };
        let sr = unsafe { ref_RtlGetOwnerSecurityDescriptor(sd.as_ptr(), &mut dr_ptr, &mut fr) };
        assert_eq!((sc, dc_ptr as usize, fc), (sr, dr_ptr as usize, fr), "GetOwner");

        // RtlGetGroup
        let (mut dc_ptr, mut dr_ptr) = (core::ptr::null_mut(), core::ptr::null_mut());
        let (mut fc, mut fr) = (0u8, 0u8);
        let sc = unsafe {
            cabi::RtlGetGroupSecurityDescriptor(sd.as_ptr() as _, &mut dc_ptr, &mut fc)
        };
        let sr = unsafe { ref_RtlGetGroupSecurityDescriptor(sd.as_ptr(), &mut dr_ptr, &mut fr) };
        assert_eq!((sc, dc_ptr as usize, fc), (sr, dr_ptr as usize, fr), "GetGroup");

        // RtlGetControl
        let (mut cc, mut cr) = (0u16, 0u16);
        let (mut revc, mut revr) = (0u32, 0u32);
        let sc =
            unsafe { cabi::RtlGetControlSecurityDescriptor(sd.as_ptr() as _, &mut cc, &mut revc) };
        let sr =
            unsafe { ref_RtlGetControlSecurityDescriptor(sd.as_ptr(), &mut cr, &mut revr) };
        assert_eq!((sc, cc, revc), (sr, cr, revr), "GetControl");

        // RtlGetSecurityDescriptorRMControl (only meaningful if RM_CONTROL_VALID set,
        // but the API gives a deterministic answer either way).
        let (mut rmc, mut rmr) = (0u8, 0u8);
        let bc = unsafe { cabi::RtlGetSecurityDescriptorRMControl(sd.as_ptr() as _, &mut rmc) };
        let br = unsafe { ref_RtlGetSecurityDescriptorRMControl(sd.as_ptr(), &mut rmr) };
        assert_eq!((bc, rmc), (br, rmr), "GetRMControl");
    }
}

#[test]
fn diff_rtl_set_security_descriptor_fields() {
    let mut rng = Rng(0x5D10_0005);
    for _ in 0..2_000 {
        // Build a fresh (absolute, no self-relative) SD with assorted control bits.
        let (mut bc, owner_v, group_v, dacl_v, sacl_v) = mk_absolute_sd(&mut rng);
        // Force absolute: clear SE_SELF_RELATIVE if our mock set it (it doesn't,
        // but be explicit).
        let mut control = u16::from_le_bytes([bc[2], bc[3]]);
        control &= !0x8000;
        bc[2..4].copy_from_slice(&control.to_le_bytes());
        let mut br = bc.clone();

        // Random new pointers (use the existing owned backing) + defaulted bits.
        let new_owner = owner_v
            .as_ref()
            .map(|v| v.as_ptr() as *mut u8)
            .unwrap_or(core::ptr::null_mut());
        let new_group = group_v
            .as_ref()
            .map(|v| v.as_ptr() as *mut u8)
            .unwrap_or(core::ptr::null_mut());
        let new_dacl = dacl_v
            .as_ref()
            .map(|v| v.as_ptr() as *mut u8)
            .unwrap_or(core::ptr::null_mut());
        let new_sacl = sacl_v
            .as_ref()
            .map(|v| v.as_ptr() as *mut u8)
            .unwrap_or(core::ptr::null_mut());

        let dpres = rng.byte() & 1;
        let ddef = rng.byte() & 1;
        let sc = unsafe { cabi::RtlSetDaclSecurityDescriptor(bc.as_mut_ptr() as _, dpres, new_dacl, ddef) };
        let sr = unsafe { ref_RtlSetDaclSecurityDescriptor(br.as_mut_ptr(), dpres, new_dacl, ddef) };
        assert_eq!(sc, sr, "SetDacl");
        assert_eq!(bc, br, "SetDacl buf");

        let spres = rng.byte() & 1;
        let sdef = rng.byte() & 1;
        let sc = unsafe { cabi::RtlSetSaclSecurityDescriptor(bc.as_mut_ptr() as _, spres, new_sacl, sdef) };
        let sr = unsafe { ref_RtlSetSaclSecurityDescriptor(br.as_mut_ptr(), spres, new_sacl, sdef) };
        assert_eq!(sc, sr, "SetSacl");
        assert_eq!(bc, br, "SetSacl buf");

        let odef = rng.byte() & 1;
        let sc = unsafe { cabi::RtlSetOwnerSecurityDescriptor(bc.as_mut_ptr() as _, new_owner, odef) };
        let sr = unsafe { ref_RtlSetOwnerSecurityDescriptor(br.as_mut_ptr(), new_owner, odef) };
        assert_eq!(sc, sr, "SetOwner");
        assert_eq!(bc, br, "SetOwner buf");

        let gdef = rng.byte() & 1;
        let sc = unsafe { cabi::RtlSetGroupSecurityDescriptor(bc.as_mut_ptr() as _, new_group, gdef) };
        let sr = unsafe { ref_RtlSetGroupSecurityDescriptor(br.as_mut_ptr(), new_group, gdef) };
        assert_eq!(sc, sr, "SetGroup");
        assert_eq!(bc, br, "SetGroup buf");

        // SetControl — random control bits and a partial mask.
        let of_int = (rng.next() as u16) & 0x3FC0; // mostly in SET_ATTRIBUTES_MASK + invalid bits
        let to_set = (rng.next() as u16) & 0x3FC0;
        let sc = unsafe {
            cabi::RtlSetControlSecurityDescriptor(bc.as_mut_ptr() as _, of_int, to_set)
        };
        let sr =
            unsafe { ref_RtlSetControlSecurityDescriptor(br.as_mut_ptr(), of_int, to_set) };
        assert_eq!(sc, sr, "SetControl");
        assert_eq!(bc, br, "SetControl buf");

        // SetAttributes
        let attr = (rng.next() as u16) & 0xFFFF;
        let (mut revc, mut revr) = (0u32, 0u32);
        let sc = unsafe {
            cabi::RtlSetAttributesSecurityDescriptor(bc.as_mut_ptr() as _, attr, &mut revc)
        };
        let sr = unsafe {
            ref_RtlSetAttributesSecurityDescriptor(br.as_mut_ptr(), attr, &mut revr)
        };
        assert_eq!((sc, revc), (sr, revr), "SetAttributes");
        assert_eq!(bc, br, "SetAttributes buf");

        // SetSecurityDescriptorRMControl — both NULL and non-NULL.
        let raw = rng.byte();
        let p = if rng.byte() & 1 != 0 { &raw as *const u8 } else { core::ptr::null() };
        unsafe { cabi::RtlSetSecurityDescriptorRMControl(bc.as_mut_ptr() as _, p) };
        unsafe { ref_RtlSetSecurityDescriptorRMControl(br.as_mut_ptr(), p) };
        assert_eq!(bc, br, "SetRMControl buf");
    }
}

#[test]
fn diff_rtl_valid_security_descriptor() {
    let mut rng = Rng(0x5D10_0006);
    for _ in 0..3_000 {
        let (sd, _o, _g, _d, _s) = mk_absolute_sd(&mut rng);
        let bc = unsafe { cabi::RtlValidSecurityDescriptor(sd.as_ptr() as _) };
        let br = unsafe { ref_RtlValidSecurityDescriptor(sd.as_ptr()) };
        assert_eq!(bc, br, "RtlValidSecurityDescriptor (well-formed)");

        // Sprinkle a bad rev in 1/4 of iterations.
        if rng.byte() & 3 == 0 {
            let mut sd2 = sd.clone();
            sd2[0] = rng.byte();
            let bc = unsafe { cabi::RtlValidSecurityDescriptor(sd2.as_ptr() as _) };
            let br = unsafe { ref_RtlValidSecurityDescriptor(sd2.as_ptr()) };
            assert_eq!(bc, br, "RtlValidSecurityDescriptor (rev={:#x})", sd2[0]);
        }
    }
}

/// Build a self-relative SD as a flat byte buffer. Layout: 20-byte
/// SECURITY_DESCRIPTOR_RELATIVE header followed by ULONG-aligned SACL,
/// DACL, Owner, Group payloads at offsets stored in the header. This
/// is exactly the shape `RtlMakeSelfRelativeSD` emits.
fn mk_relative_sd(rng: &mut Rng) -> Vec<u8> {
    let mut header = [0u8; SDREL_SIZE];
    header[0] = 1;
    let mut payload: Vec<u8> = Vec::new();
    let mut control: u16 = 0x8000; // SE_SELF_RELATIVE
    let mut current_off: u32 = SDREL_SIZE as u32;

    let owner_off;
    if rng.byte() & 1 != 0 {
        let sub = rng.byte() % 4;
        let sid = mk_sid(rng, 1, sub);
        let len = (8 + 4 * sub as usize + 3) & !3;
        payload.extend_from_slice(&sid[..len.min(sid.len())]);
        while payload.len() % 4 != 0 {
            payload.push(0);
        }
        owner_off = current_off;
        current_off = SDREL_SIZE as u32 + payload.len() as u32;
        if rng.byte() & 1 != 0 {
            control |= 0x0001;
        }
    } else {
        owner_off = 0;
    }
    let group_off;
    if rng.byte() & 1 != 0 {
        let sub = rng.byte() % 4;
        let sid = mk_sid(rng, 1, sub);
        let len = (8 + 4 * sub as usize + 3) & !3;
        payload.extend_from_slice(&sid[..len.min(sid.len())]);
        while payload.len() % 4 != 0 {
            payload.push(0);
        }
        group_off = current_off;
        current_off = SDREL_SIZE as u32 + payload.len() as u32;
        if rng.byte() & 1 != 0 {
            control |= 0x0002;
        }
    } else {
        group_off = 0;
    }
    let dacl_off;
    if rng.byte() & 1 != 0 {
        let n = rng.byte() % 3;
        let acl = mk_acl(rng, 2, n, 0);
        payload.extend_from_slice(&acl);
        while payload.len() % 4 != 0 {
            payload.push(0);
        }
        dacl_off = current_off;
        current_off = SDREL_SIZE as u32 + payload.len() as u32;
        control |= 0x0004;
    } else {
        dacl_off = 0;
    }
    let sacl_off;
    if rng.byte() & 1 != 0 {
        let n = rng.byte() % 3;
        let acl = mk_acl(rng, 2, n, 0);
        payload.extend_from_slice(&acl);
        while payload.len() % 4 != 0 {
            payload.push(0);
        }
        sacl_off = current_off;
        let _ = current_off;
        control |= 0x0010;
    } else {
        sacl_off = 0;
    }

    header[2..4].copy_from_slice(&control.to_le_bytes());
    header[4..8].copy_from_slice(&owner_off.to_le_bytes());
    header[8..12].copy_from_slice(&group_off.to_le_bytes());
    header[12..16].copy_from_slice(&sacl_off.to_le_bytes());
    header[16..20].copy_from_slice(&dacl_off.to_le_bytes());

    let mut out = Vec::with_capacity(SDREL_SIZE + payload.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&payload);
    out
}

#[test]
fn diff_rtl_valid_relative_security_descriptor() {
    let mut rng = Rng(0x5D10_0007);
    for _ in 0..3_000 {
        let sd = mk_relative_sd(&mut rng);
        let req = (rng.next() & 0xF) as u32;
        let bc = unsafe {
            cabi::RtlValidRelativeSecurityDescriptor(sd.as_ptr(), sd.len() as u32, req)
        };
        let br =
            unsafe { ref_RtlValidRelativeSecurityDescriptor(sd.as_ptr(), sd.len() as u32, req) };
        assert_eq!(bc, br, "RtlValidRelativeSecurityDescriptor (well-formed)");

        // Try a too-short length in 1/3 of iterations to drive the
        // buffer-too-small path.
        if rng.byte() % 3 == 0 && sd.len() > SDREL_SIZE + 1 {
            let short = SDREL_SIZE + (rng.byte() as usize % (sd.len() - SDREL_SIZE));
            let bc = unsafe {
                cabi::RtlValidRelativeSecurityDescriptor(sd.as_ptr(), short as u32, req)
            };
            let br = unsafe {
                ref_RtlValidRelativeSecurityDescriptor(sd.as_ptr(), short as u32, req)
            };
            assert_eq!(bc, br, "RtlValidRelativeSD (truncated, len={short})");
        }
    }
}

// ============================================================================
// Wave SD-2 — self-relative ⇄ absolute + Copy.
// ============================================================================

extern "C" {
    fn ref_RtlMakeSelfRelativeSD(src: *const u8, rel: *mut u8, len: *mut u32) -> i32;
    fn ref_RtlAbsoluteToSelfRelativeSD(src: *const u8, rel: *mut u8, len: *mut u32) -> i32;
    fn ref_RtlSelfRelativeToAbsoluteSD(
        rel: *const u8,
        abs: *mut u8,
        abs_sz: *mut u32,
        dacl: *mut u8,
        dacl_sz: *mut u32,
        sacl: *mut u8,
        sacl_sz: *mut u32,
        owner: *mut u8,
        owner_sz: *mut u32,
        group: *mut u8,
        group_sz: *mut u32,
    ) -> i32;
    fn ref_RtlSelfRelativeToAbsoluteSD2(sd: *mut u8, buf_sz: *mut u32) -> i32;
    fn ref_RtlCopySecurityDescriptor(src: *const u8, dst: *mut *mut u8) -> i32;
}

#[test]
fn diff_rtl_make_self_relative_sd() {
    let mut rng = Rng(0x5D20_0001);
    for _ in 0..2_000 {
        let (sd, _o, _g, _d, _s) = mk_absolute_sd(&mut rng);

        // First call with a too-small buffer to drive the BUFFER_TOO_SMALL
        // path AND read the needed length.
        let mut bufc = vec![0u8; 8];
        let mut bufr = bufc.clone();
        let mut lenc = 0u32;
        let mut lenr = 0u32;
        let rc = unsafe {
            cabi::RtlMakeSelfRelativeSD(sd.as_ptr() as _, bufc.as_mut_ptr() as _, &mut lenc)
        };
        let rr = unsafe {
            ref_RtlMakeSelfRelativeSD(sd.as_ptr(), bufr.as_mut_ptr(), &mut lenr)
        };
        assert_eq!(rc, rr, "MakeSelfRelative (too-small)");
        assert_eq!(lenc, lenr, "MakeSelfRelative needed-len");
        assert_eq!(bufc, bufr, "MakeSelfRelative untouched buf");

        // Now allocate the full size and emit.
        let mut bufc = vec![0u8; lenc as usize];
        let mut bufr = bufc.clone();
        let mut lenc2 = lenc;
        let mut lenr2 = lenr;
        let rc = unsafe {
            cabi::RtlMakeSelfRelativeSD(sd.as_ptr() as _, bufc.as_mut_ptr() as _, &mut lenc2)
        };
        let rr = unsafe {
            ref_RtlMakeSelfRelativeSD(sd.as_ptr(), bufr.as_mut_ptr(), &mut lenr2)
        };
        assert_eq!(rc, rr, "MakeSelfRelative status");
        assert_eq!(lenc2, lenr2, "MakeSelfRelative len");
        // The byte payload is identical because the layout (SACL→DACL→
        // Owner→Group) is identical on both sides.
        assert_eq!(bufc, bufr, "MakeSelfRelative buf");

        // RtlAbsoluteToSelfRelativeSD is the same logic + a relative-flag
        // rejection at the front; pin it too.
        let mut lenc3 = lenc;
        let mut lenr3 = lenr;
        let mut bc = vec![0u8; lenc as usize];
        let mut br = bc.clone();
        let rc = unsafe {
            cabi::RtlAbsoluteToSelfRelativeSD(
                sd.as_ptr() as _,
                bc.as_mut_ptr() as _,
                &mut lenc3,
            )
        };
        let rr = unsafe {
            ref_RtlAbsoluteToSelfRelativeSD(sd.as_ptr(), br.as_mut_ptr(), &mut lenr3)
        };
        assert_eq!((rc, lenc3, &bc), (rr, lenr3, &br), "AbsoluteToSelfRelativeSD");
    }
}

#[test]
fn diff_rtl_self_relative_to_absolute_sd() {
    let mut rng = Rng(0x5D20_0002);
    for _ in 0..2_000 {
        let rel = mk_relative_sd(&mut rng);

        // Output buffers — start zero and big enough.
        let mut abs_c = vec![0u8; SD_SIZE];
        let mut abs_r = vec![0u8; SD_SIZE];
        let mut sacl_c = vec![0u8; 256];
        let mut sacl_r = vec![0u8; 256];
        let mut dacl_c = vec![0u8; 256];
        let mut dacl_r = vec![0u8; 256];
        let mut owner_c = vec![0u8; 64];
        let mut owner_r = vec![0u8; 64];
        let mut group_c = vec![0u8; 64];
        let mut group_r = vec![0u8; 64];

        // Sometimes ask with too-small sizes (sprinkle).
        let undersized = rng.byte() % 3 == 0;
        let (abs_sz_i, sacl_sz_i, dacl_sz_i, owner_sz_i, group_sz_i) = if undersized {
            (4u32, 4u32, 4u32, 4u32, 4u32)
        } else {
            (
                SD_SIZE as u32,
                sacl_c.len() as u32,
                dacl_c.len() as u32,
                owner_c.len() as u32,
                group_c.len() as u32,
            )
        };
        let mut a = abs_sz_i;
        let mut s = sacl_sz_i;
        let mut d = dacl_sz_i;
        let mut o = owner_sz_i;
        let mut g = group_sz_i;
        let (mut ar, mut sr_, mut dr, mut or_, mut gr) = (a, s, d, o, g);

        let rc = unsafe {
            cabi::RtlSelfRelativeToAbsoluteSD(
                rel.as_ptr() as _,
                abs_c.as_mut_ptr() as _,
                &mut a,
                dacl_c.as_mut_ptr(),
                &mut d,
                sacl_c.as_mut_ptr(),
                &mut s,
                owner_c.as_mut_ptr(),
                &mut o,
                group_c.as_mut_ptr(),
                &mut g,
            )
        };
        let rr = unsafe {
            ref_RtlSelfRelativeToAbsoluteSD(
                rel.as_ptr(),
                abs_r.as_mut_ptr(),
                &mut ar,
                dacl_r.as_mut_ptr(),
                &mut dr,
                sacl_r.as_mut_ptr(),
                &mut sr_,
                owner_r.as_mut_ptr(),
                &mut or_,
                group_r.as_mut_ptr(),
                &mut gr,
            )
        };
        assert_eq!(rc, rr, "SelfRelativeToAbsolute status (undersized={undersized})");
        assert_eq!((a, s, d, o, g), (ar, sr_, dr, or_, gr), "needed sizes");

        if rc == 0 {
            // The absolute SD bytes will contain pointers (different in
            // the addresses pointed to by Rust vs C buffers) — compare
            // only the header bytes (first 4) plus the *content* of the
            // owner/group/dacl/sacl payload buffers.
            assert_eq!(abs_c[..4], abs_r[..4], "SD header bytes");
            // u16 control (offset 2) matches.
            assert_eq!(&abs_c[2..4], &abs_r[2..4], "SD control");
            // Payload bytes: SID/ACL bytes should be byte-identical (both
            // sides used RtlMoveMemory of the same source bytes).
            assert_eq!(owner_c, owner_r, "owner payload");
            assert_eq!(group_c, group_r, "group payload");
            assert_eq!(dacl_c, dacl_r, "dacl payload");
            assert_eq!(sacl_c, sacl_r, "sacl payload");
        }
    }
}

#[test]
fn diff_rtl_self_relative_to_absolute_sd2() {
    let mut rng = Rng(0x5D20_0003);
    for _ in 0..2_000 {
        let rel = mk_relative_sd(&mut rng);

        // Allocate generously so the move can succeed.
        let work = rel.len() + 64;
        let mut bc = vec![0u8; work];
        let mut br = bc.clone();
        bc[..rel.len()].copy_from_slice(&rel);
        br[..rel.len()].copy_from_slice(&rel);

        let mut szc = work as u32;
        let mut szr = work as u32;
        let rc = unsafe { cabi::RtlSelfRelativeToAbsoluteSD2(bc.as_mut_ptr() as _, &mut szc) };
        let rr = unsafe { ref_RtlSelfRelativeToAbsoluteSD2(br.as_mut_ptr(), &mut szr) };
        assert_eq!(rc, rr, "SelfRelativeToAbsoluteSD2 status");
        assert_eq!(szc, szr, "size");

        if rc == 0 {
            // Header bytes (rev + sbz1 + control) match.
            assert_eq!(bc[..4], br[..4], "header");
            // Pointer fields (16 bytes on 64-bit host: pointers at off 8/16/24/32)
            // differ in absolute address but match in NULL-ness.
            for i in 0..4 {
                let off = 8 + i * 8;
                let pc =
                    u64::from_le_bytes(bc[off..off + 8].try_into().unwrap()) != 0;
                let pr =
                    u64::from_le_bytes(br[off..off + 8].try_into().unwrap()) != 0;
                assert_eq!(pc, pr, "ptr[{i}] null-ness");
            }
            // The trailing payload (bytes after sizeof(SD)) is byte-identical.
            assert_eq!(&bc[SD_SIZE..], &br[SD_SIZE..], "payload");
        }
    }
}

#[test]
fn diff_rtl_copy_security_descriptor() {
    // RtlAllocateHeap is now heap-backed in the harness (Wave SEC).
    // Use mk_relative_sd inputs (self-relative, contiguous in one buffer)
    // so the C-faithful `memcpy(total_bytes_from_src)` actually copies
    // valid payload. Absolute-SD inputs would memcpy past the small
    // header struct into uninit memory — matches the C bug but breaks
    // the post-copy `RtlLengthSecurityDescriptor` walk.
    let mut rng = Rng(0x5D20_0004);
    for _ in 0..1_000 {
        let sd = mk_relative_sd(&mut rng);
        let (mut dc, mut dr): (*mut u8, *mut u8) = (core::ptr::null_mut(), core::ptr::null_mut());
        let rc = unsafe { cabi::RtlCopySecurityDescriptor(sd.as_ptr() as _, &mut dc as *mut _ as _) };
        let rr = unsafe { ref_RtlCopySecurityDescriptor(sd.as_ptr(), &mut dr) };
        assert_eq!(rc, rr, "RtlCopySecurityDescriptor status");
        if rc == 0 {
            let len = unsafe { cabi::RtlLengthSecurityDescriptor(dc as *const _) };
            let cs = unsafe { core::slice::from_raw_parts(dc, len as usize) };
            let rs = unsafe { core::slice::from_raw_parts(dr, len as usize) };
            assert_eq!(cs, rs, "RtlCopySecurityDescriptor content");
        } else {
            assert_eq!(dc as usize, dr as usize, "RtlCopySecurityDescriptor failure out ptr");
        }
    }
}

// =====================================================================
// Wave ENV-1 — env.c pure string-walk diff
// =====================================================================

extern "C" {
    fn ref_RtlQueryEnvironmentVariable_U(
        env: *mut u16,
        name: *const UNICODE_STRING,
        value: *mut UNICODE_STRING,
    ) -> i32;
    fn ref_RtlExpandEnvironmentStrings_U(
        env: *mut u16,
        source: *const UNICODE_STRING,
        destination: *mut UNICODE_STRING,
        length: *mut u32,
    ) -> i32;
    fn ref_RtlSetEnvironmentVariable(
        env: *mut *mut u16,
        name: *const UNICODE_STRING,
        value: *const UNICODE_STRING,
    ) -> i32;
    fn ref_RtlCreateEnvironment(inherit: u8, out_env: *mut *mut u16) -> i32;
    fn ref_RtlDestroyEnvironment(env: *mut u16);
    // Wave SEC
    fn ref_RtlpSetSecurityObject(
        object: *mut u8,
        security_information: u32,
        modification_descriptor: *const u8,
        objects_security_descriptor: *mut *mut u8,
        auto_inherit_flags: u32,
        pool_type: u32,
        generic_mapping: *const u8,
        token: *mut u8,
    ) -> i32;
    fn ref_RtlQuerySecurityObject(
        object_descriptor: *const u8,
        security_information: u32,
        resultant_descriptor: *mut u8,
        descriptor_length: u32,
        return_length: *mut u32,
    ) -> i32;
    // The shared heap-backed Nt*VirtualMemory stubs (in env ref TU).
    fn NtAllocateVirtualMemory(
        process: *mut u8,
        base: *mut *mut u8,
        zero_bits: usize,
        size: *mut usize,
        alloc_type: u32,
        protect: u32,
    ) -> i32;
}

/// Build an env block from `(name, value)` pairs. Format:
///   NAME=VALUE\0NAME=VALUE\0...\0
/// The final extra `\0` terminates the block (double-NUL).
fn mk_env(pairs: &[(&str, &str)]) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for &(n, v) in pairs {
        out.extend(n.encode_utf16());
        out.push(b'=' as u16);
        out.extend(v.encode_utf16());
        out.push(0);
    }
    out.push(0); // double-NUL terminator
    out
}

fn to_u16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn mk_us(buf: &mut [u16]) -> UNICODE_STRING {
    UNICODE_STRING {
        Length: (buf.iter().take_while(|&&c| c != 0).count() * 2) as u16,
        MaximumLength: (buf.len() * 2) as u16,
        Buffer: buf.as_mut_ptr(),
    }
}

/// Build a query-input UNICODE_STRING from a string (no trailing NUL).
fn mk_us_in(buf: &mut Vec<u16>) -> UNICODE_STRING {
    UNICODE_STRING {
        Length: (buf.len() * 2) as u16,
        MaximumLength: (buf.len() * 2) as u16,
        Buffer: buf.as_mut_ptr(),
    }
}

#[test]
fn diff_rtl_query_environment_variable_u() {
    let cases: &[&[(&str, &str)]] = &[
        &[("PATH", "/usr/bin"), ("HOME", "/root"), ("USER", "admin")],
        &[("A", "1"), ("B", "22"), ("C", "333")],
        &[("EMPTY", ""), ("X", "y")],
        &[("=C:", "C:\\foo"), ("PATH", "C:\\bin")], // hidden cwd
        &[
            ("Long_Variable_Name_42", "and_a_long_value_too_yes_yes"),
            ("short", "v"),
        ],
        &[],
        &[("ONE", "1")],
    ];
    let queries = &[
        "PATH", "path", "PaTh", "HOME", "USER", "MISSING", "A", "B", "C", "EMPTY", "ONE",
        "Long_Variable_Name_42", "long_variable_name_42",
    ];
    let buf_sizes = [0usize, 1, 8, 32, 128];

    for case in cases {
        let mut env = mk_env(case);
        for q in queries {
            for &bsz in &buf_sizes {
                let mut name_buf = to_u16(q);
                let name_us = mk_us_in(&mut name_buf);

                let mut bc = vec![0u16; bsz.max(1)];
                let mut br = vec![0u16; bsz.max(1)];
                let mut us_c = UNICODE_STRING {
                    Length: 0,
                    MaximumLength: (bsz * 2) as u16,
                    Buffer: bc.as_mut_ptr(),
                };
                let mut us_r = UNICODE_STRING {
                    Length: 0,
                    MaximumLength: (bsz * 2) as u16,
                    Buffer: br.as_mut_ptr(),
                };

                let sc = unsafe {
                    cabi::RtlQueryEnvironmentVariable_U(env.as_mut_ptr(), &name_us, &mut us_c)
                };
                let sr = unsafe {
                    ref_RtlQueryEnvironmentVariable_U(env.as_mut_ptr(), &name_us, &mut us_r)
                };
                assert_eq!(sc, sr, "status mismatch: query={:?} bsz={}", q, bsz);
                assert_eq!(us_c.Length, us_r.Length, "Length mismatch: query={:?}", q);
                // Compare buffers up to whatever MaximumLength allowed.
                let cmp_len = (us_r.Length as usize / 2).min(bsz);
                assert_eq!(
                    &bc[..cmp_len],
                    &br[..cmp_len],
                    "buffer mismatch: query={:?} bsz={}",
                    q,
                    bsz
                );
            }
        }
    }
}

#[test]
fn diff_rtl_expand_environment_strings_u() {
    let env_pairs: &[(&str, &str)] = &[
        ("PATH", "/usr/bin"),
        ("HOME", "/root"),
        ("USER", "admin"),
        ("EMPTY", ""),
        ("X", "y"),
    ];
    let sources = &[
        "no vars here",
        "%PATH%",
        "prefix %PATH% middle %HOME% suffix",
        "%MISSING% literal",
        "%PATH",                  // unfinished
        "literal % alone",
        "%%escape",               // double percent (literal first %)
        "%X%%Y%",                 // adjacent
        "",
        "%PATH%%USER%/%HOME%",
        "%EMPTY%",
        "%path%",                 // case-insensitive
        "tail%",
    ];
    let dst_sizes = [0usize, 1, 4, 16, 64, 256];

    let mut env = mk_env(env_pairs);
    for src in sources {
        let mut src_buf = to_u16(src);
        let src_us = UNICODE_STRING {
            Length: (src_buf.len() * 2) as u16,
            MaximumLength: (src_buf.len() * 2) as u16,
            Buffer: src_buf.as_mut_ptr(),
        };
        for &dsz in &dst_sizes {
            let mut bc = vec![0u16; dsz.max(1)];
            let mut br = vec![0u16; dsz.max(1)];
            let mut dc = UNICODE_STRING {
                Length: 0,
                MaximumLength: (dsz * 2) as u16,
                Buffer: bc.as_mut_ptr(),
            };
            let mut dr = UNICODE_STRING {
                Length: 0,
                MaximumLength: (dsz * 2) as u16,
                Buffer: br.as_mut_ptr(),
            };
            let mut lc: u32 = 0xDEAD_BEEF;
            let mut lr: u32 = 0xDEAD_BEEF;
            let sc = unsafe {
                cabi::RtlExpandEnvironmentStrings_U(
                    env.as_mut_ptr(),
                    &src_us,
                    &mut dc,
                    &mut lc as *mut u32,
                )
            };
            let sr = unsafe {
                ref_RtlExpandEnvironmentStrings_U(
                    env.as_mut_ptr(),
                    &src_us,
                    &mut dr,
                    &mut lr as *mut u32,
                )
            };
            assert_eq!(sc, sr, "status mismatch: src={:?} dsz={}", src, dsz);
            assert_eq!(dc.Length, dr.Length, "Length: src={:?} dsz={}", src, dsz);
            assert_eq!(lc, lr, "Length out-param: src={:?} dsz={}", src, dsz);
            let n = (dc.Length / 2) as usize;
            // Buffer must match up to chars written. (Trailing NUL
            // beyond that may or may not be present.)
            assert_eq!(&bc[..n], &br[..n], "buf: src={:?} dsz={}", src, dsz);
        }
    }
    let _ = mk_us;
}

/// Allocate a page-aligned env block via the VM stub and copy
/// `pairs` into it (double-NUL-terminated). Returns the env pointer
/// (NOT freed by this test — leaks; that's fine for fuzz coverage).
unsafe fn vm_alloc_env(pairs: &[(&str, &str)]) -> *mut u16 {
    let mut base: *mut u8 = core::ptr::null_mut();
    let mut size: usize = 0x1000; // one page; stubs round up
    let s = NtAllocateVirtualMemory(
        core::ptr::null_mut(),
        &mut base,
        0,
        &mut size,
        0x3000, /* MEM_RESERVE|MEM_COMMIT */
        4,      /* PAGE_READWRITE */
    );
    assert_eq!(s, 0, "vm alloc");
    let env = base as *mut u16;
    let mut off = 0usize;
    for &(n, v) in pairs {
        for c in n.encode_utf16() {
            env.add(off).write(c);
            off += 1;
        }
        env.add(off).write(b'=' as u16);
        off += 1;
        for c in v.encode_utf16() {
            env.add(off).write(c);
            off += 1;
        }
        env.add(off).write(0);
        off += 1;
    }
    env.add(off).write(0); // double-NUL
    env
}

/// Compute walkable env length (chars incl. final double-NUL) by
/// the same algorithm SetEnvironmentVariable uses internally.
unsafe fn env_chars(env: *mut u16) -> usize {
    if env.is_null() || *env == 0 {
        return 1;
    }
    let mut end = 0usize;
    loop {
        let mut k = 0;
        while *env.add(end + k) != 0 {
            k += 1;
        }
        end += k + 1;
        if *env.add(end) == 0 {
            return end + 1;
        }
    }
}

#[test]
fn diff_rtl_set_environment_variable_seq() {
    // Each scenario = (initial env, sequence of (name, Option<value>) ops).
    // value=None means delete the variable.
    let scenarios: &[(&[(&str, &str)], &[(&str, Option<&str>)])] = &[
        // Pure insertion-at-end (alphabetical order matters):
        (&[], &[("PATH", Some("/usr/bin"))]),
        // Insert + replace + delete:
        (
            &[("PATH", "/old"), ("HOME", "/root")],
            &[
                ("USER", Some("admin")),
                ("PATH", Some("/new/longer/path/value")),
                ("HOME", None),
            ],
        ),
        // Insert before existing var (sort order):
        (
            &[("ZZZ", "last")],
            &[("AAA", Some("first")), ("MMM", Some("middle"))],
        ),
        // Replace with shorter value (no realloc):
        (
            &[("VAR", "very_long_value_here_xxx")],
            &[("VAR", Some("y"))],
        ),
        // Replace with longer value (forces tail-shift):
        (
            &[("A", "1"), ("B", "2"), ("C", "3")],
            &[("B", Some("very_long_replacement_22"))],
        ),
        // Delete first / last / middle entries:
        (
            &[("A", "1"), ("B", "2"), ("C", "3")],
            &[("A", None), ("C", None), ("B", None)],
        ),
        // Delete non-existing (no-op):
        (&[("X", "1")], &[("NOPE", None)]),
        // Hidden cwd entries (var name starts with '='):
        (
            &[("=C:", "C:\\foo"), ("PATH", "/bin")],
            &[("PATH", Some("/usr/local/bin"))],
        ),
        // Many inserts forcing tail-shift each time:
        (
            &[("M", "5")],
            &[
                ("A", Some("1")),
                ("B", Some("2")),
                ("C", Some("3")),
                ("Z", Some("26")),
            ],
        ),
        // Empty value:
        (&[("X", "1")], &[("EMPTY", Some(""))]),
        // Case-insensitive replacement:
        (
            &[("Path", "/foo")],
            &[("PATH", Some("/bar")), ("path", Some("/baz"))],
        ),
    ];

    for (idx, (initial, ops)) in scenarios.iter().enumerate() {
        unsafe {
            let mut env_c = vm_alloc_env(initial);
            let mut env_r = vm_alloc_env(initial);

            for (step, &(name, val)) in ops.iter().enumerate() {
                let mut name_buf = to_u16(name);
                let name_us = mk_us_in(&mut name_buf);

                let (sc, sr) = match val {
                    Some(v) => {
                        let mut val_buf = to_u16(v);
                        let val_us = UNICODE_STRING {
                            Length: (val_buf.len() * 2) as u16,
                            MaximumLength: (val_buf.len() * 2) as u16,
                            Buffer: val_buf.as_mut_ptr(),
                        };
                        let sc = cabi::RtlSetEnvironmentVariable(&mut env_c, &name_us, &val_us);
                        let sr = ref_RtlSetEnvironmentVariable(&mut env_r, &name_us, &val_us);
                        (sc, sr)
                    }
                    None => {
                        let sc = cabi::RtlSetEnvironmentVariable(
                            &mut env_c,
                            &name_us,
                            core::ptr::null(),
                        );
                        let sr = ref_RtlSetEnvironmentVariable(
                            &mut env_r,
                            &name_us,
                            core::ptr::null(),
                        );
                        (sc, sr)
                    }
                };

                assert_eq!(
                    sc, sr,
                    "scenario {} step {} '{}' status",
                    idx, step, name
                );

                // Compare resulting env blocks byte-for-byte. Use the
                // larger of the two walked lengths so we catch
                // missing/extra terminator divergence.
                let lc = env_chars(env_c);
                let lr = env_chars(env_r);
                assert_eq!(
                    lc, lr,
                    "scenario {} step {} '{}' env-block length: cabi={} ref={}",
                    idx, step, name, lc, lr
                );
                let cs = core::slice::from_raw_parts(env_c, lc);
                let rs = core::slice::from_raw_parts(env_r, lr);
                assert_eq!(
                    cs, rs,
                    "scenario {} step {} '{}' env-block contents diverged",
                    idx, step, name
                );
            }
        }
    }
}

#[test]
fn diff_rtl_set_environment_variable_random() {
    // Random sequences of {insert, replace, delete} operations on a
    // shared env block. Drift between the two impls compounds at each
    // step, so a divergence at step N pins exactly which op broke.
    let mut rng = Rng(0x5E10_0001);
    // Pre-load a "vocabulary" of var names + values.
    let names = ["PATH", "HOME", "USER", "SHELL", "TEMP", "TMP", "LANG", "TZ",
                 "PWD", "ABC", "DEF", "XYZ", "Foo", "BAR", "baz", "=C:"];
    let values = ["/usr/bin", "/root", "admin", "/bin/sh", "/tmp",
                  "", "C:\\foo", "y", "a very long value indeed",
                  "x", "12345"];

    for trial in 0..30 {
        unsafe {
            let mut env_c = vm_alloc_env(&[]);
            let mut env_r = vm_alloc_env(&[]);

            for step in 0..40 {
                let name = names[(rng.byte() as usize) % names.len()];
                let do_delete = rng.byte() & 3 == 0;
                let mut name_buf = to_u16(name);
                let name_us = mk_us_in(&mut name_buf);

                let (sc, sr) = if do_delete {
                    let sc = cabi::RtlSetEnvironmentVariable(
                        &mut env_c,
                        &name_us,
                        core::ptr::null(),
                    );
                    let sr = ref_RtlSetEnvironmentVariable(
                        &mut env_r,
                        &name_us,
                        core::ptr::null(),
                    );
                    (sc, sr)
                } else {
                    let v = values[(rng.byte() as usize) % values.len()];
                    let mut val_buf = to_u16(v);
                    let val_us = UNICODE_STRING {
                        Length: (val_buf.len() * 2) as u16,
                        MaximumLength: (val_buf.len() * 2) as u16,
                        Buffer: val_buf.as_mut_ptr(),
                    };
                    let sc = cabi::RtlSetEnvironmentVariable(&mut env_c, &name_us, &val_us);
                    let sr = ref_RtlSetEnvironmentVariable(&mut env_r, &name_us, &val_us);
                    (sc, sr)
                };

                assert_eq!(sc, sr, "trial {} step {} '{}' status", trial, step, name);
                let lc = env_chars(env_c);
                let lr = env_chars(env_r);
                let cs = core::slice::from_raw_parts(env_c, lc);
                let rs = core::slice::from_raw_parts(env_r, lr);
                if cs != rs {
                    // Dump for debugging.
                    let to_str = |s: &[u16]| -> String {
                        s.iter()
                            .map(|&c| {
                                if c == 0 {
                                    "\\0".to_string()
                                } else if c < 0x80 {
                                    String::from(char::from_u32(c as u32).unwrap_or('?'))
                                } else {
                                    format!("\\x{:04x}", c)
                                }
                            })
                            .collect()
                    };
                    panic!(
                        "trial {} step {} op={} name='{}' diverged\n  cabi: {}\n  ref:  {}",
                        trial,
                        step,
                        if do_delete { "DEL" } else { "SET" },
                        name,
                        to_str(cs),
                        to_str(rs)
                    );
                }
            }
        }
    }
}

// =====================================================================
// Wave SEC — security.c byte-diff
// =====================================================================

/// Heap-alloc + populate a self-relative SD with the given owner/group
/// SIDs and dacl/sacl ACL byte slices. The result is leaked (test
/// process lifetime). Faithful to mk_relative_sd's layout from SD wave.
unsafe fn mk_sdr_for_sec(
    owner: &[u8],
    group: &[u8],
    dacl: Option<&[u8]>,
    sacl: Option<&[u8]>,
) -> Vec<u8> {
    let owner_n = (owner.len() + 3) & !3;
    let group_n = (group.len() + 3) & !3;
    let dacl_n = dacl.map(|d| (d.len() + 3) & !3).unwrap_or(0);
    let sacl_n = sacl.map(|s| (s.len() + 3) & !3).unwrap_or(0);

    let mut buf = vec![0u8; 20 + owner_n + group_n + dacl_n + sacl_n];
    buf[0] = 1; // Revision
    // Control: SE_SELF_RELATIVE
    let mut ctrl: u16 = 0x8000;
    if dacl.is_some() {
        ctrl |= 0x0004; // SE_DACL_PRESENT
    }
    if sacl.is_some() {
        ctrl |= 0x0010; // SE_SACL_PRESENT
    }
    buf[2..4].copy_from_slice(&ctrl.to_le_bytes());

    let mut off = 20u32;
    let mut sacl_off = 0u32;
    if let Some(s) = sacl {
        sacl_off = off;
        buf[off as usize..off as usize + s.len()].copy_from_slice(s);
        off += sacl_n as u32;
    }
    let mut dacl_off = 0u32;
    if let Some(d) = dacl {
        dacl_off = off;
        buf[off as usize..off as usize + d.len()].copy_from_slice(d);
        off += dacl_n as u32;
    }
    let owner_off = off;
    buf[off as usize..off as usize + owner.len()].copy_from_slice(owner);
    off += owner_n as u32;
    let group_off = off;
    buf[off as usize..off as usize + group.len()].copy_from_slice(group);

    // Owner/Group/Sacl/Dacl offsets in SDR header (offsets 4/8/12/16).
    buf[4..8].copy_from_slice(&owner_off.to_le_bytes());
    buf[8..12].copy_from_slice(&group_off.to_le_bytes());
    buf[12..16].copy_from_slice(&sacl_off.to_le_bytes());
    buf[16..20].copy_from_slice(&dacl_off.to_le_bytes());

    buf
}

/// Build a minimal valid SID. authority=NT(5), one subauthority value.
fn mk_simple_sid(rid: u32) -> Vec<u8> {
    let mut sid = Vec::with_capacity(12);
    sid.push(1); // Revision
    sid.push(1); // SubAuthorityCount
    sid.extend_from_slice(&[0u8, 0, 0, 0, 0, 5]); // NT authority
    sid.extend_from_slice(&rid.to_le_bytes());
    sid
}

/// Build a minimal ACL with the given byte content as opaque body.
/// Just the ACL header + zero ACEs (AclSize matches).
fn mk_empty_acl_sec() -> Vec<u8> {
    let mut acl = Vec::with_capacity(8);
    acl.push(2); // AclRevision
    acl.push(0); // Sbz1
    acl.extend_from_slice(&8u16.to_le_bytes()); // AclSize
    acl.extend_from_slice(&0u16.to_le_bytes()); // AceCount
    acl.extend_from_slice(&0u16.to_le_bytes()); // Sbz2
    acl
}

#[test]
fn diff_rtl_p_set_security_object() {
    // Each scenario: existing_sd_parts, modification_sd_parts, security_info bits.
    let owner1 = mk_simple_sid(0x100);
    let group1 = mk_simple_sid(0x101);
    let dacl1 = mk_empty_acl_sec();
    let sacl1 = mk_empty_acl_sec();

    let owner2 = mk_simple_sid(0x200);
    let group2 = mk_simple_sid(0x201);

    unsafe {
        // Allocate existing + modification descriptors via the vm-backed
        // RtlAllocateHeap so cabi's RtlFreeHeap works on the old one.
        let allocate = |bytes: &[u8]| -> *mut u8 {
            extern "C" {
                fn RtlAllocateHeap(h: *mut u8, f: u32, n: usize) -> *mut u8;
            }
            let p = RtlAllocateHeap(core::ptr::null_mut(), 0, bytes.len());
            assert!(!p.is_null(), "vm alloc");
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
            p
        };

        let cases: &[(u32, &[u8], &[u8], Option<&[u8]>, Option<&[u8]>)] = &[
            (
                0x1, // OWNER only
                &owner2,
                &group1,
                Some(&dacl1),
                None,
            ),
            (
                0x2, // GROUP only
                &owner1,
                &group2,
                None,
                None,
            ),
            (
                0x4, // DACL only
                &owner1,
                &group1,
                Some(&dacl1),
                None,
            ),
            (
                0x8, // SACL only
                &owner1,
                &group1,
                None,
                Some(&sacl1),
            ),
            (
                0xF, // ALL four
                &owner2,
                &group2,
                Some(&dacl1),
                Some(&sacl1),
            ),
            (
                0x0, // None — should still produce a copy
                &owner1,
                &group1,
                None,
                None,
            ),
        ];

        for (idx, (sec_info, mod_owner, mod_group, mod_dacl, mod_sacl)) in cases.iter().enumerate()
        {
            // existing SD: always full owner/group, no DACL/SACL.
            let existing = mk_sdr_for_sec(&owner1, &group1, Some(&dacl1), Some(&sacl1));
            let modification = mk_sdr_for_sec(mod_owner, mod_group, *mod_dacl, *mod_sacl);

            let mut existing_c = allocate(&existing);
            let mut existing_r = allocate(&existing);
            let mod_c = allocate(&modification);
            let mod_r = allocate(&modification);

            let sc = cabi::RtlpSetSecurityObject(
                core::ptr::null_mut(),
                *sec_info,
                mod_c,
                &mut existing_c,
                0,
                1,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            let sr = ref_RtlpSetSecurityObject(
                core::ptr::null_mut(),
                *sec_info,
                mod_r,
                &mut existing_r,
                0,
                1,
                core::ptr::null(),
                core::ptr::null_mut(),
            );

            assert_eq!(sc, sr, "scenario {} status", idx);

            if sc == 0 {
                // Both succeeded — compare new SD byte-for-byte.
                // The new SD is at *existing_c / *existing_r. Length is
                // read from offsets (group is last; its size = round_up_4(SID)).
                // Easier: rely on RtlLengthSecurityDescriptor.
                let lc = cabi::RtlLengthSecurityDescriptor(existing_c as *const _);
                let lr = cabi::RtlLengthSecurityDescriptor(existing_r as *const _);
                assert_eq!(lc, lr, "scenario {} length", idx);
                let bytes_c = core::slice::from_raw_parts(existing_c, lc as usize);
                let bytes_r = core::slice::from_raw_parts(existing_r, lc as usize);
                assert_eq!(bytes_c, bytes_r, "scenario {} bytes", idx);
            }
        }
    }
}

#[test]
fn diff_rtl_query_security_object() {
    let owner = mk_simple_sid(0x100);
    let group = mk_simple_sid(0x101);
    let dacl = mk_empty_acl_sec();
    let sacl = mk_empty_acl_sec();

    let existing = unsafe { mk_sdr_for_sec(&owner, &group, Some(&dacl), Some(&sacl)) };

    let cases: &[u32] = &[0x0, 0x1, 0x2, 0x4, 0x8, 0xF];
    let buf_sizes = [128usize, 256, 512];

    for &sec_info in cases {
        for &bsz in &buf_sizes {
            let mut bc = vec![0u8; bsz];
            let mut br = vec![0u8; bsz];
            let mut lc: u32 = 0;
            let mut lr: u32 = 0;

            let sc = unsafe {
                cabi::RtlQuerySecurityObject(
                    existing.as_ptr(),
                    sec_info,
                    bc.as_mut_ptr(),
                    bsz as u32,
                    &mut lc,
                )
            };
            let sr = unsafe {
                ref_RtlQuerySecurityObject(
                    existing.as_ptr(),
                    sec_info,
                    br.as_mut_ptr(),
                    bsz as u32,
                    &mut lr,
                )
            };
            assert_eq!(sc, sr, "sec_info={:#x} bsz={} status", sec_info, bsz);
            assert_eq!(lc, lr, "sec_info={:#x} bsz={} length", sec_info, bsz);
            if sc == 0 {
                assert_eq!(&bc[..lc as usize], &br[..lc as usize],
                           "sec_info={:#x} bsz={} bytes", sec_info, bsz);
            }
        }
    }
}
