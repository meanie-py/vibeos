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
    fn ref_strspn(s: *const u8, a: *const u8) -> usize;
    fn ref_strcspn(s: *const u8, a: *const u8) -> usize;
    fn ref_strpbrk(s: *const u8, a: *const u8) -> *mut u8;
    fn ref_wcsspn(s: *const u16, a: *const u16) -> usize;
    fn ref_wcscspn(s: *const u16, a: *const u16) -> usize;
    fn ref_wcsstr(s: *const u16, b: *const u16) -> *mut u16;
    fn ref_strtol(n: *const u8, e: *mut *mut u8, b: i32) -> i32;
    fn ref_strtoull(n: *const u8, e: *mut *mut u8, b: i32) -> u64;
    fn ref_wcstol(n: *const u16, e: *mut *mut u16, b: i32) -> i32;
    fn ref_wtoi(n: *const u16) -> i32;
    fn ref_wtol(n: *const u16) -> i32;
    fn ref_wtoi64(n: *const u16) -> i64;
    fn ref_wtoi64_l(n: *const u16, loc: *mut c_void) -> i64;
    fn ref_wcstoi64(n: *const u16, e: *mut *mut u16, b: i32) -> i64;
    fn ref_wcstoi64_l(n: *const u16, e: *mut *mut u16, b: i32, loc: *mut c_void) -> i64;
    fn ref_wcstoui64(n: *const u16, e: *mut *mut u16, b: i32) -> u64;
    fn ref_wcstoui64_l(n: *const u16, e: *mut *mut u16, b: i32, loc: *mut c_void) -> u64;
    fn ref_strlwr(s: *mut u8) -> *mut u8;
    fn ref_strupr(s: *mut u8) -> *mut u8;
    fn ref_wcslwr(s: *mut u16) -> *mut u16;
    fn ref_wcsupr(s: *mut u16) -> *mut u16;
    fn ref_tolower(c: i32) -> i32;
    fn ref_towupper(c: u16) -> u16;
    fn ref_strrev(s: *mut u8) -> *mut u8;
    fn ref_strset(s: *mut u8, fill: i32) -> *mut u8;
    fn ref_strnset(s: *mut u8, fill: i32, max: usize) -> *mut u8;
    fn ref_splitpath(p: *const u8, dr: *mut u8, di: *mut u8, fnm: *mut u8, ex: *mut u8);
    fn ref_wsplitpath(p: *const u16, dr: *mut u16, di: *mut u16, fnm: *mut u16, ex: *mut u16);
    fn ref_makepath(p: *mut u8, dr: *const u8, di: *const u8, fnm: *const u8, ex: *const u8);
    fn ref_wmakepath(p: *mut u16, dr: *const u16, di: *const u16, fnm: *const u16, ex: *const u16);
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

/// Wave G — BSD parse + endptr (strtol/wcstol signed-long clamp,
/// strtoull u64). ASCII domain, sane bases. Pins value + endptr
/// (incl. the any?s-1:nptr rule and the overflow→LONG_MIN/MAX /
/// ULLONG_MAX clamps) vs genuine strtol.c/wcstol.c/strtoull.c.
#[test]
fn diff_strtol_family() {
    let mut rng = Rng(0x6a17_0c2d_u64);
    let bases = [0i32, 2, 8, 10, 16, 36];
    let alpha = b" \t+-0xXabcdefABCDEF0123456789Zz";
    for _ in 0..9000 {
        let mut s: Vec<u8> = (0..rng.upto(24))
            .map(|_| alpha[rng.upto(alpha.len())])
            .collect();
        s.push(0);
        let base = bases[rng.upto(bases.len())];
        let off = |p: *mut u8| {
            if p == 1 as *mut u8 {
                None
            } else {
                Some(p as usize - s.as_ptr() as usize)
            }
        };
        // strtol / strtoull (narrow)
        let (mut ec, mut er): (*mut u8, *mut u8) = (1 as *mut u8, 1 as *mut u8);
        let cv = unsafe { cabi::strtol(s.as_ptr(), &mut ec, base) };
        let rv = unsafe { ref_strtol(s.as_ptr(), &mut er, base) };
        assert_eq!(cv, rv, "strtol v s={s:?} base={base}");
        assert_eq!(off(ec), off(er), "strtol endptr s={s:?} base={base}");
        let (mut uc, mut ur): (*mut u8, *mut u8) = (1 as *mut u8, 1 as *mut u8);
        let ucv = unsafe { cabi::strtoull(s.as_ptr(), &mut uc, base) };
        let urv = unsafe { ref_strtoull(s.as_ptr(), &mut ur, base) };
        assert_eq!(ucv, urv, "strtoull v s={s:?} base={base}");
        assert_eq!(off(uc), off(ur), "strtoull endptr s={s:?} base={base}");
        // wcstol (wide)
        let ws: Vec<u16> = s.iter().map(|&b| b as u16).collect();
        let woff = |p: *mut u16| {
            if p == 1 as *mut u16 {
                None
            } else {
                Some((p as usize - ws.as_ptr() as usize) / 2)
            }
        };
        let (mut wc, mut wr): (*mut u16, *mut u16) = (1 as *mut u16, 1 as *mut u16);
        let wcv = unsafe { cabi::wcstol(ws.as_ptr(), &mut wc, base) };
        let wrv = unsafe { ref_wcstol(ws.as_ptr(), &mut wr, base) };
        assert_eq!(wcv, wrv, "wcstol v s={s:?} base={base}");
        assert_eq!(woff(wc), woff(wr), "wcstol endptr s={s:?} base={base}");
        // endptr == NULL path (value still correct, no write)
        let cv2 = unsafe { cabi::strtol(s.as_ptr(), core::ptr::null_mut(), base) };
        let rv2 = unsafe { ref_strtol(s.as_ptr(), core::ptr::null_mut(), base) };
        assert_eq!(cv2, rv2, "strtol NULL-endptr s={s:?} base={base}");
    }
}

/// Wave F — charset span/search. Small alphabets so set membership
/// actually hits; empty set / empty needle exercised explicitly.
#[test]
fn diff_span() {
    let mut rng = Rng(0x5fa0_b17e_u64);
    for _ in 0..9000 {
        // narrow: full-range bytes incl >=0x80 (unsigned-char bitmap)
        let s = cstr(&mut rng, 28, true);
        let set = if rng.next() & 7 == 0 {
            vec![0] // empty set
        } else {
            cstr(&mut rng, 5, true)
        };
        unsafe {
            assert_eq!(
                cabi::strspn(s.as_ptr(), set.as_ptr()),
                ref_strspn(s.as_ptr(), set.as_ptr()),
                "strspn s={s:?} set={set:?}"
            );
            assert_eq!(
                cabi::strcspn(s.as_ptr(), set.as_ptr()),
                ref_strcspn(s.as_ptr(), set.as_ptr()),
                "strcspn s={s:?} set={set:?}"
            );
        }
        // strpbrk: strpbrk.c's 3+-char path uses a bitmap sized by
        // BIT_SIZE = sizeof(unsigned long)*8 — LP32 on the target but
        // 64 on this host, where `1 << (>=32)` is UB → the genuine
        // source computes garbage. The 0/1/2-char strchr fast paths
        // are host-correct AND fully exercise our find_any (set size
        // doesn't change `position(|c| set.contains(c))`). So fuzz
        // set ≤2 here; 3+ is the same first-in-set contract, just
        // unfuzzable on LP64 (documented, like the itoa long-family).
        {
            let s = cstr(&mut rng, 28, true);
            let mut set: Vec<u8> = (0..rng.upto(3))
                .map(|_| b"AaBb \x80\xff"[rng.upto(7)])
                .collect();
            set.push(0);
            let base = s.as_ptr() as usize;
            let off = |p: *mut u8| if p.is_null() { None } else { Some(p as usize - base) };
            unsafe {
                assert_eq!(
                    off(cabi::strpbrk(s.as_ptr(), set.as_ptr())),
                    off(ref_strpbrk(s.as_ptr(), set.as_ptr())),
                    "strpbrk s={s:?} set={set:?}"
                );
            }
        }
        // wide span + wcsstr (incl empty needle → NULL on both)
        let ws = wcstr(&mut rng, 24);
        let wset = if rng.next() & 7 == 0 {
            vec![0]
        } else {
            wcstr(&mut rng, 5)
        };
        let needle = if rng.next() & 3 == 0 {
            vec![0] // empty needle → NULL (distinct from strstr)
        } else if rng.next() & 3 == 0 && ws.len() > 2 {
            let a = rng.upto(ws.len() - 1);
            let b = a + 1 + rng.upto(ws.len() - a - 1);
            let mut n = ws[a..b].to_vec();
            n.push(0);
            n
        } else {
            wcstr(&mut rng, 4)
        };
        unsafe {
            assert_eq!(
                cabi::wcsspn(ws.as_ptr(), wset.as_ptr()),
                ref_wcsspn(ws.as_ptr(), wset.as_ptr()),
                "wcsspn ws={ws:?} set={wset:?}"
            );
            assert_eq!(
                cabi::wcscspn(ws.as_ptr(), wset.as_ptr()),
                ref_wcscspn(ws.as_ptr(), wset.as_ptr()),
                "wcscspn ws={ws:?} set={wset:?}"
            );
            let base = ws.as_ptr() as usize;
            let off = |p: *mut u16| {
                if p.is_null() {
                    None
                } else {
                    Some((p as usize - base) / 2)
                }
            };
            assert_eq!(
                off(cabi::wcsstr(ws.as_ptr(), needle.as_ptr())),
                off(ref_wcsstr(ws.as_ptr(), needle.as_ptr())),
                "wcsstr ws={ws:?} needle={needle:?}"
            );
        }
    }
}

/// Wave G2 — wide-atoi parse (owed split from Wave G). Pins the
/// MSVCRT-lineage wide parsers vs genuine wtoi64.c/wtol.c/atoi.c:
/// `_wtoi`(int-trunc), `_wtol`(u32-wrap→int), `_wtoi64{,_l}`(i64-wrap),
/// `_wcstoi64{,_l}`/`_wcstoui64{,_l}` (per-digit MAX clamp + the wide
/// endptr contract = final nptr ALWAYS, incl. invalid-base→0/no-write
/// and NULL-nptr→0/no-write). ASCII domain, incl. bases 1/37.
#[test]
fn diff_wide_atoi() {
    let mut rng = Rng(0x9c2a_d175_u64);
    let bases = [0i32, 2, 8, 10, 16, 36, 1, 37];
    let alpha = b" \t+-0xXabcdefABCDEF0123456789Zz";
    for _ in 0..9000 {
        let s: Vec<u8> = (0..rng.upto(24))
            .map(|_| alpha[rng.upto(alpha.len())])
            .collect();
        let ws: Vec<u16> = s.iter().map(|&b| b as u16).chain([0]).collect();
        let np = ws.as_ptr();
        let base = bases[rng.upto(bases.len())];
        let nul: *mut c_void = core::ptr::null_mut();
        let woff = |p: *mut u16| {
            if p == 1 as *mut u16 {
                None
            } else {
                Some((p as usize - np as usize) / 2)
            }
        };

        // no-endptr value parsers
        assert_eq!(
            unsafe { cabi::_wtoi(np) },
            unsafe { ref_wtoi(np) },
            "_wtoi ws={ws:?}"
        );
        assert_eq!(
            unsafe { cabi::_wtol(np) },
            unsafe { ref_wtol(np) },
            "_wtol ws={ws:?}"
        );
        assert_eq!(
            unsafe { cabi::_wtoi64(np) },
            unsafe { ref_wtoi64(np) },
            "_wtoi64 ws={ws:?}"
        );
        assert_eq!(
            unsafe { cabi::_wtoi64_l(np, nul) },
            unsafe { ref_wtoi64_l(np, nul) },
            "_wtoi64_l ws={ws:?}"
        );

        // signed clamp + wide endptr
        let (mut ec, mut er): (*mut u16, *mut u16) = (1 as *mut u16, 1 as *mut u16);
        let cv = unsafe { cabi::_wcstoi64(np, &mut ec, base) };
        let rv = unsafe { ref_wcstoi64(np, &mut er, base) };
        assert_eq!(cv, rv, "_wcstoi64 v ws={ws:?} base={base}");
        assert_eq!(woff(ec), woff(er), "_wcstoi64 ep ws={ws:?} base={base}");
        let (mut ecl, mut erl): (*mut u16, *mut u16) = (1 as *mut u16, 1 as *mut u16);
        let cvl = unsafe { cabi::_wcstoi64_l(np, &mut ecl, base, nul) };
        let rvl = unsafe { ref_wcstoi64_l(np, &mut erl, base, nul) };
        assert_eq!(cvl, rvl, "_wcstoi64_l v ws={ws:?} base={base}");
        assert_eq!(woff(ecl), woff(erl), "_wcstoi64_l ep ws={ws:?} base={base}");

        // unsigned clamp + wide endptr
        let (mut uc, mut ur): (*mut u16, *mut u16) = (1 as *mut u16, 1 as *mut u16);
        let ucv = unsafe { cabi::_wcstoui64(np, &mut uc, base) };
        let urv = unsafe { ref_wcstoui64(np, &mut ur, base) };
        assert_eq!(ucv, urv, "_wcstoui64 v ws={ws:?} base={base}");
        assert_eq!(woff(uc), woff(ur), "_wcstoui64 ep ws={ws:?} base={base}");
        let (mut ucl, mut url): (*mut u16, *mut u16) = (1 as *mut u16, 1 as *mut u16);
        let ucvl = unsafe { cabi::_wcstoui64_l(np, &mut ucl, base, nul) };
        let urvl = unsafe { ref_wcstoui64_l(np, &mut url, base, nul) };
        assert_eq!(ucvl, urvl, "_wcstoui64_l v ws={ws:?} base={base}");
        assert_eq!(woff(ucl), woff(url), "_wcstoui64_l ep ws={ws:?} base={base}");

        // endptr == NULL: value still correct, no write
        let cv2 = unsafe { cabi::_wcstoi64(np, core::ptr::null_mut(), base) };
        let rv2 = unsafe { ref_wcstoi64(np, core::ptr::null_mut(), base) };
        assert_eq!(cv2, rv2, "_wcstoi64 NULL-ep ws={ws:?} base={base}");
    }

    // NULL nptr → 0, endptr untouched (MSVCRT_CHECK_PMT under _LIBCNT_)
    for base in bases {
        let (mut ec, mut er): (*mut u16, *mut u16) = (7 as *mut u16, 7 as *mut u16);
        assert_eq!(
            unsafe { cabi::_wcstoi64(core::ptr::null(), &mut ec, base) },
            unsafe { ref_wcstoi64(core::ptr::null(), &mut er, base) },
            "_wcstoi64 NULL-nptr v base={base}"
        );
        assert_eq!(ec, er, "_wcstoi64 NULL-nptr ep untouched base={base}");
        let (mut uc, mut ur): (*mut u16, *mut u16) = (7 as *mut u16, 7 as *mut u16);
        assert_eq!(
            unsafe { cabi::_wcstoui64(core::ptr::null(), &mut uc, base) },
            unsafe { ref_wcstoui64(core::ptr::null(), &mut ur, base) },
            "_wcstoui64 NULL-nptr v base={base}"
        );
        assert_eq!(uc, ur, "_wcstoui64 NULL-nptr ep untouched base={base}");
    }
}

/// Wave H — ASCII case fold. In-place folders: byte-exact buffer
/// parity + returned pointer == input (or NULL for the NULL-guarded
/// _strupr/_wcslwr). Wide folders fed Latin-1/CJK units to pin the
/// ASCII-ONLY contract (À etc. must stay untouched). Scalars
/// (tolower signed-char-decision+full-int; towupper = Rtl/CP1252
/// upcase) swept across the full domain incl. negatives / >0xFFFF.
#[test]
fn diff_casemap() {
    let mut rng = Rng(0x4ca5_e009_u64);
    for _ in 0..6000 {
        // narrow string: letters + digits + punct + 0x80-0xFF
        let pool = b"AZaz mM5.\x80\xC0\xDF\xE0\xFF";
        let mut nv: Vec<u8> =
            (0..rng.upto(28)).map(|_| pool[rng.upto(pool.len())]).collect();
        nv.push(0);
        let (mut c1, mut r1) = (nv.clone(), nv.clone());
        let cp = unsafe { cabi::_strlwr(c1.as_mut_ptr()) };
        let rp = unsafe { ref_strlwr(r1.as_mut_ptr()) };
        assert_eq!(c1, r1, "_strlwr buf nv={nv:?}");
        assert_eq!(cp == c1.as_mut_ptr(), rp == r1.as_mut_ptr(), "_strlwr ret");
        let (mut c2, mut r2) = (nv.clone(), nv.clone());
        let cp = unsafe { cabi::_strupr(c2.as_mut_ptr()) };
        let rp = unsafe { ref_strupr(r2.as_mut_ptr()) };
        assert_eq!(c2, r2, "_strupr buf nv={nv:?}");
        assert_eq!(cp == c2.as_mut_ptr(), rp == r2.as_mut_ptr(), "_strupr ret");

        // wide string: ASCII + Latin-1 + CJK (must stay untouched)
        let wpool: [u16; 14] = [
            65, 90, 97, 122, 32, 0x4D, 0x35, 0xC0, 0xDF, 0xE9, 0xFF, 0x0152,
            0x4E2D, 0x017E,
        ];
        let mut wv: Vec<u16> = (0..rng.upto(24))
            .map(|_| wpool[rng.upto(wpool.len())])
            .collect();
        wv.push(0);
        let (mut wc1, mut wr1) = (wv.clone(), wv.clone());
        let cp = unsafe { cabi::_wcslwr(wc1.as_mut_ptr()) };
        let rp = unsafe { ref_wcslwr(wr1.as_mut_ptr()) };
        assert_eq!(wc1, wr1, "_wcslwr buf wv={wv:?}");
        assert_eq!(cp == wc1.as_mut_ptr(), rp == wr1.as_mut_ptr(), "_wcslwr ret");
        let (mut wc2, mut wr2) = (wv.clone(), wv.clone());
        let cp = unsafe { cabi::_wcsupr(wc2.as_mut_ptr()) };
        let rp = unsafe { ref_wcsupr(wr2.as_mut_ptr()) };
        assert_eq!(wc2, wr2, "_wcsupr buf wv={wv:?}");
        assert_eq!(cp == wc2.as_mut_ptr(), rp == wr2.as_mut_ptr(), "_wcsupr ret");
    }

    // NULL-guarded variants: NULL → NULL (no crash)
    assert_eq!(
        unsafe { cabi::_strupr(core::ptr::null_mut()) },
        unsafe { ref_strupr(core::ptr::null_mut()) },
        "_strupr NULL"
    );
    assert_eq!(
        unsafe { cabi::_wcslwr(core::ptr::null_mut()) },
        unsafe { ref_wcslwr(core::ptr::null_mut()) },
        "_wcslwr NULL"
    );

    // scalar tolower: full int domain incl. negative / >0xFF / >0xFFFF
    let probes: &[i32] = &[
        -1, -65, 0, 0x41, 0x5A, 0x60, 0x61, 0x7A, 0x7F, 0x80, 0xC1, 0xFF,
        0x141, 0x161, 0x100_0041, -0x100_0041, i32::MIN, i32::MAX,
    ];
    for &c in probes {
        assert_eq!(
            unsafe { cabi::tolower(c) },
            unsafe { ref_tolower(c) },
            "tolower c={c:#x}"
        );
    }
    let mut rng2 = Rng(0x70_10_e7_a1);
    for _ in 0..4000 {
        let c = rng2.next() as i32;
        assert_eq!(
            unsafe { cabi::tolower(c) },
            unsafe { ref_tolower(c) },
            "tolower c={c:#x}"
        );
    }

    // scalar towupper: every u16 (Rtl/CP1252 upcase)
    for c in 0u32..=0xFFFF {
        let c = c as u16;
        assert_eq!(
            unsafe { cabi::towupper(c) },
            unsafe { ref_towupper(c) },
            "towupper c={c:#x}"
        );
    }
}

/// Wave I — in-place mutate (strrev/strset/strnset). Byte-exact buffer
/// parity + returned pointer == input, across empty/odd/even lengths,
/// high bytes, szFill truncation, and the _strnset signed-int
/// sizeMaxFill misfeature (0, < / == / > strlen, and > INT_MAX → fills
/// nothing). NO NULL fuzzing (the C has no guards — derefs immediately).
#[test]
fn diff_strmut() {
    let mut rng = Rng(0x12ed_5b07_u64);
    let pool = b"ABCabc 09.\x00\x80\xFF\x01";
    for _ in 0..7000 {
        // build a string; embedded 0 from pool acts as the terminator
        let mut v: Vec<u8> = (0..rng.upto(26))
            .map(|_| {
                let b = pool[rng.upto(pool.len())];
                if b == 0 { 1 } else { b }
            })
            .collect();
        v.push(0);

        let (mut c1, mut r1) = (v.clone(), v.clone());
        let cp = unsafe { cabi::_strrev(c1.as_mut_ptr()) };
        let rp = unsafe { ref_strrev(r1.as_mut_ptr()) };
        assert_eq!(c1, r1, "_strrev buf v={v:?}");
        assert_eq!(cp == c1.as_mut_ptr(), rp == r1.as_mut_ptr(), "_strrev ret");

        let fill = (rng.next() as i32) | 1; // varied incl >0xFF / negative
        let (mut c2, mut r2) = (v.clone(), v.clone());
        let cp = unsafe { cabi::_strset(c2.as_mut_ptr(), fill) };
        let rp = unsafe { ref_strset(r2.as_mut_ptr(), fill) };
        assert_eq!(c2, r2, "_strset buf v={v:?} fill={fill:#x}");
        assert_eq!(cp == c2.as_mut_ptr(), rp == r2.as_mut_ptr(), "_strset ret");

        // _strnset: max spanning 0..past-strlen plus the >INT_MAX trap
        let slen = v.len() - 1;
        let maxes: [usize; 6] = [
            0,
            rng.upto(slen + 1),
            slen,
            slen + 5,
            usize::MAX,
            (i32::MAX as usize) + 1,
        ];
        for mx in maxes {
            let (mut c3, mut r3) = (v.clone(), v.clone());
            let cp = unsafe { cabi::_strnset(c3.as_mut_ptr(), fill, mx) };
            let rp = unsafe { ref_strnset(r3.as_mut_ptr(), fill, mx) };
            assert_eq!(c3, r3, "_strnset buf v={v:?} fill={fill:#x} max={mx}");
            assert_eq!(
                cp == c3.as_mut_ptr(),
                rp == r3.as_mut_ptr(),
                "_strnset ret max={mx}"
            );
        }
    }
}

/// Wave PATH — _splitpath/_wsplitpath/_makepath/_wmakepath. Byte-exact
/// parity of every OUT buffer (sentinel-prefilled so over/under-writes
/// AND the truncate-to-"" tail are caught) across the full
/// equivalence grid: drive present/absent, dir trailing-sep forms,
/// leading-dot fname, dot-in-dir discard, no-sep, trailing-sep,
/// per-pointer NULL, empty-component skip. NO NULL `path` for split
/// (UB in C, WINVER<0x600). makepath NULL `path` IS pinned (no-op).
/// 256-byte buffers (never the INT_MAX cap).
#[test]
fn diff_path() {
    let mut rng = Rng(0x9a17_c0de_u64);
    let pool: &[u8] = b"ab.\\/:cd.txt..\\";
    for _ in 0..8000 {
        let mut v: Vec<u8> = (0..rng.upto(24))
            .map(|_| pool[rng.upto(pool.len())])
            .collect();
        v.push(0);
        let vw: Vec<u16> = v.iter().map(|&b| b as u16).collect();
        let mask = rng.next() as u32;

        // ---- _splitpath / _wsplitpath : 4 nullable OUT bufs --------
        let (mut cd, mut ci, mut cf, mut ce) =
            (vec![0xCDu8; 260], vec![0xCDu8; 260], vec![0xCDu8; 260], vec![0xCDu8; 260]);
        let (mut rd, mut ri, mut rf, mut re) =
            (vec![0xCDu8; 260], vec![0xCDu8; 260], vec![0xCDu8; 260], vec![0xCDu8; 260]);
        macro_rules! p8 { ($b:expr, $buf:expr) => {
            if mask & $b != 0 { core::ptr::null_mut() } else { $buf.as_mut_ptr() }
        }}
        unsafe {
            cabi::_splitpath(v.as_ptr(), p8!(1, cd), p8!(2, ci), p8!(4, cf), p8!(8, ce));
            ref_splitpath(v.as_ptr(), p8!(1, rd), p8!(2, ri), p8!(4, rf), p8!(8, re));
        }
        assert_eq!(cd, rd, "_splitpath drive v={v:?} mask={mask:#x}");
        assert_eq!(ci, ri, "_splitpath dir v={v:?} mask={mask:#x}");
        assert_eq!(cf, rf, "_splitpath fname v={v:?} mask={mask:#x}");
        assert_eq!(ce, re, "_splitpath ext v={v:?} mask={mask:#x}");

        let (mut wcd, mut wci, mut wcf, mut wce) =
            (vec![0xCDCDu16; 260], vec![0xCDCDu16; 260], vec![0xCDCDu16; 260], vec![0xCDCDu16; 260]);
        let (mut wrd, mut wri, mut wrf, mut wre) =
            (vec![0xCDCDu16; 260], vec![0xCDCDu16; 260], vec![0xCDCDu16; 260], vec![0xCDCDu16; 260]);
        macro_rules! p16 { ($b:expr, $buf:expr) => {
            if mask & $b != 0 { core::ptr::null_mut() } else { $buf.as_mut_ptr() }
        }}
        unsafe {
            cabi::_wsplitpath(vw.as_ptr(), p16!(1, wcd), p16!(2, wci), p16!(4, wcf), p16!(8, wce));
            ref_wsplitpath(vw.as_ptr(), p16!(1, wrd), p16!(2, wri), p16!(4, wrf), p16!(8, wre));
        }
        assert_eq!(wcd, wrd, "_wsplitpath drive v={v:?}");
        assert_eq!(wci, wri, "_wsplitpath dir v={v:?}");
        assert_eq!(wcf, wrf, "_wsplitpath fname v={v:?}");
        assert_eq!(wce, wre, "_wsplitpath ext v={v:?}");

        // ---- _makepath / _wmakepath : 4 nullable IN components -----
        let comp = |rng: &mut Rng| -> Vec<u8> {
            let mut c: Vec<u8> = (0..rng.upto(8)).map(|_| pool[rng.upto(pool.len())]).collect();
            c.push(0);
            c
        };
        let (dr, di, fl, ex) = (comp(&mut rng), comp(&mut rng), comp(&mut rng), comp(&mut rng));
        let m2 = rng.next() as u32;
        let pk = |b: u32, c: &Vec<u8>| -> *const u8 {
            if m2 & b != 0 { core::ptr::null() } else { c.as_ptr() }
        };
        let (mut cmk, mut rmk) = (vec![0xEEu8; 280], vec![0xEEu8; 280]);
        unsafe {
            cabi::_makepath(cmk.as_mut_ptr(), pk(1, &dr), pk(2, &di), pk(4, &fl), pk(8, &ex));
            ref_makepath(rmk.as_mut_ptr(), pk(1, &dr), pk(2, &di), pk(4, &fl), pk(8, &ex));
        }
        assert_eq!(cmk, rmk, "_makepath dr={dr:?} di={di:?} fl={fl:?} ex={ex:?} m2={m2:#x}");
        unsafe {
            cabi::_makepath(core::ptr::null_mut(), dr.as_ptr(), di.as_ptr(), fl.as_ptr(), ex.as_ptr());
            ref_makepath(core::ptr::null_mut(), dr.as_ptr(), di.as_ptr(), fl.as_ptr(), ex.as_ptr());
        }
        let w = |c: &Vec<u8>| -> Vec<u16> { c.iter().map(|&b| b as u16).collect() };
        let (wdr, wdi, wfl, wex) = (w(&dr), w(&di), w(&fl), w(&ex));
        let wpk = |b: u32, c: &Vec<u16>| -> *const u16 {
            if m2 & b != 0 { core::ptr::null() } else { c.as_ptr() }
        };
        let (mut wcmk, mut wrmk) = (vec![0xEEEEu16; 280], vec![0xEEEEu16; 280]);
        unsafe {
            cabi::_wmakepath(wcmk.as_mut_ptr(), wpk(1, &wdr), wpk(2, &wdi), wpk(4, &wfl), wpk(8, &wex));
            ref_wmakepath(wrmk.as_mut_ptr(), wpk(1, &wdr), wpk(2, &wdi), wpk(4, &wfl), wpk(8, &wex));
        }
        assert_eq!(wcmk, wrmk, "_wmakepath m2={m2:#x}");
    }
}
