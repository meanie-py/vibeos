//! Live differential tests: every staged body vs the REAL Windows ntdll export of the same
//! name, on identical inputs, comparing every caller-visible byte. Run on Windows (see
//! INTEGRATION.md "Verification"). Fixed grids cover the edges; a seeded xorshift fuzzes.

use core::ffi::c_void;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, IO_RESOURCE_DESCRIPTOR, _UNICODE_STRING};

extern "system" {
    fn GetModuleHandleA(name: *const u8) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    fn GetCurrentThread() -> *mut c_void;
    fn SetThreadAffinityMask(thread: *mut c_void, mask: usize) -> usize;
    fn Sleep(ms: u32);
}

fn real(name: &str) -> *mut c_void {
    let mut n = name.as_bytes().to_vec();
    n.push(0);
    let p = unsafe { GetProcAddress(GetModuleHandleA(b"ntdll.dll\0".as_ptr()), n.as_ptr()) };
    assert!(!p.is_null(), "Windows ntdll does not export {name}");
    p
}

macro_rules! win {
    ($name:literal as $t:ty) => {
        unsafe { core::mem::transmute::<*mut c_void, $t>(real($name)) }
    };
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn ustr(v: &mut [u16]) -> _UNICODE_STRING {
    _UNICODE_STRING {
        Length: (v.len() * 2) as u16,
        MaximumLength: (v.len() * 2) as u16,
        Buffer: if v.is_empty() { core::ptr::NonNull::<u16>::dangling().as_ptr() } else { v.as_mut_ptr() },
    }
}

// name.rs allocates through the rtl glue; on the host harness that is the C heap.
#[no_mangle]
unsafe extern "system" fn RtlpAllocateMemory(bytes: u32, _tag: u32) -> *mut c_void {
    extern "C" {
        fn malloc(n: usize) -> *mut c_void;
    }
    malloc(bytes as usize)
}
#[no_mangle]
unsafe extern "system" fn RtlpFreeMemory(p: *mut c_void, _tag: u32) {
    extern "C" {
        fn free(p: *mut c_void);
    }
    free(p)
}

#[test]
fn name_long_expression_uses_heap() {
    // 2000-character expressions take the heap path; compare with Windows.
    let w = win!("RtlIsNameInExpression" as IsNameFn);
    let mut e: Vec<u16> = std::iter::repeat(b'*' as u16).take(1999).collect();
    e.push(b'z' as u16);
    for n in ["z", "az", "a", "zzzz.z"] {
        let (a, b) = name_case(w, &e, &wide(n), false, core::ptr::null_mut());
        assert_eq!(a != 0, b != 0, "long expr vs {n:?}");
    }
    let e: Vec<u16> = std::iter::repeat(b'?' as u16).take(1500).collect();
    let n: Vec<u16> = std::iter::repeat(b'x' as u16).take(1500).collect();
    let (a, b) = name_case(w, &e, &n, false, core::ptr::null_mut());
    assert_eq!((a != 0, b != 0), (true, true));
    let (a, b) = name_case(w, &e, &n[..1499], false, core::ptr::null_mut());
    assert_eq!((a != 0, b != 0), (false, false));
}

#[test]
fn helpers_are_ours() {
    // locale.rs / name.rs must exercise the TREE's helpers, not Windows' (rtl-strings is
    // linked ahead of the ntdll import library).
    let ours = rtl_strings::cabi::RtlIntegerToUnicodeString as *const c_void;
    assert_ne!(ours, real("RtlIntegerToUnicodeString") as *const c_void);
    let ours = rtl_strings::cabi::RtlUpcaseUnicodeChar as *const c_void;
    assert_ne!(ours, real("RtlUpcaseUnicodeChar") as *const c_void);
}

// ---- hashes --------------------------------------------------------------------------------

type CtxFn = unsafe extern "system" fn(*mut u8);
type UpdFn = unsafe extern "system" fn(*mut u8, *const u8, u32);
type ShaFinFn = unsafe extern "system" fn(*mut u8, *mut u32);

struct Hash {
    name: &'static str,
    init: (CtxFn, CtxFn),
    update: (UpdFn, UpdFn),
    fin: Option<(CtxFn, CtxFn)>,
    sha_fin: Option<(ShaFinFn, ShaFinFn)>,
}

fn hashes() -> Vec<Hash> {
    use crate::crypt::*;
    unsafe {
        vec![
            Hash {
                name: "MD5",
                init: (core::mem::transmute(MD5Init as unsafe extern "system" fn(_)), win!("MD5Init" as CtxFn)),
                update: (core::mem::transmute(MD5Update as unsafe extern "system" fn(_, _, _)), win!("MD5Update" as UpdFn)),
                fin: Some((core::mem::transmute(MD5Final as unsafe extern "system" fn(_)), win!("MD5Final" as CtxFn))),
                sha_fin: None,
            },
            Hash {
                name: "MD4",
                init: (core::mem::transmute(MD4Init as unsafe extern "system" fn(_)), win!("MD4Init" as CtxFn)),
                update: (core::mem::transmute(MD4Update as unsafe extern "system" fn(_, _, _)), win!("MD4Update" as UpdFn)),
                fin: Some((core::mem::transmute(MD4Final as unsafe extern "system" fn(_)), win!("MD4Final" as CtxFn))),
                sha_fin: None,
            },
            Hash {
                name: "SHA",
                init: (core::mem::transmute(A_SHAInit as unsafe extern "system" fn(_)), win!("A_SHAInit" as CtxFn)),
                update: (core::mem::transmute(A_SHAUpdate as unsafe extern "system" fn(_, _, _)), win!("A_SHAUpdate" as UpdFn)),
                fin: None,
                sha_fin: Some((core::mem::transmute(A_SHAFinal as unsafe extern "system" fn(_, _)), win!("A_SHAFinal" as ShaFinFn))),
            },
        ]
    }
}

/// Run one message (split at `cuts`) through both implementations, comparing the whole
/// context (plus untouched slack) after every call.
fn hash_case(h: &Hash, msg: &[u8], cuts: &[usize], what: &str) {
    let mut a = [0xCCu8; 160];
    let mut b = [0xCCu8; 160];
    unsafe {
        (h.init.0)(a.as_mut_ptr());
        (h.init.1)(b.as_mut_ptr());
        assert_eq!(a, b, "{} init {what}", h.name);
        let mut start = 0;
        for &end in cuts.iter().chain(core::iter::once(&msg.len())) {
            (h.update.0)(a.as_mut_ptr(), msg[start..end].as_ptr(), (end - start) as u32);
            (h.update.1)(b.as_mut_ptr(), msg[start..end].as_ptr(), (end - start) as u32);
            assert_eq!(a, b, "{} update {what} [{start}..{end}]", h.name);
            start = end;
        }
        if let Some((f0, f1)) = h.fin {
            f0(a.as_mut_ptr());
            f1(b.as_mut_ptr());
        }
        if let Some((f0, f1)) = h.sha_fin {
            let mut ra = [0xCCCC_CCCCu32; 8];
            let mut rb = [0xCCCC_CCCCu32; 8];
            f0(a.as_mut_ptr(), ra.as_mut_ptr());
            f1(b.as_mut_ptr(), rb.as_mut_ptr());
            assert_eq!(ra, rb, "{} result {what}", h.name);
        }
        assert_eq!(a, b, "{} final {what}", h.name);
    }
}

#[test]
fn hash_known_vectors() {
    // RFC 1321 / 1320 / FIPS 180 vectors, through OUR bodies alone.
    use crate::crypt::*;
    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
    let alpha = b"abcdefghijklmnopqrstuvwxyz";
    let digits = b"12345678901234567890123456789012345678901234567890123456789012345678901234567890";
    unsafe {
        let mut c: MD5_CTX = core::mem::zeroed();
        for (m, want) in [
            (&b""[..], "d41d8cd98f00b204e9800998ecf8427e"),
            (b"abc", "900150983cd24fb0d6963f7d28e17f72"),
            (alpha, "c3fcd3d76192e4007dfb496cca67e13b"),
            (digits, "57edf4a22be3c955ac49da2e2107b67a"),
        ] {
            MD5Init(&mut c);
            MD5Update(&mut c, m.as_ptr(), m.len() as u32);
            MD5Final(&mut c);
            assert_eq!(hex(&c.digest), want);
        }
        let mut c: MD4_CTX = core::mem::zeroed();
        for (m, want) in [
            (&b""[..], "31d6cfe0d16ae931b73c59d7e0c089c0"),
            (b"abc", "a448017aaf21d8525fc10ae87aa6729d"),
            (alpha, "d79e1c308aa5bbcdeea8ed63df412da9"),
            (digits, "e33b4ddc9c38f2199c3e7b164fcc0536"),
        ] {
            MD4Init(&mut c);
            MD4Update(&mut c, m.as_ptr(), m.len() as u32);
            MD4Final(&mut c);
            assert_eq!(hex(&c.digest), want);
        }
        let mut c: A_SHA_CTX = core::mem::zeroed();
        let mut r = [0u32; 5];
        let million_a = vec![b'a'; 1_000_000];
        for (m, want) in [
            (&b""[..], "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
            (b"abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
            (b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", "84983e441c3bd26ebaae4aa1f95129e5e54670f1"),
            (&million_a[..], "34aa973cd4c4daa4f61eeb2bdbad27316534016f"),
        ] {
            A_SHAInit(&mut c);
            A_SHAUpdate(&mut c, m.as_ptr(), m.len() as u32);
            A_SHAFinal(&mut c, r.as_mut_ptr());
            let bytes: Vec<u8> = r.iter().flat_map(|w| w.to_ne_bytes()).collect();
            assert_eq!(hex(&bytes), want);
        }
    }
}

#[test]
fn hash_vs_windows() {
    let hs = hashes();
    let msg: Vec<u8> = (0..4096u32).map(|i| (i.wrapping_mul(7).wrapping_add(3)) as u8).collect();
    for h in &hs {
        // Every length 0..=200 in one call, and the block-boundary lengths split everywhere.
        for n in 0..=200 {
            hash_case(h, &msg[..n], &[], &format!("len {n}"));
        }
        for n in [55usize, 56, 57, 63, 64, 65, 119, 120, 127, 128, 129] {
            for cut in 0..=n {
                hash_case(h, &msg[..n], &[cut], &format!("len {n} cut {cut}"));
            }
        }
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for case in 0..3000 {
            let n = rng.below(1500) as usize;
            let mut cuts: Vec<usize> = (0..rng.below(6)).map(|_| rng.below(n as u64 + 1) as usize).collect();
            cuts.sort();
            hash_case(h, &msg[..n], &cuts, &format!("fuzz {case} len {n} cuts {cuts:?}"));
        }
        // A context that was never Init'ed but carries an arbitrary partial count: Update and
        // Final must agree on whatever the counters say.
        for seed in 1..200u64 {
            let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D));
            let mut a = [0u8; 160];
            for x in a.iter_mut() {
                *x = rng.next() as u8;
            }
            let mut b = a;
            let n = rng.below(300) as usize;
            if seed % 4 == 0 {
                unsafe {
                    let (mut ga, mut gb) = (a, a);
                    if let Some((f0, f1)) = h.fin {
                        f0(ga.as_mut_ptr());
                        f1(gb.as_mut_ptr());
                    }
                    if let Some((f0, f1)) = h.sha_fin {
                        let (mut ra, mut rb) = ([0u32; 5], [0u32; 5]);
                        f0(ga.as_mut_ptr(), ra.as_mut_ptr());
                        f1(gb.as_mut_ptr(), rb.as_mut_ptr());
                        assert_eq!(ra, rb, "{} garbage-ctx bare final result seed {seed}", h.name);
                    }
                    assert_eq!(ga, gb, "{} garbage-ctx bare final seed {seed}", h.name);
                }
            }
            unsafe {
                (h.update.0)(a.as_mut_ptr(), msg.as_ptr(), n as u32);
                (h.update.1)(b.as_mut_ptr(), msg.as_ptr(), n as u32);
                assert_eq!(a, b, "{} garbage-ctx update seed {seed}", h.name);
                if let Some((f0, f1)) = h.fin {
                    f0(a.as_mut_ptr());
                    f1(b.as_mut_ptr());
                }
                if let Some((f0, f1)) = h.sha_fin {
                    let (mut ra, mut rb) = ([0u32; 5], [0u32; 5]);
                    f0(a.as_mut_ptr(), ra.as_mut_ptr());
                    f1(b.as_mut_ptr(), rb.as_mut_ptr());
                    assert_eq!(ra, rb);
                }
                assert_eq!(a, b, "{} garbage-ctx final seed {seed}", h.name);
            }
        }
    }
}

// ---- memory/port resource descriptors ------------------------------------------------------

const LENS: &[u64] = &[
    0, 1, 0xFF, 0x100, 0x1000, 0xFFFF_FFFF, 0x1_0000_0000, 0x1_0000_0080, 0x1_0000_0100,
    0xFF_FFFF_FF00, 0xFF_FFFF_FF01, 0x100_0000_0000, 0x100_0000_0100, 0x100_0001_0000,
    0xFFFF_FFFF_0000, 0xFFFF_FFFF_0001, 0x1_0000_0000_0000, 0x1_0001_0000_0000,
    0x100_0000_0000_0100, 0xFFFF_FFFF_0000_0000, 0xFFFF_FFFF_0000_0001, u64::MAX,
];
const ALIGNS: &[u64] = &[
    0, 1, 2, 0x81, 0x100, 0x1000, 0x10000, 0xFFFF_FFFF, 0x1_0000_0000, 0x100_0000_0000,
    0x1_0000_0000_0000, 0x8000_0000_0000_0001, u64::MAX,
];
const TYPES: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 0x80, 0xFF];
const FLAGS: &[u16] = &[0, 0x5555, 0x0E01, 0x0200, 0x0400, 0x0800, 0x0600, 0xFFFF];

fn rand_len(rng: &mut Rng) -> u64 {
    // Mostly values near the tier edges: a random width, then random low bits cleared.
    let w = rng.below(65) as u32;
    let v = if w == 64 { rng.next() } else { rng.next() & ((1u64 << w).wrapping_sub(1)) };
    match rng.below(4) {
        0 => v & !0xFF,
        1 => v & !0xFFFF,
        2 => v & !0xFFFF_FFFF,
        _ => v,
    }
}

#[test]
fn memres_vs_windows() {
    type CmEnc = unsafe extern "system" fn(*mut u8, u8, u64, u64) -> i32;
    type CmDec = unsafe extern "system" fn(*const u8, *mut u64) -> u64;
    type IoEnc = unsafe extern "system" fn(*mut u8, u8, u64, u64, u64, u64) -> i32;
    type IoDec = unsafe extern "system" fn(*const u8, *mut u64, *mut u64, *mut u64) -> u64;
    type Closest = unsafe extern "system" fn(u64, *mut u64) -> i32;
    let wcm_enc = win!("RtlCmEncodeMemIoResource" as CmEnc);
    let wcm_dec = win!("RtlCmDecodeMemIoResource" as CmDec);
    let wio_enc = win!("RtlIoEncodeMemIoResource" as IoEnc);
    let wio_dec = win!("RtlIoDecodeMemIoResource" as IoDec);
    let wclosest = win!("RtlFindClosestEncodableLength" as Closest);
    use crate::memres::*;
    let bad = std::cell::RefCell::new(Vec::<String>::new());
    macro_rules! check {
        ($l:expr, $r:expr, $($fmt:tt)*) => {
            if $l != $r {
                bad.borrow_mut().push(format!("{}: ours {:?} windows {:?}", format!($($fmt)*), $l, $r));
            }
        };
    }
    assert_eq!(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(), if cfg!(target_pointer_width = "64") { 20 } else { 16 });
    assert_eq!(core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>(), 32);

    #[derive(Clone, Copy, PartialEq, Debug)]
    #[repr(C, align(8))]
    struct A40([u8; 40]);
    #[derive(Clone, Copy, PartialEq, Debug)]
    #[repr(C, align(8))]
    struct A48([u8; 48]);
    let cm = |t: u8, f: u16, len: u64, start: u64| unsafe {
        let mut a = A40([0x55u8; 40]);
        let a = &mut a.0;
        a[2..4].copy_from_slice(&f.to_le_bytes());
        let mut bb = A40(*a);
        let b = &mut bb.0;
        let sa = RtlCmEncodeMemIoResource(a.as_mut_ptr() as *mut _, t, len, start);
        let sb = wcm_enc(b.as_mut_ptr(), t, len, start);
        check!((sa, &a[..]), (sb, &b[..]), "CmEncode t={t} f={f:#x} len={len:#x}");
        let (mut xa, mut xb) = (0xEEu64, 0xEEu64);
        let ra = RtlCmDecodeMemIoResource(a.as_ptr() as *const _, &mut xa);
        let rb = wcm_dec(b.as_ptr(), &mut xb);
        check!((ra, xa), (rb, xb), "CmDecode t={t} f={f:#x} len={len:#x}");
    };
    let io = |t: u8, f: u16, len: u64, al: u64, mn: u64, mx: u64| unsafe {
        let mut aa0 = A48([0x55u8; 48]);
        let a = &mut aa0.0;
        a[4..6].copy_from_slice(&f.to_le_bytes());
        let mut bb = A48(*a);
        let b = &mut bb.0;
        let sa = RtlIoEncodeMemIoResource(a.as_mut_ptr() as *mut _, t, len, al, mn, mx);
        let sb = wio_enc(b.as_mut_ptr(), t, len, al, mn, mx);
        check!((sa, &a[..]), (sb, &b[..]), "IoEncode t={t} f={f:#x} len={len:#x} al={al:#x}");
        let (mut aa, mut ab) = ([1u64, 2, 3], [1u64, 2, 3]);
        let ra = RtlIoDecodeMemIoResource(a.as_ptr() as *const _, &mut aa[0], &mut aa[1], &mut aa[2]);
        let rb = wio_dec(b.as_ptr(), &mut ab[0], &mut ab[1], &mut ab[2]);
        check!((ra, aa), (rb, ab), "IoDecode t={t} f={f:#x} len={len:#x} al={al:#x}");
    };
    for &t in TYPES {
        for &f in FLAGS {
            for &len in LENS {
                cm(t, f, len, 0x1234_5678_9ABC);
                for &al in ALIGNS {
                    io(t, f, len, al, 0x1000, 0xFF_FFFF_FFFF);
                }
            }
        }
    }
    // Hand-built descriptors: every type x every flag word pattern of the large bits.
    for &t in TYPES {
        for nib in 0..16u16 {
            for extra in [0u16, 0x00FF, 0xF000] {
                let f = (nib << 8) | extra;
                for raw_len in [0u32, 1, 0x0100_0000, 0xFFFF_FFFF] {
                    unsafe {
                        let mut dd = A40([0u8; 40]);
                        let d = &mut dd.0;
                        d[0] = t;
                        d[2..4].copy_from_slice(&f.to_le_bytes());
                        d[4..12].copy_from_slice(&0xABCDu64.to_le_bytes());
                        d[12..16].copy_from_slice(&raw_len.to_le_bytes());
                        let (mut xa, mut xb) = (0, 0);
                        check!(
                            (RtlCmDecodeMemIoResource(d.as_ptr() as *const _, &mut xa), xa),
                            (wcm_dec(d.as_ptr(), &mut xb), xb),
                            "raw CmDecode t={t} f={f:#x} len={raw_len:#x}"
                        );
                        let mut ee = A48([0u8; 48]);
                        let e = &mut ee.0;
                        e[1] = t;
                        e[4..6].copy_from_slice(&f.to_le_bytes());
                        e[8..12].copy_from_slice(&raw_len.to_le_bytes());
                        e[12..16].copy_from_slice(&raw_len.rotate_left(8).to_le_bytes());
                        let (mut aa, mut ab) = ([0u64; 3], [0u64; 3]);
                        check!(
                            (RtlIoDecodeMemIoResource(e.as_ptr() as *const _, &mut aa[0], &mut aa[1], &mut aa[2]), aa),
                            (wio_dec(e.as_ptr(), &mut ab[0], &mut ab[1], &mut ab[2]), ab),
                            "raw IoDecode t={t} f={f:#x} len={raw_len:#x}"
                        );
                    }
                }
            }
        }
    }
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..200_000 {
        let t = TYPES[rng.below(TYPES.len() as u64) as usize];
        let f = if rng.below(2) == 0 { FLAGS[rng.below(FLAGS.len() as u64) as usize] } else { rng.next() as u16 };
        let len = rand_len(&mut rng);
        let al = rand_len(&mut rng);
        cm(t, f, len, rng.next());
        io(t, f, len, al, rng.next(), rng.next());
        let (mut ta, mut tb) = (7u64, 7u64);
        let sa = unsafe { RtlFindClosestEncodableLength(len, &mut ta) };
        let sb = unsafe { wclosest(len, &mut tb) };
        check!((sa, ta), (sb, tb), "Closest {len:#x}");
    }
    for &len in LENS {
        let (mut ta, mut tb) = (7u64, 7u64);
        let sa = unsafe { RtlFindClosestEncodableLength(len, &mut ta) };
        let sb = unsafe { wclosest(len, &mut tb) };
        check!((sa, ta), (sb, tb), "Closest {len:#x}");
    }
    let bad = bad.into_inner();
    for m in bad.iter().take(40) {
        eprintln!("{m}");
    }
    eprintln!("memres: {} mismatches", bad.len());
    assert!(bad.is_empty());
}

// ---- RtlIsNameInExpression ------------------------------------------------------------------

type IsNameFn = unsafe extern "system" fn(*mut _UNICODE_STRING, *mut _UNICODE_STRING, u8, *mut u16) -> u8;

fn name_case(w: IsNameFn, e: &[u16], n: &[u16], ic: bool, table: *mut u16) -> (u8, u8) {
    let (mut ev, mut nv) = (e.to_vec(), n.to_vec());
    let (mut es, mut ns) = (ustr(&mut ev), ustr(&mut nv));
    let ours = unsafe { crate::name::RtlIsNameInExpression(&mut es, &mut ns, ic as u8, table) };
    let theirs = unsafe { w(&mut es, &mut ns, ic as u8, table) };
    (ours, theirs)
}

#[test]
fn is_name_in_expression_vs_windows() {
    let w = win!("RtlIsNameInExpression" as IsNameFn);
    let exprs = [
        "", "*", "**", "*.*", "?", "??", "*?", "?*?", "he*o", "*.cpl", "F0_*.*", "F0_*.", "F0_<\"*",
        "<.exe", "<.EXE", "ab<exe", "a>c.exe", "a>>>exe", "*.c.d", "ntoskrnl\"exe", "ntoskrnl.exe\"",
        "\"x", "FI<<<<<<<<", "<<<<<<<<<.<", "F<", "*a*ab*abc", "A*C", "*Z", "<", ">", "\"", "*.", ".*",
        "<.<", "x<", "x>", ">>>.>>>", "*\"*", "ABC", "abc", "\u{C9}", "*\u{E9}", "a\u{0}", "*\u{0}",
        "<\"", "\">", "*.<", "<*", "?<?", "a<b<c", "*.?.c.d", "a.b.*??.d",
    ];
    let names = [
        "", "a", "ab", ".", "..", "abc", "ABC", "hello", "helo", "x.cpl", "F0_", "F0_.", "f0_001.txt",
        "F0_001.txt", "F0_001", "test.exe", "TEST.EXE", "abcd.exe", "abcdexe", "abc.exe", "ac.exe",
        "a.b.c.d", "ntoskrnl.exe", "FILE", "FILE.TXT", "aabaabcdadabdabc", "A<>\"C", "**Z", "x", "x.",
        "x.y", "x.y.z", "abc.def.ghi", "\u{E9}", "\u{C9}", "a\u{E9}", "a\u{0}", ".a", "a..b", "...",
    ];
    // The apitest's plain ASCII table, and its deliberately "bad" one (shifted letters).
    let mut ascii: Vec<u16> = (0..256u16).map(|c| if (b'a' as u16..=b'z' as u16).contains(&c) { c - 32 } else { c }).collect();
    for c in 0xE0..0xFFu16 {
        if c != 0xF7 {
            ascii[c as usize] = c - 0x20;
        }
    }
    let mut table = vec![0u16; 65536];
    for (c, slot) in table.iter_mut().enumerate() {
        *slot = c as u16;
    }
    for c in b'a'..=b'z' {
        table[c as usize] = if c == b'z' { b'A' as u16 } else { (c + 1 - 32) as u16 };
    }
    let mut checked = 0u32;
    let mut bad: Vec<String> = Vec::new();
    for e in exprs {
        for n in names {
            for ic in [false, true] {
                for tbl in [core::ptr::null_mut(), table.as_mut_ptr()] {
                    let (a, b) = name_case(w, &wide(e), &wide(n), ic, tbl);
                    if (a != 0) != (b != 0) {
                        bad.push(format!("expr {e:?} name {n:?} ic {ic} table {} ours {a} windows {b}", !tbl.is_null()));
                    }
                    checked += 1;
                }
            }
        }
    }
    let _ = &mut ascii;
    // Fuzz: short strings over an alphabet heavy in wildcards and dots.
    let alpha: Vec<u16> = wide("ab.*?<>\"AB\u{E9}\u{C9}");
    let name_alpha: Vec<u16> = wide("ab.AB*?\u{E9}");
    let mut rng = Rng(0xA076_1D64_78BD_642F);
    for case in 0..300_000 {
        let e: Vec<u16> = (0..rng.below(9)).map(|_| alpha[rng.below(alpha.len() as u64) as usize]).collect();
        let n: Vec<u16> = (0..rng.below(11)).map(|_| name_alpha[rng.below(name_alpha.len() as u64) as usize]).collect();
        let ic = rng.below(2) == 0;
        let (a, b) = name_case(w, &e, &n, ic, core::ptr::null_mut());
        if (a != 0) != (b != 0) {
            bad.push(format!(
                "fuzz {case}: expr {:?} name {:?} ic {ic} ours {a} windows {b}",
                String::from_utf16_lossy(&e),
                String::from_utf16_lossy(&n)
            ));
        }
        checked += 1;
    }
    for m in bad.iter().take(60) {
        eprintln!("{m}");
    }
    eprintln!("RtlIsNameInExpression: {checked} cases, {} disagree with Windows", bad.len());
    assert!(bad.is_empty());
}

// ---- processor number ---------------------------------------------------------------------

#[test]
fn processor_number_vs_windows() {
    type ExFn = unsafe extern "system" fn(*mut u8) -> i32;
    let w = win!("NtGetCurrentProcessorNumberEx" as ExFn);
    assert_eq!(real("NtGetCurrentProcessorNumberEx"), real("ZwGetCurrentProcessorNumberEx"));
    for cpu in 0..2 {
        unsafe {
            SetThreadAffinityMask(GetCurrentThread(), 1usize << cpu);
            Sleep(1);
            for off in [0usize, 1, 2, 3] {
                let mut a = [0xCCu8; 12];
                let mut b = [0xCCu8; 12];
                let sa = crate::procnum::NtGetCurrentProcessorNumberEx(a.as_mut_ptr().add(off) as *mut _);
                let sb = w(b.as_mut_ptr().add(off));
                assert_eq!((sa, a), (sb, b), "cpu {cpu} offset {off}");
            }
            #[cfg(target_pointer_width = "64")]
            let bads = [0usize, 1, 0x1000, 0xFFFF, 0x7FFF_FFFF_0000, 0xFFFF_8000_0000_0000, usize::MAX - 1];
            #[cfg(target_pointer_width = "32")]
            let bads = [0usize, 1, 0x1000, 0xFFFF];
            for bad in bads {
                assert_eq!(
                    crate::procnum::NtGetCurrentProcessorNumberEx(bad as *mut _),
                    w(bad as *mut u8),
                    "pointer {bad:#x}"
                );
            }
        }
    }
}

// ---- locale -------------------------------------------------------------------------------

fn locale_init() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        crate::locale::RtlpInitializeLocaleTable();
    });
}

fn table_names() -> Vec<String> {
    crate::locale::RtlpLocaleTable
        .iter()
        .map(|e| {
            let b: Vec<u8> = e.Locale.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
            String::from_utf8(b).unwrap()
        })
        .collect()
}

#[test]
fn locale_table_sorted_under_separator_fold() {
    let fold = |s: &str| s.to_ascii_lowercase().replace('_', "-");
    let names = table_names();
    for w in names.windows(2) {
        assert!(fold(&w[0]) < fold(&w[1]), "table order breaks under the '_'='-' fold: {:?} !< {:?}", w[0], w[1]);
    }
}

#[test]
fn locale_names_vs_windows() {
    locale_init();
    type ValidFn = unsafe extern "system" fn(*const u16, u32) -> u8;
    type N2LFn = unsafe extern "system" fn(*const u16, *mut u32, u32) -> i32;
    let wvalid = win!("RtlIsValidLocaleName" as ValidFn);
    let wn2l = win!("RtlLocaleNameToLcid" as N2LFn);
    let mut probes: Vec<String> = vec![
        "", "en-US", "EN-us", "en", "de-DE", "zh-Hant", "zh-HK_radstr", "x-IV", "en-XX", "qps-ploc",
        "sr-Latn-RS", "en-US-", "en_US", "iv", "invariant", "de", "zh", "zh-CHS", "fr-FR",
        "ja-JP_radstr", "ja-JP-radstr", "_", "-", "en-US_", "zh_Hant", "sr_Latn_RS", "de-DE_phoneb",
        "de-de-PHONEB", "e", "enx", "en-U", "en-USA", "fa-AF", "la-VA", "la-001", "prs-AF", "fa", "prs",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    for n in table_names() {
        probes.push(n.replace('_', "-"));
        probes.push(n.to_ascii_uppercase());
        probes.push(n.clone());
    }
    let mut mismatches = Vec::new();
    for p in &probes {
        let mut v = wide(p);
        v.push(0);
        for flags in [0u32, 1, 2, 3, 4, 0x8000_0000] {
            let a = unsafe { crate::locale::RtlIsValidLocaleName(v.as_ptr(), flags) };
            let b = unsafe { wvalid(v.as_ptr(), flags) };
            if (a != 0) != (b != 0) {
                mismatches.push(format!("RtlIsValidLocaleName({p:?}, {flags:#x}) ours {a} windows {b}"));
            }
            if flags <= 3 {
                let (mut la, mut lb) = (0xDEAD_BEEFu32, 0xDEAD_BEEFu32);
                let sa = unsafe { crate::locale::RtlLocaleNameToLcid(v.as_ptr(), &mut la, flags) };
                let sb = unsafe { wn2l(v.as_ptr(), &mut lb, flags) };
                if (sa >= 0) != (sb >= 0) || (sa >= 0 && la != lb) {
                    mismatches.push(format!("RtlLocaleNameToLcid({p:?}, {flags}) ours {sa:#x}/{la:#x} windows {sb:#x}/{lb:#x}"));
                }
            }
        }
    }
    // Table differences from the Windows 11 NLS data are pre-existing and out of scope; the
    // staged change must not ADD any. Report everything, fail only on the staged surface:
    // a name the table HAS (in any spelling) must agree.
    for m in &mismatches {
        eprintln!("{m}");
    }
    eprintln!("locale: {} probes, {} mismatches", probes.len(), mismatches.len());
    // Zero since the Windows 11 renames (prs -> fa, fa-AF, la-VA) went in, 2026-10-17.
    assert!(mismatches.is_empty(), "{mismatches:#?}");
    let staged: Vec<_> = mismatches.iter().filter(|m| m.starts_with("RtlIsValidLocaleName")).collect();
    let n2l: Vec<_> = mismatches.iter().filter(|m| m.starts_with("RtlLocaleNameToLcid")).collect();
    // Every IsValid mismatch must be explained by a lookup mismatch on the same name.
    for m in &staged {
        let name = m.split('"').nth(1).unwrap_or("");
        assert!(
            n2l.iter().any(|x| x.contains(&format!("({name:?},"))),
            "RtlIsValidLocaleName disagrees where the lookup agrees: {m}"
        );
    }
}

#[test]
fn lcid_to_locale_name_vs_windows() {
    // The reverse of locale_names_vs_windows, over every LCID the table carries: a name
    // remapped to another LCID (Windows 11 resolves the neutral "prs" to fa's 0x29) must not
    // change what that LCID, or the one it left, prints.
    locale_init();
    type L2NFn = unsafe extern "system" fn(u32, *mut _UNICODE_STRING, u32, u8) -> i32;
    let w = win!("RtlLcidToLocaleName" as L2NFn);
    let mut lcids: Vec<u32> = crate::locale::RtlpLocaleTable.iter().map(|e| e._bitfield & 0xFFFFF).collect();
    lcids.push(0x8C); // printed through LCID_ONLY_NAMES, not a table row
    lcids.sort();
    lcids.dedup();
    let mut mismatches = Vec::new();
    for &lcid in &lcids {
        for flags in [0u32, 2] {
            let call = |f: &dyn Fn(*mut _UNICODE_STRING) -> i32| {
                let mut buf = [0u16; 86];
                let mut us: _UNICODE_STRING = unsafe { core::mem::zeroed() };
                us.Buffer = buf.as_mut_ptr();
                us.MaximumLength = (buf.len() * 2) as u16;
                let st = f(&mut us);
                let name = String::from_utf16_lossy(&buf[..(us.Length / 2) as usize]);
                (st, if st >= 0 { name } else { String::new() })
            };
            let a = call(&|u| unsafe { crate::locale::RtlLcidToLocaleName(lcid, u as _, flags, 0) });
            let b = call(&|u| unsafe { w(lcid, u, flags, 0) });
            if (a.0 >= 0) != (b.0 >= 0) || a.1 != b.1 {
                mismatches.push(format!("RtlLcidToLocaleName({lcid:#x}, {flags}) ours {:#x} {:?} windows {:#x} {:?}", a.0, a.1, b.0, b.1));
            }
        }
    }
    for m in &mismatches {
        eprintln!("{m}");
    }
    eprintln!("lcid->name: {} lcids, {} mismatches", lcids.len(), mismatches.len());
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

#[test]
fn convert_lcid_to_string_vs_windows() {
    type ConvFn = unsafe extern "system" fn(u32, u32, u32, *mut u16, u32) -> i32;
    let w = win!("RtlConvertLCIDToString" as ConvFn);
    let case = |lcid: u32, base: u32, pad: u32, size: u32| {
        let mut a = [b'#' as u16; 96];
        let mut b = a;
        let sa = unsafe { crate::locale::RtlConvertLCIDToString(lcid, base, pad, a.as_mut_ptr(), size) };
        let sb = unsafe { w(lcid, base, pad, b.as_mut_ptr(), size) };
        assert_eq!(
            (sa, &a[..]),
            (sb, &b[..]),
            "RtlConvertLCIDToString({lcid:#x}, {base}, {pad}, buf, {size})"
        );
    };
    for lcid in [0u32, 1, 0x409, 0x7F, 0x10407, 0x12345, 0x7FFF_FFFF, 0xFFFF_FFFF] {
        for base in [0u32, 1, 2, 3, 7, 8, 10, 16, 17, 36, 0xFFFF_FFFF] {
            for pad in [0u32, 1, 2, 3, 4, 5, 8, 9, 10, 20, 33, 40] {
                for size in 0..=40u32 {
                    case(lcid, base, pad, size);
                }
                case(lcid, base, pad, 90);
            }
        }
    }
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    for _ in 0..50_000 {
        let lcid = rng.next() as u32 >> rng.below(32);
        let base = [0, 2, 8, 10, 16][rng.below(5) as usize];
        case(lcid, base, rng.below(45) as u32, rng.below(45) as u32);
    }
}

// ---- i386 SList push-list ---------------------------------------------------------------------

#[cfg(target_arch = "x86")]
mod slist_i386 {
    use super::*;
    // slistex.rs calls the FASTCALL body by symbol. Here that symbol forwards to Windows'
    // own fastcall export, so the test exercises exactly the staged stdcall->fastcall hop.
    type FastFn = unsafe extern "fastcall" fn(*mut u8, *mut u8, *mut u8, u32) -> *mut u8;
    #[no_mangle]
    unsafe extern "fastcall" fn RtlInterlockedPushListSList(h: *mut u8, l: *mut u8, e: *mut u8, c: u32) -> *mut u8 {
        let f: FastFn = core::mem::transmute(real("RtlInterlockedPushListSList"));
        f(h, l, e, c)
    }

    #[repr(C, align(8))]
    struct Entry(*mut Entry);

    #[test]
    fn push_list_ex_vs_windows() {
        type ExFn = unsafe extern "system" fn(*mut u8, *mut u8, *mut u8, u32) -> *mut u8;
        let wex: ExFn = unsafe { core::mem::transmute(real("RtlInterlockedPushListSListEx")) };
        let mut headers = Vec::new();
        for which in 0..2 {
            let mut head = [0u64; 1];
            let mut e: Vec<Entry> = (0..5).map(|_| Entry(core::ptr::null_mut())).collect();
            #[allow(unused_mut)]
            let mut p: Vec<*mut Entry> = e.iter_mut().map(|x| x as *mut Entry).collect();
            unsafe {
                // Pre-push e0 by hand through the Windows Ex export.
                (*p[0]).0 = core::ptr::null_mut();
                let push = |h: *mut u64, f: *mut Entry, l: *mut Entry, c: u32| -> *mut u8 {
                    if which == 0 {
                        crate::slistex::RtlInterlockedPushListSListEx(h as *mut u8, f as *mut u8, l as *mut u8, c)
                    } else {
                        wex(h as *mut u8, f as *mut u8, l as *mut u8, c)
                    }
                };
                let r0 = push(head.as_mut_ptr(), p[0], p[0], 1);
                assert!(r0.is_null());
                (*p[1]).0 = p[2];
                (*p[2]).0 = p[3];
                (*p[3]).0 = 1 as *mut Entry;
                let r = push(head.as_mut_ptr(), p[1], p[3], 3);
                assert_eq!(r, p[0] as *mut u8, "impl {which}: returns the previous first entry");
                assert_eq!((*p[3]).0, p[0], "impl {which}: tail spliced onto the old head");
                let h = head[0];
                assert_eq!(h as u32 as usize, p[1] as usize, "impl {which}: new first entry");
                assert_eq!((h >> 32) & 0xFFFF, 4, "impl {which}: depth");
                // Sequence: whatever Windows does (this build leaves it 0 on x86 for every
                // SList operation); the staged hop must leave the same header as the Ex export.
                headers.push(h >> 32);
            }
        }
        assert_eq!(headers[0], headers[1], "depth+sequence after the staged hop vs Windows Ex");
    }
}
