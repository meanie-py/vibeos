//! `network` — Rust reimplementation of `sdk/lib/rtl/network.c` IPv4
//! address ⇄ string conversion (`RtlIpv4AddressToString[Ex][A/W]`,
//! `RtlIpv4StringToAddress[Ex][A/W]`).
//!
//! Pure algorithm, no heap / no kernel deps. The parse/format core is
//! generic over the character width (`u8`/`u16`) so the ANSI and wide
//! variants share one faithful implementation — no `RtlAnsiString…`
//! round-trip (behaviorally identical for the ASCII IP grammar). The
//! inline `RtlULongMult/Add` overflow checks become `checked_mul/add`,
//! `RtlUlongByteSwap` becomes `swap_bytes`, and the `"%u.%u.%u.%u"`
//! `RtlStringCchPrintf` becomes a hand-rolled decimal writer.
//!
//! Semantics are faithful to the C, including the inet_aton-style
//! multi-part fill (`"1.2"` → `1.0.0.2`) and base detection
//! (`0x…`=hex, `0…`=octal, `strict`=decimal only). IPv6 is a follow-on.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

const STATUS_SUCCESS: i32 = 0;
const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;

/// max length of an ipv4 address as a string (incl NUL), per network.c.
const IPV4_ADDR_STRING_MAX_LEN: usize = 16;
/// max length of an ipv4 port string (with leading ':').
const IPV4_PORT_STRING_MAX_LEN: usize = 7;

/// `struct in_addr` — only the `S_addr` (network-order u32) member is
/// needed; the `s_b1..s_b4` byte view aliases its little-endian bytes
/// (every target here is little-endian, matching the C's WN2H usage).
#[repr(C)]
pub struct InAddr {
    pub s_addr: u32,
}

/// network↔host order for a 16-bit value (htons/ntohs == byteswap).
#[inline]
fn wn2h(w: u16) -> u16 {
    w.swap_bytes()
}

// ───────────────────────── pure core ─────────────────────────────────
/// Pure, slice-based, `unsafe`-free IPv4 grammar. Indices returned are
/// element offsets (1:1 with the C `Terminator` pointer delta for both
/// `u8` and `u16`). Tested directly; the `#[no_mangle]` shims only
/// reconstruct slices and map the C ABI onto these.
pub mod pure {
    /// A string character (narrow or wide). `code()` is the codepoint;
    /// `from_ascii()` writes an ASCII output char.
    pub trait IpChar: Copy {
        fn code(self) -> u32;
        fn from_ascii(b: u8) -> Self;
    }
    impl IpChar for u8 {
        #[inline]
        fn code(self) -> u32 {
            self as u32
        }
        #[inline]
        fn from_ascii(b: u8) -> u8 {
            b
        }
    }
    impl IpChar for u16 {
        #[inline]
        fn code(self) -> u32 {
            self as u32
        }
        #[inline]
        fn from_ascii(b: u8) -> u16 {
            b as u16
        }
    }

    /// `*String` with NUL-termination semantics: past the slice end (or
    /// an embedded 0) reads as NUL, exactly like the C pointer walk.
    #[inline]
    fn at<C: IpChar>(s: &[C], i: usize) -> u32 {
        if i < s.len() {
            s[i].code()
        } else {
            0
        }
    }

    /// `towlower` restricted to ASCII (the only chars the grammar
    /// inspects); non-ASCII passes through unchanged.
    #[inline]
    fn lower(c: u32) -> u32 {
        if (b'A' as u32..=b'Z' as u32).contains(&c) {
            c + 32
        } else {
            c
        }
    }

    /// `RtlpStringToUlongBase`. Returns `(ok, value, consumed)`: `ok` is
    /// false until ≥1 digit is read and goes false again on overflow
    /// (mirroring `RtlULongMult/Add` failure → STATUS_INVALID_PARAMETER,
    /// with `value` left at the partial accumulation and `consumed`
    /// pointing AT the offending/again non-digit char).
    fn str_to_ulong_base<C: IpChar>(s: &[C], base: u32) -> (bool, u32, usize) {
        let mut result: u32 = 0;
        let mut ok = false;
        let mut i = 0usize;
        loop {
            let d = lower(at(s, i));
            let digit = if (b'0' as u32..=b'9' as u32).contains(&d)
                && (base >= 10 || d <= b'7' as u32)
            {
                d - b'0' as u32
            } else if (b'a' as u32..=b'f' as u32).contains(&d) && base >= 16 {
                d - (b'a' as u32 - 10)
            } else {
                break;
            };
            match result.checked_mul(base) {
                Some(r) => result = r,
                None => {
                    ok = false;
                    break;
                }
            }
            match result.checked_add(digit) {
                Some(r) => result = r,
                None => {
                    ok = false;
                    break;
                }
            }
            ok = true;
            i += 1;
        }
        (ok, result, i)
    }

    /// `RtlpStringToUlong` — base detection (`0x`=hex, `0`=octal,
    /// else decimal); `strict` forbids anything but decimal.
    fn str_to_ulong<C: IpChar>(s: &[C], strict: bool) -> (bool, u32, usize) {
        let mut base = 10u32;
        let mut off = 0usize;
        if at(s, 0) == b'0' as u32 {
            let n = at(s, 1);
            if n == b'x' as u32 || n == b'X' as u32 {
                off = 2;
                base = 16;
            } else if (b'0' as u32..=b'9' as u32).contains(&n) {
                off = 1;
                base = 8;
            }
        }
        if strict && base != 10 {
            return (false, 0, 0); // Terminator == String
        }
        let (ok, val, term) = str_to_ulong_base(&s[off.min(s.len())..], base);
        (ok, val, off + term)
    }

    /// `RtlpIpv4StringToAddressParserW` — extract up to 4 dot-separated
    /// parts. Returns `(ok, values, parts, consumed)`.
    fn parse_parts<C: IpChar>(s: &[C], strict: bool) -> (bool, [u32; 4], i32, usize) {
        let mut values = [0u32; 4];
        let mut parts = 0i32;
        let mut i = 0usize;
        let mut status;
        loop {
            let (ok, val, consumed) = str_to_ulong(&s[i.min(s.len())..], strict);
            values[parts as usize] = val;
            i += consumed;
            parts += 1;
            status = ok;

            if at(s, i) != b'.' as u32 {
                break;
            }
            if parts == 4 {
                status = false; // dot after 4 parts → STATUS_INVALID_PARAMETER
                break;
            }
            i += 1; // skip the dot
            if !status {
                break; // do { } while (NT_SUCCESS(Status))
            }
        }
        (status, values, parts, i)
    }

    /// `RtlIpv4StringToAddressW` core. Returns `(addr_be, terminator)`
    /// on success (`addr_be` = `in_addr.S_addr`, network byte order),
    /// or `(None terminator)` — terminator is always set (the parser's),
    /// matching the C out-param contract.
    pub fn string_to_address<C: IpChar>(
        s: &[C],
        strict: bool,
    ) -> (Result<u32, ()>, usize) {
        let (mut ok, values, parts, term) = parse_parts(s, strict);
        if strict && parts < 4 {
            ok = false;
        }
        if !ok {
            return (Err(()), term);
        }
        // Combine: last part fills the remaining low bytes (inet_aton).
        let mut result = values[(parts - 1) as usize];
        let mut i = 0i32;
        while i < parts - 1 {
            let shift = 8 * (3 - i) as u32;
            if values[i as usize] > 0xFF || (result & (0xFF << shift)) != 0 {
                return (Err(()), term);
            }
            result |= values[i as usize] << shift;
            i += 1;
        }
        (Ok(result.swap_bytes()), term)
    }

    /// Parse a host-order port (`RtlpStringToUlong` non-strict + range
    /// check). Returns `(port_host, consumed)` or `Err`.
    pub fn parse_port<C: IpChar>(s: &[C]) -> Result<u32, ()> {
        let (ok, port, term) = str_to_ulong(s, false);
        // must consume the whole remainder, be nonzero and ≤ 0xffff
        if !ok || at(s, term) != 0 || port == 0 || port > 0xffff {
            return Err(());
        }
        Ok(port)
    }

    /// Write `b0.b1.b2.b3` into `out` (no NUL). Returns the char count.
    pub fn format_address<C: IpChar>(bytes: [u8; 4], out: &mut [C]) -> usize {
        let mut n = 0usize;
        let mut part = 0;
        while part < 4 {
            if part != 0 {
                out[n] = C::from_ascii(b'.');
                n += 1;
            }
            n += write_u8(bytes[part], &mut out[n..]);
            part += 1;
        }
        n
    }

    /// Write an unsigned value's decimal digits; returns the count.
    pub fn write_u32<C: IpChar>(mut v: u32, out: &mut [C]) -> usize {
        let mut tmp = [0u8; 10];
        let mut len = 0;
        loop {
            tmp[len] = b'0' + (v % 10) as u8;
            v /= 10;
            len += 1;
            if v == 0 {
                break;
            }
        }
        for k in 0..len {
            out[k] = C::from_ascii(tmp[len - 1 - k]);
        }
        len
    }

    #[inline]
    fn write_u8<C: IpChar>(v: u8, out: &mut [C]) -> usize {
        write_u32(v as u32, out)
    }

    /// Write `v` as lowercase hex (no leading zeros). Returns the count.
    pub fn write_hex<C: IpChar>(mut v: u32, out: &mut [C]) -> usize {
        let mut tmp = [0u8; 8];
        let mut len = 0;
        loop {
            let d = (v & 0xf) as u8;
            tmp[len] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
            v >>= 4;
            len += 1;
            if v == 0 {
                break;
            }
        }
        for k in 0..len {
            out[k] = C::from_ascii(tmp[len - 1 - k]);
        }
        len
    }

    // ───────────────────── IPv6 ──────────────────────────────────────
    pub const STATUS_SUCCESS: i32 = 0;
    pub const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;

    /// ASCII → digit value (or -1), Wine's `hex_table` (covers 0x00-0x66).
    const HEX: [i8; 103] = {
        let mut t = [-1i8; 103];
        let mut c = b'0';
        while c <= b'9' {
            t[c as usize] = (c - b'0') as i8;
            c += 1;
        }
        let mut c = b'A';
        while c <= b'F' {
            t[c as usize] = (c - b'A' + 10) as i8;
            c += 1;
        }
        let mut c = b'a';
        while c <= b'f' {
            t[c as usize] = (c - b'a' + 10) as i8;
            c += 1;
        }
        t
    };

    #[inline]
    fn hex_of(code: u32) -> i8 {
        if (code as usize) < HEX.len() {
            HEX[code as usize]
        } else {
            -1
        }
    }

    /// `in6_addr` byte index helper: store host word `v` at word slot `w`
    /// in network byte order (`u.Word[w] = RtlUshortByteSwap(v)`).
    #[inline]
    fn set_word(a: &mut [u8; 16], w: usize, v: u32) {
        a[w * 2] = (v >> 8) as u8;
        a[w * 2 + 1] = v as u8;
    }
    /// host value of `s6_words[n]` (= `WN2H(u.Word[n])`).
    #[inline]
    fn word_host(a: &[u8; 16], n: usize) -> u32 {
        ((a[n * 2] as u32) << 8) | a[n * 2 + 1] as u32
    }
    /// native `s6_words[n]` (`u.Word[n]`, as the C compares it).
    #[inline]
    fn word_native(a: &[u8; 16], n: usize) -> u32 {
        (a[n * 2] as u32) | ((a[n * 2 + 1] as u32) << 8)
    }

    /// `strtoul`-style parse (base 10/16) → `(value saturating at
    /// u32::MAX, terminator index)`. No-digit ⇒ terminator == `from`.
    fn wcstoul<C: IpChar>(s: &[C], from: usize, base: u32) -> (u32, usize) {
        let mut i = from;
        while at(s, i) == b' ' as u32 || (at(s, i) >= 9 && at(s, i) <= 13) {
            i += 1; // isspace
        }
        let mut neg = false;
        match at(s, i) {
            x if x == b'+' as u32 => i += 1,
            x if x == b'-' as u32 => {
                neg = true;
                i += 1;
            }
            _ => {}
        }
        if base == 16
            && at(s, i) == b'0' as u32
            && (at(s, i + 1) == b'x' as u32 || at(s, i + 1) == b'X' as u32)
            && hex_of(at(s, i + 2)) != -1
        {
            i += 2;
        }
        let start = i;
        let mut val: u32 = 0;
        let mut ovf = false;
        loop {
            let d = hex_of(at(s, i));
            if d == -1 || d as u32 >= base {
                break;
            }
            let (m, o1) = val.overflowing_mul(base);
            let (a, o2) = m.overflowing_add(d as u32);
            if o1 || o2 {
                ovf = true;
            }
            val = a;
            i += 1;
        }
        let term = if i == start { from } else { i };
        let mut result = if ovf { u32::MAX } else { val };
        if neg {
            result = result.wrapping_neg();
        }
        (result, term)
    }

    /// Wine `parse_ipv4_component` — a scope/port number (advances `i`).
    fn parse_ipv4_component<C: IpChar>(s: &[C], i: &mut usize, strict: bool) -> Option<u32> {
        if at(s, *i) == b'.' as u32 {
            *i += 1;
            return None;
        }
        let mut base = 10u32;
        if at(s, *i) == b'0' as u32 {
            let n = at(s, *i + 1);
            if n == b'x' as u32 || n == b'X' as u32 {
                *i += 2;
                if strict {
                    return None;
                }
                base = 16;
            } else if (b'0' as u32..=b'9' as u32).contains(&n) {
                *i += 1;
                if strict {
                    return None;
                }
                base = 8;
            }
        }
        let mut cur = 0u32;
        let mut prev = 0u32;
        let mut success = false;
        loop {
            if at(s, *i) == 0 {
                break;
            }
            let d = hex_of(at(s, *i));
            if d == -1 || d as u32 >= base {
                break;
            }
            cur = cur.wrapping_mul(base).wrapping_add(d as u32);
            success = true;
            if cur < prev {
                return None; // overflow
            }
            prev = cur;
            *i += 1;
        }
        if success {
            Some(cur)
        } else {
            None
        }
    }

    /// Wine `parse_ipv6_component` (advances `i`).
    fn parse_ipv6_component<C: IpChar>(s: &[C], i: &mut usize, base: u32) -> Option<u32> {
        if hex_of(at(s, *i)) == -1 {
            return None;
        }
        let (val, mut term) = wcstoul(s, *i, base);
        let value = val.min(0x7FFF_FFFF);
        if at(s, term) == b'0' as u32 {
            term += 1; // "0x" but nothing valid after (faithful; ~dead)
        } else if term == *i {
            return None;
        }
        *i = term;
        Some(value)
    }

    /// Result of `ipv6_string_to_address`.
    pub struct Ipv6Parse {
        pub status: i32,
        pub terminator: usize,
        pub addr: [u8; 16],
        pub scope: u32,
        pub port_net: u16,
    }

    /// Wine `ipv6_string_to_address` (RtlIpv6StringToAddress[Ex]W core).
    /// `ex` enables the `%scope` / `[…]:port` suffix and forbids the
    /// trailing-`0x` feature + requires full consumption.
    pub fn ipv6_string_to_address<C: IpChar>(s: &[C], ex: bool) -> Ipv6Parse {
        let mut addr = [0u8; 16];
        let err = |term: usize, addr: [u8; 16]| Ipv6Parse {
            status: STATUS_INVALID_PARAMETER,
            terminator: term,
            addr,
            scope: 0,
            port_net: 0,
        };

        let mut expecting_port = false;
        let mut has_0x = false;
        let mut too_big = false;
        let mut n_bytes = 0i32;
        let mut n_ipv4_bytes = 0i32;
        let mut gap = -1i32;
        let mut ip_component: u32;
        let mut i = 0usize;
        let mut prev;
        let mut goto_fill = false;
        let mut term = 0usize;

        let col = b':' as u32;
        let dot = b'.' as u32;

        if at(s, 0) == b'[' as u32 {
            if !ex {
                return err(0, addr);
            }
            expecting_port = true;
            i += 1;
        }
        if at(s, i) == col {
            if at(s, i + 1) != col {
                return err(i, addr);
            }
            i += 1; // leave one ':' for the double-colon detector
        }

        loop {
            if n_ipv4_bytes == 0 && at(s, i) == col {
                // double colon
                if gap != -1 {
                    return err(i, addr);
                }
                i += 1;
                prev = i;
                gap = n_bytes;
                if n_bytes == 14 {
                    break;
                }
                let mut j = i;
                if parse_ipv6_component(s, &mut j, 16).is_none() {
                    break;
                }
                i = prev; // rewind: re-parse the component normally below
            } else {
                prev = i;
            }

            if n_ipv4_bytes == 0 && n_bytes <= (if gap != -1 { 10 } else { 12 }) {
                let mut j = i;
                if parse_ipv6_component(s, &mut j, 10).is_some() && at(s, j) == dot {
                    n_ipv4_bytes = 1;
                }
                // str = prev_str (i unchanged; lookahead used `j`)
            }

            if n_ipv4_bytes != 0 {
                // IPv4 component
                match parse_ipv6_component(s, &mut i, 10) {
                    Some(v) => ip_component = v,
                    None => return err(i, addr),
                }
                if (i - prev) > 3 || ip_component > 255 {
                    too_big = true;
                } else {
                    if at(s, i) != dot && (n_ipv4_bytes < 4 || (n_bytes < 15 && gap == -1)) {
                        return err(i, addr);
                    }
                    addr[n_bytes as usize] = ip_component as u8;
                    n_bytes += 1;
                }
                if n_ipv4_bytes == 4 || at(s, i) != dot {
                    break;
                }
                n_ipv4_bytes += 1;
            } else {
                // IPv6 component
                match parse_ipv6_component(s, &mut i, 16) {
                    Some(v) => ip_component = v,
                    None => return err(i, addr),
                }
                if at(s, prev) == b'0' as u32
                    && (at(s, prev + 1) == b'x' as u32 || at(s, prev + 1) == b'X' as u32)
                {
                    // Windows "feature": trailing component may be "0x…"
                    term = prev + 1; // the 'x' is the terminator
                    if n_bytes < 14 && gap == -1 {
                        return err(prev + 1, addr);
                    }
                    set_word(&mut addr, (n_bytes / 2) as usize, ip_component);
                    n_bytes += 2;
                    has_0x = true;
                    goto_fill = true;
                    break;
                }
                if at(s, i) != col && n_bytes < 14 && gap == -1 {
                    return err(i, addr);
                }
                if (i - prev) > 4 {
                    too_big = true;
                } else {
                    set_word(&mut addr, (n_bytes / 2) as usize, ip_component);
                }
                n_bytes += 2;
                if at(s, i) != col || (gap != -1 && at(s, i + 1) == col) {
                    break;
                }
            }
            if n_bytes == (if gap != -1 { 14 } else { 16 }) {
                break;
            }
            if too_big {
                return err(i, addr); // C returns INVALID (terminator unset here)
            }
            i += 1;
        }

        if !goto_fill {
            term = i;
            if too_big {
                return err(i, addr);
            }
        }

        // fill_gap
        if gap == -1 {
            if n_bytes < 16 {
                return err(i, addr);
            }
        } else {
            let cnt = (n_bytes - gap) as usize;
            let dst = 16 - cnt;
            addr.copy_within(gap as usize..gap as usize + cnt, dst);
            let zlen = 16 - n_bytes as usize;
            for k in gap as usize..gap as usize + zlen {
                addr[k] = 0;
            }
        }

        let mut scope = 0u32;
        let mut port_net = 0u16;
        if ex {
            if has_0x {
                return err(i, addr);
            }
            if at(s, i) == b'%' as u32 {
                i += 1;
                match parse_ipv4_component(s, &mut i, true) {
                    Some(v) => scope = v,
                    None => return err(i, addr),
                }
            }
            if expecting_port {
                if at(s, i) != b']' as u32 {
                    return err(i, addr);
                }
                i += 1;
                if at(s, i) == col {
                    i += 1;
                    let p = match parse_ipv4_component(s, &mut i, false) {
                        Some(v) => v,
                        None => return err(i, addr),
                    };
                    if p == 0 || p > 0xFFFF || at(s, i) != 0 {
                        return err(i, addr);
                    }
                    port_net = (p as u16).swap_bytes();
                }
            }
            // ex passes terminator==NULL → require full consumption
            if at(s, i) != 0 {
                return err(i, addr);
            }
        }

        Ipv6Parse {
            status: STATUS_SUCCESS,
            terminator: if goto_fill { term } else { i },
            addr,
            scope,
            port_net,
        }
    }

    /// `RtlIpv6AddressToStringW` core — write the canonical text form of
    /// `addr` (NUL-terminated). Returns the char count (excl NUL). `out`
    /// must hold at least `RTLIPV6A2S_MAX_LEN` (46) chars.
    pub fn format_ipv6<C: IpChar>(a: &[u8; 16], out: &mut [C]) -> usize {
        let mut k = 0usize;
        let put = |out: &mut [C], k: &mut usize, b: u8| {
            out[*k] = C::from_ascii(b);
            *k += 1;
        };

        // IPv4-mapped/compatible form (RFC2765-ish)
        if word_native(a, 0) == 0
            && word_native(a, 1) == 0
            && word_native(a, 2) == 0
            && word_native(a, 3) == 0
            && word_native(a, 6) != 0
        {
            let prefix: Option<&[u8]> = if word_native(a, 4) == 0xffff && word_native(a, 5) == 0 {
                Some(b"ffff:0:")
            } else if word_native(a, 4) == 0 && word_native(a, 5) == 0xffff {
                Some(b"ffff:")
            } else if word_native(a, 4) == 0 && word_native(a, 5) == 0 {
                Some(b"")
            } else {
                None
            };
            if let Some(pfx) = prefix {
                put(out, &mut k, b':');
                put(out, &mut k, b':');
                for &b in pfx {
                    put(out, &mut k, b);
                }
                for (idx, &byte) in a[12..16].iter().enumerate() {
                    if idx != 0 {
                        put(out, &mut k, b'.');
                    }
                    k += write_u32(byte as u32, &mut out[k..]);
                }
                out[k] = C::from_ascii(0);
                return k;
            }
        }

        let mut parts = 8usize;
        // ISATAP
        if (word_native(a, 4) & 0xfffd) == 0 && word_native(a, 5) == 0xfe5e {
            parts = 6;
        }

        let mut skip_once = true;
        let mut n = 0usize;
        while n < parts {
            if skip_once && (n + 1) < parts && word_native(a, n) == 0 && word_native(a, n + 1) == 0 {
                skip_once = false;
                while (n + 1) < parts && word_native(a, n + 1) == 0 {
                    n += 1;
                }
                put(out, &mut k, b':');
                if (n + 1) >= parts {
                    put(out, &mut k, b':');
                }
            } else {
                if n != 0 {
                    put(out, &mut k, b':');
                }
                k += write_hex(word_host(a, n), &mut out[k..]);
            }
            n += 1;
        }

        if parts < 8 {
            put(out, &mut k, b':');
            for (idx, &byte) in a[12..16].iter().enumerate() {
                if idx != 0 {
                    put(out, &mut k, b'.');
                }
                k += write_u32(byte as u32, &mut out[k..]);
            }
            out[k] = C::from_ascii(0);
            return k;
        }

        out[k] = C::from_ascii(0);
        k
    }
}

// ───────────────────────── FFI helpers ───────────────────────────────
/// NUL-terminated `*const C` → content slice (NUL excluded). NULL → &[].
unsafe fn cstr<'a, C: pure::IpChar>(p: *const C) -> &'a [C] {
    if p.is_null() {
        return &[];
    }
    let mut n = 0usize;
    while (*p.add(n)).code() != 0 {
        n += 1;
    }
    core::slice::from_raw_parts(p, n)
}

// ───────────────────────── IPv4: address → string ────────────────────
/// `PSTR RtlIpv4AddressToStringA(const struct in_addr*, PCHAR S)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4AddressToStringA(addr: *const InAddr, s: *mut u8) -> *mut u8 {
    if s.is_null() {
        return usize::MAX as *mut u8;
    }
    let buf = core::slice::from_raw_parts_mut(s, IPV4_ADDR_STRING_MAX_LEN);
    let n = pure::format_address((*addr).s_addr.to_le_bytes(), buf);
    buf[n] = 0;
    s.add(n)
}

/// `PWSTR RtlIpv4AddressToStringW(const struct in_addr*, PWCHAR S)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4AddressToStringW(addr: *const InAddr, s: *mut u16) -> *mut u16 {
    if s.is_null() {
        return usize::MAX as *mut u16;
    }
    let buf = core::slice::from_raw_parts_mut(s, IPV4_ADDR_STRING_MAX_LEN);
    let n = pure::format_address((*addr).s_addr.to_le_bytes(), buf);
    buf[n] = 0;
    s.add(n)
}

/// Shared Ex formatter (narrow/wide): format into a stack buffer with
/// optional `:port`, copy to the caller buffer iff it fits + NUL.
unsafe fn ipv4_to_string_ex<C: pure::IpChar>(
    addr: *const InAddr,
    port: u16,
    out: *mut C,
    out_len: *mut u32,
) -> i32 {
    if addr.is_null() || out.is_null() || out_len.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let mut tmp = [C::from_ascii(0); IPV4_ADDR_STRING_MAX_LEN + IPV4_PORT_STRING_MAX_LEN];
    let mut n = pure::format_address((*addr).s_addr.to_le_bytes(), &mut tmp);
    if port != 0 {
        tmp[n] = C::from_ascii(b':');
        n += 1;
        n += pure::write_u32(wn2h(port) as u32, &mut tmp[n..]);
    }
    let cap = *out_len as usize;
    if cap > n {
        let dst = core::slice::from_raw_parts_mut(out, n + 1);
        dst[..n].copy_from_slice(&tmp[..n]);
        dst[n] = C::from_ascii(0);
        *out_len = (n + 1) as u32;
        STATUS_SUCCESS
    } else {
        *out_len = (n + 1) as u32;
        STATUS_INVALID_PARAMETER
    }
}

/// `NTSTATUS RtlIpv4AddressToStringExA(addr, port, AddressString, *len)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4AddressToStringExA(
    addr: *const InAddr,
    port: u16,
    out: *mut u8,
    out_len: *mut u32,
) -> i32 {
    ipv4_to_string_ex::<u8>(addr, port, out, out_len)
}

/// `NTSTATUS RtlIpv4AddressToStringExW(addr, port, AddressString, *len)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4AddressToStringExW(
    addr: *const InAddr,
    port: u16,
    out: *mut u16,
    out_len: *mut u32,
) -> i32 {
    ipv4_to_string_ex::<u16>(addr, port, out, out_len)
}

// ───────────────────────── IPv4: string → address ────────────────────
unsafe fn ipv4_string_to_address<C: pure::IpChar>(
    string: *const C,
    strict: u8,
    terminator: *mut *const C,
    addr: *mut InAddr,
) -> i32 {
    let s = cstr(string);
    let (res, term) = pure::string_to_address(s, strict != 0);
    if !terminator.is_null() {
        *terminator = string.add(term);
    }
    match res {
        Ok(be) => {
            (*addr).s_addr = be;
            STATUS_SUCCESS
        }
        Err(()) => STATUS_INVALID_PARAMETER,
    }
}

/// `NTSTATUS RtlIpv4StringToAddressA(PCSTR, BOOLEAN, PCSTR*, in_addr*)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4StringToAddressA(
    string: *const u8,
    strict: u8,
    terminator: *mut *const u8,
    addr: *mut InAddr,
) -> i32 {
    ipv4_string_to_address(string, strict, terminator, addr)
}

/// `NTSTATUS RtlIpv4StringToAddressW(PCWSTR, BOOLEAN, PCWSTR*, in_addr*)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4StringToAddressW(
    string: *const u16,
    strict: u8,
    terminator: *mut *const u16,
    addr: *mut InAddr,
) -> i32 {
    ipv4_string_to_address(string, strict, terminator, addr)
}

/// Shared Ex parser (narrow/wide): parse address then optional `:port`.
unsafe fn ipv4_string_to_address_ex<C: pure::IpChar>(
    string: *const C,
    strict: u8,
    addr: *mut InAddr,
    port: *mut u16,
) -> i32 {
    if string.is_null() || addr.is_null() || port.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let s = cstr(string);
    let (res, term) = pure::string_to_address(s, strict != 0);
    if res.is_err() {
        return STATUS_INVALID_PARAMETER;
    }
    (*addr).s_addr = res.unwrap();

    // remainder after the address
    if term >= s.len() {
        *port = 0;
        return STATUS_SUCCESS;
    }
    if s[term].code() != b':' as u32 {
        return STATUS_INVALID_PARAMETER;
    }
    match pure::parse_port(&s[term + 1..]) {
        Ok(p) => {
            *port = wn2h(p as u16);
            STATUS_SUCCESS
        }
        Err(()) => STATUS_INVALID_PARAMETER,
    }
}

/// `NTSTATUS RtlIpv4StringToAddressExA(PCSTR, BOOLEAN, in_addr*, PUSHORT)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4StringToAddressExA(
    string: *const u8,
    strict: u8,
    addr: *mut InAddr,
    port: *mut u16,
) -> i32 {
    ipv4_string_to_address_ex(string, strict, addr, port)
}

/// `NTSTATUS RtlIpv4StringToAddressExW(PCWSTR, BOOLEAN, in_addr*, PUSHORT)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv4StringToAddressExW(
    string: *const u16,
    strict: u8,
    addr: *mut InAddr,
    port: *mut u16,
) -> i32 {
    ipv4_string_to_address_ex(string, strict, addr, port)
}

// ───────────────────────── IPv6 ──────────────────────────────────────
/// max length of `RtlIpv6AddressToString` output (incl NUL). The Ex
/// form (scope + port) maxes at 65; the Ex shim uses an 80-char stack
/// buffer (comfortably above that) instead of a named constant.
const RTLIPV6A2S_MAX_LEN: usize = 46;

/// `struct in6_addr` / `IN6_ADDR` — `union { Byte[16]; Word[8]; }`,
/// 16 bytes in network order.
#[repr(C)]
pub struct In6Addr {
    pub bytes: [u8; 16],
}

/// `PWSTR RtlIpv6AddressToStringW(const in6_addr*, PWSTR S)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6AddressToStringW(addr: *const In6Addr, s: *mut u16) -> *mut u16 {
    if s.is_null() {
        return usize::MAX as *mut u16;
    }
    let buf = core::slice::from_raw_parts_mut(s, RTLIPV6A2S_MAX_LEN);
    let n = pure::format_ipv6(&(*addr).bytes, buf);
    s.add(n)
}

/// `PSTR RtlIpv6AddressToStringA(const in6_addr*, PSTR S)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6AddressToStringA(addr: *const In6Addr, s: *mut u8) -> *mut u8 {
    if s.is_null() {
        return usize::MAX as *mut u8;
    }
    let buf = core::slice::from_raw_parts_mut(s, RTLIPV6A2S_MAX_LEN);
    let n = pure::format_ipv6(&(*addr).bytes, buf);
    s.add(n)
}

/// Shared Ex formatter: optional `[`, address, optional `%scope`,
/// optional `]:port`; copy to caller iff it fits.
unsafe fn ipv6_to_string_ex<C: pure::IpChar>(
    addr: *const In6Addr,
    scope: u32,
    port: u16,
    out: *mut C,
    out_len: *mut u32,
) -> i32 {
    if addr.is_null() || out.is_null() || out_len.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    // generous vs the C's exact-65 stack buffer: max valid content is
    // 1 + 45 + 1 + 10(scope) + 2 + 5(port) = 64, so 80 never overflows.
    let mut buf = [C::from_ascii(0); 80];
    let mut k = 0usize;
    if port != 0 {
        buf[k] = C::from_ascii(b'[');
        k += 1;
    }
    k += pure::format_ipv6(&(*addr).bytes, &mut buf[k..]);
    if scope != 0 {
        buf[k] = C::from_ascii(b'%');
        k += 1;
        k += pure::write_u32(scope, &mut buf[k..]);
    }
    if port != 0 {
        buf[k] = C::from_ascii(b']');
        k += 1;
        buf[k] = C::from_ascii(b':');
        k += 1;
        k += pure::write_u32(wn2h(port) as u32, &mut buf[k..]);
    }
    let length = k;
    if (*out_len as usize) > length {
        let dst = core::slice::from_raw_parts_mut(out, length + 1);
        dst[..length].copy_from_slice(&buf[..length]);
        dst[length] = C::from_ascii(0);
        *out_len = (length + 1) as u32;
        STATUS_SUCCESS
    } else {
        *out_len = (length + 1) as u32;
        STATUS_INVALID_PARAMETER
    }
}

/// `NTSTATUS RtlIpv6AddressToStringExW(addr, scope, port, str, *len)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6AddressToStringExW(
    addr: *const In6Addr,
    scope: u32,
    port: u16,
    out: *mut u16,
    out_len: *mut u32,
) -> i32 {
    ipv6_to_string_ex::<u16>(addr, scope, port, out, out_len)
}

/// `NTSTATUS RtlIpv6AddressToStringExA(addr, scope, port, str, *len)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6AddressToStringExA(
    addr: *const In6Addr,
    scope: u32,
    port: u16,
    out: *mut u8,
    out_len: *mut u32,
) -> i32 {
    ipv6_to_string_ex::<u8>(addr, scope, port, out, out_len)
}

/// `NTSTATUS RtlIpv6StringToAddressW(PCWSTR, PCWSTR*, in6_addr*)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6StringToAddressW(
    string: *const u16,
    terminator: *mut *const u16,
    addr: *mut In6Addr,
) -> i32 {
    if string.is_null() || terminator.is_null() || addr.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let s = cstr(string);
    let r = pure::ipv6_string_to_address(s, false);
    *terminator = string.add(r.terminator);
    (*addr).bytes = r.addr;
    r.status
}

/// `NTSTATUS RtlIpv6StringToAddressA(PCSTR, PCSTR*, in6_addr*)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6StringToAddressA(
    string: *const u8,
    terminator: *mut *const u8,
    addr: *mut In6Addr,
) -> i32 {
    if string.is_null() || addr.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let s = cstr(string);
    let r = pure::ipv6_string_to_address(s, false);
    // C only writes the terminator when non-NULL ("Windows doesn't
    // always write it"); map the 1:1 ASCII index.
    if !terminator.is_null() {
        *terminator = string.add(r.terminator);
    }
    (*addr).bytes = r.addr;
    r.status
}

/// `NTSTATUS RtlIpv6StringToAddressExW(PCWSTR, in6_addr*, PULONG, PUSHORT)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6StringToAddressExW(
    string: *const u16,
    addr: *mut In6Addr,
    scope: *mut u32,
    port: *mut u16,
) -> i32 {
    if string.is_null() || addr.is_null() || scope.is_null() || port.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let s = cstr(string);
    let r = pure::ipv6_string_to_address(s, true);
    (*addr).bytes = r.addr;
    if r.status == STATUS_SUCCESS {
        *scope = r.scope;
        *port = r.port_net;
    }
    r.status
}

/// `NTSTATUS RtlIpv6StringToAddressExA(PCSTR, in6_addr*, PULONG, PUSHORT)`.
#[no_mangle]
pub unsafe extern "system" fn RtlIpv6StringToAddressExA(
    string: *const u8,
    addr: *mut In6Addr,
    scope: *mut u32,
    port: *mut u16,
) -> i32 {
    if string.is_null() || addr.is_null() || scope.is_null() || port.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let s = cstr(string);
    let r = pure::ipv6_string_to_address(s, true);
    (*addr).bytes = r.addr;
    if r.status == STATUS_SUCCESS {
        *scope = r.scope;
        *port = r.port_net;
    }
    r.status
}

#[cfg(test)]
mod tests;
