//! IPv4 conversion vectors — exercise the `#[no_mangle]` FFI end-to-end
//! (narrow + wide) against hand-verified expected values. Covers the
//! load-bearing misfeatures: inet_aton multi-part fill, base detection
//! (hex/octal/strict), overflow/range rejection, and the Ex port path.

use super::*;

const OK: i32 = STATUS_SUCCESS;
const BAD: i32 = STATUS_INVALID_PARAMETER;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(core::iter::once(0)).collect()
}
fn a(s: &str) -> Vec<u8> {
    s.bytes().chain(core::iter::once(0)).collect()
}

// ---- string → address (wide) --------------------------------------
fn s2a_w(s: &str, strict: bool) -> (i32, [u8; 4], usize) {
    let buf = w(s);
    let mut addr = InAddr { s_addr: 0 };
    let mut term: *const u16 = core::ptr::null();
    let st = unsafe {
        RtlIpv4StringToAddressW(buf.as_ptr(), strict as u8, &mut term, &mut addr)
    };
    let idx = (term as usize - buf.as_ptr() as usize) / 2;
    (st, addr.s_addr.to_le_bytes(), idx)
}

// ---- string → address (narrow) ------------------------------------
fn s2a_a(s: &str, strict: bool) -> (i32, [u8; 4], usize) {
    let buf = a(s);
    let mut addr = InAddr { s_addr: 0 };
    let mut term: *const u8 = core::ptr::null();
    let st = unsafe {
        RtlIpv4StringToAddressA(buf.as_ptr(), strict as u8, &mut term, &mut addr)
    };
    let idx = term as usize - buf.as_ptr() as usize;
    (st, addr.s_addr.to_le_bytes(), idx)
}

#[test]
fn string_to_address_basic() {
    for &(s, bytes) in &[
        ("1.2.3.4", [1u8, 2, 3, 4]),
        ("127.0.0.1", [127, 0, 0, 1]),
        ("255.255.255.255", [255, 255, 255, 255]),
        ("0.0.0.0", [0, 0, 0, 0]),
    ] {
        let (st, b, _) = s2a_w(s, false);
        assert_eq!(st, OK, "W {s}");
        assert_eq!(b, bytes, "W {s}");
        let (st, b, _) = s2a_a(s, false);
        assert_eq!(st, OK, "A {s}");
        assert_eq!(b, bytes, "A {s}");
    }
}

#[test]
fn string_to_address_inet_aton_fill() {
    // last part fills the remaining low bytes
    assert_eq!(s2a_w("1.2", false), (OK, [1, 0, 0, 2], 3));
    assert_eq!(s2a_w("1.2.3", false), (OK, [1, 2, 0, 3], 5));
    assert_eq!(s2a_w("16909060", false).0, OK); // 0x01020304 host → fill
    assert_eq!(s2a_w("16909060", false).1, [1, 2, 3, 4]);
}

#[test]
fn string_to_address_base_detection() {
    assert_eq!(s2a_w("0x7f.0.0.1", false), (OK, [127, 0, 0, 1], 10));
    assert_eq!(s2a_w("0X7F.0.0.1", false).1, [127, 0, 0, 1]);
    assert_eq!(s2a_w("010.0.0.1", false), (OK, [8, 0, 0, 1], 9)); // octal 010=8
    // strict forbids non-decimal
    assert_eq!(s2a_w("0x7f.0.0.1", true).0, BAD);
    assert_eq!(s2a_w("010.0.0.1", true).0, BAD);
}

#[test]
fn string_to_address_rejects() {
    assert_eq!(s2a_w("256.0.0.1", false).0, BAD); // non-last part > 255
    assert_eq!(s2a_w("1.2.3.4.5", false).0, BAD); // dot after 4 parts
    assert_eq!(s2a_w("", false).0, BAD); // no digits
    assert_eq!(s2a_w(".", false).0, BAD);
    // strict requires exactly 4 parts
    assert_eq!(s2a_w("1.2.3", true).0, BAD);
    assert_eq!(s2a_w("1.2.3.4", true).0, OK);
    // overflow of a single part
    assert_eq!(s2a_w("4294967296.0.0.0", false).0, BAD);
}

#[test]
fn string_to_address_terminator() {
    // non-Ex parse succeeds and leaves terminator at the trailing text
    let (st, b, idx) = s2a_w("1.2.3.4:80", false);
    assert_eq!((st, b), (OK, [1, 2, 3, 4]));
    assert_eq!(idx, 7); // points at ':'
}

// ---- address → string ---------------------------------------------
#[test]
fn address_to_string() {
    for &(bytes, s) in &[
        ([1u8, 2, 3, 4], "1.2.3.4"),
        ([127, 0, 0, 1], "127.0.0.1"),
        ([255, 255, 255, 255], "255.255.255.255"),
        ([0, 0, 0, 0], "0.0.0.0"),
    ] {
        let addr = InAddr {
            s_addr: u32::from_le_bytes(bytes),
        };
        // wide
        let mut buf = [0u16; IPV4_ADDR_STRING_MAX_LEN];
        let end = unsafe { RtlIpv4AddressToStringW(&addr, buf.as_mut_ptr()) };
        let n = (end as usize - buf.as_ptr() as usize) / 2;
        assert_eq!(String::from_utf16(&buf[..n]).unwrap(), s);
        assert_eq!(buf[n], 0);
        // narrow
        let mut bufa = [0u8; IPV4_ADDR_STRING_MAX_LEN];
        let end = unsafe { RtlIpv4AddressToStringA(&addr, bufa.as_mut_ptr()) };
        let n = end as usize - bufa.as_ptr() as usize;
        assert_eq!(core::str::from_utf8(&bufa[..n]).unwrap(), s);
    }
}

#[test]
fn address_to_string_null() {
    unsafe {
        assert_eq!(RtlIpv4AddressToStringW(core::ptr::null(), core::ptr::null_mut()), usize::MAX as *mut u16);
        assert_eq!(RtlIpv4AddressToStringA(core::ptr::null(), core::ptr::null_mut()), usize::MAX as *mut u8);
    }
}

// ---- Ex: address → string with port -------------------------------
#[test]
fn address_to_string_ex() {
    let addr = InAddr {
        s_addr: u32::from_le_bytes([127, 0, 0, 1]),
    };
    // no port
    let mut buf = [0u16; 32];
    let mut len = buf.len() as u32;
    let st = unsafe { RtlIpv4AddressToStringExW(&addr, 0, buf.as_mut_ptr(), &mut len) };
    assert_eq!(st, OK);
    assert_eq!(String::from_utf16(&buf[..len as usize - 1]).unwrap(), "127.0.0.1");
    // with port (port arg is network order; display is host order)
    let mut buf = [0u16; 32];
    let mut len = buf.len() as u32;
    let st = unsafe {
        RtlIpv4AddressToStringExW(&addr, 8080u16.swap_bytes(), buf.as_mut_ptr(), &mut len)
    };
    assert_eq!(st, OK);
    assert_eq!(String::from_utf16(&buf[..len as usize - 1]).unwrap(), "127.0.0.1:8080");
    // too-small buffer → INVALID + required length
    let mut small = [0u16; 4];
    let mut len = small.len() as u32;
    let st = unsafe { RtlIpv4AddressToStringExW(&addr, 0, small.as_mut_ptr(), &mut len) };
    assert_eq!(st, BAD);
    assert_eq!(len, "127.0.0.1".len() as u32 + 1);
}

// ---- Ex: string → address with port -------------------------------
#[test]
fn string_to_address_ex() {
    let parse = |s: &str| -> (i32, [u8; 4], u16) {
        let buf = w(s);
        let mut addr = InAddr { s_addr: 0 };
        let mut port: u16 = 0;
        let st = unsafe {
            RtlIpv4StringToAddressExW(buf.as_ptr(), 0, &mut addr, &mut port)
        };
        (st, addr.s_addr.to_le_bytes(), port)
    };
    // with port → *Port is network order
    let (st, b, p) = parse("127.0.0.1:8080");
    assert_eq!((st, b), (OK, [127, 0, 0, 1]));
    assert_eq!(p, 8080u16.swap_bytes());
    // no port → 0
    assert_eq!(parse("127.0.0.1"), (OK, [127, 0, 0, 1], 0));
    // bad: trailing garbage, port 0, port overflow
    assert_eq!(parse("127.0.0.1:0").0, BAD);
    assert_eq!(parse("127.0.0.1:99999").0, BAD);
    assert_eq!(parse("127.0.0.1:80x").0, BAD);
    assert_eq!(parse("127.0.0.1x").0, BAD);
}

// ───────────────────────── IPv6 ──────────────────────────────────────
/// network-order bytes from 8 host words.
fn words(w: [u16; 8]) -> [u8; 16] {
    let mut b = [0u8; 16];
    for i in 0..8 {
        b[2 * i] = (w[i] >> 8) as u8;
        b[2 * i + 1] = w[i] as u8;
    }
    b
}

fn a2s6_w(bytes: [u8; 16]) -> String {
    let addr = In6Addr { bytes };
    let mut buf = [0u16; 64];
    let end = unsafe { RtlIpv6AddressToStringW(&addr, buf.as_mut_ptr()) };
    let n = (end as usize - buf.as_ptr() as usize) / 2;
    assert_eq!(buf[n], 0, "NUL-terminated");
    String::from_utf16(&buf[..n]).unwrap()
}

fn s2a6_w(s: &str) -> (i32, [u8; 16]) {
    let buf = w(s);
    let mut addr = In6Addr { bytes: [0xAA; 16] };
    let mut term: *const u16 = core::ptr::null();
    let st = unsafe { RtlIpv6StringToAddressW(buf.as_ptr(), &mut term, &mut addr) };
    (st, addr.bytes)
}

#[test]
fn ipv6_format() {
    assert_eq!(a2s6_w(words([0, 0, 0, 0, 0, 0, 0, 0])), "::");
    assert_eq!(a2s6_w(words([0, 0, 0, 0, 0, 0, 0, 1])), "::1");
    assert_eq!(a2s6_w(words([1, 0, 0, 0, 0, 0, 0, 0])), "1::");
    assert_eq!(a2s6_w(words([0x2001, 0xdb8, 0, 0, 0, 0, 0, 1])), "2001:db8::1");
    assert_eq!(
        a2s6_w(words([1, 2, 3, 4, 5, 6, 7, 8])),
        "1:2:3:4:5:6:7:8"
    );
    assert_eq!(
        a2s6_w(words([0xfe80, 0, 0, 0, 0, 0, 0, 1])),
        "fe80::1"
    );
    // only the FIRST zero-run is compressed
    assert_eq!(a2s6_w(words([1, 0, 0, 2, 0, 0, 0, 3])), "1::2:0:0:0:3");
}

#[test]
fn ipv6_format_ipv4_mapped() {
    // ::ffff:1.2.3.4 (word4=0, word5=0xffff, words0..3=0, word6 != 0)
    let mut b = [0u8; 16];
    b[10] = 0xff;
    b[11] = 0xff;
    b[12..16].copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(a2s6_w(b), "::ffff:1.2.3.4");
    // ::1.2.3.4 (words0..5 == 0, word6 != 0)
    let mut b = [0u8; 16];
    b[12..16].copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(a2s6_w(b), "::1.2.3.4");
    // ::ffff:0:1.2.3.4 (word4=0xffff, word5=0)
    let mut b = [0u8; 16];
    b[8] = 0xff;
    b[9] = 0xff;
    b[12..16].copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(a2s6_w(b), "::ffff:0:1.2.3.4");
}

#[test]
fn ipv6_parse() {
    assert_eq!(s2a6_w("::"), (OK, words([0, 0, 0, 0, 0, 0, 0, 0])));
    assert_eq!(s2a6_w("::1"), (OK, words([0, 0, 0, 0, 0, 0, 0, 1])));
    assert_eq!(s2a6_w("1::"), (OK, words([1, 0, 0, 0, 0, 0, 0, 0])));
    assert_eq!(
        s2a6_w("2001:db8::1"),
        (OK, words([0x2001, 0xdb8, 0, 0, 0, 0, 0, 1]))
    );
    assert_eq!(
        s2a6_w("1:2:3:4:5:6:7:8"),
        (OK, words([1, 2, 3, 4, 5, 6, 7, 8]))
    );
    assert_eq!(
        s2a6_w("fe80::1"),
        (OK, words([0xfe80, 0, 0, 0, 0, 0, 0, 1]))
    );
}

#[test]
fn ipv6_parse_embedded_ipv4() {
    let mut want = [0u8; 16];
    want[10] = 0xff;
    want[11] = 0xff;
    want[12..16].copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(s2a6_w("::ffff:1.2.3.4"), (OK, want));

    let mut want = [0u8; 16];
    want[12..16].copy_from_slice(&[127, 0, 0, 1]);
    assert_eq!(s2a6_w("::127.0.0.1"), (OK, want));
}

#[test]
fn ipv6_parse_rejects() {
    // True failures: invalid from the start / insufficient bytes.
    assert_eq!(s2a6_w("12345::").0, BAD); // group > 4 hex digits
    assert_eq!(s2a6_w("1.2.3.4").0, BAD); // bare ipv4, no gap, <16 bytes
    assert_eq!(s2a6_w("xyz").0, BAD);
    assert_eq!(s2a6_w("").0, BAD);
    // Non-Ex W stops at the first invalid token and returns SUCCESS with
    // a mid-string terminator (faithful to Windows/Wine) — NOT failures:
    assert_eq!(s2a6_w("1::2::3").0, OK); // parses "1::2"
    assert_eq!(s2a6_w("1:2:3:4:5:6:7:8:9").0, OK); // parses 8 groups
    assert_eq!(s2a6_w(":::").0, OK); // parses "::"
    // ...but the Ex form requires full consumption, so it rejects them:
    let ex = |s: &str| -> i32 {
        let buf = w(s);
        let mut addr = In6Addr { bytes: [0; 16] };
        let (mut sc, mut po) = (0u32, 0u16);
        unsafe { RtlIpv6StringToAddressExW(buf.as_ptr(), &mut addr, &mut sc, &mut po) }
    };
    assert_eq!(ex("1::2::3"), BAD);
    assert_eq!(ex("1:2:3:4:5:6:7:8:9"), BAD);
    assert_eq!(ex(":::"), BAD);
}

#[test]
fn ipv6_round_trip() {
    for s in &[
        "::", "::1", "1::", "2001:db8::1", "1:2:3:4:5:6:7:8", "fe80::1",
        "::ffff:1.2.3.4", "::1.2.3.4", "1::2:0:0:0:3",
    ] {
        let (st, b) = s2a6_w(s);
        assert_eq!(st, OK, "parse {s}");
        assert_eq!(&a2s6_w(b), s, "round-trip {s}");
    }
}

// ---- Ex: string → address with scope + port -----------------------
#[test]
fn ipv6_string_to_address_ex() {
    let parse = |s: &str| -> (i32, [u8; 16], u32, u16) {
        let buf = w(s);
        let mut addr = In6Addr { bytes: [0; 16] };
        let mut scope: u32 = 0xdead;
        let mut port: u16 = 0xbe;
        let st = unsafe {
            RtlIpv6StringToAddressExW(buf.as_ptr(), &mut addr, &mut scope, &mut port)
        };
        (st, addr.bytes, scope, port)
    };
    // scope id
    let (st, b, scope, port) = parse("fe80::1%5");
    assert_eq!((st, scope, port), (OK, 5, 0));
    assert_eq!(b, words([0xfe80, 0, 0, 0, 0, 0, 0, 1]));
    // bracketed port → network order
    let (st, b, scope, port) = parse("[::1]:8080");
    assert_eq!((st, scope), (OK, 0));
    assert_eq!(port, 8080u16.swap_bytes());
    assert_eq!(b, words([0, 0, 0, 0, 0, 0, 0, 1]));
    // scope + port together
    let (st, _, scope, port) = parse("[fe80::1%3]:443");
    assert_eq!((st, scope, port), (OK, 3, 443u16.swap_bytes()));
    // plain (no scope/port) requires full consumption
    assert_eq!(parse("::1").0, OK);
    assert_eq!(parse("::1 ").0, BAD); // trailing junk
    assert_eq!(parse("[::1]:0").0, BAD); // port 0
    assert_eq!(parse("[::1]:99999").0, BAD); // port overflow
}

// ---- Ex: address → string with scope + port -----------------------
#[test]
fn ipv6_address_to_string_ex() {
    let fmt = |bytes: [u8; 16], scope: u32, port: u16| -> (i32, String) {
        let addr = In6Addr { bytes };
        let mut buf = [0u16; 80];
        let mut len = buf.len() as u32;
        let st = unsafe {
            RtlIpv6AddressToStringExW(&addr, scope, port, buf.as_mut_ptr(), &mut len)
        };
        (st, String::from_utf16(&buf[..len as usize - 1]).unwrap())
    };
    let lo = words([0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(fmt(lo, 0, 0), (OK, "::1".into()));
    assert_eq!(fmt(lo, 5, 0), (OK, "::1%5".into()));
    // port arg is network order; displayed host order, wrapped in []
    assert_eq!(fmt(lo, 0, 8080u16.swap_bytes()), (OK, "[::1]:8080".into()));
    assert_eq!(
        fmt(lo, 3, 443u16.swap_bytes()),
        (OK, "[::1%3]:443".into())
    );
}
