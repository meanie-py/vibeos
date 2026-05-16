//! Differential parity: cnt-rust `cabi` shims vs the genuine compiled
//! ReactOS libcntpr reference (build.rs), across each symbol's
//! fuzz_domain. A mismatch fails loudly — this is the net that makes
//! cheap-model manifest rows safe and pins the load-bearing
//! misfeatures (strncpy non-term, signed-char strcmp, overlap-safe
//! memcpy, atoi i64-wrap→i32).

use core::ffi::c_void;

extern "C" {
    fn ref_strlen(s: *const u8) -> usize;
    fn ref_strncpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn ref_strcmp(a: *const u8, b: *const u8) -> i32;
    fn ref_strstr(h: *const u8, n: *const u8) -> *mut u8;
    fn ref_atoi64(s: *const u8) -> i64;
    fn ref_atol(s: *const u8) -> i64; // C `long` (LP32) → i64-wide here, compare as i32
    fn ref_memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn ref_memchr(s: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn ref_memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn ref_memmove(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn ref_memset(d: *mut c_void, v: i32, n: usize) -> *mut c_void;
    fn ref_strnlen(s: *const u8, cap: usize) -> usize;
    fn ref_strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref_strchr(s: *const u8, c: i32) -> *mut u8;
    fn ref_strrchr(s: *const u8, c: i32) -> *mut u8;
    fn ref_strtoi64(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> i64;
    fn ref_strtoul(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> u32;
    fn ref_i64toa(v: i64, s: *mut u8, r: i32) -> *mut u8;
    fn ref_ui64toa(v: u64, s: *mut u8, r: i32) -> *mut u8;
    fn ref_itoa(v: i32, s: *mut u8, r: i32) -> *mut u8;
    fn ref_ltoa32(v: i32, s: *mut u8, r: i32) -> *mut u8;
    fn ref_ultoa32(v: u32, s: *mut u8, r: i32) -> *mut u8;
    fn ref_i64tow(v: i64, s: *mut u16, r: i32) -> *mut u16;
    fn ref_ui64tow(v: u64, s: *mut u16, r: i32) -> *mut u16;
    fn ref_itow(v: i32, s: *mut u16, r: i32) -> *mut u16;
    fn ref_ltow32(v: i32, s: *mut u16, r: i32) -> *mut u16;
    fn ref_ultow32(v: u32, s: *mut u16, r: i32) -> *mut u16;
    fn ref_i64toa_s(v: i64, s: *mut u8, sz: usize, r: i32) -> i32;
    fn ref_ui64toa_s(v: u64, s: *mut u8, sz: usize, r: i32) -> i32;
    fn ref_itoa_s(v: i32, s: *mut u8, sz: usize, r: i32) -> i32;
    fn ref_ltoa_s32(v: i32, s: *mut u8, sz: usize, r: i32) -> i32;
    fn ref_i64tow_s(v: i64, s: *mut u16, sz: usize, r: i32) -> i32;
    fn ref_ui64tow_s(v: u64, s: *mut u16, sz: usize, r: i32) -> i32;
    fn ref_itow_s(v: i32, s: *mut u16, sz: usize, r: i32) -> i32;
    fn ref_ltow_s32(v: i32, s: *mut u16, sz: usize, r: i32) -> i32;
    fn ref_stricmp(a: *const u8, b: *const u8) -> i32;
    fn ref_strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref_wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn ref_wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref_wcstoul(n: *const u16, e: *mut *mut u16, b: i32) -> u32;
    fn ref_mbtowc(w: *mut u16, mb: *const u8, n: usize) -> i32;
    fn ref_mbstowcs(w: *mut u16, mb: *const u8, n: usize) -> usize;
    fn ref_wctomb(mb: *mut u8, w: u16) -> i32;
    fn ref_wcstombs(mb: *mut u8, w: *const u16, n: usize) -> usize;
    fn ref_toupper_nt_mb(c: i32) -> i32;
    fn ref_wcslen(s: *const u16) -> usize;
    fn ref_wcsnlen(s: *const u16, n: usize) -> usize;
    fn ref_wcscmp(a: *const u16, b: *const u16) -> i32;
    fn ref_wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref_wcschr(s: *const u16, c: u16) -> *mut u16;
    fn ref_wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn ref_wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    fn ref_strcpy(d: *mut u8, s: *const u8) -> *mut u8;
    fn ref_wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    fn ref_strcat(d: *mut u8, s: *const u8) -> *mut u8;
    fn ref_wcscat(d: *mut u16, s: *const u16) -> *mut u16;
    fn ref_strncat(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn ref_wcsncat(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
}

use cnt_rust::cabi;

/// Deterministic xorshift64 — reproducible failures, no rand dep.
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
    fn byte(&mut self) -> u8 {
        (self.next() & 0xff) as u8
    }
    fn upto(&mut self, n: usize) -> usize {
        (self.next() as usize) % n
    }
}

/// Random NUL-terminated C string. `alpha` restricts to a small set so
/// substring/compare collisions actually happen; otherwise full-range
/// bytes (incl. >=0x80 to exercise signed-char strcmp).
fn cstr(rng: &mut Rng, max: usize, alpha: bool) -> Vec<u8> {
    let len = rng.upto(max + 1);
    let mut v: Vec<u8> = (0..len)
        .map(|_| {
            if alpha {
                b"AaBb \x80\xff"[rng.upto(7)]
            } else {
                let b = rng.byte();
                if b == 0 {
                    1
                } else {
                    b
                }
            }
        })
        .collect();
    v.push(0);
    v
}

#[test]
fn diff_strlen() {
    let mut rng = Rng(0x1234_5678);
    for _ in 0..5000 {
        let s = cstr(&mut rng, 64, false);
        unsafe { assert_eq!(cabi::strlen(s.as_ptr()), ref_strlen(s.as_ptr()), "strlen {s:?}") }
    }
    // NULL → 0 (preserved misfeature).
    unsafe { assert_eq!(cabi::strlen(core::ptr::null()), ref_strlen(core::ptr::null())) }
}

#[test]
fn diff_strcmp() {
    let mut rng = Rng(0xdead_beef);
    for _ in 0..8000 {
        // mix alpha (collisions) + raw (>=0x80 signed-char path)
        let alpha_a = rng.next() & 1 == 0;
        let a = cstr(&mut rng, 24, alpha_a);
        let same = rng.next() & 3 == 0;
        let alpha_b = rng.next() & 1 == 0;
        let b = if same {
            a.clone() // equal / prefix cases
        } else {
            cstr(&mut rng, 24, alpha_b)
        };
        let c = unsafe { cabi::strcmp(a.as_ptr(), b.as_ptr()) };
        let r = unsafe { ref_strcmp(a.as_ptr(), b.as_ptr()) };
        assert_eq!(c, r, "strcmp {a:?} vs {b:?}");
    }
}

#[test]
fn diff_strstr() {
    let mut rng = Rng(0x0bad_f00d);
    for _ in 0..8000 {
        let hay = cstr(&mut rng, 40, true);
        let needle = if rng.next() & 3 == 0 {
            vec![0] // empty needle → returns hay
        } else if rng.next() & 3 == 0 && hay.len() > 2 {
            let s = rng.upto(hay.len() - 1);
            let e = s + 1 + rng.upto(hay.len() - s - 1);
            let mut n = hay[s..e].to_vec();
            n.push(0);
            n
        } else {
            cstr(&mut rng, 6, true)
        };
        let c = unsafe { cabi::strstr(hay.as_ptr(), needle.as_ptr()) };
        let r = unsafe { ref_strstr(hay.as_ptr(), needle.as_ptr()) };
        let base = hay.as_ptr() as usize;
        let off = |p: *mut u8| if p.is_null() { None } else { Some(p as usize - base) };
        assert_eq!(off(c), off(r), "strstr hay={hay:?} needle={needle:?}");
    }
}

#[test]
fn diff_atoi() {
    let mut rng = Rng(0xfeed_face);
    let fixed: &[&[u8]] = &[
        b"\0",
        b"   +12\0",
        b"-0\0",
        b"2147483648\0",          // i32 overflow
        b"-2147483649\0",
        b"99999999999999999999\0", // i64 overflow → wrap
        b"12abc\0",
        b"  -\0",
        b"+\0",
        b"\t\n 42xyz\0",
    ];
    for f in fixed {
        let c = unsafe { cabi::atoi(f.as_ptr()) };
        let r = unsafe { ref_atoi64(f.as_ptr()) as i32 };
        assert_eq!(c, r, "atoi {:?}", core::str::from_utf8(f).unwrap_or("?"));
    }
    for _ in 0..5000 {
        let mut s = Vec::new();
        for _ in 0..rng.upto(24) {
            s.push(b" \t+-0123456789xyz"[rng.upto(17)]);
        }
        s.push(0);
        let c = unsafe { cabi::atoi(s.as_ptr()) };
        let r = unsafe { ref_atoi64(s.as_ptr()) as i32 };
        assert_eq!(c, r, "atoi {s:?}");
    }
    unsafe { assert_eq!(cabi::atoi(core::ptr::null()), ref_atoi64(core::ptr::null()) as i32) }
}

#[test]
fn diff_strncpy() {
    let mut rng = Rng(0xc0ff_ee11);
    for _ in 0..8000 {
        let src = cstr(&mut rng, 20, true);
        let n = rng.upto(24); // spans src<n (pad), src==n, src>n (no-term), n==0
        let mut a = vec![0xAAu8; n];
        let mut b = vec![0xAAu8; n];
        let ra = unsafe { cabi::strncpy(a.as_mut_ptr(), src.as_ptr(), n) };
        let rb = unsafe { ref_strncpy(b.as_mut_ptr(), src.as_ptr(), n) };
        assert_eq!(ra, a.as_mut_ptr(), "strncpy returns dst");
        assert_eq!(rb, b.as_mut_ptr());
        assert_eq!(a, b, "strncpy buffer mismatch (misfeature!) src={src:?} n={n}");
    }
}

#[test]
fn diff_memcpy() {
    let mut rng = Rng(0x5151_5151);
    for _ in 0..6000 {
        let n = rng.upto(64);
        // shared region to force overlap variety
        let total = n + rng.upto(16);
        let base: Vec<u8> = (0..total + n).map(|_| rng.byte()).collect();
        // pick dst/src offsets within the same buffer (overlap)
        let so = if total == 0 { 0 } else { rng.upto(total) };
        let do_ = if total == 0 { 0 } else { rng.upto(total) };
        let mut a = base.clone();
        let mut b = base.clone();
        unsafe {
            cabi::memcpy(
                a.as_mut_ptr().add(do_) as *mut c_void,
                a.as_ptr().add(so) as *const c_void,
                n,
            );
            ref_memcpy(
                b.as_mut_ptr().add(do_) as *mut c_void,
                b.as_ptr().add(so) as *const c_void,
                n,
            );
        }
        assert_eq!(a, b, "memcpy overlap mismatch n={n} so={so} do={do_}");
    }
}

#[test]
fn diff_memset() {
    let mut rng = Rng(0x9e37_79b9);
    for _ in 0..6000 {
        let n = rng.upto(64);
        let v = rng.next() as i32;
        let mut a = vec![0u8; n];
        let mut b = vec![0u8; n];
        let ra = unsafe { cabi::memset(a.as_mut_ptr() as *mut c_void, v, n) };
        let rb = unsafe { ref_memset(b.as_mut_ptr() as *mut c_void, v, n) };
        assert_eq!(ra as usize, a.as_mut_ptr() as usize);
        assert_eq!(rb as usize, b.as_mut_ptr() as usize);
        assert_eq!(a, b, "memset n={n} v={v}");
    }
}

#[test]
fn diff_memcmp() {
    let mut rng = Rng(0x2545_f491);
    for _ in 0..8000 {
        let n = rng.upto(48);
        let mut a: Vec<u8> = (0..n).map(|_| rng.byte()).collect();
        let mut b = a.clone();
        if n > 0 && rng.next() & 1 == 0 {
            let i = rng.upto(n);
            b[i] = rng.byte();
        }
        if n > 0 && rng.next() & 3 == 0 {
            a[0] = 0xFF;
            b[0] = 0x00;
        }
        let c = unsafe { cabi::memcmp(a.as_ptr() as *const c_void, b.as_ptr() as *const c_void, n) };
        let r = unsafe { ref_memcmp(a.as_ptr() as *const c_void, b.as_ptr() as *const c_void, n) };
        assert_eq!(c, r, "memcmp n={n} a={a:?} b={b:?}");
    }
}

#[test]
fn diff_memmove() {
    let mut rng = Rng(0x1357_9bdf);
    for _ in 0..6000 {
        let n = rng.upto(64);
        let total = n + rng.upto(16);
        let basev: Vec<u8> = (0..total + n).map(|_| rng.byte()).collect();
        let so = if total == 0 { 0 } else { rng.upto(total) };
        let do_ = if total == 0 { 0 } else { rng.upto(total) };
        let mut a = basev.clone();
        let mut b = basev.clone();
        unsafe {
            cabi::memmove(a.as_mut_ptr().add(do_) as *mut c_void, a.as_ptr().add(so) as *const c_void, n);
            ref_memmove(b.as_mut_ptr().add(do_) as *mut c_void, b.as_ptr().add(so) as *const c_void, n);
        }
        assert_eq!(a, b, "memmove n={n} so={so} do={do_}");
    }
}

#[test]
fn diff_memchr() {
    let mut rng = Rng(0xa5a5_5a5a);
    for _ in 0..8000 {
        let n = rng.upto(48);
        let buf: Vec<u8> = (0..n).map(|_| rng.byte()).collect();
        let c: i32 = match rng.upto(4) {
            0 if n > 0 => buf[rng.upto(n)] as i32,
            1 => 0,
            2 => (0x80 | (rng.byte() as i32)) & 0xff,
            _ => rng.next() as i32,
        };
        let cc = unsafe { cabi::memchr(buf.as_ptr() as *const c_void, c, n) };
        let rr = unsafe { ref_memchr(buf.as_ptr() as *const c_void, c, n) };
        let base = buf.as_ptr() as usize;
        let off = |p: *mut c_void| if p.is_null() { None } else { Some(p as usize - base) };
        assert_eq!(off(cc), off(rr), "memchr n={n} c={c} buf={buf:?}");
    }
}

#[test]
fn diff_atoi64_atol() {
    let mut rng = Rng(0x0123_4567);
    let fixed: &[&[u8]] = &[
        b"\0", b"  +9223372036854775807\0", b"-9223372036854775808\0",
        b"9223372036854775808\0", b"99999999999999999999\0",
        b"2147483648\0", b"-2147483649\0", b"  -42xyz\0", b"+\0",
    ];
    for f in fixed {
        unsafe {
            assert_eq!(cabi::_atoi64(f.as_ptr()), ref_atoi64(f.as_ptr()),
                "_atoi64 {:?}", core::str::from_utf8(f).unwrap_or("?"));
            assert_eq!(cabi::atol(f.as_ptr()), ref_atol(f.as_ptr()) as i32,
                "atol {:?}", core::str::from_utf8(f).unwrap_or("?"));
        }
    }
    for _ in 0..4000 {
        let mut s = Vec::new();
        for _ in 0..rng.upto(28) {
            s.push(b" \t+-0123456789zz"[rng.upto(16)]);
        }
        s.push(0);
        unsafe {
            assert_eq!(cabi::_atoi64(s.as_ptr()), ref_atoi64(s.as_ptr()), "_atoi64 {s:?}");
            assert_eq!(cabi::atol(s.as_ptr()), ref_atol(s.as_ptr()) as i32, "atol {s:?}");
        }
    }
    unsafe { assert_eq!(cabi::_atoi64(core::ptr::null()), ref_atoi64(core::ptr::null())); }
}

#[test]
fn diff_strnlen() {
    let mut rng = Rng(0x7777_3333);
    for _ in 0..6000 {
        let s = cstr(&mut rng, 40, false);
        let cap = rng.upto(48);
        unsafe {
            assert_eq!(cabi::strnlen(s.as_ptr(), cap), ref_strnlen(s.as_ptr(), cap),
                "strnlen s={s:?} cap={cap}");
        }
    }
    unsafe { assert_eq!(cabi::strnlen(core::ptr::null(), 8), ref_strnlen(core::ptr::null(), 8)); }
}

#[test]
fn diff_strncmp() {
    let mut rng = Rng(0x3333_7777);
    for _ in 0..8000 {
        let aa = rng.next() & 1 == 0;
        let a = cstr(&mut rng, 20, aa);
        let same = rng.next() & 3 == 0;
        let ab = rng.next() & 1 == 0;
        let b = if same { a.clone() } else { cstr(&mut rng, 20, ab) };
        let n = rng.upto(24);
        let c = unsafe { cabi::strncmp(a.as_ptr(), b.as_ptr(), n) };
        let r = unsafe { ref_strncmp(a.as_ptr(), b.as_ptr(), n) };
        assert_eq!(c, r, "strncmp a={a:?} b={b:?} n={n}");
    }
}

#[test]
fn diff_strchr_rchr() {
    let mut rng = Rng(0x9999_1111);
    for _ in 0..8000 {
        let s = cstr(&mut rng, 40, true);
        let c: i32 = match rng.upto(4) {
            0 if s.len() > 1 => s[rng.upto(s.len() - 1)] as i32,
            1 => 0, // NUL target → ptr to terminator
            2 => (0x80 | (rng.byte() as i32)) & 0xff,
            _ => rng.next() as i32,
        };
        let base = s.as_ptr() as usize;
        let off = |p: *mut u8| if p.is_null() { None } else { Some(p as usize - base) };
        unsafe {
            assert_eq!(off(cabi::strchr(s.as_ptr(), c)), off(ref_strchr(s.as_ptr(), c)),
                "strchr s={s:?} c={c}");
            assert_eq!(off(cabi::strrchr(s.as_ptr(), c)), off(ref_strrchr(s.as_ptr(), c)),
                "strrchr s={s:?} c={c}");
        }
    }
}

#[test]
fn diff_strtoi64_strtoul() {
    let mut rng = Rng(0x57f0_1640_u64);
    let bases = [0i32, 2, 8, 10, 16, 36, 1, 37];
    let alpha = b" \t+-0xXabcdefABCDEF0123456789Zz";
    for _ in 0..9000 {
        let mut s = Vec::new();
        for _ in 0..rng.upto(24) {
            s.push(alpha[rng.upto(alpha.len())]);
        }
        s.push(0);
        let base = bases[rng.upto(bases.len())];
        // i64 / endptr parity
        let (mut ep_c, mut ep_r): (*mut u8, *mut u8) =
            (1 as *mut u8, 1 as *mut u8);
        let cv = unsafe { cabi::_strtoi64(s.as_ptr(), &mut ep_c, base) };
        let rv = unsafe { ref_strtoi64(s.as_ptr(), &mut ep_r, base) };
        assert_eq!(cv, rv, "_strtoi64 v s={s:?} base={base}");
        let off = |p: *mut u8| if p == 1 as *mut u8 { None } else { Some(p as usize - s.as_ptr() as usize) };
        assert_eq!(off(ep_c), off(ep_r), "_strtoi64 endptr s={s:?} base={base}");
        // unsigned-long parity (asymmetric clamp incl. neg→1)
        let (mut uc, mut ur): (*mut u8, *mut u8) = (1 as *mut u8, 1 as *mut u8);
        let ucv = unsafe { cabi::strtoul(s.as_ptr(), &mut uc, base) };
        let urv = unsafe { ref_strtoul(s.as_ptr(), &mut ur, base) };
        assert_eq!(ucv, urv, "strtoul v s={s:?} base={base}");
        assert_eq!(off(uc), off(ur), "strtoul endptr s={s:?} base={base}");
    }
}

/// Wave C1 — unchecked narrow int→str. Valid radices only (C unchecked
/// is UB for radix 1/37 — div-by-zero / OOB digit map — never fuzzed).
/// Generous 80-byte dst (unchecked path has no bounds check). `_i64toa`
/// /`_ui64toa` get the full domain incl. negative+radix!=10 (host
/// `__int64`==model u64); the `long`/int-input symbols skip the
/// width-divergent neg+radix!=10 case (host LP64 vs Win LP32).
#[test]
fn diff_itoa_unchecked() {
    let mut rng = Rng(0x171a_0c01_u64);
    let radices = [2i32, 8, 10, 16, 36];
    for _ in 0..6000 {
        let r = radices[rng.upto(radices.len())];
        let mut a = [0xAAu8; 80];
        let mut b = [0xAAu8; 80];
        // _i64toa / _ui64toa: __int64 width == model, full domain.
        let v64 = rng.next() as i64;
        let pa = unsafe { cabi::_i64toa(v64, a.as_mut_ptr(), r) };
        let pb = unsafe { ref_i64toa(v64, b.as_mut_ptr(), r) };
        assert_eq!(pa, a.as_mut_ptr());
        assert_eq!(pb, b.as_mut_ptr());
        assert_eq!(a, b, "_i64toa v={v64} r={r}");

        let u64v = rng.next();
        let (mut a, mut b) = ([0xAAu8; 80], [0xAAu8; 80]);
        unsafe {
            cabi::_ui64toa(u64v, a.as_mut_ptr(), r);
            ref_ui64toa(u64v, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_ui64toa v={u64v} r={r}");

        // _ltoa / _itoa: i32; neg+radix!=10 is width-divergent on a
        // LP64 host → constrain to value>=0 or radix==10.
        let mut v32 = rng.next() as i32;
        if v32 < 0 && r != 10 {
            v32 = v32.wrapping_neg();
        }
        let (mut a, mut b) = ([0xAAu8; 80], [0xAAu8; 80]);
        unsafe {
            cabi::_ltoa(v32, a.as_mut_ptr(), r);
            ref_ltoa32(v32, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_ltoa v={v32} r={r}");
        let (mut a, mut b) = ([0xAAu8; 80], [0xAAu8; 80]);
        unsafe {
            cabi::_itoa(v32, a.as_mut_ptr(), r);
            ref_itoa(v32, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_itoa v={v32} r={r}");

        // _ultoa: u32, host adapter zero-extends → full u32 domain.
        let uv = rng.next() as u32;
        let (mut a, mut b) = ([0xAAu8; 80], [0xAAu8; 80]);
        unsafe {
            cabi::_ultoa(uv, a.as_mut_ptr(), r);
            ref_ultoa32(uv, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_ultoa v={uv} r={r}");
    }
    // Edge values.
    for &(v64, r) in &[
        (0i64, 10i32), (1, 10), (-1, 10), (i64::MIN, 10), (i64::MAX, 10),
        (-1, 16), (i64::MIN, 16), (i64::MAX, 2), (-1, 2), (255, 16), (46655, 36),
    ] {
        let (mut a, mut b) = ([0xAAu8; 80], [0xAAu8; 80]);
        unsafe {
            cabi::_i64toa(v64, a.as_mut_ptr(), r);
            ref_i64toa(v64, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_i64toa edge v={v64} r={r}");
    }
}

/// Wave C2 — unchecked wide int→str. Same domain split as C1; plus the
/// wide misfeature: NULL dst is accepted and the input ptr is returned
/// unchanged (no write, no crash).
#[test]
fn diff_itow_unchecked() {
    let mut rng = Rng(0x2c02_77a0_u64);
    let radices = [2i32, 8, 10, 16, 36];
    for _ in 0..6000 {
        let r = radices[rng.upto(radices.len())];
        let v64 = rng.next() as i64;
        let (mut a, mut b) = ([0xAAAAu16; 80], [0xAAAAu16; 80]);
        let pa = unsafe { cabi::_i64tow(v64, a.as_mut_ptr(), r) };
        let pb = unsafe { ref_i64tow(v64, b.as_mut_ptr(), r) };
        assert_eq!(pa, a.as_mut_ptr());
        assert_eq!(pb, b.as_mut_ptr());
        assert_eq!(a, b, "_i64tow v={v64} r={r}");

        let u64v = rng.next();
        let (mut a, mut b) = ([0xAAAAu16; 80], [0xAAAAu16; 80]);
        unsafe {
            cabi::_ui64tow(u64v, a.as_mut_ptr(), r);
            ref_ui64tow(u64v, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_ui64tow v={u64v} r={r}");

        let mut v32 = rng.next() as i32;
        if v32 < 0 && r != 10 {
            v32 = v32.wrapping_neg();
        }
        let (mut a, mut b) = ([0xAAAAu16; 80], [0xAAAAu16; 80]);
        unsafe {
            cabi::_ltow(v32, a.as_mut_ptr(), r);
            ref_ltow32(v32, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_ltow v={v32} r={r}");
        let (mut a, mut b) = ([0xAAAAu16; 80], [0xAAAAu16; 80]);
        unsafe {
            cabi::_itow(v32, a.as_mut_ptr(), r);
            ref_itow(v32, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_itow v={v32} r={r}");

        let uv = rng.next() as u32;
        let (mut a, mut b) = ([0xAAAAu16; 80], [0xAAAAu16; 80]);
        unsafe {
            cabi::_ultow(uv, a.as_mut_ptr(), r);
            ref_ultow32(uv, b.as_mut_ptr(), r);
        }
        assert_eq!(a, b, "_ultow v={uv} r={r}");
    }
    // NULL dst → returns NULL, no write (the wide misfeature).
    unsafe {
        assert!(cabi::_i64tow(42, core::ptr::null_mut(), 10).is_null());
        assert!(ref_i64tow(42, core::ptr::null_mut(), 10).is_null());
        assert!(cabi::_ui64tow(7, core::ptr::null_mut(), 16).is_null());
        assert!(cabi::_ltow(-9, core::ptr::null_mut(), 10).is_null());
        assert!(cabi::_itow(-9, core::ptr::null_mut(), 10).is_null());
        assert!(cabi::_ultow(123, core::ptr::null_mut(), 8).is_null());
    }
}

/// Wave C3 — secure `_s` narrow. Pins return code AND the full dst
/// buffer (the ERANGE reverse-backfill / str[0]=0 / no-fill contracts).
/// Sizes span 0..40 (tiny→exact→over) over many iters; radix set
/// includes invalid (1/37/0/-1) to exercise the EINVAL path. `_ltoa_s`
/// /`_itoa_s` skip neg+valid-radix!=10 (LP64-host width divergence).
#[test]
fn diff_itoa_s() {
    let mut rng = Rng(0x53c0_3a11_u64);
    let radices = [2i32, 8, 10, 16, 36, 1, 37, 0, -1];
    let valid = |r: i32| matches!(r, 2 | 8 | 16 | 36);
    for _ in 0..7000 {
        let r = radices[rng.upto(radices.len())];
        let sz = rng.upto(40);

        let v64 = rng.next() as i64;
        let (mut a, mut b) = ([0xAAu8; 90], [0xAAu8; 90]);
        let rca = unsafe { cabi::_i64toa_s(v64, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_i64toa_s(v64, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_i64toa_s rc v={v64} sz={sz} r={r}");
        assert_eq!(a, b, "_i64toa_s buf v={v64} sz={sz} r={r}");

        let uv = rng.next();
        let (mut a, mut b) = ([0xAAu8; 90], [0xAAu8; 90]);
        let rca = unsafe { cabi::_ui64toa_s(uv, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_ui64toa_s(uv, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_ui64toa_s rc v={uv} sz={sz} r={r}");
        assert_eq!(a, b, "_ui64toa_s buf v={uv} sz={sz} r={r}");

        let mut v32 = rng.next() as i32;
        if v32 < 0 && valid(r) {
            v32 = v32.wrapping_neg();
        }
        let (mut a, mut b) = ([0xAAu8; 90], [0xAAu8; 90]);
        let rca = unsafe { cabi::_ltoa_s(v32, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_ltoa_s32(v32, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_ltoa_s rc v={v32} sz={sz} r={r}");
        assert_eq!(a, b, "_ltoa_s buf v={v32} sz={sz} r={r}");
        let (mut a, mut b) = ([0xAAu8; 90], [0xAAu8; 90]);
        let rca = unsafe { cabi::_itoa_s(v32, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_itoa_s(v32, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_itoa_s rc v={v32} sz={sz} r={r}");
        assert_eq!(a, b, "_itoa_s buf v={v32} sz={sz} r={r}");
    }
    // str==NULL → EINVAL (22), no deref. _ui64toa_s also: no str[0]=0.
    unsafe {
        assert_eq!(cabi::_i64toa_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(ref_i64toa_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(cabi::_ui64toa_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(ref_ui64toa_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(cabi::_itoa_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(cabi::_ltoa_s(1, core::ptr::null_mut(), 8, 10), 22);
    }
}

/// Wave C4 — secure `_s` wide. s_erange_w mirrors C3 with u16; the
/// star is `_ui64tow_s`'s genuine byte-count bug (memcpy ndigits+1
/// BYTES, not * sizeof(wchar_t)) — the diff pins that exact half-wide
/// destination state against the real compiled itow.c.
#[test]
fn diff_itow_s() {
    let mut rng = Rng(0x77c0_4b22_u64);
    let radices = [2i32, 8, 10, 16, 36, 1, 37, 0, -1];
    let valid = |r: i32| matches!(r, 2 | 8 | 16 | 36);
    for _ in 0..7000 {
        let r = radices[rng.upto(radices.len())];
        let sz = rng.upto(40);

        let v64 = rng.next() as i64;
        let (mut a, mut b) = ([0xAAAAu16; 90], [0xAAAAu16; 90]);
        let rca = unsafe { cabi::_i64tow_s(v64, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_i64tow_s(v64, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_i64tow_s rc v={v64} sz={sz} r={r}");
        assert_eq!(a, b, "_i64tow_s buf v={v64} sz={sz} r={r}");

        let uv = rng.next();
        let (mut a, mut b) = ([0xAAAAu16; 90], [0xAAAAu16; 90]);
        let rca = unsafe { cabi::_ui64tow_s(uv, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_ui64tow_s(uv, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_ui64tow_s rc v={uv} sz={sz} r={r}");
        assert_eq!(a, b, "_ui64tow_s buf (byte-count bug) v={uv} sz={sz} r={r}");

        let mut v32 = rng.next() as i32;
        if v32 < 0 && valid(r) {
            v32 = v32.wrapping_neg();
        }
        let (mut a, mut b) = ([0xAAAAu16; 90], [0xAAAAu16; 90]);
        let rca = unsafe { cabi::_ltow_s(v32, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_ltow_s32(v32, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_ltow_s rc v={v32} sz={sz} r={r}");
        assert_eq!(a, b, "_ltow_s buf v={v32} sz={sz} r={r}");
        let (mut a, mut b) = ([0xAAAAu16; 90], [0xAAAAu16; 90]);
        let rca = unsafe { cabi::_itow_s(v32, a.as_mut_ptr(), sz, r) };
        let rcb = unsafe { ref_itow_s(v32, b.as_mut_ptr(), sz, r) };
        assert_eq!(rca, rcb, "_itow_s rc v={v32} sz={sz} r={r}");
        assert_eq!(a, b, "_itow_s buf v={v32} sz={sz} r={r}");
    }
    unsafe {
        assert_eq!(cabi::_i64tow_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(ref_i64tow_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(cabi::_ui64tow_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(ref_ui64tow_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(cabi::_itow_s(1, core::ptr::null_mut(), 8, 10), 22);
        assert_eq!(cabi::_ltow_s(1, core::ptr::null_mut(), 8, 10), 22);
    }
}

/// NUL-terminated wide string from a case/fold-stressing alphabet:
/// A/a/B/b/Z/z, the fold-boundary neighbours '@'(0x3F<'A') / '['(0x5B
/// >'Z'), and >=0x80 / >0xFF values that must NEVER be folded.
fn wcstr(rng: &mut Rng, max: usize) -> Vec<u16> {
    const A: [u16; 12] = [
        0x41, 0x61, 0x42, 0x62, 0x5A, 0x7A, 0x3F, 0x5B, 0x00C0, 0x0100, 0xFF21, 0x0080,
    ];
    let len = rng.upto(max + 1);
    let mut v: Vec<u16> = (0..len).map(|_| A[rng.upto(A.len())]).collect();
    v.push(0);
    v
}

/// Wave D — case-insensitive NT variants. Narrow uses the existing
/// `cstr` alpha (incl. >=0x80 for the signed-char path); wide uses
/// `wcstr` (incl. >0xFF, never folded). Pins the raw (un-clamped)
/// difference + the n==0/NUL-break bounds against compiled _*_nt.c.
#[test]
fn diff_icmp() {
    let mut rng = Rng(0x1cb0_5d33_u64);
    for _ in 0..9000 {
        // narrow _stricmp / _strnicmp
        let a = cstr(&mut rng, 22, true);
        let b = if rng.next() & 3 == 0 {
            a.clone()
        } else {
            cstr(&mut rng, 22, true)
        };
        let n = rng.upto(26);
        unsafe {
            assert_eq!(
                cabi::_stricmp(a.as_ptr(), b.as_ptr()),
                ref_stricmp(a.as_ptr(), b.as_ptr()),
                "_stricmp a={a:?} b={b:?}"
            );
            assert_eq!(
                cabi::_strnicmp(a.as_ptr(), b.as_ptr(), n),
                ref_strnicmp(a.as_ptr(), b.as_ptr(), n),
                "_strnicmp a={a:?} b={b:?} n={n}"
            );
        }
        // wide _wcsicmp / _wcsnicmp
        let wa = wcstr(&mut rng, 22);
        let wb = if rng.next() & 3 == 0 {
            wa.clone()
        } else {
            wcstr(&mut rng, 22)
        };
        let wn = rng.upto(26);
        unsafe {
            assert_eq!(
                cabi::_wcsicmp(wa.as_ptr(), wb.as_ptr()),
                ref_wcsicmp(wa.as_ptr(), wb.as_ptr()),
                "_wcsicmp wa={wa:?} wb={wb:?}"
            );
            assert_eq!(
                cabi::_wcsnicmp(wa.as_ptr(), wb.as_ptr(), wn),
                ref_wcsnicmp(wa.as_ptr(), wb.as_ptr(), wn),
                "_wcsnicmp wa={wa:?} wb={wb:?} n={wn}"
            );
        }
    }
}

/// Wave E1 — wcstoul (wide BSD cutoff/cutlim parse). ASCII domain
/// (iswctype for ASCII is standard); bases {0,2,8,10,16,36} (a
/// negative base's (unsigned long)base cast is host-divergent and
/// never fuzzed). Pins value AND endptr offset incl. the saturate-
/// but-still-advance + negate-after-saturate (→1) misfeatures, and
/// the endptr=NULL path, vs compiled wcstoul.c.
#[test]
fn diff_wcstoul() {
    let mut rng = Rng(0x7c50_1a44_u64);
    let alpha: &[u16] = &[
        0x20, 0x09, b'+' as u16, b'-' as u16, b'0' as u16, b'x' as u16, b'X' as u16,
        b'1' as u16, b'7' as u16, b'9' as u16, b'a' as u16, b'f' as u16, b'F' as u16,
        b'A' as u16, b'Z' as u16, b'z' as u16, b'g' as u16, 0x00C0,
    ];
    let bases = [0i32, 2, 8, 10, 16, 36];
    for _ in 0..9000 {
        let len = rng.upto(24);
        let mut sv: Vec<u16> = (0..len).map(|_| alpha[rng.upto(alpha.len())]).collect();
        sv.push(0);
        let base = bases[rng.upto(bases.len())];
        let (mut ec, mut er): (*mut u16, *mut u16) = (1 as *mut u16, 1 as *mut u16);
        let cv = unsafe { cabi::wcstoul(sv.as_ptr(), &mut ec, base) };
        let rv = unsafe { ref_wcstoul(sv.as_ptr(), &mut er, base) };
        assert_eq!(cv, rv, "wcstoul v s={sv:?} base={base}");
        let off = |p: *mut u16| {
            if p == 1 as *mut u16 {
                None
            } else {
                Some((p as usize - sv.as_ptr() as usize) / 2)
            }
        };
        assert_eq!(off(ec), off(er), "wcstoul endptr s={sv:?} base={base}");
        // endptr == NULL path (no write, value still correct).
        let cv2 = unsafe { cabi::wcstoul(sv.as_ptr(), core::ptr::null_mut(), base) };
        let rv2 = unsafe { ref_wcstoul(sv.as_ptr(), core::ptr::null_mut(), base) };
        assert_eq!(cv2, rv2, "wcstoul NULL-endptr s={sv:?} base={base}");
    }
}

/// Wave E2 — multibyte ⇄ wide (CP1252 single-byte model). Genuine
/// wrapper .c linked against an identical C CP1252 Rtl* stub, so the
/// diff pins the wrapper logic over the whole 0x00-0xFF byte domain
/// (incl. the 0x80-0x9F table + undefined slots) and the wide side
/// over ASCII/Latin-1/table-targets/unmappable.
#[test]
fn diff_mbcp() {
    // toupper_nt_mb — whole low-byte domain + negative / >0xFF inputs.
    for c in -4i32..=300 {
        unsafe {
            assert_eq!(
                cabi::toupper_nt_mb(c),
                ref_toupper_nt_mb(c),
                "toupper_nt_mb c={c}"
            );
        }
    }
    for &c in &[0x1234i32, 0x7F41, -256, i32::MIN, i32::MAX, 0xFF, 0xB5, 0xDF] {
        unsafe {
            assert_eq!(cabi::toupper_nt_mb(c), ref_toupper_nt_mb(c), "toupper c={c}");
        }
    }

    // mbtowc — every byte, count {0,1,2,3}, + NULL guards.
    for b in 0u16..=255 {
        for &cnt in &[0usize, 1, 2, 3] {
            let mbbuf = [b as u8, 0x41, 0x42, 0];
            let (mut wc, mut wr): (u16, u16) = (0xEEEE, 0xEEEE);
            let rc = unsafe { cabi::mbtowc(&mut wc, mbbuf.as_ptr(), cnt) };
            let rr = unsafe { ref_mbtowc(&mut wr, mbbuf.as_ptr(), cnt) };
            assert_eq!(rc, rr, "mbtowc ret b={b:#x} cnt={cnt}");
            if rc != -1 {
                assert_eq!(wc, wr, "mbtowc *wchar b={b:#x} cnt={cnt}");
            }
        }
    }
    unsafe {
        assert_eq!(cabi::mbtowc(core::ptr::null_mut(), b"a\0".as_ptr(), 1), 0);
        assert_eq!(ref_mbtowc(core::ptr::null_mut(), b"a\0".as_ptr(), 1), 0);
        let mut w = 0u16;
        assert_eq!(cabi::mbtowc(&mut w, core::ptr::null(), 1), 0);
        assert_eq!(ref_mbtowc(&mut w, core::ptr::null(), 1), 0);
    }

    // wctomb — wide domain incl. table targets + unmappable, + NULL.
    for w in 0u16..=0x21FF {
        let (mut a, mut b) = (0xAAu8, 0xAAu8);
        let rc = unsafe { cabi::wctomb(&mut a, w) };
        let rr = unsafe { ref_wctomb(&mut b, w) };
        assert_eq!(rc, rr, "wctomb ret w={w:#x}");
        assert_eq!(a, b, "wctomb *mbchar w={w:#x}");
    }
    unsafe {
        assert_eq!(cabi::wctomb(core::ptr::null_mut(), 0x41), 0);
        assert_eq!(ref_wctomb(core::ptr::null_mut(), 0x41), 0);
    }

    let mut rng = Rng(0x3b9a_ca07_u64);
    for _ in 0..4000 {
        // mbstowcs: NUL-terminated mb string (bytes 1..=255).
        let len = rng.upto(20);
        let mut mb: Vec<u8> = (0..len).map(|_| (1 + rng.upto(255)) as u8).collect();
        mb.push(0);
        let count = rng.upto(24);
        // query (wcstr == NULL)
        unsafe {
            assert_eq!(
                cabi::mbstowcs(core::ptr::null_mut(), mb.as_ptr(), count),
                ref_mbstowcs(core::ptr::null_mut(), mb.as_ptr(), count),
                "mbstowcs query mb={mb:?}"
            );
        }
        let (mut a, mut b) = ([0xEEEEu16; 32], [0xEEEEu16; 32]);
        let ra = unsafe { cabi::mbstowcs(a.as_mut_ptr(), mb.as_ptr(), count) };
        let rb = unsafe { ref_mbstowcs(b.as_mut_ptr(), mb.as_ptr(), count) };
        assert_eq!(ra, rb, "mbstowcs ret mb={mb:?} count={count}");
        assert_eq!(a, b, "mbstowcs buf mb={mb:?} count={count}");

        // wcstombs: NUL-terminated wide string (ASCII/Latin-1/table/
        // unmappable, no embedded 0).
        let wl = rng.upto(20);
        const WSET: [u16; 8] = [0x41, 0x7A, 0xA9, 0xE9, 0xFF, 0x20AC, 0x2013, 0x4E2D];
        let mut ws: Vec<u16> = (0..wl).map(|_| WSET[rng.upto(8)]).collect();
        ws.push(0);
        let wc = rng.upto(24);
        unsafe {
            assert_eq!(
                cabi::wcstombs(core::ptr::null_mut(), ws.as_ptr(), wc),
                ref_wcstombs(core::ptr::null_mut(), ws.as_ptr(), wc),
                "wcstombs query ws={ws:?}"
            );
        }
        let (mut a, mut b) = ([0xAAu8; 32], [0xAAu8; 32]);
        let ra = unsafe { cabi::wcstombs(a.as_mut_ptr(), ws.as_ptr(), wc) };
        let rb = unsafe { ref_wcstombs(b.as_mut_ptr(), ws.as_ptr(), wc) };
        assert_eq!(ra, rb, "wcstombs ret ws={ws:?} wc={wc}");
        assert_eq!(a, b, "wcstombs buf ws={ws:?} wc={wc}");
    }
}

/// tcs* tchar family auto-derive — the 7 wide twins of the live
/// narrow cores + strcpy/strcat/strncat (narrow+wide). One generic
/// `core_rs::string` impl over `Elem`; the diff pins each variant
/// vs the genuine tcs*.h logic header instantiated narrow/wide.
#[test]
fn diff_tcs_family() {
    let mut rng = Rng(0x7c5f_a311_u64);
    let off16 = |p: *mut u16, base: *const u16| {
        if p.is_null() {
            None
        } else {
            Some((p as usize - base as usize) / 2)
        }
    };
    for _ in 0..8000 {
        let wa = wcstr(&mut rng, 24);
        let wb = if rng.next() & 3 == 0 {
            wa.clone()
        } else {
            wcstr(&mut rng, 24)
        };
        let n = rng.upto(28);
        unsafe {
            // wcslen / wcsnlen / wcscmp / wcsncmp
            assert_eq!(cabi::wcslen(wa.as_ptr()), ref_wcslen(wa.as_ptr()), "wcslen {wa:?}");
            assert_eq!(
                cabi::wcsnlen(wa.as_ptr(), n),
                ref_wcsnlen(wa.as_ptr(), n),
                "wcsnlen {wa:?} n={n}"
            );
            assert_eq!(
                cabi::wcscmp(wa.as_ptr(), wb.as_ptr()),
                ref_wcscmp(wa.as_ptr(), wb.as_ptr()),
                "wcscmp {wa:?} {wb:?}"
            );
            assert_eq!(
                cabi::wcsncmp(wa.as_ptr(), wb.as_ptr(), n),
                ref_wcsncmp(wa.as_ptr(), wb.as_ptr(), n),
                "wcsncmp {wa:?} {wb:?} n={n}"
            );
            // wcschr / wcsrchr
            let c: u16 = match rng.upto(4) {
                0 if wa.len() > 1 => wa[rng.upto(wa.len() - 1)],
                1 => 0,
                2 => 0x0100 + (rng.next() as u16 & 0xFF),
                _ => rng.next() as u16,
            };
            assert_eq!(
                off16(cabi::wcschr(wa.as_ptr(), c), wa.as_ptr()),
                off16(ref_wcschr(wa.as_ptr(), c), wa.as_ptr()),
                "wcschr {wa:?} c={c:#x}"
            );
            assert_eq!(
                off16(cabi::wcsrchr(wa.as_ptr(), c), wa.as_ptr()),
                off16(ref_wcsrchr(wa.as_ptr(), c), wa.as_ptr()),
                "wcsrchr {wa:?} c={c:#x}"
            );
            // wcsncpy — fixed-n dst
            let (mut da, mut db) = ([0xBEEFu16; 40], [0xBEEFu16; 40]);
            let nn = rng.upto(36);
            let pa = cabi::wcsncpy(da.as_mut_ptr(), wa.as_ptr(), nn);
            let pb = ref_wcsncpy(db.as_mut_ptr(), wa.as_ptr(), nn);
            assert_eq!(pa, da.as_mut_ptr());
            assert_eq!(pb, db.as_mut_ptr());
            assert_eq!(da, db, "wcsncpy {wa:?} n={nn}");
            // wcscpy / wcscat / wcsncat (raw unbounded dst)
            let (mut da, mut db) = ([0xBEEFu16; 64], [0xBEEFu16; 64]);
            assert_eq!(cabi::wcscpy(da.as_mut_ptr(), wa.as_ptr()), da.as_mut_ptr());
            ref_wcscpy(db.as_mut_ptr(), wa.as_ptr());
            assert_eq!(da, db, "wcscpy {wa:?}");
            // cat onto a base
            let base = wcstr(&mut rng, 10);
            let mk = || {
                let mut v = [0xBEEFu16; 72];
                v[..base.len()].copy_from_slice(&base);
                v
            };
            let (mut da, mut db) = (mk(), mk());
            cabi::wcscat(da.as_mut_ptr(), wb.as_ptr());
            ref_wcscat(db.as_mut_ptr(), wb.as_ptr());
            assert_eq!(da, db, "wcscat base={base:?} ap={wb:?}");
            let (mut da, mut db) = (mk(), mk());
            cabi::wcsncat(da.as_mut_ptr(), wb.as_ptr(), n);
            ref_wcsncat(db.as_mut_ptr(), wb.as_ptr(), n);
            assert_eq!(da, db, "wcsncat base={base:?} ap={wb:?} n={n}");
        }

        // narrow strcpy / strcat / strncat
        let sa = cstr(&mut rng, 24, false);
        let sb = cstr(&mut rng, 16, false);
        unsafe {
            let (mut da, mut db) = ([0xAAu8; 64], [0xAAu8; 64]);
            assert_eq!(cabi::strcpy(da.as_mut_ptr(), sa.as_ptr()), da.as_mut_ptr());
            ref_strcpy(db.as_mut_ptr(), sa.as_ptr());
            assert_eq!(da, db, "strcpy {sa:?}");
            let nbase = cstr(&mut rng, 10, false);
            let mkn = || {
                let mut v = [0xAAu8; 72];
                v[..nbase.len()].copy_from_slice(&nbase);
                v
            };
            let (mut da, mut db) = (mkn(), mkn());
            cabi::strcat(da.as_mut_ptr(), sb.as_ptr());
            ref_strcat(db.as_mut_ptr(), sb.as_ptr());
            assert_eq!(da, db, "strcat base={nbase:?} ap={sb:?}");
            let (mut da, mut db) = (mkn(), mkn());
            cabi::strncat(da.as_mut_ptr(), sb.as_ptr(), n);
            ref_strncat(db.as_mut_ptr(), sb.as_ptr(), n);
            assert_eq!(da, db, "strncat base={nbase:?} ap={sb:?} n={n}");
        }
    }
    unsafe {
        assert_eq!(cabi::wcslen(core::ptr::null()), ref_wcslen(core::ptr::null()));
        assert_eq!(
            cabi::wcsnlen(core::ptr::null(), 5),
            ref_wcsnlen(core::ptr::null(), 5)
        );
    }
}
