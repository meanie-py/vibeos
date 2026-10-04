#![no_builtins] // host-harness only: cnt's #[no_mangle] libc-named shims (isdigit/strlen/...) must be CALLED, not folded to LLVM builtins (rustc-1.92 release UB-exploits out-of-range args). No effect on cnt-rust/cnt-ctype/kernel (separate crates/targets).
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
    fn ref_memccpy(to: *mut c_void, from: *const c_void, c: i32, count: usize) -> *mut c_void;
    fn ref_memicmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32;
    fn ref_rand() -> i32;
    fn ref_srand(seed: u32);
    fn ref_qsort(base: *mut c_void, n: usize, es: usize, cmp: QCmp);
    fn ref_bsearch(key: *const c_void, base: *const c_void, nelem: usize, size: usize, cmp: QCmp) -> *mut c_void;
    fn ref_lfind(key: *const c_void, base: *const c_void, nelp: *const u32, width: u32, cmp: QCmp) -> *mut c_void;
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
    fn ref_strtok(s: *mut u8, d: *const u8) -> *mut u8;
    fn ref_strtok_s(s: *mut u8, d: *const u8, c: *mut *mut u8) -> *mut u8;
    fn ref_wcstok(s: *mut u16, d: *const u16) -> *mut u16;
    fn ref_wcstok_s(s: *mut u16, d: *const u16, n: *mut *mut u16) -> *mut u16;
    // cnt-rust's sscanf/swscanf/_snscanf/_snwscanf come from its C
    // trampoline (scanf_trampoline.c → cnt_v*scanf in cabi), plain
    // SysV. Variadic *callees* are stable to declare+call (only
    // DEFINING a variadic needs nightly).
    fn sscanf(s: *const u8, f: *const u8, ...) -> i32;
    fn swscanf(s: *const u16, f: *const u16, ...) -> i32;
    fn _snscanf(s: *const u8, n: usize, f: *const u8, ...) -> i32;
    fn _snwscanf(s: *const u16, n: usize, f: *const u16, ...) -> i32;
    fn ref_sscanf(s: *const u8, f: *const u8, ...) -> i32;
    fn ref_swscanf(s: *const u16, f: *const u16, ...) -> i32;
    fn ref__snscanf(s: *const u8, n: usize, f: *const u8, ...) -> i32;
    fn ref__snwscanf(s: *const u16, n: usize, f: *const u16, ...) -> i32;
    // scanf float: winesup.c's _internal_handle_float va_list*
    // indirection is unportable to the host harness, so the genuine
    // store call site is overridden by a shim running winesup.c's
    // EXACT synthesis math into this captured pointer (the genuine
    // PARSE in scanf.h is still fully exercised). Set before a
    // genuine-side float scan; the genuine return value is unaffected.
    fn cnt_set_fout(p: *mut core::ffi::c_void, is_double: i32);
    // env — genuine getenv.c/putenv.c/wputenv.c + environ.c SetEnv,
    // seeded via ref_env_set_block (BlockEnvToEnvironW+A over a
    // CP1252-modeled Win32 environment surface).
    fn ref_env_set_block(w: *const u16, n: i32);
    fn ref_getenv(name: *const u8) -> *mut u8;
    fn ref__wgetenv(name: *const u16) -> *mut u16;
    fn ref__putenv(val: *const u8) -> i32;
    fn ref__wputenv(val: *const u16) -> i32;
    fn ref___p__environ() -> *mut *mut *mut u8;
    fn ref___p__wenviron() -> *mut *mut *mut u16;
    fn ref___p___initenv() -> *mut *mut *mut u8;
    fn ref___p__winitenv() -> *mut *mut *mut u16;
    // float-cvt — genuine fcvtbuf.c `cvt()` + fcvt.c.
    fn ref__fcvt(v: f64, ndigits: i32, decpt: *mut i32, sign: *mut i32) -> *mut u8;
    // _ecvt — genuine ecvt.c + streamout.c sprintf("%-+.*E"),
    // -mlong-double-64, floor/log10/pow modeled == core_rs::fl.
    fn ref__ecvt(v: f64, ndigits: i32, decpt: *mut i32, sign: *mut i32) -> *mut u8;
    // _gcvt/_gcvt_s — genuine gcvt.c + streamout.c sprintf("%.*g").
    fn ref__gcvt(number: f64, ndigit: i32, buff: *mut u8) -> *mut u8;
    fn ref__gcvt_s(buff: *mut u8, size: usize, number: f64, digits: i32) -> i32;
    // strtod — genuine string/strtod.c, -mlong-double-64 (f64 semantics,
    // == the cnt-rust f64 core; the documented i386 f80 deviation is the
    // gap vs the OLD i386 C, not vs this reference).
    fn ref_strtod(s: *const u8, sret: *mut *mut u8) -> f64;
    // Wave WCS — kernel (_LIBCNT_) slice of string/wcs.c (8 wide leaves).
    fn ref__wcsnset(s: *mut u16, c: u16, n: usize) -> *mut u16;
    fn ref__wcsrev(s: *mut u16) -> *mut u16;
    fn ref_wcscoll(a: *const u16, b: *const u16) -> i32;
    fn ref_wcspbrk(s: *const u16, accept: *const u16) -> *mut u16;
    fn ref_wcscpy_s(d: *mut u16, num: usize, s: *const u16) -> i32;
    fn ref_wcsncpy_s(d: *mut u16, num: usize, s: *const u16, count: usize) -> i32;
    fn ref_wcscat_s(d: *mut u16, elem: usize, s: *const u16) -> i32;
    fn ref_wcsncat_s(d: *mut u16, elem: usize, s: *const u16, count: usize) -> i32;
    // Wave CTYPE — ctype.c. Tables + narrow classification + wide
    // forwarders (the genuine compiled C, build.rs TU-A). The C names
    // with leading underscores map to suffixed ref names (build.rs).
    static ref_ctype: [u16; 257];
    static ref_wctype: [u16; 257];
    fn ref_isctype(c: i32, f: i32) -> i32;
    fn ref_isalpha(c: i32) -> i32;
    fn ref_isalnum(c: i32) -> i32;
    fn ref_iscntrl(c: i32) -> i32;
    fn ref_isdigit(c: i32) -> i32;
    fn ref_isgraph(c: i32) -> i32;
    fn ref_islower(c: i32) -> i32;
    fn ref_isprint(c: i32) -> i32;
    fn ref_ispunct(c: i32) -> i32;
    fn ref_isspace(c: i32) -> i32;
    fn ref_isupper(c: i32) -> i32;
    fn ref_isxdigit(c: i32) -> i32;
    fn ref_isascii_n(c: i32) -> i32; // __isascii
    fn ref_toascii_n(c: i32) -> i32; // __toascii
    fn ref_iscsym_n(c: i32) -> i32; // __iscsym
    fn ref_iscsymf_n(c: i32) -> i32; // __iscsymf
    fn ref_tolower_u(c: i32) -> i32; // _tolower
    fn ref_toupper_u(c: i32) -> i32; // _toupper
    fn ref_toupper(c: i32) -> i32;
    fn ref_iswascii(c: u16) -> i32;
    fn ref_iswalnum(c: u16) -> i32;
    fn ref_iswalpha(c: u16) -> i32;
    fn ref_iswcntrl(c: u16) -> i32;
    fn ref_iswdigit(c: u16) -> i32;
    fn ref_iswgraph(c: u16) -> i32;
    fn ref_iswprint(c: u16) -> i32;
    fn ref_iswpunct(c: u16) -> i32;
    fn ref_iswlower(c: u16) -> i32;
    fn ref_iswupper(c: u16) -> i32;
    fn ref_iswspace(c: u16) -> i32;
    fn ref_iswxdigit(c: u16) -> i32;
    fn ref_towlower(c: u16) -> u16;
    fn ref_iswctype(c: u16, t: u16) -> i32; // iswctype_nt.c (flipped)
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

fn diff_strlen() {
    let mut rng = Rng(0x1234_5678);
    for _ in 0..5000 {
        let s = cstr(&mut rng, 64, false);
        unsafe { assert_eq!(cabi::strlen(s.as_ptr()), ref_strlen(s.as_ptr()), "strlen {s:?}") }
    }
    // NULL → 0 (preserved misfeature).
    unsafe { assert_eq!(cabi::strlen(core::ptr::null()), ref_strlen(core::ptr::null())) }
}

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

/// Wave STRTOK — strtok/strtok_s/wcstok/wcstok_s. Drives cnt-rust and
/// the genuine reference through the SAME full tokenize sequence on
/// independent buffer copies; after each call asserts (1) token
/// offset+nullness parity, (2) the whole buffer (NUL writes) parity,
/// (3) for the _s variants the *ctx offset parity — incl. the
/// strtok_s-vs-wcstok_s no-token-path asymmetry. Plus the bad-param
/// guards. The hidden statics are independent per side but the
/// identical call stream keeps them in lockstep.
fn diff_strtok() {
    let mut rng = Rng(0x704b_17aa_u64);
    let pool: &[u8] = b"ab,; :.c";
    let dpool: &[u8] = b",; :.";
    for _ in 0..6000 {
        let mut base: Vec<u8> = (0..rng.upto(20))
            .map(|_| pool[rng.upto(pool.len())])
            .collect();
        base.push(0);
        let dl = 1 + rng.upto(3);
        let mut delim: Vec<u8> = (0..dl).map(|_| dpool[rng.upto(dpool.len())]).collect();
        delim.push(0);
        let dw: Vec<u16> = delim.iter().map(|&b| b as u16).collect();

        // ---- strtok (hidden static) ----
        let (mut cb, mut rb) = (base.clone(), base.clone());
        let mut first = true;
        loop {
            let cs = if first { cb.as_mut_ptr() } else { core::ptr::null_mut() };
            let rs = if first { rb.as_mut_ptr() } else { core::ptr::null_mut() };
            let cr = unsafe { cabi::strtok(cs, delim.as_ptr()) };
            let rr = unsafe { ref_strtok(rs, delim.as_ptr()) };
            let co = if cr.is_null() { None } else { Some(cr as usize - cb.as_ptr() as usize) };
            let ro = if rr.is_null() { None } else { Some(rr as usize - rb.as_ptr() as usize) };
            assert_eq!(co, ro, "strtok off base={base:?} delim={delim:?}");
            assert_eq!(cb, rb, "strtok buf base={base:?} delim={delim:?}");
            if cr.is_null() { break; }
            first = false;
        }

        // ---- strtok_s (explicit ctx) ----
        let (mut cb, mut rb) = (base.clone(), base.clone());
        let (mut cc, mut rc): (*mut u8, *mut u8) =
            (core::ptr::null_mut(), core::ptr::null_mut());
        let mut first = true;
        loop {
            let cs = if first { cb.as_mut_ptr() } else { core::ptr::null_mut() };
            let rs = if first { rb.as_mut_ptr() } else { core::ptr::null_mut() };
            let cr = unsafe { cabi::strtok_s(cs, delim.as_ptr(), &mut cc) };
            let rr = unsafe { ref_strtok_s(rs, delim.as_ptr(), &mut rc) };
            let co = if cr.is_null() { None } else { Some(cr as usize - cb.as_ptr() as usize) };
            let ro = if rr.is_null() { None } else { Some(rr as usize - rb.as_ptr() as usize) };
            let cx = if cc.is_null() { None } else { Some(cc as usize - cb.as_ptr() as usize) };
            let rx = if rc.is_null() { None } else { Some(rc as usize - rb.as_ptr() as usize) };
            assert_eq!(co, ro, "strtok_s off base={base:?} delim={delim:?}");
            assert_eq!(cx, rx, "strtok_s ctx base={base:?} delim={delim:?}");
            assert_eq!(cb, rb, "strtok_s buf base={base:?} delim={delim:?}");
            if cr.is_null() { break; }
            first = false;
        }

        // ---- wcstok (hidden static) ----
        let bw: Vec<u16> = base.iter().map(|&b| b as u16).collect();
        let (mut cw, mut rw) = (bw.clone(), bw.clone());
        let mut first = true;
        loop {
            let cs = if first { cw.as_mut_ptr() } else { core::ptr::null_mut() };
            let rs = if first { rw.as_mut_ptr() } else { core::ptr::null_mut() };
            let cr = unsafe { cabi::wcstok(cs, dw.as_ptr()) };
            let rr = unsafe { ref_wcstok(rs, dw.as_ptr()) };
            let co = if cr.is_null() { None } else { Some((cr as usize - cw.as_ptr() as usize) / 2) };
            let ro = if rr.is_null() { None } else { Some((rr as usize - rw.as_ptr() as usize) / 2) };
            assert_eq!(co, ro, "wcstok off base={base:?}");
            assert_eq!(cw, rw, "wcstok buf base={base:?}");
            if cr.is_null() { break; }
            first = false;
        }

        // ---- wcstok_s (explicit ctx; no-token leaves *nt unchanged) ----
        let (mut cw, mut rw) = (bw.clone(), bw.clone());
        let (mut cn, mut rn): (*mut u16, *mut u16) =
            (core::ptr::null_mut(), core::ptr::null_mut());
        let mut first = true;
        loop {
            let cs = if first { cw.as_mut_ptr() } else { core::ptr::null_mut() };
            let rs = if first { rw.as_mut_ptr() } else { core::ptr::null_mut() };
            let cr = unsafe { cabi::wcstok_s(cs, dw.as_ptr(), &mut cn) };
            let rr = unsafe { ref_wcstok_s(rs, dw.as_ptr(), &mut rn) };
            let co = if cr.is_null() { None } else { Some((cr as usize - cw.as_ptr() as usize) / 2) };
            let ro = if rr.is_null() { None } else { Some((rr as usize - rw.as_ptr() as usize) / 2) };
            let cx = if cn.is_null() { None } else { Some((cn as usize - cw.as_ptr() as usize) / 2) };
            let rx = if rn.is_null() { None } else { Some((rn as usize - rw.as_ptr() as usize) / 2) };
            assert_eq!(co, ro, "wcstok_s off base={base:?}");
            assert_eq!(cx, rx, "wcstok_s nt base={base:?}");
            assert_eq!(cw, rw, "wcstok_s buf base={base:?}");
            if cr.is_null() { break; }
            first = false;
        }
    }

    // ---- bad-param guards (strtok_s / wcstok_s) ----
    let mut s = *b"a,b\0";
    let d = *b",\0";
    let mut ctx: *mut u8 = core::ptr::null_mut();
    unsafe {
        assert!(cabi::strtok_s(core::ptr::null_mut(), d.as_ptr(), &mut ctx).is_null());
        assert!(ref_strtok_s(core::ptr::null_mut(), d.as_ptr(), &mut ctx).is_null());
        assert!(cabi::strtok_s(s.as_mut_ptr(), core::ptr::null(), &mut ctx).is_null());
        assert!(ref_strtok_s(s.as_mut_ptr(), core::ptr::null(), &mut ctx).is_null());
        assert!(cabi::strtok_s(s.as_mut_ptr(), d.as_ptr(), core::ptr::null_mut()).is_null());
        assert!(ref_strtok_s(s.as_mut_ptr(), d.as_ptr(), core::ptr::null_mut()).is_null());
    }
    let mut sw: [u16; 4] = [b'a' as u16, b',' as u16, b'b' as u16, 0];
    let dw2: [u16; 2] = [b',' as u16, 0];
    let mut ntw: *mut u16 = core::ptr::null_mut();
    unsafe {
        assert!(cabi::wcstok_s(core::ptr::null_mut(), dw2.as_ptr(), &mut ntw).is_null());
        assert!(ref_wcstok_s(core::ptr::null_mut(), dw2.as_ptr(), &mut ntw).is_null());
        assert!(cabi::wcstok_s(sw.as_mut_ptr(), core::ptr::null(), &mut ntw).is_null());
        assert!(ref_wcstok_s(sw.as_mut_ptr(), core::ptr::null(), &mut ntw).is_null());
        assert!(cabi::wcstok_s(sw.as_mut_ptr(), dw2.as_ptr(), core::ptr::null_mut()).is_null());
        assert!(ref_wcstok_s(sw.as_mut_ptr(), dw2.as_ptr(), core::ptr::null_mut()).is_null());
    }
}

/// Wave SCANF Phase 1 — integer/string/char/scanset/%n/literal/ws/
/// width/suppress/prefixes for sscanf+swscanf. Float (%e/%f/%g) is
/// Phase 2 (x87-long-double-vs-f64 parity, owed) and NOT fuzzed here.
/// Fixed per-shape concrete arg lists (Rust can't build a dynamic
/// va_list); each shape pins return value + every out + a trailing
/// %n (the consumed-1 quirk). Buffers sentinel-prefilled so over/
/// under-write and the %s/%[/%c terminator asymmetry are caught.
fn diff_scanf() {
    let mut rng = Rng(0x5caf_d1f0_u64);
    let cstr0 = |v: &[u8]| {
        let mut x = v.to_vec();
        x.push(0);
        x
    };
    for _ in 0..7000 {
        // a token-ish input with ws/sign/digits/letters/punct
        let pool = b"  \t12+-0xabZ.,;0789AF[]^ qword";
        let raw: Vec<u8> = (0..rng.upto(22))
            .map(|_| pool[rng.upto(pool.len())])
            .collect();
        let inp = cstr0(&raw);
        match rng.upto(18) {
            0 => {
                // %d + %n
                let f = cstr0(b"%d%n");
                let (mut a, mut b) = (-7i32, -7i32);
                let (mut na, mut nb) = (-9i32, -9i32);
                let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                let q = unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                assert_eq!((r, a, na), (q, b, nb), "%d inp={raw:?}");
            }
            1 => {
                let f = cstr0(b"%i%n");
                let (mut a, mut b, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
                let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                let q = unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                assert_eq!((r, a, na), (q, b, nb), "%i inp={raw:?}");
            }
            2 => {
                for fmt in [&b"%x%n"[..], b"%o%n", b"%u%n", b"%X%n"] {
                    let f = cstr0(fmt);
                    let (mut a, mut b, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
                    let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                    let q =
                        unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                    assert_eq!((r, a, na), (q, b, nb), "{fmt:?} inp={raw:?}");
                }
            }
            3 => {
                // width + suppression
                for fmt in [&b"%3d%n"[..], b"%*d%d%n", b"%2x%n"] {
                    let f = cstr0(fmt);
                    let (mut a, mut b, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
                    let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                    let q =
                        unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                    assert_eq!((r, a, na), (q, b, nb), "{fmt:?} inp={raw:?}");
                }
            }
            4 => {
                // length prefixes: %ld(LONG=i32) %hd(short) %lld/%I64d(i64)
                let f = cstr0(b"%ld%n");
                let (mut a, mut b, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
                let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                let q = unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                assert_eq!((r, a, na), (q, b, nb), "%ld inp={raw:?}");
                let f = cstr0(b"%hd%n");
                let (mut a, mut b) = (0i16, 0i16);
                let (mut na, mut nb) = (-1i32, -1i32);
                let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                let q = unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                assert_eq!((r, a, na), (q, b, nb), "%hd inp={raw:?}");
                for fmt in [&b"%lld%n"[..], b"%I64d%n"] {
                    let f = cstr0(fmt);
                    let (mut a, mut b) = (0i64, 0i64);
                    let (mut na, mut nb) = (-1i32, -1i32);
                    let r = unsafe { sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                    let q =
                        unsafe { ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                    assert_eq!((r, a, na), (q, b, nb), "{fmt:?} inp={raw:?}");
                }
            }
            5 => {
                // %s + %n (sentinel buffers)
                let f = cstr0(b"%s%n");
                let mut ba = [0xCCu8; 64];
                let mut bb = [0xCCu8; 64];
                let (mut na, mut nb) = (-1i32, -1i32);
                let r =
                    unsafe { sscanf(inp.as_ptr(), f.as_ptr(), ba.as_mut_ptr(), &mut na) };
                let q = unsafe {
                    ref_sscanf(inp.as_ptr(), f.as_ptr(), bb.as_mut_ptr(), &mut nb)
                };
                assert_eq!((r, na), (q, nb), "%s ret inp={raw:?}");
                assert_eq!(ba, bb, "%s buf inp={raw:?}");
            }
            6 => {
                // %c (no terminator) and %5c
                for fmt in [&b"%c%n"[..], b"%5c%n"] {
                    let f = cstr0(fmt);
                    let mut ba = [0xCCu8; 16];
                    let mut bb = [0xCCu8; 16];
                    let (mut na, mut nb) = (-1i32, -1i32);
                    let r = unsafe {
                        sscanf(inp.as_ptr(), f.as_ptr(), ba.as_mut_ptr(), &mut na)
                    };
                    let q = unsafe {
                        ref_sscanf(inp.as_ptr(), f.as_ptr(), bb.as_mut_ptr(), &mut nb)
                    };
                    assert_eq!((r, na), (q, nb), "{fmt:?} ret inp={raw:?}");
                    assert_eq!(ba, bb, "{fmt:?} buf inp={raw:?}");
                }
            }
            7 => {
                // scanset
                for fmt in
                    [&b"%[a-z]%n"[..], b"%[^,]%n", b"%[]ab]%n", b"%[0-9A-F]%n", b"%[^ ]%n"]
                {
                    let f = cstr0(fmt);
                    let mut ba = [0xCCu8; 48];
                    let mut bb = [0xCCu8; 48];
                    let (mut na, mut nb) = (-1i32, -1i32);
                    let r = unsafe {
                        sscanf(inp.as_ptr(), f.as_ptr(), ba.as_mut_ptr(), &mut na)
                    };
                    let q = unsafe {
                        ref_sscanf(inp.as_ptr(), f.as_ptr(), bb.as_mut_ptr(), &mut nb)
                    };
                    assert_eq!((r, na), (q, nb), "{fmt:?} ret inp={raw:?}");
                    assert_eq!(ba, bb, "{fmt:?} buf inp={raw:?}");
                }
            }
            8 => {
                // literal + %% + ws-in-format
                for fmt in [&b"%d,%d%n"[..], b"%%%d%n", b"  %d  %n", b"ab%d%n"] {
                    let f = cstr0(fmt);
                    let (mut a, mut b, mut c, mut d) = (0i32, 0i32, 0i32, 0i32);
                    let (mut na, mut nb) = (-1i32, -1i32);
                    let r = unsafe {
                        sscanf(inp.as_ptr(), f.as_ptr(), &mut a, &mut c, &mut na)
                    };
                    let q = unsafe {
                        ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, &mut d, &mut nb)
                    };
                    assert_eq!((r, a, c, na), (q, b, d, nb), "{fmt:?} inp={raw:?}");
                }
            }
            9 => {
                // multi: int + string + hex
                let f = cstr0(b"%d %s %x%n");
                let (mut a, mut b) = (0i32, 0i32);
                let mut sa = [0xCCu8; 40];
                let mut sb = [0xCCu8; 40];
                let (mut xa, mut xb, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
                let r = unsafe {
                    sscanf(inp.as_ptr(), f.as_ptr(), &mut a, sa.as_mut_ptr(), &mut xa, &mut na)
                };
                let q = unsafe {
                    ref_sscanf(inp.as_ptr(), f.as_ptr(), &mut b, sb.as_mut_ptr(), &mut xb, &mut nb)
                };
                assert_eq!((r, a, xa, na), (q, b, xb, nb), "multi ret inp={raw:?}");
                assert_eq!(sa, sb, "multi str inp={raw:?}");
            }
            10 => {
                // empty input / empty format edge
                let e = cstr0(b"");
                let f = cstr0(b"%d");
                let (mut a, mut b) = (5i32, 5i32);
                let r = unsafe { sscanf(e.as_ptr(), f.as_ptr(), &mut a) };
                let q = unsafe { ref_sscanf(e.as_ptr(), f.as_ptr(), &mut b) };
                assert_eq!((r, a), (q, b), "empty-input");
                let ef = cstr0(b"");
                let r = unsafe { sscanf(inp.as_ptr(), ef.as_ptr()) };
                let q = unsafe { ref_sscanf(inp.as_ptr(), ef.as_ptr()) };
                assert_eq!(r, q, "empty-format");
            }
            _ => {
                // swscanf wide twins (u16 buffers / int* outs)
                let win: Vec<u16> = inp.iter().map(|&b| b as u16).collect();
                let wf = |v: &[u8]| -> Vec<u16> {
                    v.iter().map(|&b| b as u16).chain([0]).collect()
                };
                let f = wf(b"%d%n");
                let (mut a, mut b, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
                let r = unsafe { swscanf(win.as_ptr(), f.as_ptr(), &mut a, &mut na) };
                let q = unsafe { ref_swscanf(win.as_ptr(), f.as_ptr(), &mut b, &mut nb) };
                assert_eq!((r, a, na), (q, b, nb), "w%d inp={raw:?}");
                let f = wf(b"%s%n");
                let mut ba = [0xCCCCu16; 48];
                let mut bb = [0xCCCCu16; 48];
                let (mut na, mut nb) = (-1i32, -1i32);
                let r =
                    unsafe { swscanf(win.as_ptr(), f.as_ptr(), ba.as_mut_ptr(), &mut na) };
                let q = unsafe {
                    ref_swscanf(win.as_ptr(), f.as_ptr(), bb.as_mut_ptr(), &mut nb)
                };
                assert_eq!((r, na), (q, nb), "w%s ret inp={raw:?}");
                assert_eq!(ba, bb, "w%s buf inp={raw:?}");
                let f = wf(b"%[a-z]%n");
                let mut ba = [0xCCCCu16; 48];
                let mut bb = [0xCCCCu16; 48];
                let (mut na, mut nb) = (-1i32, -1i32);
                let r =
                    unsafe { swscanf(win.as_ptr(), f.as_ptr(), ba.as_mut_ptr(), &mut na) };
                let q = unsafe {
                    ref_swscanf(win.as_ptr(), f.as_ptr(), bb.as_mut_ptr(), &mut nb)
                };
                assert_eq!((r, na), (q, nb), "w%[ ret inp={raw:?}");
                assert_eq!(ba, bb, "w%[ buf inp={raw:?}");
            }
        }
    }
}

/// Wave SCANF Phase 3 — _snscanf/_snwscanf (STRING_LEN). The input is
/// length-bounded (NOT NUL-bounded — reads embedded NULs up to
/// `length`); pins the genuine `consumed==length ? '\0'` (no advance)
/// quirk across length ∈ {0, <strlen, ==strlen, >strlen} + an
/// embedded-NUL buffer. Float excluded (Phase 2). Fixed concrete arg
/// shapes; return + every out + trailing %n parity, sentinel buffers.
fn diff_snscanf() {
    let mut rng = Rng(0x5caf_3a30_u64);
    for _ in 0..7000 {
        // raw input buffer (NOT NUL-terminated; may contain a NUL)
        let pool = b"  12+-0xabZ.,;\x0089AF[] qword";
        let mut buf: Vec<u8> = (0..(4 + rng.upto(20)))
            .map(|_| pool[rng.upto(pool.len())])
            .collect();
        // sometimes plant an explicit embedded NUL mid-buffer
        if rng.upto(2) == 0 && buf.len() > 3 {
            let p = 1 + rng.upto(buf.len() - 2);
            buf[p] = 0;
        }
        let blen = buf.len();
        let lengths = [
            0usize,
            rng.upto(blen + 1),
            blen,
            blen + 5, // > buffer: genuine reads OOB too, but our slice
                      // is exactly `length`; cap to blen to stay sound
                      // while still exercising length>consumed-able.
        ];
        let fmts: [&[u8]; 5] =
            [b"%d%n\0", b"%s%n\0", b"%[a-z]%n\0", b"%c%n\0", b"%x%n\0"];
        for &ln in &lengths {
            let ln = ln.min(blen); // keep from_raw_parts in-bounds
            for f in fmts {
                let fp = f.as_ptr();
                match f[1] {
                    b'd' | b'x' => {
                        let (mut a, mut b) = (0i32, 0i32);
                        let (mut na, mut nb) = (-1i32, -1i32);
                        let r = unsafe {
                            _snscanf(buf.as_ptr(), ln, fp, &mut a, &mut na)
                        };
                        let q = unsafe {
                            ref__snscanf(buf.as_ptr(), ln, fp, &mut b, &mut nb)
                        };
                        assert_eq!(
                            (r, a, na),
                            (q, b, nb),
                            "_snscanf {f:?} buf={buf:?} ln={ln}"
                        );
                    }
                    _ => {
                        let mut sa = [0xCCu8; 64];
                        let mut sb = [0xCCu8; 64];
                        let (mut na, mut nb) = (-1i32, -1i32);
                        let r = unsafe {
                            _snscanf(buf.as_ptr(), ln, fp, sa.as_mut_ptr(), &mut na)
                        };
                        let q = unsafe {
                            ref__snscanf(buf.as_ptr(), ln, fp, sb.as_mut_ptr(), &mut nb)
                        };
                        assert_eq!((r, na), (q, nb), "_snscanf {f:?} ret ln={ln}");
                        assert_eq!(sa, sb, "_snscanf {f:?} buf={buf:?} ln={ln}");
                    }
                }
            }
            // wide twin: %d + %s + %[
            let wbuf: Vec<u16> = buf.iter().map(|&b| b as u16).collect();
            let wf = |v: &[u8]| -> Vec<u16> {
                v.iter().map(|&b| b as u16).chain([0]).collect()
            };
            let fd = wf(b"%d%n");
            let (mut a, mut b, mut na, mut nb) = (0i32, 0i32, -1i32, -1i32);
            let r = unsafe {
                _snwscanf(wbuf.as_ptr(), ln, fd.as_ptr(), &mut a, &mut na)
            };
            let q = unsafe {
                ref__snwscanf(wbuf.as_ptr(), ln, fd.as_ptr(), &mut b, &mut nb)
            };
            assert_eq!((r, a, na), (q, b, nb), "_snwscanf %d buf={buf:?} ln={ln}");
            for ff in [&b"%s%n"[..], b"%[a-z]%n"] {
                let fw = wf(ff);
                let mut sa = [0xCCCCu16; 64];
                let mut sb = [0xCCCCu16; 64];
                let (mut na, mut nb) = (-1i32, -1i32);
                let r = unsafe {
                    _snwscanf(wbuf.as_ptr(), ln, fw.as_ptr(), sa.as_mut_ptr(), &mut na)
                };
                let q = unsafe {
                    ref__snwscanf(wbuf.as_ptr(), ln, fw.as_ptr(), sb.as_mut_ptr(), &mut nb)
                };
                assert_eq!((r, na), (q, nb), "_snwscanf {ff:?} ret ln={ln}");
                assert_eq!(sa, sb, "_snwscanf {ff:?} buf={buf:?} ln={ln}");
            }
        }
    }
}

/// Wave SCANF Phase 2 — float %e/%f/%g/%E/%G (+ l/L → double). The
/// cnt-rust side runs its real engine + VaSink (f64 synthesis). The
/// genuine side runs scanf.h's real PARSE; its `_internal_handle_float`
/// store call site is overridden by a shim running winesup.c's EXACT
/// synthesis math (verbatim copy, `-mlong-double-64` == Windows 64-bit
/// long double == f64) into `cnt_set_fout`'s captured pointer — the
/// genuine va_list* indirection is unportable to the host harness, and
/// the va store mechanism itself is identical-by-inspection to the
/// integer path (already diff-pinned). Single conversion (no %n on the
/// genuine side, so the overridden shim is the only va consumer).
/// Compares RAW BITS (exact incl. sign-zero / inf / NaN) + return
/// value. Pins the genuine d·10^exp synthesis, the mantissa overflow
/// guard, INT_MAX/MIN exponent saturation, ".8"→0.8, sign, the
/// l/L→double vs default→float selection.
fn diff_scanf_float() {
    let mut rng = Rng(0xf10a_5caf_u64);
    let cstr0 = |v: &[u8]| {
        let mut x = v.to_vec();
        x.push(0);
        x
    };
    // DOMAIN LIMIT (documented, tracked-owed — task #280): the
    // genuine d·10^exp synthesis + exponent combine are riddled with
    // signed-`int` overflow (C UB; gcc 2's-complement-wraps) once the
    // exponent saturates to INT_MAX / combines to INT_MIN
    // (`1e±999999999`, > ~25-digit mantissas). cnt-rust's `wrapping_*`
    // matches gcc in the common range but the corner cases of that UB
    // are not bit-portable and are NOT fuzzed here (same precedent as
    // the LP32 / strpbrk≤2 / iswctype-ASCII documented limits). The
    // real-world float domain (|exp| ≲ 308, sane mantissas) IS pinned
    // bit-exact. The UB-exponent edge remains owed (memory note).
    let frag: &[&[u8]] = &[
        b"0", b"1", b"9", b"12", b"345", b"+", b"-", b".", b"0.5",
        b"3.14159", b".8", b"8.", b"1e3", b"1E-3", b"2.5e+2", b"1e",
        b"1e+", b"0.000123", b"123456789012", b"  -42.0e1", b"+.0",
        b"6.022e23", b"1.6e-19", b"2.5e2", b"7e-7", b"00012.300",
        b"123.456", b"9e9", b"0.0009", b"42", b"xz9", b"1e30",
        b"1e-30", b"8.5", b".25",
    ];
    for _ in 0..8000 {
        // ONE fragment per iteration — concatenation could chain into
        // an INT_MAX-saturated exponent (the excluded UB domain).
        let s = frag[rng.upto(frag.len())].to_vec();
        let inp = cstr0(&s);
        let mut dummy = 0u8; // genuine side: never read (no va consumer)

        // float (no prefix): %e/%f/%g/%E/%G — 4-byte target.
        for fmt in [&b"%e\0"[..], b"%f\0", b"%g\0", b"%E\0", b"%G\0"] {
            let mut cv = 7.5f32;
            let rc = unsafe { sscanf(inp.as_ptr(), fmt.as_ptr(), &mut cv) };
            let mut rv = 7.5f32;
            unsafe {
                cnt_set_fout(&mut rv as *mut f32 as *mut core::ffi::c_void, 0)
            };
            let rr = unsafe { ref_sscanf(inp.as_ptr(), fmt.as_ptr(), &mut dummy) };
            assert_eq!(
                (rc, cv.to_bits()),
                (rr, rv.to_bits()),
                "{fmt:?} f32 inp={s:?}"
            );
        }
        // double (l / L prefix): %le/%lf/%lg/%Lf — 8-byte target.
        for fmt in [&b"%le\0"[..], b"%lf\0", b"%lg\0", b"%Lf\0"] {
            let mut cv = 7.5f64;
            let rc = unsafe { sscanf(inp.as_ptr(), fmt.as_ptr(), &mut cv) };
            let mut rv = 7.5f64;
            unsafe {
                cnt_set_fout(&mut rv as *mut f64 as *mut core::ffi::c_void, 1)
            };
            let rr = unsafe { ref_sscanf(inp.as_ptr(), fmt.as_ptr(), &mut dummy) };
            assert_eq!(
                (rc, cv.to_bits()),
                (rr, rv.to_bits()),
                "{fmt:?} f64 inp={s:?}"
            );
        }
        // width on a float
        {
            let fmt = b"%5f\0";
            let mut cv = 7.5f32;
            let rc = unsafe { sscanf(inp.as_ptr(), fmt.as_ptr(), &mut cv) };
            let mut rv = 7.5f32;
            unsafe {
                cnt_set_fout(&mut rv as *mut f32 as *mut core::ffi::c_void, 0)
            };
            let rr = unsafe { ref_sscanf(inp.as_ptr(), fmt.as_ptr(), &mut dummy) };
            assert_eq!((rc, cv.to_bits()), (rr, rv.to_bits()), "%5f inp={s:?}");
        }
        // wide twin: swscanf %lf
        {
            let win: Vec<u16> = inp.iter().map(|&c| c as u16).collect();
            let wf: Vec<u16> =
                b"%lf".iter().map(|&c| c as u16).chain([0]).collect();
            let mut cv = 7.5f64;
            let rc = unsafe { swscanf(win.as_ptr(), wf.as_ptr(), &mut cv) };
            let mut rv = 7.5f64;
            unsafe {
                cnt_set_fout(&mut rv as *mut f64 as *mut core::ffi::c_void, 1)
            };
            let rr = unsafe { ref_swscanf(win.as_ptr(), wf.as_ptr(), &mut dummy) };
            assert_eq!((rc, cv.to_bits()), (rr, rv.to_bits()), "w%lf inp={s:?}");
        }
    }
}

/// env — getenv/_wgetenv/_putenv/_wputenv + the 4 __p__* accessors,
/// stateful (process-global block; getenv returns an interior pointer)
/// vs the genuine compiled getenv.c/putenv.c/wputenv.c + environ.c
/// SetEnv. Pins the load-bearing contracts: getenv's exact-name-length
/// NT-ASCII-fold match; SetEnv's PREFIX-match asymmetry; `"NAME="`
/// (empty value) deletes; the duplicate-on-first-write
/// `_environ==__initenv` identity flip (incl. on a no-op delete, since
/// the genuine dup precedes the found-search); a no-`=` option → -1
/// (NO dup flip); CP1252 narrow/wide sync. Both sides seed from the
/// same wide block via the same CP1252 Win32 model — deterministic, no
/// global NLS state.
fn diff_env() {
    // double-NUL "GetEnvironmentStringsW"-format block from "K=V" list.
    fn wblock(entries: &[&str]) -> Vec<u16> {
        let mut v = Vec::new();
        for e in entries {
            v.extend(e.encode_utf16());
            v.push(0);
        }
        v.push(0); // terminating empty entry
        v
    }
    fn read_n(p: *const u8) -> Vec<u8> {
        if p.is_null() {
            return Vec::new();
        }
        let mut v = Vec::new();
        let mut i = 0;
        unsafe {
            while *p.add(i) != 0 {
                v.push(*p.add(i));
                i += 1;
            }
        }
        v
    }
    fn read_w(p: *const u16) -> Vec<u16> {
        if p.is_null() {
            return Vec::new();
        }
        let mut v = Vec::new();
        let mut i = 0;
        unsafe {
            while *p.add(i) != 0 {
                v.push(*p.add(i));
                i += 1;
            }
        }
        v
    }
    unsafe fn walk_n(t: *mut *mut *mut u8) -> Vec<Vec<u8>> {
        let arr = *t;
        if arr.is_null() {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut i = 0;
        while !(*arr.add(i)).is_null() {
            out.push(read_n(*arr.add(i)));
            i += 1;
        }
        out
    }
    unsafe fn walk_w(t: *mut *mut *mut u16) -> Vec<Vec<u16>> {
        let arr = *t;
        if arr.is_null() {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut i = 0;
        while !(*arr.add(i)).is_null() {
            out.push(read_w(*arr.add(i)));
            i += 1;
        }
        out
    }
    // identity bool: is `_environ` the SAME pointer array as
    // `__initenv` (genuine: true iff no successful SetEnv yet).
    unsafe fn id_n(e: *mut *mut *mut u8, i: *mut *mut *mut u8) -> bool {
        (*e as usize) == (*i as usize)
    }
    unsafe fn id_w(e: *mut *mut *mut u16, i: *mut *mut *mut u16) -> bool {
        (*e as usize) == (*i as usize)
    }

    // Full cross-impl state assertion (block content + frozen init +
    // the narrow==wide images + the identity flip).
    let chk = |msg: &str| unsafe {
        assert_eq!(
            walk_n(cabi::__p__environ()),
            walk_n(ref___p__environ()),
            "{msg}: _environ"
        );
        assert_eq!(
            walk_w(cabi::__p__wenviron()),
            walk_w(ref___p__wenviron()),
            "{msg}: _wenviron"
        );
        assert_eq!(
            walk_n(cabi::__p___initenv()),
            walk_n(ref___p___initenv()),
            "{msg}: __initenv (frozen)"
        );
        assert_eq!(
            walk_w(cabi::__p___winitenv()),
            walk_w(ref___p__winitenv()),
            "{msg}: __winitenv (frozen)"
        );
        assert_eq!(
            id_n(cabi::__p__environ(), cabi::__p___initenv()),
            id_n(ref___p__environ(), ref___p___initenv()),
            "{msg}: _environ==__initenv identity"
        );
        assert_eq!(
            id_w(cabi::__p__wenviron(), cabi::__p___winitenv()),
            id_w(ref___p__wenviron(), ref___p__winitenv()),
            "{msg}: _wenviron==__winitenv identity"
        );
    };

    let seed = |entries: &[&str]| {
        let b = wblock(entries);
        unsafe {
            cabi::cnt_env_seed_w(b.as_ptr());
            ref_env_set_block(b.as_ptr(), b.len() as i32);
        }
    };
    let getn = |name: &str| unsafe {
        let nm: Vec<u8> = name.bytes().chain([0]).collect();
        let c = cabi::getenv(nm.as_ptr());
        let r = ref_getenv(nm.as_ptr());
        assert_eq!(c.is_null(), r.is_null(), "getenv({name}) nullness");
        if !c.is_null() {
            assert_eq!(read_n(c), read_n(r), "getenv({name}) value");
        }
    };
    let wgetn = |name: &str| unsafe {
        let nm: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let c = cabi::_wgetenv(nm.as_ptr());
        let r = ref__wgetenv(nm.as_ptr());
        assert_eq!(c.is_null(), r.is_null(), "_wgetenv({name}) nullness");
        if !c.is_null() {
            assert_eq!(read_w(c), read_w(r), "_wgetenv({name}) value");
        }
    };
    let putn = |opt: &[u8], msg: &str| unsafe {
        let mut v = opt.to_vec();
        v.push(0);
        let c = cabi::_putenv(v.as_ptr());
        let r = ref__putenv(v.as_ptr());
        assert_eq!(c, r, "_putenv({msg}) ret");
        chk(msg);
    };
    let wputn = |opt: &str, msg: &str| unsafe {
        let v: Vec<u16> = opt.encode_utf16().chain([0]).collect();
        let c = cabi::_wputenv(v.as_ptr());
        let r = ref__wputenv(v.as_ptr());
        assert_eq!(c, r, "_wputenv({msg}) ret");
        chk(msg);
    };

    // ---- explicit pins ----
    // duplicate-on-first-write + the no-`=` exception (no dup flip).
    seed(&["PATH=/bin", "Home=/root", "=Z:=Z:\\drive", "Lang=C"]);
    chk("seed");
    unsafe {
        // pre-mutation: _environ IS __initenv (both impls).
        assert!(id_n(cabi::__p__environ(), cabi::__p___initenv()));
        assert!(id_n(ref___p__environ(), ref___p___initenv()));
    }
    getn("PATH");
    getn("path"); // NT ASCII fold (case-insensitive)
    getn("HOME");
    getn("Z:"); // the `=`-leading drive entry was skipped → NULL
    getn("missing");
    wgetn("Lang");
    putn(b"NOEQ", "no '=' → -1, NO dup flip");
    unsafe {
        // a failed (no-`=`) putenv must NOT have duplicated.
        assert!(id_n(cabi::__p__environ(), cabi::__p___initenv()));
        assert!(id_n(ref___p__environ(), ref___p___initenv()));
    }
    putn(b"NEW=1", "append (first real SetEnv → dup flip)");
    unsafe {
        assert!(!id_n(cabi::__p__environ(), cabi::__p___initenv()));
        assert!(!id_n(ref___p__environ(), ref___p___initenv()));
    }
    getn("NEW");
    putn(b"path=/usr/bin", "replace, fold-matched, full option stored");
    getn("PATH");
    putn(b"Home=", "empty value DELETES");
    getn("Home");
    putn(b"Home=", "delete-of-missing → 0, but still dup (already duped)");

    // SetEnv PREFIX-match asymmetry vs getenv's exact-length match.
    seed(&["ABC=one"]);
    putn(b"AB=two", "prefix-asymmetry: option 'AB' matches entry 'ABC'");
    getn("ABC"); // entry was replaced by the WHOLE option → "AB=two"
    getn("AB");
    chk("after prefix-asymmetry");

    // delete-of-missing still flips identity (genuine dup precedes the
    // found-search) — seed fresh, first op is a no-op delete.
    seed(&["A=1", "B=2"]);
    unsafe {
        assert!(id_n(cabi::__p__environ(), cabi::__p___initenv()));
    }
    putn(b"NOPE=", "no-op delete → 0, but dup flips");
    unsafe {
        assert!(!id_n(cabi::__p__environ(), cabi::__p___initenv()));
        assert!(!id_n(ref___p__environ(), ref___p___initenv()));
    }

    // CP1252 narrow/wide sync (high bytes round-trip enc∘dec).
    seed(&["X=hi"]);
    putn(b"CAF\xE9=r\xE9sum\xE9", "Latin-1 value, CP1252 narrow/wide sync");
    putn(b"EUR\x80=cost\xA9", "CP1252 0x80/0xA9 round-trip");
    wputn("Uni=\u{00e9}\u{00ff}", "wide _wputenv → CP1252 narrow image");
    chk("after CP1252");

    // ---- randomized op fuzz ----
    let names = ["A", "AB", "ABC", "B", "Bb", "XY", "ab", "aBc"];
    let vals = ["", "1", "xy", "long-ish", "a=b", "\u{00e9}z"];
    let mut rng = Rng(0x91e0_5a17_u64);
    for outer in 0..400 {
        // fresh random seed each round (exercises re-seed + repeated
        // dup-flip; keeps the arena well under MAXE).
        let ns = 1 + rng.upto(5);
        let entries: Vec<String> = (0..ns)
            .map(|_| {
                format!(
                    "{}={}",
                    names[rng.upto(names.len())],
                    vals[rng.upto(vals.len())]
                )
            })
            .collect();
        let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
        seed(&refs);
        chk(&format!("round {outer} seed"));
        for _ in 0..24 {
            match rng.upto(6) {
                0 => getn(names[rng.upto(names.len())]),
                1 => wgetn(names[rng.upto(names.len())]),
                2 => {
                    let o = format!(
                        "{}={}",
                        names[rng.upto(names.len())],
                        vals[rng.upto(vals.len())]
                    );
                    putn(o.as_bytes(), "rand set");
                }
                3 => {
                    let o = format!("{}=", names[rng.upto(names.len())]);
                    putn(o.as_bytes(), "rand delete");
                }
                4 => putn(names[rng.upto(names.len())].as_bytes(), "rand no-eq"),
                _ => {
                    let o = format!(
                        "{}={}",
                        names[rng.upto(names.len())],
                        vals[rng.upto(vals.len())]
                    );
                    wputn(&o, "rand wset");
                }
            }
        }
    }
}

/// float-cvt Phase 1 — _fcvt/fcvtbuf vs the genuine compiled
/// fcvtbuf.c `cvt()` (the sanos modf digit loop) + fcvt.c. The
/// genuine uses host `modf` (exact for binary64); cnt uses the
/// fdlibm bit-`modf` (also exact) → bit-for-bit on every finite
/// double, the whole magnitude range incl. subnormals. Pins the
/// `(fj+.03)*10` integer-part fudge, the `arg<1` exponent-down
/// loop, the `*p1+=5` ripple carry (incl. the new-leading-digit /
/// `(*decpt)++` case) and the negative-`ndigits+r2` empty-string
/// path. NaN/Inf (`snprintf(buf,ndigits,...)` size quirk) is the
/// documented UNFUZZED edge (scanf-extreme-exponent precedent).
fn diff_fcvt() {
    let mut rng = Rng(0xf0a7_c021_u64);
    let mut cd = 0i32;
    let mut cs = 0i32;
    let mut rd = 0i32;
    let mut rs = 0i32;
    // a spread of fixed magnitudes + fully-random finite bit patterns.
    let fixed: &[f64] = &[
        0.0, -0.0, 1.0, -1.0, 0.5, 2.5, 9.999_999, 0.1, 0.2, 0.3,
        123.456, -123.456, 1e-7, 9.999_999_999e9, 1.0 / 3.0,
        2.0 / 3.0, 1234567890.123_456, 4.940_656e-308, 1.797_693e308,
    ];
    for it in 0..40_000 {
        let v: f64 = if it < fixed.len() {
            fixed[it]
        } else if it % 3 == 0 {
            // structured: mantissa in [1,10) × 10^k, random sign
            let m = 1.0 + (rng.next() as f64 / u64::MAX as f64) * 9.0;
            let k = (rng.upto(620) as i32) - 310;
            let s = if rng.upto(2) == 0 { 1.0 } else { -1.0 };
            s * m * 10f64.powi(k)
        } else {
            // raw bit pattern (covers subnormals/huge); skip non-finite
            f64::from_bits(rng.next())
        };
        if !v.is_finite() {
            continue;
        }
        let nd = (rng.upto(46) as i32) - 4; // -4 ..= 41
        unsafe {
            let c = cabi::_fcvt(v, nd, &mut cd, &mut cs);
            let r = ref__fcvt(v, nd, &mut rd, &mut rs);
            let cb = {
                let mut x = Vec::new();
                let mut i = 0;
                while *c.add(i) != 0 {
                    x.push(*c.add(i));
                    i += 1;
                }
                x
            };
            let rb = {
                let mut x = Vec::new();
                let mut i = 0;
                while *r.add(i) != 0 {
                    x.push(*r.add(i));
                    i += 1;
                }
                x
            };
            assert_eq!(cb, rb, "_fcvt({v:e}, {nd}) digits");
            assert_eq!(cd, rd, "_fcvt({v:e}, {nd}) decpt");
            assert_eq!(cs, rs, "_fcvt({v:e}, {nd}) sign");
        }
    }
}

/// float-cvt Phase 2 — _ecvt vs the genuine compiled ecvt.c +
/// printf/streamout.c `sprintf("%-+.*E")` (one TU, -mlong-double-64
/// so the genuine `long double` == f64 == this core; the scanf-P2
/// precedent). `floor`/`log10`/`pow` are modeled BIT-IDENTICALLY on
/// both sides (the Wave E2 CP1252 shared-model discipline), so the
/// diff pins the genuine ecvt + format_float SURGERY deterministically
/// with libm-exactness factored out: the case_e
/// `round(sign*v*pow(10,p-exp))` + the `>= pow(10,p+1)` exponent
/// compensation + the `(u64)fpval2 % 10; fpval2/=10.` f64-divide peel
/// + ecvt's `cvtbuf[1]=='0' → decpt=0` and `ndigits<1` enhanced-
/// precision `tbuf[1]>='5'` nudge. NaN/Inf and extreme magnitudes
/// (|v| ≲ 1e-292: `pow(10, p-exp)` overflows f64 → inf → the
/// `(u64)inf` cast is C-UB / not bit-portable, exactly scanf-P2) are
/// the documented UNFUZZED owed domain limit (strpbrk≤2 precedent).
fn diff_ecvt() {
    let mut rng = Rng(0xec_47_a1_d0_u64);
    let mut cd = 0i32;
    let mut cs = 0i32;
    let mut rd = 0i32;
    let mut rs = 0i32;
    let read = |p: *const u8| -> Vec<u8> {
        let mut v = Vec::new();
        let mut i = 0;
        unsafe {
            while *p.add(i) != 0 {
                v.push(*p.add(i));
                i += 1;
            }
        }
        v
    };
    let fixed: &[f64] = &[
        0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 2.5, 9.999_999, 0.1, 0.2,
        0.3, 123.456, -123.456, 1e-7, 9.999_999_999e9, 1.0 / 3.0,
        2.0 / 3.0, 1234567890.123_456, 99999.5, 0.000_999_5, 1e20,
        -1e-20, 5.0, 0.000_05, 9.5, 0.95, 100.0, 0.001,
    ];
    let nds: &[i32] = &[0, 1, 2, 3, 4, 5, 6, 7, 10, 15, 17, 18, 20, 25];
    for it in 0..60_000 {
        let v: f64 = if it < fixed.len() {
            fixed[it]
        } else if it % 2 == 0 {
            // mantissa [1,10) × 10^k, |k| bounded so pow(10,p-exp)
            // stays finite (the extreme-magnitude inf-cast corner is
            // the documented owed edge — scanf-P2 precedent).
            let m = 1.0 + (rng.next() as f64 / u64::MAX as f64) * 9.0;
            let k = (rng.upto(560) as i32) - 280; // 10^-280 .. 10^279
            let s = if rng.upto(2) == 0 { 1.0 } else { -1.0 };
            s * m * 10f64.powi(k)
        } else {
            let b = f64::from_bits(rng.next());
            if !b.is_finite() || b == 0.0 {
                1.0
            } else if b.abs() < 1e-280 || b.abs() > 1e280 {
                // clamp into the fuzzed magnitude band
                b.signum() * 1e10
            } else {
                b
            }
        };
        if !v.is_finite() {
            continue;
        }
        let nd = nds[rng.upto(nds.len())];
        unsafe {
            let c = cabi::_ecvt(v, nd, &mut cd, &mut cs);
            let r = ref__ecvt(v, nd, &mut rd, &mut rs);
            assert_eq!(read(c), read(r), "_ecvt({v:e}, {nd}) digits");
            assert_eq!(cd, rd, "_ecvt({v:e}, {nd}) decpt");
            assert_eq!(cs, rs, "_ecvt({v:e}, {nd}) sign");
        }
    }
}

/// float-cvt Phase 3 — _gcvt/_gcvt_s vs the genuine gcvt.c + the same
/// streamout.c format_float (chr='g'), `-mlong-double-64`,
/// floor/log10/pow modeled BIT-IDENTICAL to core_rs::floatcvt::fl
/// (Wave E2 shared-model). Pins the `%g` algorithm: `precision--`,
/// the `exponent<-4 || exponent>=precision` e-form-vs-f-form decision,
/// the `while(precision && (u64)fpval2%10==0)` trailing-zero strip,
/// the generic int/frac emission (lowercase 'e' separator), the
/// `_gcvt` NULL/ndigit<0 → NULL guards and `_gcvt_s` EINVAL /
/// `digits>=size` & `len>size` ERANGE boundary (incl. the genuine
/// `len==size` off-by-one). NaN/Inf + extreme magnitude are the same
/// documented UNFUZZED owed edge as Phase 2 (scanf-P2 precedent).
fn diff_gcvt() {
    let mut rng = Rng(0x9c_07_5a_e2_u64);
    let read = |p: *const u8| -> Vec<u8> {
        if p.is_null() {
            return Vec::new();
        }
        let mut v = Vec::new();
        let mut i = 0;
        unsafe {
            while *p.add(i) != 0 {
                v.push(*p.add(i));
                i += 1;
            }
        }
        v
    };
    let fixed: &[f64] = &[
        0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 2.5, 9.999_999, 0.1, 0.2,
        0.3, 123.456, -123.456, 1e-7, 9.999_999_999e9, 1.0 / 3.0,
        2.0 / 3.0, 1234567890.123_456, 99999.5, 0.000_999_5, 1e20,
        -1e-20, 100.0, 0.001, 1e-5, 12345.0, 0.000_012_345, 1e6,
    ];
    let nds: &[i32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 15, 17, 18, 20, 25];
    // generous backing storage (the genuine _gcvt_s can write len+1
    // even when len==size — the preserved off-by-one — so the alloc
    // must have slack beyond the `size` we pass).
    let mut cbuf = [0u8; 512];
    let mut rbuf = [0u8; 512];
    for it in 0..60_000 {
        let v: f64 = if it < fixed.len() {
            fixed[it]
        } else if it % 2 == 0 {
            let m = 1.0 + (rng.next() as f64 / u64::MAX as f64) * 9.0;
            let k = (rng.upto(560) as i32) - 280;
            let s = if rng.upto(2) == 0 { 1.0 } else { -1.0 };
            s * m * 10f64.powi(k)
        } else {
            let b = f64::from_bits(rng.next());
            if !b.is_finite() || b == 0.0 {
                1.0
            } else if b.abs() < 1e-280 || b.abs() > 1e280 {
                b.signum() * 1e10
            } else {
                b
            }
        };
        if !v.is_finite() {
            continue;
        }
        let nd = nds[rng.upto(nds.len())];
        // %g f-form computes fpval2 = round(|v| * 10^precision); when
        // that exceeds 2^63 the genuine `(unsigned __int64)` cast is
        // C-UB while Rust `as u64` saturates — not bit-portable (the
        // scanf-P2 / Phase-2 owed edge). Bound the fuzz to the
        // bit-exact regime; the extreme region is the documented owed
        // domain limit (e-form's fpval2 ≈ 10^min(nd,17) is always
        // safe; this only gates the large-|v|·10^nd corner).
        let mag = if v == 0.0 { 0.0 } else { v.abs().log10() };
        if mag + (nd as f64) > 17.0 {
            continue;
        }
        unsafe {
            // ---- _gcvt ----
            cbuf.fill(0xAA);
            rbuf.fill(0xAA);
            let c = cabi::_gcvt(v, nd, cbuf.as_mut_ptr());
            let r = ref__gcvt(v, nd, rbuf.as_mut_ptr());
            assert_eq!(c.is_null(), r.is_null(), "_gcvt({v:e},{nd}) null");
            if !c.is_null() {
                assert_eq!(read(c), read(r), "_gcvt({v:e},{nd}) text");
            }
            // ndigit<0 → NULL (both)
            let cn = cabi::_gcvt(v, -1, cbuf.as_mut_ptr());
            let rn = ref__gcvt(v, -1, rbuf.as_mut_ptr());
            assert!(cn.is_null() && rn.is_null(), "_gcvt neg ndigit");

            // ---- _gcvt_s ---- vary size to hit ok / ERANGE / off-by-one
            let size = match rng.upto(4) {
                0 => nd as usize,            // digits>=size ⇒ ERANGE
                1 => (nd as usize) + 1,      // tight
                2 => (nd as usize) + 8,      // around the len==size edge
                _ => 64,                     // roomy
            };
            cbuf.fill(0xAA);
            rbuf.fill(0xAA);
            let cr = cabi::_gcvt_s(cbuf.as_mut_ptr(), size, v, nd);
            let rr = ref__gcvt_s(rbuf.as_mut_ptr(), size, v, nd);
            assert_eq!(cr, rr, "_gcvt_s({v:e},{nd},size={size}) ret");
            // Buffer is only DEFINED when something was written:
            // success (cr==0 ⇒ the %g string) or ERANGE with size>0
            // (genuine `buff[0]='\0'`). With size==0 & ERANGE the
            // genuine writes nothing — undefined, nothing to compare.
            if cr == 0 || size > 0 {
                let rd = |p: *const u8| -> Vec<u8> {
                    let mut v = Vec::new();
                    for i in 0..512usize {
                        let b = *p.add(i);
                        if b == 0 {
                            break;
                        }
                        v.push(b);
                    }
                    v
                };
                assert_eq!(rd(cbuf.as_ptr()), rd(rbuf.as_ptr()),
                    "_gcvt_s({v:e},{nd},size={size}) buf");
            }
        }
    }
    // NULL-buff guards.
    unsafe {
        assert!(cabi::_gcvt(1.0, 4, core::ptr::null_mut()).is_null());
        assert!(ref__gcvt(1.0, 4, core::ptr::null_mut()).is_null());
        assert_eq!(
            cabi::_gcvt_s(core::ptr::null_mut(), 16, 1.0, 4),
            ref__gcvt_s(core::ptr::null_mut(), 16, 1.0, 4)
        );
    }
}

// ── Wave MEM2 + RAND ────────────────────────────────────────────────

fn diff_memccpy() {
    let mut rng = Rng(0x5eed_0cc9);
    for _ in 0..10000 {
        let n = rng.upto(40);
        let cap = n.max(1);
        let src: Vec<u8> = (0..cap).map(|_| rng.byte()).collect();
        // c: signed-char match / NUL precedence / absent / >0xFF.
        let c: i32 = match rng.upto(6) {
            0 => 0,                                      // NUL break wins → NULL
            1 if n > 0 => src[rng.upto(n)] as i8 as i32, // signed form → can match
            2 if n > 0 => src[rng.upto(n)] as i32,       // unsigned form: matches iff <128
            3 => 0x90,                                   // high positive int: never matches a char
            4 => -112,                                   // 0x90 as signed char → matches a 0x90 byte
            _ => 0x100 | (rng.byte() as i32),            // >0xFF: never matches
        };
        let mut d1 = vec![0xAAu8; cap];
        let mut d2 = vec![0xAAu8; cap];
        let r1 = unsafe {
            cabi::_memccpy(d1.as_mut_ptr() as *mut c_void, src.as_ptr() as *const c_void, c, n)
        };
        let r2 = unsafe {
            ref_memccpy(d2.as_mut_ptr() as *mut c_void, src.as_ptr() as *const c_void, c, n)
        };
        assert_eq!(d1, d2, "memccpy buffers c={c} n={n} src={src:?}");
        let off = |r: *mut c_void, base: *const u8| {
            if r.is_null() { -1isize } else { (r as usize as isize) - (base as usize as isize) }
        };
        assert_eq!(
            off(r1, d1.as_ptr()),
            off(r2, d2.as_ptr()),
            "memccpy ret c={c} n={n} src={src:?}"
        );
    }
}

fn diff_memicmp() {
    let mut rng = Rng(0x1357_9bdf);
    for _ in 0..10000 {
        let n = rng.upto(40);
        let cap = n.max(1);
        let a: Vec<u8> = (0..cap)
            .map(|_| {
                if rng.next() & 1 == 0 {
                    b"AaBbZz09 \x80\xff"[rng.upto(11)]
                } else {
                    rng.byte()
                }
            })
            .collect();
        let b: Vec<u8> = match rng.upto(3) {
            // case-variant: fold-equal but differs pre-fold (→ 0)
            0 => a.iter().map(|&c| if c.is_ascii_alphabetic() { c ^ 0x20 } else { c }).collect(),
            1 => a.clone(),
            _ => (0..cap).map(|_| rng.byte()).collect(),
        };
        let c = unsafe { cabi::_memicmp(a.as_ptr() as *const c_void, b.as_ptr() as *const c_void, n) };
        let r = unsafe { ref_memicmp(a.as_ptr() as *const c_void, b.as_ptr() as *const c_void, n) };
        assert_eq!(c, r, "memicmp a={a:?} b={b:?} n={n}");
    }
}

fn diff_rand() {
    // fresh (state 0): both statics init to 0 and step identically.
    for _ in 0..50 {
        assert_eq!(unsafe { cabi::rand() }, unsafe { ref_rand() }, "rand fresh");
    }
    let mut rng = Rng(0x2468_ace0);
    for _ in 0..3000 {
        let seed = rng.next() as u32;
        unsafe {
            cabi::srand(seed);
            ref_srand(seed);
        }
        let len = 1 + rng.upto(64);
        for _ in 0..len {
            assert_eq!(
                unsafe { cabi::rand() },
                unsafe { ref_rand() },
                "rand after srand({seed})"
            );
        }
    }
}

// ── Wave SORT: qsort ────────────────────────────────────────────────

type QCmp = unsafe extern "C" fn(*const c_void, *const c_void) -> i32;

// Element width for cmp_memcmp_es (the C comparator gets only two
// pointers, so the width travels via this global — set before each call).
static QSORT_ES: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

unsafe extern "C" fn cmp_memcmp_es(a: *const c_void, b: *const c_void) -> i32 {
    let n = QSORT_ES.load(core::sync::atomic::Ordering::Relaxed);
    let pa = a as *const u8;
    let pb = b as *const u8;
    for k in 0..n {
        let x = *pa.add(k);
        let y = *pb.add(k);
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}

// First byte only → heavy ties → exercises equal-key final order.
unsafe extern "C" fn cmp_firstbyte(a: *const c_void, b: *const c_void) -> i32 {
    (*(a as *const u8) as i32) - (*(b as *const u8) as i32)
}

fn diff_qsort() {
    let mut rng = Rng(0x90a0_b0c0_d0e0_f001);
    let sizes = [1usize, 2, 3, 4, 8, 12];
    for _ in 0..4000 {
        let es = sizes[rng.upto(sizes.len())];
        let n = rng.upto(60);
        let total = (n * es).max(1);
        // small alphabet biases toward equal keys (the unstable-order
        // stress); full bytes for the distinct-key path.
        let small = rng.next() & 1 == 0;
        let buf: Vec<u8> = (0..total)
            .map(|_| if small { (rng.next() % 5) as u8 } else { rng.byte() })
            .collect();
        let mut b1 = buf.clone();
        let mut b2 = buf.clone();
        QSORT_ES.store(es, core::sync::atomic::Ordering::Relaxed);
        let cmp: QCmp = if rng.next() & 1 == 0 { cmp_memcmp_es } else { cmp_firstbyte };
        unsafe {
            cabi::qsort(b1.as_mut_ptr() as *mut c_void, n, es, cmp);
            ref_qsort(b2.as_mut_ptr() as *mut c_void, n, es, cmp);
        }
        assert_eq!(b1, b2, "qsort es={es} n={n} small={small}");
    }
}

// ── Wave SEARCH: bsearch / _lfind ──────────────────────────────────
// Compare the returned element pointer as an *index* into the buffer
// (raw pointers differ between the two builds; the offset must match).
fn ptr_index(ret: *mut c_void, base: *const u8, es: usize) -> isize {
    if ret.is_null() {
        -1
    } else {
        ((ret as usize - base as usize) / es) as isize
    }
}

fn diff_bsearch() {
    let mut rng = Rng(0x51a9_7c3e_0d62_b105);
    let sizes = [1usize, 2, 3, 4, 8];
    for _ in 0..6000 {
        let es = sizes[rng.upto(sizes.len())];
        let n = rng.upto(40);
        // Build a SORTED buffer (bsearch's precondition). Small alphabet
        // → duplicate keys exercise the tie path (the C returns whichever
        // probe lands first; the port replicates the exact probe order).
        let mut elems: Vec<Vec<u8>> = (0..n)
            .map(|_| (0..es).map(|_| (rng.next() % 6) as u8).collect())
            .collect();
        QSORT_ES.store(es, core::sync::atomic::Ordering::Relaxed);
        elems.sort();
        let buf: Vec<u8> = elems.into_iter().flatten().collect();
        // key: half the time an element drawn from the array, half random.
        let key: Vec<u8> = if n > 0 && rng.next() & 1 == 0 {
            let i = rng.upto(n);
            buf[i * es..i * es + es].to_vec()
        } else {
            (0..es).map(|_| (rng.next() % 6) as u8).collect()
        };
        let cmp: QCmp = if rng.next() & 1 == 0 { cmp_memcmp_es } else { cmp_firstbyte };
        unsafe {
            let r1 = cabi::bsearch(key.as_ptr() as *const c_void, buf.as_ptr() as *const c_void, n, es, cmp);
            let r2 = ref_bsearch(key.as_ptr() as *const c_void, buf.as_ptr() as *const c_void, n, es, cmp);
            assert_eq!(
                ptr_index(r1, buf.as_ptr(), es),
                ptr_index(r2, buf.as_ptr(), es),
                "bsearch es={es} n={n}"
            );
        }
    }
}

fn diff_lfind() {
    let mut rng = Rng(0x7b32_19fe_c4a0_d886);
    let sizes = [1u32, 2, 3, 4, 8];
    for _ in 0..6000 {
        let es = sizes[rng.upto(sizes.len())];
        let n = rng.upto(40) as u32;
        let total = (n as usize * es as usize).max(1);
        let buf: Vec<u8> = (0..total).map(|_| (rng.next() % 6) as u8).collect();
        let key: Vec<u8> = if n > 0 && rng.next() & 1 == 0 {
            let i = rng.upto(n as usize);
            buf[i * es as usize..(i + 1) * es as usize].to_vec()
        } else {
            (0..es).map(|_| (rng.next() % 6) as u8).collect()
        };
        QSORT_ES.store(es as usize, core::sync::atomic::Ordering::Relaxed);
        let cmp: QCmp = if rng.next() & 1 == 0 { cmp_memcmp_es } else { cmp_firstbyte };
        unsafe {
            let r1 = cabi::_lfind(key.as_ptr() as *const c_void, buf.as_ptr() as *const c_void, &n, es, cmp);
            let r2 = ref_lfind(key.as_ptr() as *const c_void, buf.as_ptr() as *const c_void, &n, es, cmp);
            assert_eq!(
                ptr_index(r1, buf.as_ptr(), es as usize),
                ptr_index(r2, buf.as_ptr(), es as usize),
                "lfind es={es} n={n}"
            );
        }
    }
}


fn diff_ctype() {
    // ctype lives in its own crate (cnt-ctype); shadow the module-level
    // `cabi` alias so the assertions below read uniformly.
    use cnt_ctype as cabi;
    // Tables byte-for-byte: the highest-risk part (514 entries). cnt's
    // statics are machine-generated from ctype.c; pin vs the genuine
    // compiled C arrays.
    for i in 0..257 {
        assert_eq!(cabi::_ctype[i], unsafe { ref_ctype[i] }, "_ctype[{i}]");
        assert_eq!(cabi::_wctype[i], unsafe { ref_wctype[i] }, "_wctype[{i}]");
    }
    // _pctype / _pwctype point one past the table start (the EOF slot).
    assert_eq!(unsafe { *cabi::_pctype.0 }, cabi::_ctype[1], "_pctype[0]");
    assert_eq!(unsafe { *cabi::_pwctype.0 }, cabi::_wctype[1], "_pwctype[0]");

    // Narrow classification + case + ascii/csym over the full int byte
    // domain plus negatives and >0xFF (exercises the `(unsigned char)`
    // truncation the C `_isctype` does).
    for c in -260i32..=520 {
        unsafe {
            assert_eq!(cabi::_isctype(c, 0xFFFF), ref_isctype(c, 0xFFFF), "_isctype {c}");
            assert_eq!(cabi::isalpha(c), ref_isalpha(c), "isalpha {c}");
            assert_eq!(cabi::isalnum(c), ref_isalnum(c), "isalnum {c}");
            assert_eq!(cabi::iscntrl(c), ref_iscntrl(c), "iscntrl {c}");
            assert_eq!(cabi::isdigit(c), ref_isdigit(c), "isdigit {c}");
            assert_eq!(cabi::isgraph(c), ref_isgraph(c), "isgraph {c}");
            assert_eq!(cabi::islower(c), ref_islower(c), "islower {c}");
            assert_eq!(cabi::isprint(c), ref_isprint(c), "isprint {c}");
            assert_eq!(cabi::ispunct(c), ref_ispunct(c), "ispunct {c}");
            assert_eq!(cabi::isspace(c), ref_isspace(c), "isspace {c}");
            assert_eq!(cabi::isupper(c), ref_isupper(c), "isupper {c}");
            assert_eq!(cabi::isxdigit(c), ref_isxdigit(c), "isxdigit {c}");
            assert_eq!(cabi::__isascii(c), ref_isascii_n(c), "__isascii {c}");
            assert_eq!(cabi::__toascii(c), ref_toascii_n(c), "__toascii {c}");
            assert_eq!(cabi::__iscsym(c), ref_iscsym_n(c), "__iscsym {c}");
            assert_eq!(cabi::__iscsymf(c), ref_iscsymf_n(c), "__iscsymf {c}");
            assert_eq!(cabi::_tolower(c), ref_tolower_u(c), "_tolower {c}");
            assert_eq!(cabi::_toupper(c), ref_toupper_u(c), "_toupper {c}");
            assert_eq!(cabi::toupper(c), ref_toupper(c), "toupper {c}");
        }
    }

    // Wide forwarders (→ iswctype) over the full wint_t domain.
    for c in 0u32..=0xFFFF {
        let c = c as u16;
        unsafe {
            assert_eq!(cabi::iswascii(c), ref_iswascii(c), "iswascii {c}");
            assert_eq!(cabi::iswalnum(c), ref_iswalnum(c), "iswalnum {c}");
            assert_eq!(cabi::iswalpha(c), ref_iswalpha(c), "iswalpha {c}");
            assert_eq!(cabi::iswcntrl(c), ref_iswcntrl(c), "iswcntrl {c}");
            assert_eq!(cabi::iswdigit(c), ref_iswdigit(c), "iswdigit {c}");
            assert_eq!(cabi::iswgraph(c), ref_iswgraph(c), "iswgraph {c}");
            assert_eq!(cabi::iswprint(c), ref_iswprint(c), "iswprint {c}");
            assert_eq!(cabi::iswpunct(c), ref_iswpunct(c), "iswpunct {c}");
            assert_eq!(cabi::iswlower(c), ref_iswlower(c), "iswlower {c}");
            assert_eq!(cabi::iswupper(c), ref_iswupper(c), "iswupper {c}");
            assert_eq!(cabi::iswspace(c), ref_iswspace(c), "iswspace {c}");
            assert_eq!(cabi::iswxdigit(c), ref_iswxdigit(c), "iswxdigit {c}");
            assert_eq!(cabi::towlower(c), ref_towlower(c), "towlower {c}");
        }
    }

    // iswctype / is_wctype (iswctype_nt.c + is_wctype.c flipped) over
    // the full wint_t domain × every mask combo the wide funcs use.
    for &t in &[
        1u16, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0x103, 0x107, 0xFFFF, 0,
    ] {
        for c in 0u32..=0xFFFF {
            let c = c as u16;
            unsafe {
                assert_eq!(cabi::iswctype(c, t), ref_iswctype(c, t), "iswctype c={c} t={t}");
                assert_eq!(cabi::is_wctype(c, t), ref_iswctype(c, t), "is_wctype c={c} t={t}");
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// printf engine (streamout) — narrow differential
//
// cnt-rust's Rust `streamout` (cabi::streamout) vs the genuine compiled
// `printf/streamout.c` (`ref_streamout`, build.rs). Both are driven
// with the SAME hand-built flat va_list (one 8-byte slot per arg — the
// 64-bit host slot size cnt-rust's `va_next` uses) over the SAME `FILE`
// setup, so this pins the parser + va_arg layout + the `_cnt` overflow
// guard + the format_float path against ReactOS's own code (no
// glibc-vs-ReactOS quirk divergence). Narrow only (the wide twin needs
// 16-bit L"" literals the host can't express; wide is covered by the
// ACPI swprintf boot path). See build.rs for the oracle setup.
// ═══════════════════════════════════════════════════════════════════
extern "C" {
    fn ref_run(base: *mut u8, cnt: i32, fmt: *const u8, va: *mut u8) -> i32;
}

/// Lay a printf arg into one flat va slot (8 bytes, little-endian) the
/// way cnt-rust's `va_next::<T>` reads it.
trait ToSlot {
    fn slot(self) -> u64;
}
impl ToSlot for i32 {
    fn slot(self) -> u64 {
        self as u32 as u64
    }
}
impl ToSlot for u32 {
    fn slot(self) -> u64 {
        self as u64
    }
}
impl ToSlot for i64 {
    fn slot(self) -> u64 {
        self as u64
    }
}
impl ToSlot for u64 {
    fn slot(self) -> u64 {
        self
    }
}
impl ToSlot for f64 {
    fn slot(self) -> u64 {
        self.to_bits()
    }
}
impl ToSlot for *const u8 {
    fn slot(self) -> u64 {
        self as u64
    }
}

/// Run a format through both engines with the given initial `_cnt`
/// (pass a negative value to exercise the unbounded swprintf overflow
/// path) and assert identical return value AND identical 512-byte
/// output buffer.
fn so_both(fmt: &str, slots: &[u64], cnt_init: i32) {
    let mut fmtz: Vec<u8> = fmt.as_bytes().to_vec();
    fmtz.push(0);
    let va = if slots.is_empty() {
        core::ptr::null_mut()
    } else {
        slots.as_ptr() as *mut u8
    };

    let mut rbuf = [0u8; 512];
    let rn = unsafe { ref_run(rbuf.as_mut_ptr(), cnt_init, fmtz.as_ptr(), va) };

    let mut cbuf = [0u8; 512];
    let mut f = cabi::File {
        _ptr: cbuf.as_mut_ptr(),
        _cnt: cnt_init,
        _base: cbuf.as_mut_ptr(),
        _flag: 0x40 | 0x2, // _IOSTRG | _IOWRT
        _file: 0,
        _charbuf: 0,
        _bufsiz: 0,
        _tmpfname: core::ptr::null_mut(),
    };
    let cn = unsafe { cabi::streamout(&mut f, fmtz.as_ptr(), va) };

    assert_eq!(
        cn, rn,
        "return mismatch: fmt={fmt:?} cnt_init={cnt_init} cnt={cn} ref={rn}"
    );
    assert_eq!(
        &cbuf[..], &rbuf[..],
        "output mismatch: fmt={:?} cnt_init={} \n cnt={:?}\n ref={:?}",
        fmt,
        cnt_init,
        String::from_utf8_lossy(&cbuf[..rn.max(0) as usize]),
        String::from_utf8_lossy(&rbuf[..rn.max(0) as usize]),
    );
}

macro_rules! so {
    ($cnt:expr, $fmt:literal $(, $a:expr)* $(,)?) => {
        so_both($fmt, &[$( ToSlot::slot($a) ),*], $cnt)
    };
}

fn diff_streamout_integers() {
    // d/i/u/x/X/o across signs, zero, extremes; flags; width; precision.
    for &cnt in &[512i32, -2, 8, 1, 0] {
        so!(cnt, "plain");
        so!(cnt, "%d", 0i32);
        so!(cnt, "%d", 42i32);
        so!(cnt, "%d", -42i32);
        so!(cnt, "%i", -1i32);
        so!(cnt, "%d", i32::MIN);
        so!(cnt, "%d", i32::MAX);
        so!(cnt, "%u", 0u32);
        so!(cnt, "%u", 4000000000u32);
        so!(cnt, "%x", 0xdeadbeefu32);
        so!(cnt, "%X", 0xabcdu32);
        so!(cnt, "%o", 0o777u32);
        so!(cnt, "%5d", 42i32);
        so!(cnt, "%-5d", 42i32);
        so!(cnt, "%05d", 42i32);
        so!(cnt, "%+d", 42i32);
        so!(cnt, "% d", 42i32);
        so!(cnt, "%8.5d", 42i32);
        so!(cnt, "%.0d", 0i32);
        so!(cnt, "%#x", 0x2au32);
        so!(cnt, "%#o", 0o17u32);
        so!(cnt, "%10.4x", 0xabu32);
        so!(cnt, "%-#10x", 0x2au32);
        so!(cnt, "a=%d b=%x c=%u", 1i32, 0x10u32, 99u32);
        so!(cnt, "%lld", 0x1_0000_0000i64);
        so!(cnt, "%llx", 0xdead_beef_cafeu64);
        so!(cnt, "%llu", u64::MAX);
        so!(cnt, "%I64d", -5i64);
        // '*' width / precision
        so!(cnt, "%*d", 6i32, 42i32);
        so!(cnt, "%.*d", 4i32, 42i32);
        so!(cnt, "%-*d", 6i32, 42i32);
    }
}

fn diff_streamout_chars_strings_percent() {
    let hello = b"Hello, world\0";
    let empty = b"\0";
    for &cnt in &[512i32, -2, 6, 1, 0] {
        so!(cnt, "100%%");
        so!(cnt, "%c", b'A' as i32);
        so!(cnt, "%5c", b'Z' as i32);
        so!(cnt, "%-5c", b'Z' as i32);
        so!(cnt, "%s", hello.as_ptr() as *const u8);
        so!(cnt, "%s", empty.as_ptr() as *const u8);
        so!(cnt, "%s", core::ptr::null::<u8>()); // (null)
        so!(cnt, "%10s", hello.as_ptr() as *const u8);
        so!(cnt, "%-15s", hello.as_ptr() as *const u8);
        so!(cnt, "%.5s", hello.as_ptr() as *const u8);
        so!(cnt, "[%8.3s]", hello.as_ptr() as *const u8);
    }
}

fn diff_streamout_float() {
    // %f/%e/%g/%E/%G — ref uses the SAME fl libm model as cnt's
    // format_float (build.rs #defines floor/log10/pow bit-identically),
    // so finite values pin format_float exactly.
    let vals = [
        0.0f64, 1.0, -1.0, 3.14159265, -2.5, 0.5, 1234.5678, 0.000123, 1e6, 1e-6, 42.0,
    ];
    for &cnt in &[512i32, -2] {
        for &v in &vals {
            so!(cnt, "%f", v);
            so!(cnt, "%.2f", v);
            so!(cnt, "%.0f", v);
            so!(cnt, "%10.3f", v);
            so!(cnt, "%-12.4f", v);
            so!(cnt, "%+.3f", v);
            so!(cnt, "%e", v);
            so!(cnt, "%.3e", v);
            so!(cnt, "%E", v);
            so!(cnt, "%g", v);
            so!(cnt, "%.8g", v);
            so!(cnt, "%G", v);
        }
    }

    // Specials — ±Inf and NaN. These rows are the tripwire for the
    // exp10_floor non-finite guard (fl::exp10_floor returns 308 for
    // biased==0x7ff; the recipe's cnt_log10 carries the identical
    // `!isfinite → 308.` guard): before the guard BOTH sides spun
    // forever in the log10 correction loop on ±Inf (`pow10(309)`
    // saturates to +Inf and `Inf <= Inf` holds) — a silent kernel hang
    // on any `%f` of an infinity, found by the msvcrt printf probe
    // (2026-07-14). The effective output is the genuine `#INF`/`#QNAN`
    // prepend-and-truncate path in format_float.
    for &v in &[f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        for &cnt in &[512i32, -2] {
            so!(cnt, "%f", v);
            so!(cnt, "%e", v);
            so!(cnt, "%g", v);
            so!(cnt, "%.3E", v);
            so!(cnt, "%12.2f", v);
        }
    }
}

// ── Wave MATH-CLASS — IEEE-754 predicates + integer abs ─────────────────
extern "C" {
    fn ref_abs(j: i32) -> i32;
    fn ref_labs(j: i64) -> i64; // host C `long` is 64-bit; compare truncated to i32
    fn ref_isnan(x: f64) -> i32;
    fn ref_isnanf(x: f32) -> i32;
    fn ref_finite(x: f64) -> i32;
    fn ref_finitef(x: f32) -> i32;
    fn ref_dsign(x: f64) -> i32;
    fn ref_fdsign(x: f32) -> i32;
    fn ref_dclass(x: f64) -> i16;
    fn ref_fdclass(x: f32) -> i16;
    fn ref_dtest(px: *const f64) -> i16;
    fn ref_fdtest(px: *const f32) -> i16;
    // Wave MATH-TIERB
    fn ref_div(num: i32, denom: i32) -> RDivT;
    fn ref_chgsignf(x: f32) -> f32;
    fn ref_scalbn(x: f64, n: i32) -> f64;
    fn ref_scalbnf(x: f32, n: i32) -> f32;
    fn ref_round(x: f64) -> f64;
    fn ref_roundf(x: f32) -> f32;
    fn ref_nextafter(x: f64, y: f64) -> f64;
    fn ref_nextafterf(x: f32, y: f32) -> f32;
    fn ref_nexttowardf(x: f32, y: f64) -> f32;
    // Wave MATH-TIERC sub-wave 1
    fn ref_sin(x: f64) -> f64;
    fn ref_cos(x: f64) -> f64;
    fn ref_sqrt(x: f64) -> f64;
    fn ref_sqrtf(x: f32) -> f32;
    // Wave MATH-TIERC sub-wave 2
    fn ref_exp(x: f64) -> f64;
    fn ref_log(x: f64) -> f64;
    fn ref_fmod(x: f64, y: f64) -> f64;
    // Wave MATH-TIERC sub-wave 3
    fn ref_tan(x: f64) -> f64;
    // Wave MATH-TIERC sub-wave 4
    fn ref_ceil(x: f64) -> f64;
    fn ref_floor(x: f64) -> f64;
    fn ref_ceilf(x: f32) -> f32;
    fn ref_floorf(x: f32) -> f32;
    // Wave MATH-TIERC sub-wave 5
    fn ref_ldexp(x: f64, n: i32) -> f64;
    fn ref__logb(x: f64) -> f64;
    fn ref__logbf(x: f32) -> f32;
    // Wave MATH-TIERC sub-wave 6
    fn ref_fabs(x: f64) -> f64;
    fn ref_fabsf(x: f32) -> f32;
    // Wave MATH-TIERC sub-wave 7
    fn ref_modf(x: f64, iptr: *mut f64) -> f64;
    fn ref_atan(x: f64) -> f64;
    fn ref_atan2(y: f64, x: f64) -> f64;
    fn ref_atan_p(x: f64) -> f64;
    fn ref_atan2_p(y: f64, x: f64) -> f64;
    // Wave MATH-TIERC sub-wave 8
    fn ref_exp2(x: f64) -> f64;
    fn ref_tanh(x: f64) -> f64;
    // Wave MATH-TIERC sub-wave 9
    fn ref_sinh(x: f64) -> f64;
    fn ref_cosh(x: f64) -> f64;
    // Wave MATH-TIERC sub-wave 10
    fn ref_asin(x: f64) -> f64;
    fn ref_acos(x: f64) -> f64;
    // Wave MATH-TIERC sub-wave 11
    fn ref__hypot(x: f64, y: f64) -> f64;
    // Wave MATH-TIERC sub-wave 12
    fn ref_modff(x: f32, iptr: *mut f32) -> f32;
    fn ref__hypotf(x: f32, y: f32) -> f32;
    // Wave MATH-TIERC sub-wave 13 (f32 transcendental twins)
    fn ref_acosf(x: f32) -> f32;
    fn ref_asinf(x: f32) -> f32;
    fn ref_atanf(x: f32) -> f32;
    fn ref_sinhf(x: f32) -> f32;
    fn ref_coshf(x: f32) -> f32;
    fn ref_tanhf(x: f32) -> f32;
    fn ref_tanf(x: f32) -> f32;
    fn ref_atan2f(y: f32, x: f32) -> f32;
    // The last of crtmath's plain-C float/misc surface. (The seven f32
    // transcendental wrappers have no independent oracle — see build.rs.)
    fn ref_copysign(d: f64, s: f64) -> f64;
    fn ref_copysignf(x: f32, y: f32) -> f32;
    fn ref_frexp(x: f64, e: *mut i32) -> f64;
    fn ref_ldiv(num: i32, denom: i32) -> RDivT;
    fn ref_fpclass(d: f64) -> i32;
    // The Bessel wrappers. Both sides call the SAME `libm` numerics
    // underneath (build.rs), so a divergence here is in the
    // 102 lines of errno/_fpclass shell that are ours.
    fn ref__j0(num: f64) -> f64;
    fn ref__j1(num: f64) -> f64;
    fn ref__jn(n: i32, num: f64) -> f64;
    fn ref__y0(num: f64) -> f64;
    fn ref__y1(num: f64) -> f64;
    fn ref__yn(order: i32, num: f64) -> f64;
    /// The one errno slot, shared by both sides (build.rs's `cnt_errno_v`).
    /// Rows therefore clear it before each call and read it after.
    fn _errno() -> *mut i32;
}

/// Mirror of the C `div_t` for the struct-return ABI (SysV: {i32,i32}
/// in a single register — same on the Rust side via `#[repr(C)]`).
#[repr(C)]
struct RDivT {
    quot: i32,
    rem: i32,
}

/// NaN-tolerant bit equality: two results match if their bit images are
/// identical, or both are NaN (any NaN payload). Non-NaN results from
/// identical algorithms on the same host FPU are bit-for-bit equal.
fn same_f64(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}
fn same_f32(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// A spread of interesting f64 bit images: zeros, inf, the two NaN
/// kinds, subnormals (incl. smallest), normals, and DBL_MAX.
fn f64_edge_bits() -> Vec<u64> {
    vec![
        0x0000_0000_0000_0000, // +0
        0x8000_0000_0000_0000, // -0
        0x7FF0_0000_0000_0000, // +inf
        0xFFF0_0000_0000_0000, // -inf
        0x7FF8_0000_0000_0000, // qNaN
        0xFFF8_0000_0000_0000, // -qNaN
        0x7FF0_0000_0000_0001, // sNaN
        0xFFF0_0000_0000_0001, // -sNaN
        0x0000_0000_0000_0001, // smallest +subnormal
        0x8000_0000_0000_0001, // smallest -subnormal
        0x000F_FFFF_FFFF_FFFF, // largest +subnormal
        0x0010_0000_0000_0000, // smallest +normal
        0x3FF0_0000_0000_0000, // 1.0
        0xBFF0_0000_0000_0000, // -1.0
        0x7FEF_FFFF_FFFF_FFFF, // DBL_MAX
        0xFFEF_FFFF_FFFF_FFFF, // -DBL_MAX
        0x4093_4A45_6D5C_FAAD, // an arbitrary normal
    ]
}

fn f32_edge_bits() -> Vec<u32> {
    vec![
        0x0000_0000, // +0
        0x8000_0000, // -0
        0x7F80_0000, // +inf
        0xFF80_0000, // -inf
        0x7FC0_0000, // qNaN
        0xFFC0_0000, // -qNaN
        0x7F80_0001, // sNaN
        0xFF80_0001, // -sNaN
        0x0000_0001, // smallest +subnormal
        0x8000_0001, // smallest -subnormal
        0x007F_FFFF, // largest +subnormal
        0x0080_0000, // smallest +normal
        0x3F80_0000, // 1.0
        0xBF80_0000, // -1.0
        0x7F7F_FFFF, // FLT_MAX
        0xFF7F_FFFF, // -FLT_MAX
        0x4048_F5C3, // arbitrary normal (~3.14)
    ]
}

fn diff_math_abs() {
    let mut rng = Rng(0x00ab_5123);
    // INT_MIN is UB for the C `int abs` (excluded for abs); for labs we
    // compare the host `long` result truncated to i32 — at i32::MIN both
    // sides land on i32::MIN, so it's covered.
    let mut cases: Vec<i32> = vec![0, 1, -1, 7, -7, i32::MAX, i32::MIN, i32::MIN + 1];
    for _ in 0..5000 {
        cases.push(rng.next() as i32);
    }
    for &v in &cases {
        unsafe {
            assert_eq!(cabi::labs(v), ref_labs(v as i64) as i32, "labs {v}");
            if v != i32::MIN {
                assert_eq!(cabi::abs(v), ref_abs(v), "abs {v}");
            }
        }
    }
}

fn diff_math_class_f64() {
    let mut rng = Rng(0xd00d_f00d);
    let mut bits = f64_edge_bits();
    for _ in 0..20000 {
        bits.push(rng.next());
    }
    for &b in &bits {
        let x = f64::from_bits(b);
        unsafe {
            assert_eq!(cabi::_isnan(x), ref_isnan(x), "_isnan {b:#018x}");
            assert_eq!(cabi::_finite(x), ref_finite(x), "_finite {b:#018x}");
            assert_eq!(cabi::_dsign(x), ref_dsign(x), "_dsign {b:#018x}");
            assert_eq!(cabi::_dclass(x), ref_dclass(x), "_dclass {b:#018x}");
            assert_eq!(cabi::_dtest(&x), ref_dtest(&x), "_dtest {b:#018x}");
            // float/fpclass.c. Note this sweep drives SIGNALLING NaNs
            // through it too (f64_edge_bits carries both quiet-bit
            // states): Windows reports every NaN as _FPCLASS_QNAN and
            // never _FPCLASS_SNAN, so an "improved" port that
            // distinguished them would fail right here.
            assert_eq!(cabi::_fpclass(x), ref_fpclass(x), "_fpclass {b:#018x}");
        }
    }
}

/// `_copysign` / `_copysignf` / `frexp` / `ldiv` — the self-contained tail
/// of crtmath's plain-C surface. Every one is bit or integer logic with a
/// standalone C body, which is exactly what a difftest can settle.
fn diff_math_lastmile() {
    let mut rng = Rng(0x1ea5_7a11_d00d_f00d);
    let mut bits = f64_edge_bits();
    for _ in 0..20000 {
        bits.push(rng.next());
    }

    for (i, &b) in bits.iter().enumerate() {
        let x = f64::from_bits(b);
        // Pair each value with another from the corpus so the sign SOURCE
        // sweeps independently of the sign DESTINATION — a body that
        // returned |d| or copied the wrong operand's bit passes a
        // same-value sweep.
        let y = f64::from_bits(bits[(i * 7 + 1) % bits.len()]);
        unsafe {
            assert!(
                same_f64(cabi::_copysign(x, y), ref_copysign(x, y)),
                "_copysign {b:#018x} {:#018x}",
                y.to_bits()
            );

            // frexp writes through an out-param, and for +-inf/NaN the C
            // RETURNS BEFORE ASSIGNING IT (math/frexp.c:24-26). Both sides
            // start from the same poison so that "left untouched" is part
            // of what is compared, not a hole in it.
            let (mut es, mut er) = (0x5AFE_5AFEi32, 0x5AFE_5AFEi32);
            let rs = cabi::frexp(x, &mut es);
            let rr = ref_frexp(x, &mut er);
            assert!(same_f64(rs, rr), "frexp mantissa {b:#018x}");
            assert_eq!(es, er, "frexp exponent {b:#018x}");
        }
    }

    let mut fbits = f32_edge_bits();
    for _ in 0..20000 {
        fbits.push(rng.next() as u32);
    }
    for (i, &b) in fbits.iter().enumerate() {
        let x = f32::from_bits(b);
        let y = f32::from_bits(fbits[(i * 7 + 1) % fbits.len()]);
        unsafe {
            assert!(
                same_f32(cabi::_copysignf(x, y), ref_copysignf(x, y)),
                "_copysignf {b:#010x} {:#010x}",
                y.to_bits()
            );
        }
    }

    // ldiv: the truncation fixup only fires for num<0 && denom>0 && rem>0,
    // so the corpus has to reach negative numerators with positive
    // denominators on purpose. i32::MIN is in there for the negation
    // overflow the C leaves undefined.
    let edges: [i32; 9] = [i32::MIN, i32::MIN + 1, -7, -1, 0, 1, 7, i32::MAX - 1, i32::MAX];
    let mut pairs: Vec<(i32, i32)> = Vec::new();
    // Every edge against every edge — all four sign quadrants, which is
    // what decides whether the fixup fires.
    for &n in &edges {
        for &d in &edges {
            pairs.push((n, d));
        }
    }
    // Then random pairs, plus each random numerator against a small
    // positive denominator: num<0 && denom>0 is the ONLY case the fixup
    // touches, and uniformly random pairs hit it only a quarter of the time.
    for _ in 0..20000 {
        let n = rng.next() as i32;
        pairs.push((n, rng.next() as i32));
        pairs.push((n, (rng.next() as i32).abs().max(1)));
    }
    for (n, d) in pairs {
        // Division by zero faults identically on both sides, and MIN/-1
        // traps in hardware rather than wrapping. Neither is a divergence
        // this harness can observe, so neither is graded.
        if d == 0 || (n == i32::MIN && d == -1) {
            continue;
        }
        unsafe {
            let a = cabi::ldiv(n, d);
            let e = ref_ldiv(n, d);
            assert_eq!((a.quot, a.rem), (e.quot, e.rem), "ldiv {n} {d}");
        }
    }
}

/// `_j0` `_j1` `_jn` `_y0` `_y1` `_yn` — `math/{j0_y0,j1_y1,jn_yn}.c`.
///
/// The numerics are `libm`'s and identical on both sides by
/// construction, so what these rows actually grade is the shell: WHICH
/// arguments set `EDOM`, and the `-inf -> sqrt(-1)` epilogue. Both are easy
/// to get subtly wrong and invisible to a value-only comparison, so errno is
/// part of the compared image, not an afterthought.
///
/// NEGATIVE-TESTED, and the result is worth writing down. Deleting the
/// `_FPCLASS_NN` arm from `_y0` fails here at `-1.0`. Deleting the
/// `_FPCLASS_NZ` arm does NOT fail, and that is not a gap in the corpus: for
/// `-0.0` the FDLIBM body returns -inf, so the `-inf -> sqrt(-1)` epilogue
/// sets EDOM anyway and the prologue's negative-zero test cannot be observed
/// through errno at all. So these rows pin `_y0`/`_y1`'s NN and ND arms but
/// are structurally unable to pin NZ. Don't read a green run as covering it.
// The numerics under the C wrappers: the `libm` functions cnt-rust's wrappers call (see
// build.rs), so `diff_math_bessel` grades only the shell.
#[no_mangle]
extern "C" fn __ieee754_j0(x: f64) -> f64 { libm::j0(x) }
#[no_mangle]
extern "C" fn __ieee754_j1(x: f64) -> f64 { libm::j1(x) }
#[no_mangle]
extern "C" fn __ieee754_jn(n: i32, x: f64) -> f64 { libm::jn(n, x) }
#[no_mangle]
extern "C" fn __ieee754_y0(x: f64) -> f64 { libm::y0(x) }
#[no_mangle]
extern "C" fn __ieee754_y1(x: f64) -> f64 { libm::y1(x) }
#[no_mangle]
extern "C" fn __ieee754_yn(n: i32, x: f64) -> f64 { libm::yn(n, x) }

fn diff_math_bessel() {
    let mut rng = Rng(0xbe55_e100_d00d_f00d);

    // The corpus has to reach each errno arm on purpose:
    //   * non-finite (both infinities and a NaN) -- every one of the six,
    //   * NEGATIVE normal / denormal / zero -- _y0 and _y1 only,
    //   * small positive values, where y0/y1 go to -inf and fire the epilogue.
    let mut xs: Vec<f64> = vec![
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        -0.0,
        0.0,
        -1.0,
        -1e-320,      // negative denormal -> _FPCLASS_ND
        1e-320,       // positive denormal
        -f64::MAX,
        f64::MIN_POSITIVE,
        1e-300,       // y0/y1 -> -inf here: the sqrt(-1) epilogue
        1e-30,
        1.0,
        2.0,
        3.0,
        10.0,
        1e3,
        1e10,
    ];
    xs.extend(f64_edge_bits().into_iter().map(f64::from_bits));
    for _ in 0..2000 {
        xs.push(f64::from_bits(rng.next()));
        // Ordinary-magnitude arguments too -- a corpus of pure bit noise is
        // almost entirely NaN and huge exponents, and would never exercise
        // the arms that matter.
        xs.push((rng.next() as i64 as f64) / 1e6);
    }

    // Drive one function, returning (value bits, errno). errno is cleared
    // first BECAUSE THE SLOT IS SHARED: without the reset a row would read
    // whatever the previous call left, and every row after the first domain
    // error would trivially agree.
    unsafe fn call(f: impl Fn() -> f64) -> (f64, i32) {
        *_errno() = 0;
        let v = f();
        (v, *_errno())
    }

    for &x in &xs {
        let b = x.to_bits();
        unsafe {
            let (sv, se) = call(|| cabi::_j0(x));
            let (rv, re) = call(|| ref__j0(x));
            assert!(same_f64(sv, rv), "_j0 value {b:#018x}");
            assert_eq!(se, re, "_j0 errno {b:#018x}");

            let (sv, se) = call(|| cabi::_j1(x));
            let (rv, re) = call(|| ref__j1(x));
            assert!(same_f64(sv, rv), "_j1 value {b:#018x}");
            assert_eq!(se, re, "_j1 errno {b:#018x}");

            let (sv, se) = call(|| cabi::_y0(x));
            let (rv, re) = call(|| ref__y0(x));
            assert!(same_f64(sv, rv), "_y0 value {b:#018x}");
            assert_eq!(se, re, "_y0 errno {b:#018x}");

            let (sv, se) = call(|| cabi::_y1(x));
            let (rv, re) = call(|| ref__y1(x));
            assert!(same_f64(sv, rv), "_y1 value {b:#018x}");
            assert_eq!(se, re, "_y1 errno {b:#018x}");

            // _jn/_yn take an order. 0 and 1 overlap the dedicated bodies,
            // negative orders are a distinct branch in FDLIBM, and the shell
            // must be transparent to all of them.
            for n in [-3i32, -1, 0, 1, 2, 7] {
                let (sv, se) = call(|| cabi::_jn(n, x));
                let (rv, re) = call(|| ref__jn(n, x));
                assert!(same_f64(sv, rv), "_jn({n}) value {b:#018x}");
                assert_eq!(se, re, "_jn({n}) errno {b:#018x}");

                let (sv, se) = call(|| cabi::_yn(n, x));
                let (rv, re) = call(|| ref__yn(n, x));
                assert!(same_f64(sv, rv), "_yn({n}) value {b:#018x}");
                assert_eq!(se, re, "_yn({n}) errno {b:#018x}");
            }
        }
    }
}

fn diff_math_class_f32() {
    let mut rng = Rng(0xbeef_cafe);
    let mut bits = f32_edge_bits();
    for _ in 0..20000 {
        bits.push(rng.next() as u32);
    }
    for &b in &bits {
        let x = f32::from_bits(b);
        unsafe {
            assert_eq!(cabi::_isnanf(x), ref_isnanf(x), "_isnanf {b:#010x}");
            assert_eq!(cabi::_finitef(x), ref_finitef(x), "_finitef {b:#010x}");
            assert_eq!(cabi::_fdsign(x), ref_fdsign(x), "_fdsign {b:#010x}");
            assert_eq!(cabi::_fdclass(x), ref_fdclass(x), "_fdclass {b:#010x}");
            assert_eq!(cabi::_fdtest(&x), ref_fdtest(&x), "_fdtest {b:#010x}");
        }
    }
}

fn diff_math_div() {
    let mut rng = Rng(0x0d1d_0d1d);
    let mut cases: Vec<(i32, i32)> = vec![
        (7, 3),
        (-7, 3),
        (7, -3),
        (-7, -3),
        (6, 3),
        (-6, 3),
        (0, 5),
        (0, -5),
        (i32::MAX, 7),
        (i32::MIN + 1, 7), // avoid INT_MIN / -1 overflow
        (i32::MAX, -7),
        (1, i32::MAX),
        // num > 0 && denom == i32::MIN takes the `denom = -denom` branch,
        // where the negation overflows. C wraps; Rust's unary `-` panics in
        // debug. The corpus above dodged INT_MIN/-1 but had never put
        // INT_MIN in the DENOMINATOR, so the panic shipped unseen.
        (7, i32::MIN),
        (1, i32::MIN),
        (i32::MAX, i32::MIN),
        (i32::MIN, i32::MIN),
        (0, i32::MIN),
    ];
    for _ in 0..5000 {
        let num = rng.next() as i32;
        let mut denom = rng.next() as i32;
        if denom == 0 {
            denom = 1;
        }
        // Skip INT_MIN / -1 (the one C division-overflow UB case).
        if num == i32::MIN && denom == -1 {
            continue;
        }
        cases.push((num, denom));
    }
    for &(n, d) in &cases {
        if n == i32::MIN && d == -1 {
            continue;
        }
        unsafe {
            let r = cabi::div(n, d);
            let e = ref_div(n, d);
            assert_eq!(r.quot, e.quot, "div quot {n}/{d}");
            assert_eq!(r.rem, e.rem, "div rem {n}/{d}");
        }
    }
}

fn diff_math_chgsignf() {
    let mut rng = Rng(0xc465_1167);
    let mut bits = f32_edge_bits();
    for _ in 0..20000 {
        bits.push(rng.next() as u32);
    }
    for &b in &bits {
        let x = f32::from_bits(b);
        unsafe {
            assert!(
                same_f32(cabi::_chgsignf(x), ref_chgsignf(x)),
                "_chgsignf {b:#010x}"
            );
        }
    }
}

fn diff_math_scalbn() {
    let mut rng = Rng(0x5ca1_b00f);
    let mut xs = f64_edge_bits();
    for _ in 0..4000 {
        xs.push(rng.next());
    }
    // n spans the clamp/overflow/underflow paths (musl branches at
    // ±1023/±1022 and clamps after two iterations).
    let ns: Vec<i32> = vec![
        0, 1, -1, 52, 53, -53, 1023, 1024, 2046, 2100, 5000, -1022, -1023, -1074, -2100, -5000,
    ];
    for &b in &xs {
        let x = f64::from_bits(b);
        for &n in &ns {
            unsafe {
                assert!(
                    same_f64(cabi::scalbn(x, n), ref_scalbn(x, n)),
                    "scalbn {b:#018x} n={n}"
                );
            }
        }
        let n = (rng.next() as i32) % 4200 - 2100;
        unsafe {
            assert!(
                same_f64(cabi::scalbn(x, n), ref_scalbn(x, n)),
                "scalbn {b:#018x} n={n}"
            );
        }
    }
}

fn diff_math_scalbnf() {
    let mut rng = Rng(0x5ca1_b0ff);
    let mut xs = f32_edge_bits();
    for _ in 0..4000 {
        xs.push(rng.next() as u32);
    }
    let ns: Vec<i32> = vec![
        0, 1, -1, 23, 24, -24, 127, 128, 254, 300, 700, -126, -127, -149, -300, -700,
    ];
    for &b in &xs {
        let x = f32::from_bits(b);
        for &n in &ns {
            unsafe {
                assert!(
                    same_f32(cabi::scalbnf(x, n), ref_scalbnf(x, n)),
                    "scalbnf {b:#010x} n={n}"
                );
            }
        }
        let n = (rng.next() as i32) % 600 - 300;
        unsafe {
            assert!(
                same_f32(cabi::scalbnf(x, n), ref_scalbnf(x, n)),
                "scalbnf {b:#010x} n={n}"
            );
        }
    }
}

fn diff_math_round() {
    let mut rng = Rng(0x5004_d123);
    let mut xs = f64_edge_bits();
    // Half-integer / near-boundary cases where round's ±0.5 matters.
    for v in [
        0.5f64, -0.5, 1.5, -1.5, 2.5, -2.5, 0.4999, -0.4999, 0.5001, 1e16, -1e16,
    ] {
        xs.push(v.to_bits());
    }
    for _ in 0..20000 {
        xs.push(rng.next());
    }
    for &b in &xs {
        let x = f64::from_bits(b);
        unsafe {
            assert!(same_f64(cabi::round(x), ref_round(x)), "round {b:#018x}");
        }
    }
}

fn diff_math_roundf() {
    let mut rng = Rng(0x5004_d1ff);
    let mut xs = f32_edge_bits();
    for v in [0.5f32, -0.5, 1.5, -1.5, 2.5, -2.5, 0.4999, -0.4999, 0.5001] {
        xs.push(v.to_bits());
    }
    for _ in 0..20000 {
        xs.push(rng.next() as u32);
    }
    for &b in &xs {
        let x = f32::from_bits(b);
        unsafe {
            assert!(same_f32(cabi::roundf(x), ref_roundf(x)), "roundf {b:#010x}");
        }
    }
}

fn diff_math_nextafter() {
    let mut rng = Rng(0x0ea7_a47e);
    let mut bits = f64_edge_bits();
    for _ in 0..2000 {
        bits.push(rng.next());
    }
    for &bx in &bits {
        let x = f64::from_bits(bx);
        for &by in &bits {
            let y = f64::from_bits(by);
            unsafe {
                assert!(
                    same_f64(cabi::nextafter(x, y), ref_nextafter(x, y)),
                    "nextafter {bx:#018x} {by:#018x}"
                );
            }
        }
    }
}

fn diff_math_nextafterf() {
    let mut rng = Rng(0x0ea7_a4ff);
    let mut bits = f32_edge_bits();
    for _ in 0..2000 {
        bits.push(rng.next() as u32);
    }
    for &bx in &bits {
        let x = f32::from_bits(bx);
        for &by in &bits {
            let y = f32::from_bits(by);
            unsafe {
                assert!(
                    same_f32(cabi::nextafterf(x, y), ref_nextafterf(x, y)),
                    "nextafterf {bx:#010x} {by:#010x}"
                );
            }
        }
    }
}

/// The exports over `libm`: sin/cos/tan, asin/acos/atan, atan2, the hyperbolics and hypot
/// return exactly `libm`'s value (the AMD ports they replaced keep their rows against the
/// C above, as `*_amd`). No error hook is linked here (`crt-math-errors` is off), so this
/// pins the values; the guest's matherrs payload pins the reports.
fn diff_math_libm_exports() {
    let mut rng = Rng(0x11b3_c0de_5eed_0001);
    let mut xs: Vec<f64> = f64_edge_bits().into_iter().map(f64::from_bits).collect();
    for _ in 0..200_000 {
        xs.push(f64::from_bits(rng.next()));
        xs.push((rng.next() as i64 as f64) / 1e15);
    }
    for &x in &xs {
        let y = f64::from_bits(rng.next());
        let b = x.to_bits();
        macro_rules! one { ($($f:ident),*) => {$(
            assert!(same_f64(cabi::$f(x), libm::$f(x)), concat!(stringify!($f), " {:#018x}"), b);
        )*} }
        one!(sin, cos, tan, asin, acos, atan, sinh, cosh, tanh);
        assert!(same_f64(cabi::atan2(y, x), libm::atan2(y, x)), "atan2 {b:#018x}");
        assert!(same_f64(cabi::_hypot(y, x), libm::hypot(y, x)), "_hypot {b:#018x}");
        let (xf, yf) = (x as f32, y as f32);
        let bf = xf.to_bits();
        macro_rules! onef { ($($f:ident),*) => {$(
            assert!(cabi::$f(xf).to_bits() == libm::$f(xf).to_bits() || (cabi::$f(xf).is_nan() && libm::$f(xf).is_nan()),
                concat!(stringify!($f), " {:#010x}"), bf);
        )*} }
        onef!(sinf, cosf, tanf, asinf, acosf, atanf, sinhf, coshf, tanhf);
        let (a, e) = (cabi::atan2f(yf, xf), libm::atan2f(yf, xf));
        assert!(a.to_bits() == e.to_bits() || (a.is_nan() && e.is_nan()), "atan2f {bf:#010x}");
        let (a, e) = (cabi::_hypotf(yf, xf), libm::hypotf(yf, xf));
        assert!(a.to_bits() == e.to_bits() || (a.is_nan() && e.is_nan()), "_hypotf {bf:#010x}");
    }
}

fn diff_math_sincos() {
    // cnt-rust ports AMD's libm_sse2/{sin,cos}.asm; the oracle is libm, to 1
    // ULP. Bit-exactness is not available (different algorithms) and the old
    // bit-exact oracle, math/{sin,cos}.c, was deleted rather than loosened —
    // see build.rs for why grading against it was worse than not grading.
    //
    // 1 ULP is a real bound, not a rubber stamp: glibc is under 0.55 ULP here,
    // so anything that drifts algorithmically fails. For scale, the C this
    // replaced misses this bound by eight orders of magnitude at its BEST
    // (1.3e8 ULP for |x| < pi/4) and by infinitely much above 1e9, where it
    // returns values larger than 1e160.
    const BOUND: i64 = 1;
    let mut rng = Rng(0x713c_0d1a);

    // Exact cases first. These are the ones the old implementation got wrong
    // in kind rather than in degree, so they are asserted EXACTLY even though
    // the sweep below is a tolerance: sin(±0) must carry the sign of zero, and
    // cos(0) must be exactly 1.0.
    assert_eq!(cabi::sin_amd(0.0).to_bits(), 0u64, "sin(+0.0) must be +0.0");
    assert_eq!(cabi::sin_amd(-0.0).to_bits(), 1u64 << 63, "sin(-0.0) must be -0.0");
    assert_eq!(cabi::cos_amd(0.0), 1.0, "cos(0) must be exactly 1");
    assert_eq!(cabi::cos_amd(-0.0), 1.0, "cos(-0) must be exactly 1");

    let edges = [
        0.0f64, -0.0, 1.0, -1.0, 0.5, -0.5, 3.0, -3.0, 100.0, -100.0, 1000.0, -1000.0,
        // The reduction boundaries: pi/4 (fast path ends), 2^-13 and 2^-27
        // (cos's small-x branches), 5e5 (Cody-Waite gives way to Payne-Hanek).
        core::f64::consts::FRAC_PI_4, -core::f64::consts::FRAC_PI_4,
        core::f64::consts::FRAC_PI_2, core::f64::consts::PI,
        1.220703125e-4, 7.450580596923828e-9, 1e-300,
        499_999.0, 500_000.0, 500_001.0, -500_000.0,
        // Large arguments, where the old body returned >1e160 instead of a
        // value in [-1, 1]. Payne-Hanek has to carry these exactly.
        1e10, -1e10, 1e18, 1e17, 1.234_567_89e17, f64::MAX,
    ];
    for &x in &edges {
        let (s, c) = (cabi::sin_amd(x), cabi::cos_amd(x));
        assert!(near_ulps(s, unsafe { ref_sin(x) }, BOUND), "sin {x:e}");
        assert!(near_ulps(c, unsafe { ref_cos(x) }, BOUND), "cos {x:e}");
        // A trig function that leaves [-1, 1] is the failure that made this
        // lane exist, and a ULP comparison against a NaN/garbage reference
        // could in principle pass it through. Assert the range outright.
        assert!(s.abs() <= 1.0 && c.abs() <= 1.0, "sin/cos out of range at {x:e}");
    }

    // Sweep every band with its own reduction path, not just the ±2000 the
    // old test used — the failure modes here are all reduction failures, and
    // ±2000 never leaves the Cody-Waite branch at all.
    for &(lo, hi) in &[
        (0.0f64, 0.785f64),
        (0.785, 10.0),
        (10.0, 1e5),
        (1e5, 5e5),
        (5e5, 1e9),
        (1e9, 1e17),
    ] {
        for _ in 0..60_000 {
            let u = rng.next();
            let t = (u >> 11) as f64 / (1u64 << 53) as f64;
            for x in [lo + t * (hi - lo), -(lo + t * (hi - lo))] {
                let (s, c) = (cabi::sin_amd(x), cabi::cos_amd(x));
                assert!(near_ulps(s, unsafe { ref_sin(x) }, BOUND), "sin {x:e}");
                assert!(near_ulps(c, unsafe { ref_cos(x) }, BOUND), "cos {x:e}");
                assert!(s.abs() <= 1.0 && c.abs() <= 1.0, "sin/cos out of range at {x:e}");
            }
        }
    }
}

fn diff_math_sqrt() {
    // Hardware sqrt = correctly rounded = libm reference, bit-for-bit.
    let mut rng = Rng(0x5012_3a4b_c0de_f00d);
    let mut bits = f64_edge_bits();
    for v in [1.0f64, 2.0, 4.0, 0.25, 1e300, 1e-300] {
        bits.push(v.to_bits());
    }
    for _ in 0..200_000 {
        bits.push(rng.next());
    }
    for &b in &bits {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::sqrt(x), unsafe { ref_sqrt(x) }), "sqrt {b:#018x}");
        let xf = f32::from_bits((b >> 32) as u32);
        assert!(same_f32(cabi::sqrtf(xf), unsafe { ref_sqrtf(xf) }), "sqrtf");
    }
}

fn diff_math_nexttowardf() {
    let mut rng = Rng(0x0ea7_70a4);
    let mut xb = f32_edge_bits();
    for _ in 0..1500 {
        xb.push(rng.next() as u32);
    }
    let mut yb = f64_edge_bits();
    for _ in 0..1500 {
        yb.push(rng.next());
    }
    for &bx in &xb {
        let x = f32::from_bits(bx);
        for &by in &yb {
            let y = f64::from_bits(by);
            unsafe {
                assert!(
                    same_f32(cabi::nexttowardf(x, y), ref_nexttowardf(x, y)),
                    "nexttowardf {bx:#010x} {by:#018x}"
                );
            }
        }
    }
}

// --- Wave MATH-TIERC sub-wave 2: exp / log / log10 / pow / fmod -------------

fn diff_math_exp() {
    // Two contracts. The exported `exp` is musl's (musl-math, graded bit-exact against
    // the musl C in musl-math-difftest) wherever libm-class crates build; `exp_fdlibm`,
    // what the no-SSE i686 build exports instead, is
    // range-reduce + degree-5 rational; bit-exact vs the genuine .c under
    // -ffp-contract=off. Random bit images cover NaN/±inf/subnormal/
    // overflow(>709)/underflow(<-745)/tiny(<2^-28); a dense moderate sweep
    // exercises the polynomial + 2^k scaling path.
    let mut rng = Rng(0xe5a1_0e7c);
    let mut bits = f64_edge_bits();
    for _ in 0..50_000 {
        bits.push(rng.next());
    }
    for &b in &bits {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::exp_fdlibm(x), unsafe { ref_exp(x) }), "exp {b:#018x}");
        assert!(same_f64(cabi::exp(x), musl_math::exp(x).0), "exp(musl) {b:#018x}");
    }
    for _ in 0..200_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 1500.0 - 750.0;
        assert!(same_f64(cabi::exp_fdlibm(x), unsafe { ref_exp(x) }), "exp {x}");
        assert!(same_f64(cabi::exp(x), musl_math::exp(x).0), "exp(musl) {x}");
    }
}

fn diff_math_log() {
    // Natural log, fdlibm-style. Random bit images cover negative→NaN,
    // 0→-inf, inf/NaN passthrough, and the subnormal-normalize (k-=54)
    // path; a dense positive sweep exercises the polynomial across decades.
    let mut rng = Rng(0x10c0_9a31);
    let mut bits = f64_edge_bits();
    for _ in 0..50_000 {
        bits.push(rng.next());
    }
    for &b in &bits {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::log_fdlibm(x), unsafe { ref_log(x) }), "log {b:#018x}");
        assert!(same_f64(cabi::log(x), musl_math::log(x).0), "log(musl) {b:#018x}");
    }
    for _ in 0..200_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 1.0e6;
        assert!(same_f64(cabi::log_fdlibm(x), unsafe { ref_log(x) }), "log {x}");
        assert!(same_f64(cabi::log(x), musl_math::log(x).0), "log(musl) {x}");
    }
}

/// Within `ulps` of `want`, which is the correctly rounded value (exact decimal
/// arithmetic, 80 digits). `log10`/`pow` are the `libm` crate's now, which does not promise
/// correct rounding, so a reference that IS correctly rounded is graded to an ulp budget.
fn within_ulps(got: f64, want: f64, ulps: u64) -> bool {
    let (g, w) = (got.to_bits() as i64, want.to_bits() as i64);
    (got.is_sign_negative() == want.is_sign_negative()) && g.abs_diff(w) <= ulps
}

fn diff_math_log10() {
    // Was bit-exact against upstream's `math/arm64/log10.c`, which is not a logarithm: it
    // divided by ten in a loop and added `(t - 1) / 9`, so log10(2) was 0.111, and log10(inf)
    // never returned (this fuzz had to exclude it). cnt-rust's `log10` is libm's now.
    let table: [(f64, f64); 10] = [
        (2.0, f64::from_bits(0x3fd34413509f79ff)),
        (3.0, f64::from_bits(0x3fde8927964fd5fd)),
        (50.0, f64::from_bits(0x3ffb2efb2bd82180)),
        (1000.0, f64::from_bits(0x4008000000000000)),
        (0.5, f64::from_bits(0xbfd34413509f79ff)),
        (0.001, f64::from_bits(0xc008000000000000)),
        (7.5e-300, f64::from_bits(0xc072b1ffbfc2bbc8)),
        (f64::MAX, f64::from_bits(0x40734413509f79ff)),
        (123456.789, f64::from_bits(0x40145db61a282512)),
        (0.3, f64::from_bits(0xbfe0bb6c34d81502)),
    ];
    for (x, want) in table {
        assert!(within_ulps(cabi::log10(x), want, 1), "log10 {x} = {}", cabi::log10(x));
    }
    // Exact powers of ten are exact, and the specials terminate.
    for k in -22i32..=22 {
        let x = format!("1e{k}").parse::<f64>().unwrap();
        assert_eq!(cabi::log10(x), k as f64, "log10 1e{k}");
    }
    assert_eq!(cabi::log10(f64::INFINITY), f64::INFINITY);
    assert_eq!(cabi::log10(0.0), f64::NEG_INFINITY);
    assert!(cabi::log10(-1.0).is_nan() && cabi::log10(f64::NAN).is_nan());
}

fn diff_math_pow() {
    // Was bit-exact against upstream's `math/arm64/pow.c`, whose general case was
    // `return 0.0; // FIXME` -- pow(2, 0.5) == 0 shipped in msvcrt.dll and ucrtbase.dll.
    let table: [(f64, f64, f64); 10] = [
        (2.0, 0.5, f64::from_bits(0x3ff6a09e667f3bcd)),
        (8.0, 1.0 / 3.0, f64::from_bits(0x4000000000000000)),
        (1.5, 2.5, f64::from_bits(0x40060b9fd68a4554)),
        (10.0, -0.5, f64::from_bits(0x3fd43d136248490f)),
        (2.0, 10.0, f64::from_bits(0x4090000000000000)),
        (3.0, -3.0, f64::from_bits(0x3fa2f684bda12f68)),
        (0.5, 0.25, f64::from_bits(0x3feae89f995ad3ad)),
        (100.0, 1.5, f64::from_bits(0x408f400000000000)),
        (7.0, 0.1, f64::from_bits(0x3ff36fe0d9dde8a3)),
        (1e-5, 0.75, f64::from_bits(0x3f274eea61c12624)),
    ];
    for (b, e, want) in table {
        assert!(within_ulps(cabi::pow(b, e), want, 1), "pow {b} {e} = {}", cabi::pow(b, e));
    }
    // Integer powers that are exact in binary stay exact; C99 F.9.4.4's specials.
    for k in -60i32..=60 {
        assert_eq!(cabi::pow(2.0, k as f64), f64::from_bits(((1023 + k) as u64) << 52), "pow 2 {k}");
    }
    assert_eq!(cabi::pow(-2.0, 3.0), -8.0);
    assert_eq!(cabi::pow(f64::NAN, 0.0), 1.0);
    assert_eq!(cabi::pow(1.0, f64::NAN), 1.0);
    assert_eq!(cabi::pow(0.0, -1.0), f64::INFINITY);
    assert!(cabi::pow(-1.0, 0.5).is_nan());
}

fn diff_math_fmod() {
    // Pure bit-twiddling remainder, faithful to the ReactOS arm64 stub.
    //
    // DOMAIN NOTE: the upstream C normalizes a *subnormal* operand to a
    // different binade than a normal one (its `ex -= 11; xi <<= (1-ex)`
    // bookkeeping lands the leading bit at ~2^63, vs ~2^52 for normals —
    // unlike musl's `ux <<= -ex+1`). With a normal x and a subnormal y the
    // mismatched scales drive xi to 0 and the C's own `while(!(xi>>52))`
    // loop spins forever. So a *subnormal operand* is outside the
    // reference's defined domain (and the kernel's: fmod is fed only finite
    // normals by printf float formatting). We therefore fuzz normal /
    // signed-zero / inf / NaN operands — every case the C actually handles
    // — and the Rust matches it bit-for-bit there. (Both sides are
    // identical on subnormals too, including the hang; there is nothing to
    // compare.) `compose_normal` forces the exponent field into [1, 0x7FE].
    let mut rng = Rng(0xf3d0_b007);
    let compose_normal = |r: u64| -> f64 {
        let sign = r & 0x8000_0000_0000_0000;
        let exp = 1 + (r >> 52) % 0x7FE; // 1 ..= 2046 (normal)
        let mant = r & 0x000F_FFFF_FFFF_FFFF;
        f64::from_bits(sign | (exp << 52) | mant)
    };
    // Non-subnormal specials: signed zeros, ±inf, NaN, and a few normals.
    let specials: [f64; 11] = [
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        1.0,
        -1.0,
        2.0,
        -2.0,
        0.5,
        f64::MAX,
    ];
    for &x in &specials {
        for &y in &specials {
            assert!(
                same_f64(cabi::fmod(x, y), unsafe { ref_fmod(x, y) }),
                "fmod {:#018x} {:#018x}",
                x.to_bits(),
                y.to_bits()
            );
        }
    }
    // Random normal × normal — the reduction + result-construct paths.
    for _ in 0..60_000 {
        let x = compose_normal(rng.next());
        let y = compose_normal(rng.next());
        assert!(
            same_f64(cabi::fmod(x, y), unsafe { ref_fmod(x, y) }),
            "fmod {:#018x} {:#018x}",
            x.to_bits(),
            y.to_bits()
        );
    }
    // Mixed: normal against the specials (fmod(x,0)=NaN, fmod(x,inf)=x, …).
    for _ in 0..20_000 {
        let x = compose_normal(rng.next());
        for &y in &specials {
            assert!(
                same_f64(cabi::fmod(x, y), unsafe { ref_fmod(x, y) }),
                "fmod {:#018x} {:#018x}",
                x.to_bits(),
                y.to_bits()
            );
            assert!(
                same_f64(cabi::fmod(y, x), unsafe { ref_fmod(y, x) }),
                "fmod {:#018x} {:#018x}",
                y.to_bits(),
                x.to_bits()
            );
        }
    }
    // The case the upstream C (and this fuzz's former domain note) never returned from:
    // a normal x and a subnormal y. libm's fmod is exact here too.
    let y = f64::from_bits(3); // 3 * 2^-1074
    // 1.0 is 2^1074 units of 2^-1074, and 2^1074 = 4^537 = 1 (mod 3): the remainder is 1 unit.
    assert_eq!(cabi::fmod(1.0, y), f64::from_bits(1), "fmod 1 subnormal");
}

// --- Wave MATH-TIERC sub-wave 3: tan ---------------------------------------

fn diff_math_tan() {
    // cnt-rust's tan is a hand port of AMD's math/libm_sse2/tan.c, which is
    // what amd64 links; the oracle is that same C, bit-exact.
    //
    // It was `sin(x)/cos(x)` graded against a reference composed from
    // ref_sin/ref_cos — an oracle that could only ever confirm the quotient,
    // never the implementation amd64 actually shipped, which is how a measured
    // 0.056-absolute regression stayed invisible. The domain below is also
    // wider than that test's ±2000: it now includes the |x| >= 5e5 arguments
    // that take the Payne-Hanek path in `remainder_piby2`, which the old
    // quotient had no equivalent of and which nothing exercised.
    let mut rng = Rng(0x7a17_c05c);
    for &b in &f64_edge_bits() {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::tan_amd(x), unsafe { ref_tan(x) }), "tan {b:#018x}");
    }
    let edges = [
        0.0f64, -0.0, 1.0, -1.0, 0.5, -0.5, 3.0, -3.0, 100.0, -100.0, 1000.0, -1000.0,
        // the branch boundaries the C dispatches on, from both sides
        core::f64::consts::FRAC_PI_4, -core::f64::consts::FRAC_PI_4, 0.68, -0.68,
        f64::from_bits(0x3f20_0000_0000_0000), f64::from_bits(0x3e40_0000_0000_0000),
        f64::from_bits(0x400f_6a7a_2955_385e), f64::from_bits(0x4002_d97c_7f33_21d2),
        f64::from_bits(0x401c_463a_becc_b2bb), f64::from_bits(0x4015_fdbb_e9bb_a775),
        5.0e5, -5.0e5, 4.999_999e5, 5.000_001e5, 1.0e300, -1.0e300,
    ];
    for &x in &edges {
        assert!(same_f64(cabi::tan_amd(x), unsafe { ref_tan(x) }), "tan {x:e}");
    }
    // dense over the hand-reduction range, then the whole f64 line (which is
    // overwhelmingly the Payne-Hanek path).
    for _ in 0..200_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 4000.0 - 2000.0;
        assert!(same_f64(cabi::tan_amd(x), unsafe { ref_tan(x) }), "tan {x}");
    }
    for _ in 0..200_000 {
        let b = rng.next();
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::tan_amd(x), unsafe { ref_tan(x) }), "tan {b:#018x}");
    }
}

// --- Wave MATH-TIERC sub-wave 4: ceil / floor / ceilf / floorf -------------

fn diff_math_ceil_floor() {
    // Exact directed rounding — bit-exact vs hardware roundsd/roundss
    // (__builtin reference). Full edge bit set + dense random bit images
    // (covers integral, fractional, |x|<1, subnormal, inf, NaN, ±0) + a
    // sweep through magnitudes that straddle the 2^52 / 2^23 integral cliff.
    let mut rng = Rng(0xce11_f100);
    let mut b64 = f64_edge_bits();
    for v in [0.5f64, -0.5, 1.5, -1.5, 2.5, -2.5, 1e16, -1e16, 4503599627370495.5] {
        b64.push(v.to_bits());
    }
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::ceil(x), unsafe { ref_ceil(x) }), "ceil {b:#018x}");
        assert!(same_f64(cabi::floor(x), unsafe { ref_floor(x) }), "floor {b:#018x}");
    }
    let mut b32 = f32_edge_bits();
    for v in [0.5f32, -0.5, 1.5, -1.5, 2.5, -2.5, 8388607.5] {
        b32.push(v.to_bits());
    }
    for _ in 0..200_000 {
        b32.push(rng.next() as u32);
    }
    for &b in &b32 {
        let x = f32::from_bits(b);
        assert!(same_f32(cabi::ceilf(x), unsafe { ref_ceilf(x) }), "ceilf {b:#010x}");
        assert!(same_f32(cabi::floorf(x), unsafe { ref_floorf(x) }), "floorf {b:#010x}");
    }
}

// --- Wave MATH-TIERC sub-wave 5: ldexp / _logb / _logbf --------------------

fn diff_math_ldexp_logb() {
    // ldexp(x,n) = x*2^n (portable stepping); _logb/_logbf extract the
    // unbiased exponent. Bit-exact vs the genuine .c. Edge bits + random
    // bit images + (for ldexp) a spread of exponents incl. the clamping
    // extremes (>1023 / <-1022) and 0/±1.
    let mut rng = Rng(0x1de4_106b);
    let mut b64 = f64_edge_bits();
    for _ in 0..40_000 {
        b64.push(rng.next());
    }
    let ns = [
        0i32, 1, -1, 52, -52, 1023, -1022, 1024, -1023, 2048, -2048, 3000, -3000, 100, -100,
    ];
    for &b in &b64 {
        let x = f64::from_bits(b);
        for &n in &ns {
            let got = cabi::ldexp(x, n);
            let amd = unsafe { ref_ldexp(x, n) };
            if same_f64(got, amd) {
                continue;
            }
            // AMD's ldexp.c has a REAL BUG, and this is the only place the two
            // disagree. Its denormal path normalizes by 2^53, range-checks the
            // *pre*-compensation exponent (`<= 2046`), then adds the 53 back
            // before packing -- so a subnormal x with a large n can pack a
            // biased exponent above 2047, which overflows the 11-bit field
            // straight into the SIGN bit and is then scaled by 2^-53.
            // ldexp(0x1p-1074, 2048) returns 0x8000000000000001 (minus the
            // smallest subnormal) where the answer is 0x7cd0000000000000
            // (2^974) -- confirmed against glibc.
            //
            // So here the Rust is CORRECTER than the C it replaces on x86, and
            // the assertion is against musl's scalbn (math/scalbn.c, the same
            // function under its C99 name, built for every arch) instead. The
            // exception is pinned to the exact shape of the defect so it cannot
            // quietly widen into cover for a translation bug.
            let subnormal = b & 0x7FF0_0000_0000_0000 == 0 && b & 0x000F_FFFF_FFFF_FFFF != 0;
            assert!(subnormal && n > 1023, "ldexp {b:#018x} n={n} (unexpected AMD divergence)");
            assert!(
                same_f64(got, unsafe { ref_scalbn(x, n) }),
                "ldexp {b:#018x} n={n} (vs musl scalbn)"
            );
        }
        assert!(same_f64(cabi::_logb(x), unsafe { ref__logb(x) }), "_logb {b:#018x}");
    }
    let mut b32 = f32_edge_bits();
    for _ in 0..100_000 {
        b32.push(rng.next() as u32);
    }
    for &b in &b32 {
        let x = f32::from_bits(b);
        assert!(same_f32(cabi::_logbf(x), unsafe { ref__logbf(x) }), "_logbf {b:#010x}");
    }
}

// --- Wave MATH-TIERC sub-wave 6: fabs / fabsf -----------------------------

fn diff_math_fabs() {
    // Magnitude via sign-bit clear, with the function-specific NaN handling
    // (fabs returns NaN unchanged; fabsf ORs in the quiet bit). Bit-exact vs
    // the genuine reference across all edge bits + a heavy random bit image,
    // so every NaN/±0/±inf/subnormal pattern is exercised.
    let mut rng = Rng(0xfab5_0006);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::fabs(x), unsafe { ref_fabs(x) }), "fabs {b:#018x}");
    }
    let mut b32 = f32_edge_bits();
    for _ in 0..200_000 {
        b32.push(rng.next() as u32);
    }
    for &b in &b32 {
        let x = f32::from_bits(b);
        assert!(same_f32(cabi::fabsf(x), unsafe { ref_fabsf(x) }), "fabsf {b:#010x}");
    }
}

// --- Wave MATH-TIERC sub-wave 7: modf --------------------------------------

fn diff_math_modf() {
    // Integer/fraction split — exact, so bit-exact on BOTH the return value
    // and the integer part written through *iptr. Check both against the
    // reference across all edge bits + 200k random bit images.
    let mut rng = Rng(0x70df_0007);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        let mut ri = 0.0f64;
        let mut ci = 0.0f64;
        let cf = unsafe { cabi::modf(x, &mut ci) };
        let rf = unsafe { ref_modf(x, &mut ri) };
        assert!(same_f64(cf, rf), "modf frac {b:#018x}");
        assert!(same_f64(ci, ri), "modf int {b:#018x}");
    }
}

fn diff_math_atan() {
    // TWO oracles, because the tree ships two different C implementations of
    // these and cnt-rust's Rust replaces both (math.cmake: math/arm64/*.c on
    // arm64, AMD's math/libm_sse2/*.c on amd64 and i386):
    //
    //   ref_atan_p / ref_atan2_p — the portable fdlibm C, the algorithm the
    //       Rust actually implements. Graded BIT-EXACT, as before.
    //   ref_atan  / ref_atan2    — AMD's, what x86 linked before the flip.
    //       Graded to 1 ULP. A different algorithm cannot be bit-exact, but it
    //       is a real regression detector for the arches that lost AMD's body,
    //       and it is the check that did not exist at all before: atan/atan2
    //       had only the portable oracle, i.e. the very source the port was
    //       made from, so amd64's shipped behaviour was graded by nothing.
    //
    // The swap itself is a wash on accuracy, measured over 2M random finite
    // inputs against `atanl`: AMD is not correctly rounded 0.019% of the time,
    // fdlibm 0.023%, both with a 0.795 ULP maximum error. The two differ on
    // exactly two shapes, each 1 ULP: AMD quiets a signalling NaN and fdlibm
    // returns it raw, and just past the ±pi/2 boundary (|y| large, x a tiny
    // negative) fdlibm rounds up to the correctly-rounded neighbour where AMD
    // stays at pi/2.
    let mut rng = Rng(0xa7a2_0007);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        let got = cabi::atan_amd(x);
        assert!(same_f64(got, unsafe { ref_atan_p(x) }), "atan {b:#018x}");
        assert!(near_ulps(got, unsafe { ref_atan(x) }, 1), "atan vs AMD {b:#018x}");
    }
    // atan2 over all (edge × edge) pairs + random pairs.
    let edges = f64_edge_bits();
    for &by in &edges {
        for &bx in &edges {
            let y = f64::from_bits(by);
            let x = f64::from_bits(bx);
            let got = cabi::atan2_amd(y, x);
            assert!(
                same_f64(got, unsafe { ref_atan2_p(y, x) }),
                "atan2 y={by:#018x} x={bx:#018x}"
            );
            assert!(
                near_ulps(got, unsafe { ref_atan2(y, x) }, 1),
                "atan2 vs AMD y={by:#018x} x={bx:#018x}"
            );
        }
    }
    for _ in 0..200_000 {
        let y = f64::from_bits(rng.next());
        let x = f64::from_bits(rng.next());
        let got = cabi::atan2_amd(y, x);
        assert!(
            same_f64(got, unsafe { ref_atan2_p(y, x) }),
            "atan2 y={:#018x} x={:#018x}",
            y.to_bits(),
            x.to_bits()
        );
        assert!(
            near_ulps(got, unsafe { ref_atan2(y, x) }, 1),
            "atan2 vs AMD y={:#018x} x={:#018x}",
            y.to_bits(),
            x.to_bits()
        );
    }
}

/// Two finite results within `max` ULPs, or both NaN / both the same infinity.
///
/// The loose comparison exists for ONE purpose: grading the Rust against a
/// DIFFERENT algorithm than the one it implements (AMD's atan/atan2 vs the
/// fdlibm the port follows). Everywhere the tree grades a port against its own
/// algorithm's C, `same_f64` is the bar and stays the bar -- this must never be
/// reached for to soften a bit-exactness failure.
fn near_ulps(a: f64, b: f64, max: i64) -> bool {
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    if a == b {
        return true;
    }
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    // Map to a monotone ordering across the sign boundary, then subtract.
    let key = |v: f64| -> i64 {
        let bits = v.to_bits() as i64;
        if bits < 0 {
            i64::MIN - bits
        } else {
            bits
        }
    };
    (key(a) - key(b)).abs() <= max
}

fn diff_math_exp2() {
    // 2**x, AMD-libm splitexp path + small-arg rational. Bit-exact vs the
    // genuine libm_sse2/exp2.c across edge bits, a heavy random bit image,
    // and a dense sweep over [-1100, 1100] (covers the overflow/underflow
    // thresholds, the small-arg branch, and the splitexp main path).
    let mut rng = Rng(0xe2b2_0008);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::exp2_amd(x), unsafe { ref_exp2(x) }), "exp2 {b:#018x}");
        assert!(same_f64(cabi::exp2(x), musl_math::exp2(x).0), "exp2(musl) {b:#018x}");
    }
    for _ in 0..200_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 2200.0 - 1100.0;
        assert!(same_f64(cabi::exp2_amd(x), unsafe { ref_exp2(x) }), "exp2 {x}");
        assert!(same_f64(cabi::exp2(x), musl_math::exp2(x).0), "exp2(musl) {x}");
    }
}

fn diff_math_tanh() {
    // tanh: small-|x| identity, two Remez sub-ranges on (0,1], splitexp path,
    // ±1 saturation, NaN. Bit-exact vs libm_sse2/tanh.c across edge bits, a
    // random bit image, and a dense sweep over [-25, 25].
    let mut rng = Rng(0x7a24_0008);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::tanh_amd(x), unsafe { ref_tanh(x) }), "tanh {b:#018x}");
    }
    for _ in 0..200_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 50.0 - 25.0;
        assert!(same_f64(cabi::tanh_amd(x), unsafe { ref_tanh(x) }), "tanh {x}");
    }
}

fn diff_math_sinh_cosh() {
    // sinh (odd) and cosh (even): table reconstruction for small |x|, the
    // splitexp exp/2 path for larger, ±Inf saturation past max_*_arg, NaN.
    // Bit-exact vs libm_sse2/{sinh,cosh}.c across edge bits, a random bit
    // image, and a dense sweep over [-720, 720] (spans both table-branch
    // thresholds — sinh 36.12 / cosh 20 — the splitexp range, and overflow).
    let mut rng = Rng(0x51c8_0009);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::sinh_amd(x), unsafe { ref_sinh(x) }), "sinh {b:#018x}");
        assert!(same_f64(cabi::cosh_amd(x), unsafe { ref_cosh(x) }), "cosh {b:#018x}");
    }
    for _ in 0..200_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 1440.0 - 720.0;
        assert!(same_f64(cabi::sinh_amd(x), unsafe { ref_sinh(x) }), "sinh {x}");
        assert!(same_f64(cabi::cosh_amd(x), unsafe { ref_cosh(x) }), "cosh {x}");
    }
}

fn diff_math_asin_acos() {
    // asin/acos: small-|x| identity, |x|<=0.5 rational, |x|>0.5 sqrt
    // half-angle reconstruction, the ±1 / |x|>=1 domain edges, NaN. Bit-exact
    // vs libm_sse2/{asin,acos}.c across edge bits, a random bit image, and a
    // dense sweep over the valid domain [-1, 1] plus a little past it.
    let mut rng = Rng(0xac5a_0010);
    let mut b64 = f64_edge_bits();
    for _ in 0..200_000 {
        b64.push(rng.next());
    }
    for &b in &b64 {
        let x = f64::from_bits(b);
        assert!(same_f64(cabi::asin_amd(x), unsafe { ref_asin(x) }), "asin {b:#018x}");
        assert!(same_f64(cabi::acos_amd(x), unsafe { ref_acos(x) }), "acos {b:#018x}");
    }
    for _ in 0..400_000 {
        let u = rng.next();
        let x = (u as f64) / (u64::MAX as f64) * 2.2 - 1.1;
        assert!(same_f64(cabi::asin_amd(x), unsafe { ref_asin(x) }), "asin {x}");
        assert!(same_f64(cabi::acos_amd(x), unsafe { ref_acos(x) }), "acos {x}");
    }
}

fn diff_math_hypot() {
    // _hypot: sqrt(x*x+y*y) with exponent scaling (FAST path — the .c defines
    // FAST_BUT_GREATER_THAN_ONE_ULP, so r = x*x+y*y directly).
    // Bit-exact vs libm_sse2/hypot.c. Exercises the NaN/Inf cases, the
    // zero/insignificant shortcuts, the over/underflow scaling branches
    // (incl. denormals), and the equal-exponent (dexp==0) precise path —
    // via edge×edge pairs, random pairs, and same-exponent random pairs.
    let mut rng = Rng(0x4970_0011);
    let edges = f64_edge_bits();
    for &by in &edges {
        for &bx in &edges {
            let y = f64::from_bits(by);
            let x = f64::from_bits(bx);
            assert!(
                same_f64(cabi::_hypot_amd(x, y), unsafe { ref__hypot(x, y) }),
                "hypot x={bx:#018x} y={by:#018x}"
            );
        }
    }
    for _ in 0..300_000 {
        let x = f64::from_bits(rng.next());
        let y = f64::from_bits(rng.next());
        assert!(
            same_f64(cabi::_hypot_amd(x, y), unsafe { ref__hypot(x, y) }),
            "hypot x={:#018x} y={:#018x}",
            x.to_bits(),
            y.to_bits()
        );
    }
    // Same-magnitude pairs to hit the dexp==0 precise branch heavily.
    for _ in 0..200_000 {
        let bx = rng.next();
        // copy x's exponent field into y, randomize y's sign+mantissa.
        let by = (bx & 0x7ff0_0000_0000_0000) | (rng.next() & 0x800f_ffff_ffff_ffff);
        let x = f64::from_bits(bx);
        let y = f64::from_bits(by);
        assert!(
            same_f64(cabi::_hypot_amd(x, y), unsafe { ref__hypot(x, y) }),
            "hypot eqexp x={bx:#018x} y={by:#018x}"
        );
    }
}

fn diff_math_sincos_modff_hypotf() {
    let mut rng = Rng(0x5c0f_0012);
    // sincos's contract is that it IS the Rust sin/cos written through the two
    // out-pointers — so its outputs must equal cabi::sin/cabi::cos exactly
    // (bit-for-bit, NaN payloads included) for every input, validating the
    // wiring + the write_volatile. (sin/cos themselves are pinned vs the
    // genuine C on the finite domain by diff_math_sincos; sin(±inf)/sin(NaN)
    // range-reduction is implementation-defined and not compared there.)
    // Finite moderate domain only (matching diff_math_sincos): sin/cos's
    // range reduction is implementation-defined for ±inf/NaN/huge and, in this
    // debug-built harness, would arithmetic-overflow-panic on them (release/
    // kernel wraps). That's a property of sin/cos, not sincos.
    let mut sweep64: Vec<f64> = vec![0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 3.0, -3.0, 100.0, -100.0];
    for _ in 0..400_000 {
        let u = rng.next();
        sweep64.push((u as f64) / (u64::MAX as f64) * 4000.0 - 2000.0);
    }
    for &x in &sweep64 {
        let mut s = 0.0f64;
        let mut c = 0.0f64;
        unsafe { cabi::sincos(x, &mut s, &mut c) };
        assert!(s.to_bits() == cabi::sin(x).to_bits(), "sincos.s {x}");
        assert!(c.to_bits() == cabi::cos(x).to_bits(), "sincos.c {x}");
    }

    // modff: exact integer/fraction split — return + *iptr both bit-exact.
    let mut b32 = f32_edge_bits();
    for _ in 0..300_000 {
        b32.push(rng.next() as u32);
    }
    for &b in &b32 {
        let x = f32::from_bits(b);
        let mut ci = 0.0f32;
        let mut ri = 0.0f32;
        let cf = unsafe { cabi::modff(x, &mut ci) };
        let rf = unsafe { ref_modff(x, &mut ri) };
        assert!(same_f32(cf, rf), "modff frac {b:#010x}");
        assert!(same_f32(ci, ri), "modff int {b:#010x}");
    }

    // hypotf: f64-internal then round to f32. edge×edge + random pairs.
    let edges = f32_edge_bits();
    for &by in &edges {
        for &bx in &edges {
            let x = f32::from_bits(bx);
            let y = f32::from_bits(by);
            assert!(
                same_f32(cabi::_hypotf_amd(x, y), unsafe { ref__hypotf(x, y) }),
                "hypotf x={bx:#010x} y={by:#010x}"
            );
        }
    }
    for _ in 0..300_000 {
        let x = f32::from_bits(rng.next() as u32);
        let y = f32::from_bits(rng.next() as u32);
        assert!(
            same_f32(cabi::_hypotf_amd(x, y), unsafe { ref__hypotf(x, y) }),
            "hypotf x={:#010x} y={:#010x}",
            x.to_bits(),
            y.to_bits()
        );
    }
}

fn diff_math_f32_trig() {
    // The f32 transcendental twins: acosf/asinf (rational + sqrt half-angle),
    // atanf (f64 core, 5-region reduction), sinhf/coshf (table + splitexp/2),
    // tanhf (Remez sub-ranges + splitexpf), tanf (pi/2 reduction incl. the
    // Payne-Hanek __remainder_piby2f path for |x|>=5e5), atan2f (atan(j/256)
    // table + quadrant dispatch). Bit-exact vs the genuine libm_sse2/*f.c.
    //
    // A full-domain random u32 bit image (every exponent → hits all branches,
    // incl. tanf's >5e5 reducer and the sinh/cosh overflow) + the edge bits,
    // plus dense sweeps over the domain-sensitive ranges.
    let mut rng = Rng(0xf327_0013);
    let mut b32 = f32_edge_bits();
    for _ in 0..1_500_000 {
        b32.push(rng.next() as u32);
    }
    for &b in &b32 {
        let x = f32::from_bits(b);
        assert!(same_f32(cabi::acosf_amd(x), unsafe { ref_acosf(x) }), "acosf {b:#010x}");
        assert!(same_f32(cabi::asinf_amd(x), unsafe { ref_asinf(x) }), "asinf {b:#010x}");
        assert!(same_f32(cabi::atanf_amd(x), unsafe { ref_atanf(x) }), "atanf {b:#010x}");
        assert!(same_f32(cabi::sinhf_amd(x), unsafe { ref_sinhf(x) }), "sinhf {b:#010x}");
        assert!(same_f32(cabi::coshf_amd(x), unsafe { ref_coshf(x) }), "coshf {b:#010x}");
        assert!(same_f32(cabi::tanhf_amd(x), unsafe { ref_tanhf(x) }), "tanhf {b:#010x}");
        assert!(same_f32(cabi::tanf_amd(x), unsafe { ref_tanf(x) }), "tanf {b:#010x}");
    }
    // asin/acos: dense over the valid domain [-1,1] + a little past.
    for _ in 0..400_000 {
        let u = rng.next();
        let x = ((u as f64) / (u64::MAX as f64) * 2.2 - 1.1) as f32;
        assert!(same_f32(cabi::asinf_amd(x), unsafe { ref_asinf(x) }), "asinf {x}");
        assert!(same_f32(cabi::acosf_amd(x), unsafe { ref_acosf(x) }), "acosf {x}");
    }
    // tanf: dense near pi/2 multiples (small + large), and big magnitudes that
    // drive the __remainder_piby2f Payne-Hanek path (|x| >= 5e5).
    for _ in 0..400_000 {
        let u = rng.next();
        let x = ((u as f64) / (u64::MAX as f64) * 200.0 - 100.0) as f32;
        assert!(same_f32(cabi::tanf_amd(x), unsafe { ref_tanf(x) }), "tanf small {x}");
        let big = ((rng.next() as f64) / (u64::MAX as f64) * 2.0e8 - 1.0e8) as f32;
        assert!(same_f32(cabi::tanf_amd(big), unsafe { ref_tanf(big) }), "tanf big {big}");
    }
    // sinh/cosh/tanh/atan: dense over their active ranges.
    for _ in 0..400_000 {
        let u = rng.next();
        let x = ((u as f64) / (u64::MAX as f64) * 200.0 - 100.0) as f32;
        assert!(same_f32(cabi::sinhf_amd(x), unsafe { ref_sinhf(x) }), "sinhf {x}");
        assert!(same_f32(cabi::coshf_amd(x), unsafe { ref_coshf(x) }), "coshf {x}");
        assert!(same_f32(cabi::tanhf_amd(x), unsafe { ref_tanhf(x) }), "tanhf {x}");
        assert!(same_f32(cabi::atanf_amd(x), unsafe { ref_atanf(x) }), "atanf {x}");
    }
    // atan2f: all edge×edge pairs + random pairs (every sign/zero/inf quadrant).
    let edges = f32_edge_bits();
    for &by in &edges {
        for &bx in &edges {
            let y = f32::from_bits(by);
            let x = f32::from_bits(bx);
            assert!(
                same_f32(cabi::atan2f_amd(y, x), unsafe { ref_atan2f(y, x) }),
                "atan2f y={by:#010x} x={bx:#010x}"
            );
        }
    }
    for _ in 0..600_000 {
        let y = f32::from_bits(rng.next() as u32);
        let x = f32::from_bits(rng.next() as u32);
        assert!(
            same_f32(cabi::atan2f_amd(y, x), unsafe { ref_atan2f(y, x) }),
            "atan2f y={:#010x} x={:#010x}",
            y.to_bits(),
            x.to_bits()
        );
    }
}

/// strtod / atof — genuine DJ Delorie `strtod.c` (build.rs ref,
/// -mlong-double-64 → f64 semantics) vs the cnt-rust f64 port. Fuzzes
/// float-shaped strings (sign, int/frac digits, exponent, the e-no-digit
/// and no-digit quirks) over the same NUL-terminated buffer; compares
/// value bit-exact (same_f64) AND the endptr offset, plus the NULL-sret
/// and atof paths. Pins the documented "f64 on every arch" contract.
fn diff_strtod() {
    let mut rng = Rng(0x57c0_d00d_u64);
    // Structured float strings. The exponent is bounded to <=3 digits:
    // the genuine C accumulates the exponent into a signed `int` and a
    // >10-digit exponent overflows it (signed-overflow UB — gcc -O2 then
    // diverges from any well-defined wrap), so multi-billion exponents
    // are OUTSIDE strtod's defined domain (the kernel never feeds them;
    // same documented-limit precedent as fmod-subnormal / scanf-extreme-
    // exponent). Everything else — sign, int/frac digit runs, the no-digit
    // and e-without-digit quirks, leading ws, trailing junk — is fuzzed.
    let dig = |rng: &mut Rng, v: &mut Vec<u8>, max: usize| {
        for _ in 0..rng.upto(max + 1) {
            v.push(b"0123456789"[rng.upto(10)]);
        }
    };
    for _ in 0..40000 {
        let mut s: Vec<u8> = Vec::new();
        // leading whitespace
        for _ in 0..rng.upto(3) {
            s.push(if rng.next() & 1 == 0 { b' ' } else { b'\t' });
        }
        match rng.upto(3) {
            0 => s.push(b'+'),
            1 => s.push(b'-'),
            _ => {}
        }
        dig(&mut rng, &mut s, 16); // integer digits (may be 0)
        if rng.next() & 1 == 0 {
            s.push(b'.');
            dig(&mut rng, &mut s, 16); // fraction digits (may be 0)
        }
        if rng.upto(3) != 0 {
            s.push(if rng.next() & 1 == 0 { b'e' } else { b'E' });
            match rng.upto(3) {
                0 => s.push(b'+'),
                1 => s.push(b'-'),
                _ => {}
            }
            dig(&mut rng, &mut s, 3); // <=3 exponent digits (may be 0 → quirk)
        }
        // occasional trailing junk to exercise the endptr stop point
        if rng.next() & 3 == 0 {
            s.push(b"xyz.+- "[rng.upto(7)]);
        }
        s.push(0);
        let off = |p: *mut u8| {
            if p == 1 as *mut u8 {
                None
            } else {
                Some(p as usize - s.as_ptr() as usize)
            }
        };
        let (mut ec, mut er): (*mut u8, *mut u8) = (1 as *mut u8, 1 as *mut u8);
        let cv = unsafe { cabi::strtod(s.as_ptr(), &mut ec) };
        let rv = unsafe { ref_strtod(s.as_ptr(), &mut er) };
        assert!(same_f64(cv, rv), "strtod v s={s:?} cnt={cv} ref={rv}");
        assert_eq!(off(ec), off(er), "strtod endptr s={s:?}");
        // NULL-sret path (value unchanged, no write).
        let cv2 = unsafe { cabi::strtod(s.as_ptr(), core::ptr::null_mut()) };
        assert!(same_f64(cv2, rv), "strtod NULL-sret s={s:?}");
        // atof == strtod(s, 0).
        let av = unsafe { cabi::atof(s.as_ptr()) };
        assert!(same_f64(av, rv), "atof s={s:?} cnt={av} ref={rv}");
    }
}

/// Wave WCS — the 8 kernel wide leaves from wcs.c vs the transcribed
/// genuine refs (build.rs). Runs cnt and ref on SEPARATE copies of the
/// same buffer and compares both the buffer contents and the
/// return value / returned-pointer offset.
fn diff_wcs() {
    let mut rng = Rng(0x5c5_4ee7_d00d_u64);
    // wide string from a no-NUL alphabet (collisions for pbrk/coll).
    let wstr = |rng: &mut Rng, max: usize| -> Vec<u16> {
        let a = b"abABcC xy.+-12";
        let n = rng.upto(max + 1);
        let mut v: Vec<u16> = (0..n).map(|_| a[rng.upto(a.len())] as u16).collect();
        v.push(0);
        v
    };
    let off = |p: *mut u16, base: *const u16| {
        if p.is_null() {
            None
        } else {
            Some((p as usize - base as usize) / 2)
        }
    };
    for _ in 0..30000 {
        // _wcsnset
        {
            let s = wstr(&mut rng, 12);
            let (mut a, mut b) = (s.clone(), s.clone());
            let c = b"AZ*0 "[rng.upto(5)] as u16;
            let n = rng.upto(14);
            let ra = unsafe { cabi::_wcsnset(a.as_mut_ptr(), c, n) };
            let rb = unsafe { ref__wcsnset(b.as_mut_ptr(), c, n) };
            assert_eq!(off(ra, a.as_ptr()), off(rb, b.as_ptr()), "_wcsnset ret");
            assert_eq!(a, b, "_wcsnset buf s={s:?} c={c} n={n}");
        }
        // _wcsrev
        {
            let s = wstr(&mut rng, 14);
            let (mut a, mut b) = (s.clone(), s.clone());
            unsafe {
                cabi::_wcsrev(a.as_mut_ptr());
                ref__wcsrev(b.as_mut_ptr());
            }
            assert_eq!(a, b, "_wcsrev s={s:?}");
        }
        // wcscoll / wcspbrk
        {
            let x = wstr(&mut rng, 10);
            let y = wstr(&mut rng, 10);
            let cc = unsafe { cabi::wcscoll(x.as_ptr(), y.as_ptr()) };
            let rc = unsafe { ref_wcscoll(x.as_ptr(), y.as_ptr()) };
            assert_eq!(cc.signum(), rc.signum(), "wcscoll x={x:?} y={y:?}");
            let acc = wstr(&mut rng, 5);
            let cp = unsafe { cabi::wcspbrk(x.as_ptr(), acc.as_ptr()) };
            let rp = unsafe { ref_wcspbrk(x.as_ptr(), acc.as_ptr()) };
            assert_eq!(off(cp, x.as_ptr()), off(rp, x.as_ptr()), "wcspbrk x={x:?} acc={acc:?}");
        }
        // wcscpy_s / wcsncpy_s
        {
            let src = wstr(&mut rng, 12);
            let num = rng.upto(16);
            let cnt = rng.upto(16);
            let (mut a, mut b) = (vec![0xAAu16; 18], vec![0xAAu16; 18]);
            let rca = unsafe { cabi::wcscpy_s(a.as_mut_ptr(), num, src.as_ptr()) };
            let rcb = unsafe { ref_wcscpy_s(b.as_mut_ptr(), num, src.as_ptr()) };
            assert_eq!(rca, rcb, "wcscpy_s ret src={src:?} num={num}");
            assert_eq!(a, b, "wcscpy_s buf src={src:?} num={num}");
            let (mut a, mut b) = (vec![0xAAu16; 18], vec![0xAAu16; 18]);
            let rna = unsafe { cabi::wcsncpy_s(a.as_mut_ptr(), num, src.as_ptr(), cnt) };
            let rnb = unsafe { ref_wcsncpy_s(b.as_mut_ptr(), num, src.as_ptr(), cnt) };
            assert_eq!(rna, rnb, "wcsncpy_s ret src={src:?} num={num} cnt={cnt}");
            assert_eq!(a, b, "wcsncpy_s buf");
        }
        // wcscat_s / wcsncat_s — dst prefilled with a NUL-terminated prefix
        {
            let elem = 4 + rng.upto(12);
            let src = wstr(&mut rng, 10);
            let cnt = if rng.next() & 7 == 0 { usize::MAX } else { rng.upto(14) };
            // dst prefilled with a random NUL-terminated prefix (plen may
            // == elem → the not-NUL-terminated branch). Build once, clone
            // the Vec so cnt and ref get byte-identical input.
            let mut dst0 = vec![0xAAu16; elem + 6];
            let plen = rng.upto(elem);
            for i in 0..plen {
                dst0[i] = b"pq"[i % 2] as u16;
            }
            if plen < dst0.len() {
                dst0[plen] = 0;
            }
            let (mut a, mut b) = (dst0.clone(), dst0.clone());
            let rca = unsafe { cabi::wcscat_s(a.as_mut_ptr(), elem, src.as_ptr()) };
            let rcb = unsafe { ref_wcscat_s(b.as_mut_ptr(), elem, src.as_ptr()) };
            assert_eq!(rca, rcb, "wcscat_s ret elem={elem} src={src:?}");
            assert_eq!(a, b, "wcscat_s buf elem={elem} src={src:?}");
            let (mut a, mut b) = (dst0.clone(), dst0.clone());
            let rna = unsafe { cabi::wcsncat_s(a.as_mut_ptr(), elem, src.as_ptr(), cnt) };
            let rnb = unsafe { ref_wcsncat_s(b.as_mut_ptr(), elem, src.as_ptr(), cnt) };
            assert_eq!(rna, rnb, "wcsncat_s ret elem={elem} src={src:?} cnt={cnt}");
            assert_eq!(a, b, "wcsncat_s buf elem={elem} src={src:?} cnt={cnt}");
        }
    }
}

// ---------------------------------------------------------------------------
// Custom single-threaded runner (replaces libtest — see Cargo.toml `[[test]]
// harness = false`). cnt-rust/cnt-ctype `#[no_mangle]` shims override host
// glibc once linked, which breaks libtest (its `getenv` for RUST_TEST_THREADS
// hits cnt's empty env → NULL → forced multithreading → a race on cnt's
// process-global setenv state → spurious abort). This runner never calls
// getenv and never spawns a thread, so all of that evaporates. Every check
// here is deterministic and order-independent; sequential execution is both
// correct and reproducible. Each test prints its name (flushed) BEFORE it
// runs, so even an abort under `panic = "abort"` names the culprit, and the
// panic message itself carries the failing assertion's file:line.
fn main() {
    macro_rules! run {
        ($($t:ident),* $(,)?) => {{
            use std::io::Write;
            let args: Vec<String> = std::env::args().skip(1).collect();
            let selected = if args.is_empty() {
                &args[..]
            } else {
                assert_eq!(args[0], "--exact", "only exact case selection is supported");
                assert!(args.len() > 1, "--exact requires a case name");
                &args[1..]
            };
            let known = [$(stringify!($t)),*];
            for name in selected {
                assert!(known.contains(&name.as_str()), "unknown differential case: {name}");
            }
            let mut n = 0usize;
            $(
                if selected.is_empty() || selected.iter().any(|name| name == stringify!($t)) {
                print!("test {} ... ", stringify!($t));
                std::io::stdout().flush().ok();
                $t();
                println!("ok");
                n += 1;
                }
            )*
            n
        }};
    }
    let n = run!(
        diff_strlen,
        diff_strcmp,
        diff_strstr,
        diff_atoi,
        diff_strncpy,
        diff_memcpy,
        diff_memset,
        diff_memcmp,
        diff_memmove,
        diff_memchr,
        diff_atoi64_atol,
        diff_strnlen,
        diff_strncmp,
        diff_strchr_rchr,
        diff_strtoi64_strtoul,
        diff_itoa_unchecked,
        diff_itow_unchecked,
        diff_itoa_s,
        diff_itow_s,
        diff_icmp,
        diff_wcstoul,
        diff_mbcp,
        diff_tcs_family,
        diff_strtol_family,
        diff_strtod,
        diff_wcs,
        diff_span,
        diff_wide_atoi,
        diff_casemap,
        diff_strmut,
        diff_path,
        diff_strtok,
        diff_scanf,
        diff_snscanf,
        diff_scanf_float,
        diff_env,
        diff_fcvt,
        diff_ecvt,
        diff_gcvt,
        diff_memccpy,
        diff_memicmp,
        diff_rand,
        diff_qsort,
        diff_bsearch,
        diff_lfind,
        diff_ctype,
        diff_streamout_integers,
        diff_streamout_chars_strings_percent,
        diff_streamout_float,
        diff_math_abs,
        diff_math_class_f64,
        diff_math_class_f32,
        diff_math_lastmile,
        diff_math_bessel,
        diff_math_div,
        diff_math_chgsignf,
        diff_math_scalbn,
        diff_math_scalbnf,
        diff_math_round,
        diff_math_roundf,
        diff_math_nextafter,
        diff_math_nextafterf,
        diff_math_libm_exports,
        diff_math_sincos,
        diff_math_sqrt,
        diff_math_nexttowardf,
        diff_math_exp,
        diff_math_log,
        diff_math_log10,
        diff_math_pow,
        diff_math_fmod,
        diff_math_tan,
        diff_math_ceil_floor,
        diff_math_ldexp_logb,
        diff_math_fabs,
        diff_math_modf,
        diff_math_atan,
        diff_math_exp2,
        diff_math_tanh,
        diff_math_sinh_cosh,
        diff_math_asin_acos,
        diff_math_hypot,
        diff_math_sincos_modff_hypotf,
        diff_math_f32_trig,
    );
    println!("\ndifftest result: ok. {n} passed; 0 failed");
}
