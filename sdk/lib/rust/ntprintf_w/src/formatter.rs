//! Wide format-string evaluator. Mirrors `_vsnwprintf` for the same
//! Windows-flavored printf subset as the ANSI sibling, but every emit
//! goes through a `WriterW` (one `u16` per char, not one byte).
//!
//! Two responsibilities:
//!  - Parse each `%…` specifier (re-walking the same wide format string the
//!    scanner saw) and convert the matching `slots[i]` into wide chars.
//!  - Emit those chars through a [`WriterW`] sink. Two sinks ship in this
//!    crate: [`BoundedBufW`] for fixed C `WCHAR` buffers, and a `Vec<u16>`
//!    sink for tests.
//!
//! ASCII bytes (digits, `0123456789abcdefABCDEF`, `(null)`, sign chars) are
//! emitted by widening to `u16`; non-ASCII codepoints inside literal text
//! are passed through unchanged. Inside a `%c`/`%C` spec, the rendering
//! follows MS rules (`%c` = wchar in W fmt, `%C` = single-byte char).

/// Sink for formatted wide chars. Implementations track total chars
/// "written" (including chars dropped because the underlying buffer
/// filled up) so the FFI return value matches `_vsnwprintf` semantics.
pub trait WriterW {
    /// Append wide chars. The implementation MUST update its written
    /// counter even if it drops chars due to capacity.
    fn write_chars(&mut self, chars: &[u16]);

    /// Total wide chars the sink has been asked to write (including dropped).
    fn written(&self) -> usize;
}

/// Bounded WCHAR buffer — like `vsnwprintf`'s `(dst, cap)` pair where
/// `cap` is **wide-character count**, not bytes. NUL-terminates at the
/// truncation boundary if `cap > 0`.
pub struct BoundedBufW {
    dst: *mut u16,
    cap: usize,
    /// Position in `dst` where the next *kept* wide char will land.
    pos: usize,
    /// Total wide chars the formatter has emitted (incl. dropped past `cap`).
    total: usize,
}

impl BoundedBufW {
    /// `dst` may be null only if `cap == 0`.
    pub fn new(dst: *mut u16, cap: usize) -> Self {
        Self { dst, cap, pos: 0, total: 0 }
    }

    /// Place a NUL wchar at the truncation boundary. Safe to call multiple
    /// times.
    pub fn terminate(&mut self) {
        if self.cap == 0 || self.dst.is_null() {
            return;
        }
        let nul_pos = if self.pos >= self.cap { self.cap - 1 } else { self.pos };
        unsafe { *self.dst.add(nul_pos) = 0; }
    }
}

impl WriterW for BoundedBufW {
    fn write_chars(&mut self, chars: &[u16]) {
        self.total += chars.len();
        if self.dst.is_null() || self.cap == 0 {
            return;
        }
        // Reserve one wchar for terminator → never write into pos == cap-1.
        let max_pos = self.cap.saturating_sub(1);
        let mut i = 0;
        while i < chars.len() && self.pos < max_pos {
            unsafe { *self.dst.add(self.pos) = chars[i]; }
            self.pos += 1;
            i += 1;
        }
    }

    fn written(&self) -> usize { self.total }
}

/// Test-only sink that wraps a `Vec<u16>`.
#[cfg(test)]
pub struct VecBufW<'a>(pub &'a mut Vec<u16>, pub usize);
#[cfg(test)]
impl<'a> VecBufW<'a> {
    pub fn new(v: &'a mut Vec<u16>) -> Self { Self(v, 0) }
}
#[cfg(test)]
impl<'a> WriterW for VecBufW<'a> {
    fn write_chars(&mut self, chars: &[u16]) {
        self.0.extend_from_slice(chars);
        self.1 += chars.len();
    }
    fn written(&self) -> usize { self.1 }
}

// ---------------------------------------------------------------------------
// Format-spec parsing (mirror of ANSI but reading `&[u16]`).

#[derive(Default, Debug, Clone, Copy)]
struct Spec {
    left_align: bool,
    plus: bool,
    space: bool,
    alt: bool,        // # flag
    zero_pad: bool,
    width: i32,       // -1 = unset
    precision: i32,   // -1 = unset
    is64: bool,
    is_long: bool,    // l modifier
    is_short: bool,   // h modifier
    is_wide_str: bool, // w modifier (UNICODE_STRING)
    conv: u8,
}

impl Spec {
    const fn empty() -> Self {
        Self {
            left_align: false, plus: false, space: false, alt: false,
            zero_pad: false, width: -1, precision: -1,
            is64: false, is_long: false, is_short: false, is_wide_str: false,
            conv: 0,
        }
    }
}

#[inline]
fn ascii(fmt: &[u16], p: usize) -> Option<u8> {
    let c = fmt[p];
    if c <= 0x7F { Some(c as u8) } else { None }
}

/// Parse one specifier starting just past the `%`.
/// Returns `(spec, end-of-specifier index, slots_consumed)`.
fn parse_spec(fmt: &[u16], start: usize, slots: &[u64], next_slot: usize) -> (Spec, usize, usize) {
    let mut spec = Spec::empty();
    let mut p = start;
    let mut consumed = 0usize;

    // Flags
    while p < fmt.len() {
        match ascii(fmt, p) {
            Some(b'-') => spec.left_align = true,
            Some(b'+') => spec.plus = true,
            Some(b' ') => spec.space = true,
            Some(b'#') => spec.alt = true,
            Some(b'0') => spec.zero_pad = true,
            _ => break,
        }
        p += 1;
    }
    // Width
    if p < fmt.len() && ascii(fmt, p) == Some(b'*') {
        let v = slots.get(next_slot + consumed).copied().unwrap_or(0) as i32;
        if v < 0 {
            spec.left_align = true;
            spec.width = v.wrapping_neg();
        } else {
            spec.width = v;
        }
        consumed += 1;
        p += 1;
    } else {
        let mut have = false;
        let mut w: i32 = 0;
        while p < fmt.len() {
            match ascii(fmt, p) {
                Some(b) if b.is_ascii_digit() => {
                    w = w.saturating_mul(10).saturating_add((b - b'0') as i32);
                    p += 1;
                    have = true;
                }
                _ => break,
            }
        }
        if have { spec.width = w; }
    }
    // Precision
    if p < fmt.len() && ascii(fmt, p) == Some(b'.') {
        p += 1;
        if p < fmt.len() && ascii(fmt, p) == Some(b'*') {
            let v = slots.get(next_slot + consumed).copied().unwrap_or(0) as i32;
            spec.precision = if v < 0 { 0 } else { v };
            consumed += 1;
            p += 1;
        } else {
            let mut prec: i32 = 0;
            while p < fmt.len() {
                match ascii(fmt, p) {
                    Some(b) if b.is_ascii_digit() => {
                        prec = prec.saturating_mul(10).saturating_add((b - b'0') as i32);
                        p += 1;
                    }
                    _ => break,
                }
            }
            spec.precision = prec;
        }
    }
    // Length modifier
    while p < fmt.len() {
        match ascii(fmt, p) {
            Some(b'h') => {
                p += 1;
                spec.is_short = true;
                if p < fmt.len() && ascii(fmt, p) == Some(b'h') { p += 1; }
            }
            Some(b'l') => {
                p += 1;
                spec.is_long = true;
                if p < fmt.len() && ascii(fmt, p) == Some(b'l') { p += 1; spec.is64 = true; }
            }
            Some(b'L') | Some(b'q') | Some(b'j') | Some(b'z') | Some(b't') => { p += 1; }
            Some(b'I') => {
                p += 1;
                if p + 1 < fmt.len()
                    && ascii(fmt, p) == Some(b'6') && ascii(fmt, p + 1) == Some(b'4')
                {
                    p += 2; spec.is64 = true;
                } else if p + 1 < fmt.len()
                    && ascii(fmt, p) == Some(b'3') && ascii(fmt, p + 1) == Some(b'2')
                {
                    p += 2;
                }
            }
            Some(b'w') => { p += 1; spec.is_wide_str = true; }
            _ => break,
        }
    }
    if p < fmt.len() {
        if let Some(b) = ascii(fmt, p) {
            spec.conv = b;
            p += 1;
            if spec.conv != b'%' {
                consumed += 1;
            }
        }
    }
    (spec, p, consumed)
}

/// Format `fmt` against `slots`, write result through `out`. Returns the total
/// wide chars the formatter emitted (matches `_vsnwprintf` return).
pub fn format_into<W: WriterW>(out: &mut W, fmt: &[u16], slots: &[u64]) -> usize {
    let mut p = 0usize;
    let mut next_slot = 0usize;
    while p < fmt.len() {
        if fmt[p] != b'%' as u16 {
            // Run literal chars until next '%'
            let start = p;
            while p < fmt.len() && fmt[p] != b'%' as u16 { p += 1; }
            out.write_chars(&fmt[start..p]);
            continue;
        }
        p += 1;
        if p >= fmt.len() {
            // dangling '%' — emit literally
            out.write_chars(&[b'%' as u16]);
            break;
        }
        let (spec, end, consumed) = parse_spec(fmt, p, slots, next_slot);
        p = end;
        if spec.conv == 0 {
            break;
        }
        if spec.conv == b'%' {
            out.write_chars(&[b'%' as u16]);
            continue;
        }
        let value_idx = next_slot + consumed - 1;
        let value = slots.get(value_idx).copied().unwrap_or(0);
        next_slot += consumed;

        emit_spec(out, &spec, value);
    }
    out.written()
}

/// Emit one specifier's value through `out`.
fn emit_spec<W: WriterW>(out: &mut W, spec: &Spec, value: u64) {
    match spec.conv {
        b'd' | b'i' => {
            let v = if spec.is64 { value as i64 } else { value as i32 as i64 };
            emit_signed_int(out, spec, v);
        }
        b'u' => {
            let v = if spec.is64 { value } else { value as u32 as u64 };
            emit_unsigned_int(out, spec, v, 10, false, b"");
        }
        b'o' => {
            let v = if spec.is64 { value } else { value as u32 as u64 };
            let prefix: &[u8] = if spec.alt && v != 0 { b"0" } else { b"" };
            emit_unsigned_int(out, spec, v, 8, false, prefix);
        }
        b'x' => {
            let v = if spec.is64 { value } else { value as u32 as u64 };
            let prefix: &[u8] = if spec.alt && v != 0 { b"0x" } else { b"" };
            emit_unsigned_int(out, spec, v, 16, false, prefix);
        }
        b'X' => {
            let v = if spec.is64 { value } else { value as u32 as u64 };
            let prefix: &[u8] = if spec.alt && v != 0 { b"0X" } else { b"" };
            emit_unsigned_int(out, spec, v, 16, true, prefix);
        }
        b'c' => {
            // W-fmt %c — wchar by default; %hc → char (low byte widened to wchar).
            let wc = if spec.is_short { (value as u8) as u16 } else { value as u16 };
            emit_padded_chars(out, spec, &[wc]);
        }
        b'C' => {
            // W-fmt %C — char (low byte widened); %lC → wchar.
            let wc = if spec.is_long { value as u16 } else { (value as u8) as u16 };
            emit_padded_chars(out, spec, &[wc]);
        }
        b's' => {
            // W-fmt %s — wide string by default; %hs → ansi string.
            if spec.is_short {
                emit_ansi_string(out, spec, value as usize as *const u8);
            } else {
                emit_wide_string(out, spec, value as usize as *const u16);
            }
        }
        b'S' => {
            // W-fmt %S — ANSI string (the inverse of A-fmt %S=wide).
            if spec.is_long {
                emit_wide_string(out, spec, value as usize as *const u16);
            } else {
                emit_ansi_string(out, spec, value as usize as *const u8);
            }
        }
        b'Z' => {
            if spec.is_wide_str {
                emit_unicode_string(out, spec, value as usize as *const UnicodeStringSlim);
            } else {
                // %Z — STRING* (ANSI counterpart, rare in W fmt)
                emit_ansi_string_struct(out, spec, value as usize as *const AnsiStringSlim);
            }
        }
        b'p' => {
            let mut tmp_spec = *spec;
            tmp_spec.alt = false;
            tmp_spec.zero_pad = true;
            #[cfg(target_pointer_width = "64")]
            { tmp_spec.width = 16; tmp_spec.is64 = true; }
            #[cfg(target_pointer_width = "32")]
            { tmp_spec.width = 8; tmp_spec.is64 = false; }
            let v = if tmp_spec.is64 { value } else { value as u32 as u64 };
            emit_unsigned_int(out, &tmp_spec, v, 16, false, b"");
        }
        _ => {
            out.write_chars(&[b'%' as u16, spec.conv as u16]);
        }
    }
}

/// Match the C UNICODE_STRING layout (Length, MaximumLength, Buffer).
#[repr(C)]
pub struct UnicodeStringSlim {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *const u16,
}

/// Match the C STRING/ANSI_STRING layout.
#[repr(C)]
pub struct AnsiStringSlim {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *const u8,
}

// ---------------------------------------------------------------------------
// Helpers.

/// Emit a buffer of ASCII bytes by widening each one to a u16, applying width
/// padding using ASCII space.
fn emit_padded_ascii<W: WriterW>(out: &mut W, spec: &Spec, body: &[u8]) {
    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(body.len());
    if pad > 0 && !spec.left_align {
        emit_repeat(out, b' ' as u16, pad);
    }
    // Widen the body byte-by-byte into u16 in chunks.
    const CHUNK: usize = 32;
    let mut buf = [0u16; CHUNK];
    let mut i = 0;
    while i < body.len() {
        let take = (body.len() - i).min(CHUNK);
        for j in 0..take { buf[j] = body[i + j] as u16; }
        out.write_chars(&buf[..take]);
        i += take;
    }
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ' as u16, pad);
    }
}

/// Emit a buffer of u16 chars (already wide), applying width padding.
fn emit_padded_chars<W: WriterW>(out: &mut W, spec: &Spec, body: &[u16]) {
    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(body.len());
    if pad > 0 && !spec.left_align {
        emit_repeat(out, b' ' as u16, pad);
    }
    out.write_chars(body);
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ' as u16, pad);
    }
}

fn emit_signed_int<W: WriterW>(out: &mut W, spec: &Spec, v: i64) {
    let mut buf = [0u8; 24];
    let mut len = 0;
    let abs: u64 = if v < 0 { (v as i128).wrapping_neg() as u64 } else { v as u64 };
    if abs == 0 {
        if spec.precision != 0 {
            buf[0] = b'0'; len = 1;
        }
    } else {
        let mut n = abs;
        while n > 0 {
            buf[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
    }
    let sign: &[u8] = if v < 0 { b"-" } else if spec.plus { b"+" } else if spec.space { b" " } else { b"" };

    let digits_len = len;
    let prec = if spec.precision >= 0 { spec.precision as usize } else { 0 };
    let total_digits = digits_len.max(prec);
    let body_len = sign.len() + total_digits;

    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(body_len);

    let zero_pad = spec.zero_pad && spec.precision < 0 && !spec.left_align;

    if pad > 0 && !spec.left_align && !zero_pad {
        emit_repeat(out, b' ' as u16, pad);
    }
    write_ascii(out, sign);
    if zero_pad && pad > 0 {
        emit_repeat(out, b'0' as u16, pad);
    }
    if total_digits > digits_len {
        emit_repeat(out, b'0' as u16, total_digits - digits_len);
    }
    let mut i = digits_len;
    while i > 0 {
        i -= 1;
        out.write_chars(&[buf[i] as u16]);
    }
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ' as u16, pad);
    }
}

fn emit_unsigned_int<W: WriterW>(out: &mut W, spec: &Spec, v: u64, base: u32, upper: bool, prefix: &[u8]) {
    let mut buf = [0u8; 24];
    let mut len = 0;
    let digits = if upper { b"0123456789ABCDEF" } else { b"0123456789abcdef" };

    if v == 0 {
        if spec.precision != 0 {
            buf[0] = b'0'; len = 1;
        }
    } else {
        let mut n = v;
        while n > 0 {
            buf[len] = digits[(n % base as u64) as usize];
            n /= base as u64;
            len += 1;
        }
    }

    let prec = if spec.precision >= 0 { spec.precision as usize } else { 0 };
    let total_digits = len.max(prec);
    let body_len = prefix.len() + total_digits;
    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(body_len);

    let zero_pad = spec.zero_pad && spec.precision < 0 && !spec.left_align;

    if pad > 0 && !spec.left_align && !zero_pad {
        emit_repeat(out, b' ' as u16, pad);
    }
    write_ascii(out, prefix);
    if zero_pad && pad > 0 {
        emit_repeat(out, b'0' as u16, pad);
    }
    if total_digits > len {
        emit_repeat(out, b'0' as u16, total_digits - len);
    }
    let mut i = len;
    while i > 0 {
        i -= 1;
        out.write_chars(&[buf[i] as u16]);
    }
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ' as u16, pad);
    }
}

fn emit_ansi_string<W: WriterW>(out: &mut W, spec: &Spec, ptr: *const u8) {
    if ptr.is_null() {
        emit_padded_ascii(out, spec, b"(null)");
        return;
    }
    let max_scan = if spec.precision >= 0 { spec.precision as usize } else { 0x10000 };
    let mut n = 0usize;
    unsafe {
        while n < max_scan {
            if *ptr.add(n) == 0 { break; }
            n += 1;
        }
    }
    let body = unsafe { core::slice::from_raw_parts(ptr, n) };
    emit_padded_ascii(out, spec, body);
}

fn emit_wide_string<W: WriterW>(out: &mut W, spec: &Spec, ptr: *const u16) {
    if ptr.is_null() {
        emit_padded_ascii(out, spec, b"(null)");
        return;
    }
    let max_scan = if spec.precision >= 0 { spec.precision as usize } else { 0x10000 };
    let mut n = 0usize;
    unsafe {
        while n < max_scan {
            if *ptr.add(n) == 0 { break; }
            n += 1;
        }
    }
    let body = unsafe { core::slice::from_raw_parts(ptr, n) };
    emit_padded_chars(out, spec, body);
}

fn emit_unicode_string<W: WriterW>(out: &mut W, spec: &Spec, p: *const UnicodeStringSlim) {
    if p.is_null() {
        emit_padded_ascii(out, spec, b"(null)");
        return;
    }
    let s = unsafe { &*p };
    let nchars = (s.length as usize) / 2;
    if s.buffer.is_null() || nchars == 0 {
        emit_padded_chars(out, spec, &[]);
        return;
    }
    let body = unsafe { core::slice::from_raw_parts(s.buffer, nchars) };
    emit_padded_chars(out, spec, body);
}

fn emit_ansi_string_struct<W: WriterW>(out: &mut W, spec: &Spec, p: *const AnsiStringSlim) {
    if p.is_null() {
        emit_padded_ascii(out, spec, b"(null)");
        return;
    }
    let s = unsafe { &*p };
    if s.buffer.is_null() || s.length == 0 {
        emit_padded_ascii(out, spec, b"");
        return;
    }
    let body = unsafe { core::slice::from_raw_parts(s.buffer, s.length as usize) };
    emit_padded_ascii(out, spec, body);
}

/// Widen ASCII bytes inline and write them.
#[inline]
fn write_ascii<W: WriterW>(out: &mut W, body: &[u8]) {
    if body.is_empty() { return; }
    const CHUNK: usize = 32;
    let mut buf = [0u16; CHUNK];
    let mut i = 0;
    while i < body.len() {
        let take = (body.len() - i).min(CHUNK);
        for j in 0..take { buf[j] = body[i + j] as u16; }
        out.write_chars(&buf[..take]);
        i += take;
    }
}

fn emit_repeat<W: WriterW>(out: &mut W, ch: u16, n: usize) {
    let chunk = [ch; 32];
    let mut left = n;
    while left >= 32 {
        out.write_chars(&chunk);
        left -= 32;
    }
    if left > 0 {
        out.write_chars(&chunk[..left]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Widen ASCII bytes into a Vec<u16> for tests.
    fn wstr(s: &[u8]) -> Vec<u16> { s.iter().map(|&b| b as u16).collect() }

    /// Convert wide-output back to a String for comparison (assumes ASCII-only output).
    fn to_string(buf: &[u16]) -> String {
        buf.iter().map(|&c| c as u8 as char).collect()
    }

    fn fmt(fmt_bytes: &[u8], slots: &[u64]) -> String {
        let f = wstr(fmt_bytes);
        let mut out: Vec<u16> = Vec::new();
        let mut w = VecBufW::new(&mut out);
        format_into(&mut w, &f, slots);
        to_string(&out)
    }

    #[test]
    fn literals() {
        assert_eq!(fmt(b"hello world", &[]), "hello world");
        assert_eq!(fmt(b"100%%", &[]), "100%");
        assert_eq!(fmt(b"", &[]), "");
    }

    #[test]
    fn decimals() {
        assert_eq!(fmt(b"%d", &[42]), "42");
        assert_eq!(fmt(b"%d", &[(-42i32) as u32 as u64]), "-42");
        assert_eq!(fmt(b"%u", &[0xFFFFFFFF]), "4294967295");
    }

    #[test]
    fn hex_and_octal() {
        assert_eq!(fmt(b"%x", &[0xdeadbeef]), "deadbeef");
        assert_eq!(fmt(b"%X", &[0xdeadbeef]), "DEADBEEF");
        assert_eq!(fmt(b"%#x", &[0x42]), "0x42");
        assert_eq!(fmt(b"%o", &[8]), "10");
    }

    #[test]
    fn width_and_padding() {
        assert_eq!(fmt(b"%5d", &[42]), "   42");
        assert_eq!(fmt(b"%-5d|", &[42]), "42   |");
        assert_eq!(fmt(b"%05d", &[42]), "00042");
        assert_eq!(fmt(b"%08x", &[0xab]), "000000ab");
        assert_eq!(fmt(b"%03ld", &[7]), "007");
    }

    #[test]
    fn long_long_and_i64() {
        assert_eq!(fmt(b"%lld", &[0x1_0000_0001]), "4294967297");
        assert_eq!(fmt(b"%I64x", &[0x12345678abcdef00]), "12345678abcdef00");
    }

    #[test]
    fn percent_chars_W_semantics() {
        // W-fmt %c is wchar (the slot is u32-promoted)
        assert_eq!(fmt(b"%c", &[b'A' as u64]), "A");
        // %hc — 8-bit char (low byte)
        assert_eq!(fmt(b"%hc", &[b'X' as u64]), "X");
    }

    #[test]
    fn percent_s_W_semantics() {
        // %s in W fmt → wide string
        let buf: [u16; 5] = [b'H' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b'o' as u16];
        let nul: [u16; 1] = [0];
        let mut full: Vec<u16> = buf.to_vec();
        full.extend_from_slice(&nul);
        let slot = full.as_ptr() as usize as u64;
        assert_eq!(fmt(b"%s", &[slot]), "Hello");

        // %hs in W fmt → narrow string (ANSI)
        let ansi = b"World\0";
        let aslot = ansi.as_ptr() as usize as u64;
        assert_eq!(fmt(b"%hs", &[aslot]), "World");

        // %S in W fmt → narrow string (inverse of %s)
        assert_eq!(fmt(b"%S", &[aslot]), "World");
    }

    #[test]
    fn pointer_emit() {
        let s = fmt(b"%p", &[0xdeadbeef]);
        if cfg!(target_pointer_width = "64") {
            assert_eq!(s, "00000000deadbeef");
        } else {
            assert_eq!(s, "deadbeef");
        }
    }

    #[test]
    fn unicode_string() {
        let buf: [u16; 2] = [b'H' as u16, b'i' as u16];
        let s = UnicodeStringSlim {
            length: 4,
            maximum_length: 4,
            buffer: buf.as_ptr(),
        };
        let slot = (&s as *const _) as usize as u64;
        assert_eq!(fmt(b"%wZ", &[slot]), "Hi");
    }

    #[test]
    fn null_string() {
        assert_eq!(fmt(b"%s", &[0]), "(null)");
        assert_eq!(fmt(b"%hs", &[0]), "(null)");
    }

    #[test]
    fn star_width() {
        let s = b"Hi\0";
        let slot = s.as_ptr() as usize as u64;
        assert_eq!(fmt(b"%*hs|", &[5, slot]), "   Hi|");
    }

    #[test]
    fn bounded_truncation() {
        let mut dst = [0u16; 8];
        let mut w = BoundedBufW::new(dst.as_mut_ptr(), 8);
        let written = format_into(&mut w, &wstr(b"Hello, World!"), &[]);
        w.terminate();
        assert_eq!(written, 13);
        assert_eq!(to_string(&dst[..7]), "Hello, ");
        assert_eq!(dst[7], 0);
    }

    #[test]
    fn real_world_partition_fmt() {
        // From fstub/disksup.rs: W_HARDDISK_PARTITION_FMT
        assert_eq!(fmt(b"\\Device\\Harddisk%d\\Partition%d", &[0, 1]),
                   "\\Device\\Harddisk0\\Partition1");
        // From cmsysini.rs:
        assert_eq!(fmt(b"\\Registry\\Machine\\System\\ControlSet%03ld", &[7]),
                   "\\Registry\\Machine\\System\\ControlSet007");
    }
}
