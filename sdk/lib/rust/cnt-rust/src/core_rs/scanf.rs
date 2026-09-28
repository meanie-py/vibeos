//! cnt-rust scanf engine — faithful port of the genuine ReactOS
//! `sdk/lib/crt/string/scanf.h` `_tscanf` engine, the `_LIBCNT_` +
//! `STRING`/`STRING_LEN` instantiations only (libcntpr has exactly
//! `sscanf`/`swscanf`/`_snscanf`/`_snwscanf`). Pure, `no_std`, NO
//! `unsafe`, NO varargs, NO raw pointers.
//!
//! The varargs ABI is irreducibly C and Rust cannot DEFINE a variadic
//! fn on stable (the `c_variadic` feature is nightly; this project's
//! discipline is "declare, never define" variadics). So the public
//! `sscanf`/… symbols are a tiny C trampoline (mirrors genuine
//! scanf.c, which is itself a 4-line `va_start; vsscanf_l(); va_end`
//! shell over the engine). The trampoline hands the engine a `va_list`
//! and a `Sink`; the `Sink` impl (in cabi, unsafe) performs each
//! `va_arg` via a C helper at the EXACT per-directive point the
//! genuine engine does, and writes the typed result. This module
//! never touches a pointer — it only tells the sink what to store.
//!
//! Phase 1 pins: integer (`%d %i %u %o %x %X %p`), string (`%s %S`),
//! char (`%c %C`), scanset (`%[`), `%n`, literal/`%%`, whitespace,
//! width, `*` suppression, `h l ll L w I64` prefixes — for `sscanf`
//! and `swscanf`. Float (`%e %f %g`) is parsed + the genuine
//! `_internal_handle_float` synthesis is wired, but is NOT fuzzed
//! until Phase 2 (the x87-`long double`-vs-`f64` parity question is
//! owed, tracked — never silently approximated).

/// Input/format element: `u8` (sscanf/_snscanf) or `u16` (swscanf/
/// _snwscanf). `WIDE` selects the `%s`/`%S`/`%c`/`%C` cross-width
/// table and the `_CHAR_` scanset target. Classification is ASCII
/// (the genuine `isspace`/`iswspace`/`isdigit`/`iswdigit` agree on
/// the fuzzed ASCII domain; non-ASCII is locale/codepage-dependent
/// and never fuzzed — documented domain limit, like radix.rs).
pub trait Elem: Copy + Eq {
    const WIDE: bool;
    fn to_u32(self) -> u32;
    fn is_space(self) -> bool {
        matches!(self.to_u32(), 0x20 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d)
    }
    fn is_digit(self) -> bool {
        (0x30..=0x39).contains(&self.to_u32())
    }
}
impl Elem for u8 {
    const WIDE: bool = false;
    fn to_u32(self) -> u32 {
        self as u32
    }
}
impl Elem for u16 {
    const WIDE: bool = true;
    fn to_u32(self) -> u32 {
        self as u32
    }
}

/// `char2digit`/`wchar2digit` (scanf.c:39-57) — value of `c` in
/// `base`, or `-1`. The genuine off-by quirks are preserved verbatim:
/// `c<='0'+base-1` for 0-9 (so `base<=10` only accepts digits) and
/// `c<='A'+base-11` / `c<='a'+base-11` for letters.
#[inline]
fn char2digit(c: u32, base: i32) -> i32 {
    let cc = c as i32;
    if (b'0' as i32..=b'9' as i32).contains(&cc) && cc <= b'0' as i32 + base - 1 {
        return cc - b'0' as i32;
    }
    if base <= 10 {
        return -1;
    }
    if (b'A' as i32..=b'Z' as i32).contains(&cc) && cc <= b'A' as i32 + base - 11 {
        return cc - b'A' as i32 + 10;
    }
    if (b'a' as i32..=b'z' as i32).contains(&cc) && cc <= b'a' as i32 + base - 11 {
        return cc - b'a' as i32 + 10;
    }
    -1
}

/// Integer target width (`_SET_NUMBER_` selector, scanf.h:301-304).
/// Windows `LONG`==32-bit; `int`==32; `short`==16; `LONGLONG`==64.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IntWidth {
    I64,
    Long,
    Short,
    Int,
}

/// String/char/scanset destination element width (the `char*` vs
/// `wchar_t*` the genuine `va_arg` fetches per the `%s/%S/%c/%C`
/// cross-width table; `%[` is always the engine-native `_CHAR_`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Narrow,
    Wide,
}

/// The cabi shim implements this over the C `va_list`. Methods are
/// invoked in the EXACT order / at the exact points the genuine C
/// calls `va_arg`/writes, so the va_list stays in lockstep:
///  * `begin_target` — at a string/char/scanset directive ENTRY when
///    not suppressed (genuine `sptr = va_arg(ap, T*)`).
///  * `push`/`terminate` — streamed during/after the conversion.
///  * `store_*` — at the genuine `_SET_NUMBER_`/`%n`/float store point
///    (after the conversion), only when not suppressed.
/// Suppressed directives invoke NONE of these (the engine still
/// consumes input identically — faithful to `suppress ? NULL : …`).
pub trait Sink {
    fn store_int(&mut self, w: IntWidth, value: u64);
    fn store_float(&mut self, negative: bool, exp: i32, d: u64, is_double: bool);
    fn store_n(&mut self, value: i32);
    fn begin_target(&mut self, kind: TargetKind);
    fn push(&mut self, e: u32);
    fn terminate(&mut self);
}

struct Cursor<'a, E: Elem> {
    input: &'a [E],
    has_len_limit: bool,
    consumed: i32,
    fpos: usize,
}
impl<'a, E: Elem> Cursor<'a, E> {
    /// `_GETC_`. STRING: `(consumed++, *file++)`, reads 0 at/after the
    /// implicit NUL. STRING_LEN: `consumed==length ? '\0' :
    /// (consumed++, *file++)` — at the limit yields 0 WITHOUT
    /// advancing or counting (the genuine quirk). `_EOF_`==0.
    fn getc(&mut self) -> u32 {
        if self.has_len_limit && self.consumed as usize == self.input.len() {
            return 0;
        }
        self.consumed += 1;
        let v = self.input.get(self.fpos).map(|e| e.to_u32()).unwrap_or(0);
        self.fpos += 1;
        v
    }
    // `_UNGETC_` (`file--; consumed--;`) is only ever the genuine
    // TRAILING push-back, which is unobservable on a string source
    // (it only adjusts `consumed`, read solely by `%n` which always
    // precedes it) — so it is intentionally not modeled.
}

/// Faithful `_tscanf`. `input` is the NUL-excluded content for STRING
/// (sscanf/swscanf) or exactly `length` elements for STRING_LEN
/// (_snscanf/_snwscanf, `has_len_limit=true`). Returns the genuine C
/// value: number of assigned fields, or the first-getc-EOF value
/// `_EOF_RET` (== `EOF` == -1 for narrow STRING; `(short)WEOF` == -1
/// for wide — both `-1`; `_EOF_` is redefined to 0 for STRING but
/// `_EOF_RET` is NOT, so empty input returns -1, NOT 0).
pub fn scanf_engine<E: Elem, S: Sink>(
    input: &[E],
    has_len_limit: bool,
    format: &[E],
    sink: &mut S,
) -> i32 {
    const EOF: u32 = 0; // nolint: const-contradiction (this is scanf.h:69 `#define _EOF_ 0` for STRING, not stdio EOF=-1)
    if format.is_empty() {
        return 0; // C: if (!*format) return 0;
    }
    let mut cur = Cursor { input, has_len_limit, consumed: 0, fpos: 0 };
    let mut rd: i32 = 0;
    let mut nch = cur.getc();
    if nch == EOF {
        return -1; // _EOF_RET (narrow EOF / wide (short)WEOF == -1)
    }

    let dot = b'.' as u32; // locinfo->lconv->decimal_point == "."
    let flen = format.len();
    let fch = |i: usize| -> u32 { format.get(i).map(|c| c.to_u32()).unwrap_or(0) };
    let mut fi = 0usize;

    while fi < flen {
        let fc = format[fi];
        if fc.is_space() {
            while nch != EOF && is_sp(nch) {
                nch = cur.getc();
            }
        } else if fc.to_u32() == b'%' as u32 {
            let mut st = 0i32;
            let mut suppress = false;
            let mut width: i32 = 0;
            let mut h_prefix = 0i32;
            let mut l_prefix = false;
            let mut big_l = false;
            let mut w_prefix = false;
            let mut i64_prefix = false;
            fi += 1;
            if fch(fi) == b'*' as u32 {
                fi += 1;
                suppress = true;
            }
            while fi < flen && format[fi].is_digit() {
                width = width * 10 + (fch(fi) as i32 - '0' as i32);
                fi += 1;
            }
            if width == 0 {
                width = -1; // no width spec (scanf.h:197)
            }
            let mut prefix_finished = false;
            while !prefix_finished {
                match fch(fi) {
                    0x68 => h_prefix += 1,        // 'h'
                    0x6c => {                     // 'l'
                        if fch(fi + 1) == 0x6c {
                            i64_prefix = true;
                            fi += 1;
                        }
                        l_prefix = true;
                    }
                    0x77 => w_prefix = true,      // 'w'
                    0x4c => big_l = true,         // 'L'
                    0x49 => {                     // 'I' (I64)
                        if fch(fi + 1) == 0x36 && fch(fi + 2) == 0x34 {
                            i64_prefix = true;
                            fi += 2;
                        }
                    }
                    _ => prefix_finished = true,
                }
                if !prefix_finished {
                    fi += 1;
                }
            }
            let ty = fch(fi);
            match ty {
                // ---- integer: p P x X o u d i ------------------------
                0x70 | 0x50 | 0x78 | 0x58 | 0x6f | 0x75 | 0x64 | 0x69 => {
                    let mut base: i32 = match ty {
                        0x70 | 0x50 | 0x78 | 0x58 => 16,
                        0x6f => 8,
                        0x75 | 0x64 => 10,
                        _ => 0, // 'i'
                    };
                    // %p/%P: I64 iff host pointer is 64-bit
                    // (genuine `sizeof(void*)==sizeof(LONGLONG)`).
                    if (ty == 0x70 || ty == 0x50)
                        && core::mem::size_of::<*const ()>() == core::mem::size_of::<i64>()
                    {
                        i64_prefix = true;
                    }
                    let is_p = ty == 0x70 || ty == 0x50;
                    let mut acc: u64 = 0;
                    let mut negative = false;
                    let mut seendigit = false;
                    while nch != EOF && is_sp(nch) {
                        nch = cur.getc();
                    }
                    if nch == b'-' as u32 || nch == b'+' as u32 {
                        negative = nch == b'-' as u32;
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                    }
                    if width != 0 && nch == b'0' as u32 && !is_p {
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                        if width != 0 && (nch == b'x' as u32 || nch == b'X' as u32) {
                            if base == 0 {
                                base = 16;
                            }
                            if base == 16 {
                                nch = cur.getc();
                                if width > 0 {
                                    width -= 1;
                                }
                                seendigit = false;
                            }
                        } else if base == 0 {
                            base = 8;
                        }
                    }
                    if base == 0 {
                        base = 10;
                    }
                    while width != 0 && nch == b'0' as u32 {
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                    }
                    if width != 0 && char2digit(nch, base) != -1 {
                        acc = char2digit(nch, base) as u64;
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                    }
                    while width != 0 && nch != EOF && char2digit(nch, base) != -1 {
                        acc = acc
                            .wrapping_mul(base as u64)
                            .wrapping_add(char2digit(nch, base) as u64);
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                    }
                    if !seendigit {
                        break; // not a valid number (matching failure)
                    }
                    st = 1;
                    if !suppress {
                        let v = if negative {
                            (acc as i64).wrapping_neg() as u64
                        } else {
                            acc
                        };
                        let w = if i64_prefix {
                            IntWidth::I64
                        } else if l_prefix {
                            IntWidth::Long
                        } else if h_prefix == 1 {
                            IntWidth::Short
                        } else {
                            IntWidth::Int
                        };
                        sink.store_int(w, v);
                    }
                }
                // ---- float: e E f g G (parsed; synth in sink) --------
                0x65 | 0x45 | 0x66 | 0x67 | 0x47 => {
                    let mut d: u64;
                    let mut exp: i32 = 0;
                    let mut negative = false;
                    while nch != EOF && is_sp(nch) {
                        nch = cur.getc();
                    }
                    if nch == b'-' as u32 || nch == b'+' as u32 {
                        negative = nch == b'-' as u32;
                        if width > 0 {
                            width -= 1;
                        }
                        if width == 0 {
                            break;
                        }
                        nch = cur.getc();
                    }
                    if dot != nch {
                        if !is_dig(nch) {
                            break;
                        }
                        d = (nch - b'0' as u32) as u64;
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        while width != 0 && nch != EOF && is_dig(nch) {
                            let hlp = d.wrapping_mul(10).wrapping_add((nch - b'0' as u32) as u64);
                            nch = cur.getc();
                            if width > 0 {
                                width -= 1;
                            }
                            if d > u64::MAX / 10 || hlp < d {
                                exp += 1;
                                break;
                            } else {
                                d = hlp;
                            }
                        }
                        while width != 0 && nch != EOF && is_dig(nch) {
                            exp += 1;
                            nch = cur.getc();
                            if width > 0 {
                                width -= 1;
                            }
                        }
                    } else {
                        d = 0;
                    }
                    if width != 0 && nch == dot {
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        while width != 0 && nch != EOF && is_dig(nch) {
                            let hlp = d.wrapping_mul(10).wrapping_add((nch - b'0' as u32) as u64);
                            nch = cur.getc();
                            if width > 0 {
                                width -= 1;
                            }
                            if d > u64::MAX / 10 || hlp < d {
                                break;
                            }
                            d = hlp;
                            exp -= 1;
                        }
                        while width != 0 && nch != EOF && is_dig(nch) {
                            nch = cur.getc();
                            if width > 0 {
                                width -= 1;
                            }
                        }
                    }
                    if width != 0 && (nch == b'e' as u32 || nch == b'E' as u32) {
                        let mut sign = 1i32;
                        let mut e = 0i32;
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        if width != 0 && (nch == b'+' as u32 || nch == b'-' as u32) {
                            if nch == b'-' as u32 {
                                sign = -1;
                            }
                            nch = cur.getc();
                            if width > 0 {
                                width -= 1;
                            }
                        }
                        while width != 0 && nch != EOF && is_dig(nch) {
                            // C short-circuit: `if(e > INT_MAX/10 ||
                            // (e = e*10+d) < 0) e = INT_MAX;` — the
                            // multiply is NOT evaluated once e exceeds
                            // INT_MAX/10. Signed overflow is UB in C
                            // (gcc wraps); model with wrapping.
                            if e > i32::MAX / 10 {
                                e = i32::MAX;
                            } else {
                                e = e
                                    .wrapping_mul(10)
                                    .wrapping_add((nch - b'0' as u32) as i32);
                                if e < 0 {
                                    e = i32::MAX;
                                }
                            }
                            nch = cur.getc();
                            if width > 0 {
                                width -= 1;
                            }
                        }
                        e = e.wrapping_mul(sign);
                        // C `e+exp` is signed int (UB on overflow; gcc
                        // wraps) — wrapping_add keeps debug from
                        // panicking while staying byte-faithful.
                        if exp < 0 && e < 0 && e.wrapping_add(exp) > 0 {
                            exp = i32::MIN;
                        } else if exp > 0 && e > 0 && e.wrapping_add(exp) < 0 {
                            exp = i32::MAX;
                        } else {
                            exp = exp.wrapping_add(e);
                        }
                    }
                    st = 1;
                    if !suppress {
                        sink.store_float(negative, exp, d, l_prefix || big_l);
                    }
                }
                // ---- string: s S ------------------------------------
                0x73 | 0x53 => {
                    let kind = str_kind(ty, w_prefix, l_prefix, h_prefix, E::WIDE);
                    if !suppress {
                        sink.begin_target(kind);
                    }
                    while nch != EOF && is_sp(nch) {
                        nch = cur.getc();
                    }
                    while width != 0 && nch != EOF && !is_sp(nch) {
                        if !suppress {
                            sink.push(nch);
                        }
                        st += 1;
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                    }
                    if st != 0 && !suppress {
                        sink.terminate();
                    }
                }
                // ---- char: c C --------------------------------------
                0x63 | 0x43 => {
                    let kind = chr_kind(ty, w_prefix, l_prefix, h_prefix, E::WIDE);
                    if !suppress {
                        sink.begin_target(kind);
                    }
                    if width == -1 {
                        width = 1;
                    }
                    while width != 0 && nch != EOF {
                        if !suppress {
                            sink.push(nch);
                        }
                        st += 1;
                        width -= 1;
                        nch = cur.getc();
                    }
                    // NO terminate (genuine: %c never NUL-terminates)
                }
                // ---- %n ---------------------------------------------
                0x6e => {
                    if !suppress {
                        sink.store_n(cur.consumed - 1);
                    }
                    suppress = true; // does not count toward rd
                    st = 1;
                }
                // ---- scanset %[ -------------------------------------
                0x5b => {
                    let kind = if E::WIDE { TargetKind::Wide } else { TargetKind::Narrow };
                    if !suppress {
                        sink.begin_target(kind);
                    }
                    let mut bits = [0u64; 1024]; // 65536 bits (256 narrow)
                    let bsize: u32 = if E::WIDE { 256 * 256 } else { 256 };
                    let mut set = |idx: u32, count: u32| {
                        let mut k = idx;
                        let endk = idx.saturating_add(count).min(bsize);
                        while k < endk {
                            bits[(k >> 6) as usize] |= 1u64 << (k & 63);
                            k += 1;
                        }
                    };
                    let mut invert = false;
                    fi += 1; // past '['
                    if fch(fi) == b'^' as u32 {
                        invert = true;
                        fi += 1;
                    }
                    if fch(fi) == b']' as u32 {
                        set(b']' as u32, 1);
                        fi += 1;
                    }
                    while fi < flen && fch(fi) != 0 && fch(fi) != b']' as u32 {
                        if fch(fi) == b'-' as u32 && fch(fi + 1) != b']' as u32 {
                            let lo = fch(fi - 1);
                            let hi = fch(fi + 1);
                            if lo < hi {
                                set(lo + 1, hi - lo);
                            } else {
                                set(hi, lo - hi);
                            }
                            fi += 1; // past '-'
                        } else {
                            set(fch(fi), 1);
                        }
                        fi += 1;
                    }
                    // fi now at ']' (or end); the loop-bottom fi+=1
                    // advances past it (genuine outer format++).
                    let is_set = |c: u32| -> bool {
                        c < bsize && (bits[(c >> 6) as usize] >> (c & 63)) & 1 == 1
                    };
                    while width != 0 && nch != EOF {
                        let hit = if invert { !is_set(nch) } else { is_set(nch) };
                        if hit {
                            if !suppress {
                                sink.push(nch);
                            }
                        } else {
                            break;
                        }
                        st += 1;
                        nch = cur.getc();
                        if width > 0 {
                            width -= 1;
                        }
                    }
                    if !suppress {
                        sink.terminate(); // %[ ALWAYS terminates
                    }
                }
                // ---- default (incl. %%) -----------------------------
                _ => {
                    while nch != EOF && is_sp(nch) {
                        nch = cur.getc();
                    }
                    if nch == ty {
                        suppress = true;
                        st = 1;
                        nch = cur.getc();
                    }
                }
            }
            if st != 0 && !suppress {
                rd += 1;
            } else if st == 0 {
                break;
            }
        } else {
            // literal non-ws: must match, else stop
            if nch == fc.to_u32() {
                nch = cur.getc();
            } else {
                break;
            }
        }
        fi += 1;
    }
    // genuine trailing `_UNGETC_` is unobservable on a string source
    // (only adjusts `consumed`, read solely by %n which precedes it).
    rd
}

#[inline]
fn is_sp(c: u32) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d)
}
#[inline]
fn is_dig(c: u32) -> bool {
    (0x30..=0x39).contains(&c)
}

/// `%s`/`%S` target width (scanf.h:449-464). `w`/`l` force wide, `h`
/// forces narrow; otherwise `%s` is engine-native and `%S` is the
/// opposite.
fn str_kind(ty: u32, w: bool, l: bool, h: i32, wide_engine: bool) -> TargetKind {
    if w || l {
        TargetKind::Wide
    } else if h != 0 {
        TargetKind::Narrow
    } else {
        let s_is_native = ty == 0x73; // 's'
        let native_wide = wide_engine;
        let wide = if s_is_native { native_wide } else { !native_wide };
        if wide { TargetKind::Wide } else { TargetKind::Narrow }
    }
}
/// `%c`/`%C` target width (scanf.h:527-542). Same rule shape as `%s`.
fn chr_kind(ty: u32, w: bool, l: bool, h: i32, wide_engine: bool) -> TargetKind {
    if w || l {
        TargetKind::Wide
    } else if h != 0 {
        TargetKind::Narrow
    } else {
        let c_is_native = ty == 0x63; // 'c'
        let native_wide = wide_engine;
        let wide = if c_is_native { native_wide } else { !native_wide };
        if wide { TargetKind::Wide } else { TargetKind::Narrow }
    }
}
