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
    fn ref_memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
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
