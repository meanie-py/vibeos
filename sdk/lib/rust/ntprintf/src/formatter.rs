//! Format-string evaluator. Mirrors `_vsnprintf` for the Windows-flavored
//! printf subset enumerated in `scanner.rs`.
//!
//! Two responsibilities:
//!  - Parse each `%…` specifier (re-walking the same format string the scanner
//!    saw) and convert the matching `slots[i]` into bytes.
//!  - Emit those bytes through a [`Writer`] sink. Two sinks ship in this
//!    crate: [`BoundedBuf`] for fixed C buffers, and any closure
//!    `FnMut(&[u8])` for callback-style consumers.

/// Sink for formatted bytes. Implementations track total bytes "written"
/// (including bytes that were dropped because the underlying buffer filled
/// up) so the FFI return value matches `_vsnprintf` semantics.
pub trait Writer {
    /// Append bytes. The implementation MUST update its written counter
    /// even if it drops bytes due to capacity.
    fn write_bytes(&mut self, bytes: &[u8]);

    /// Total bytes the sink has been asked to write (including dropped).
    fn written(&self) -> usize;
}

/// Bounded byte buffer — like `vsnprintf`'s `(dst, cap)` pair. NUL-terminates
/// at the truncation boundary if `cap > 0`.
pub struct BoundedBuf {
    dst: *mut u8,
    cap: usize,
    /// Position in `dst` where the next *kept* byte will land.
    pos: usize,
    /// Total bytes the formatter has emitted (incl. dropped past `cap`).
    total: usize,
}

impl BoundedBuf {
    /// `dst` may be null only if `cap == 0`.
    pub fn new(dst: *mut u8, cap: usize) -> Self {
        Self { dst, cap, pos: 0, total: 0 }
    }

    /// Place a NUL at the truncation boundary. Safe to call multiple times.
    pub fn terminate(&mut self) {
        if self.cap == 0 || self.dst.is_null() {
            return;
        }
        let nul_pos = if self.pos >= self.cap { self.cap - 1 } else { self.pos };
        unsafe { *self.dst.add(nul_pos) = 0; }
    }
}

impl Writer for BoundedBuf {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.total += bytes.len();
        if self.dst.is_null() || self.cap == 0 {
            return;
        }
        // Reserve one byte for terminator → never write into pos == cap-1.
        let max_pos = self.cap.saturating_sub(1);
        let mut i = 0;
        while i < bytes.len() && self.pos < max_pos {
            unsafe { *self.dst.add(self.pos) = bytes[i]; }
            self.pos += 1;
            i += 1;
        }
    }

    fn written(&self) -> usize { self.total }
}

/// Sink that wraps a borrowed `&mut Vec<u8>` — used in tests and as a
/// reference for callers that want unbounded output.
#[cfg(test)]
pub struct VecBuf<'a>(pub &'a mut Vec<u8>, pub usize);
#[cfg(test)]
impl<'a> VecBuf<'a> {
    pub fn new(v: &'a mut Vec<u8>) -> Self { Self(v, 0) }
}
#[cfg(test)]
impl<'a> Writer for VecBuf<'a> {
    fn write_bytes(&mut self, bytes: &[u8]) { self.0.extend_from_slice(bytes); self.1 += bytes.len(); }
    fn written(&self) -> usize { self.1 }
}

// ---------------------------------------------------------------------------
// Format-spec parsing (same syntax as scanner.rs but here we keep all the
// modifier values, not just emit one tag).

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
    is_wide_str: bool,
    conv: u8,
}

impl Spec {
    const fn empty() -> Self {
        Self {
            left_align: false, plus: false, space: false, alt: false,
            zero_pad: false, width: -1, precision: -1,
            is64: false, is_wide_str: false, conv: 0,
        }
    }
}

/// Parse one specifier starting just past the `%`.
/// Returns `(spec, end-of-specifier index, slots_consumed)`.
/// `slots_consumed` is the number of `slots[]` entries this spec used,
/// counting `*` width and `.*` precision in addition to the value itself.
/// `slots_consumed` for a trailing `%` (literal) is 0.
fn parse_spec(fmt: &[u8], start: usize, slots: &[u64], next_slot: usize) -> (Spec, usize, usize) {
    let mut spec = Spec::empty();
    let mut p = start;
    let mut consumed = 0usize;

    // Flags
    while p < fmt.len() {
        match fmt[p] {
            b'-' => spec.left_align = true,
            b'+' => spec.plus = true,
            b' ' => spec.space = true,
            b'#' => spec.alt = true,
            b'0' => spec.zero_pad = true,
            _ => break,
        }
        p += 1;
    }
    // Width
    if p < fmt.len() && fmt[p] == b'*' {
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
        while p < fmt.len() && fmt[p].is_ascii_digit() {
            w = w.saturating_mul(10).saturating_add((fmt[p] - b'0') as i32);
            p += 1;
            have = true;
        }
        if have { spec.width = w; }
    }
    // Precision
    if p < fmt.len() && fmt[p] == b'.' {
        p += 1;
        if p < fmt.len() && fmt[p] == b'*' {
            let v = slots.get(next_slot + consumed).copied().unwrap_or(0) as i32;
            spec.precision = if v < 0 { 0 } else { v };
            consumed += 1;
            p += 1;
        } else {
            let mut prec: i32 = 0;
            while p < fmt.len() && fmt[p].is_ascii_digit() {
                prec = prec.saturating_mul(10).saturating_add((fmt[p] - b'0') as i32);
                p += 1;
            }
            spec.precision = prec;
        }
    }
    // Length modifier
    while p < fmt.len() {
        match fmt[p] {
            b'h' => { p += 1; if p < fmt.len() && fmt[p] == b'h' { p += 1; } }
            b'l' => {
                p += 1;
                if p < fmt.len() && fmt[p] == b'l' { p += 1; spec.is64 = true; }
            }
            b'L' | b'q' | b'j' | b'z' | b't' => { p += 1; }
            b'I' => {
                p += 1;
                if p + 1 < fmt.len() && fmt[p] == b'6' && fmt[p+1] == b'4' {
                    p += 2; spec.is64 = true;
                } else if p + 1 < fmt.len() && fmt[p] == b'3' && fmt[p+1] == b'2' {
                    p += 2;
                }
            }
            b'w' => { p += 1; spec.is_wide_str = true; }
            _ => break,
        }
    }
    if p < fmt.len() {
        spec.conv = fmt[p];
        p += 1;
        // Value slot consumed unless conv == '%'
        if spec.conv != b'%' {
            consumed += 1;
        }
    }
    (spec, p, consumed)
}

/// Format `fmt` against `slots`, write result through `out`. Returns the total
/// bytes the formatter emitted (matches `_vsnprintf` return).
pub fn format_into<W: Writer>(out: &mut W, fmt: &[u8], slots: &[u64]) -> usize {
    let mut p = 0usize;
    let mut next_slot = 0usize;
    while p < fmt.len() {
        if fmt[p] != b'%' {
            // Run literal bytes until next '%'
            let start = p;
            while p < fmt.len() && fmt[p] != b'%' { p += 1; }
            out.write_bytes(&fmt[start..p]);
            continue;
        }
        p += 1;
        if p >= fmt.len() {
            // dangling '%' — emit literally to match common printf behavior
            out.write_bytes(b"%");
            break;
        }
        let (spec, end, consumed) = parse_spec(fmt, p, slots, next_slot);
        p = end;
        if spec.conv == 0 {
            // Truncated specifier — bail
            break;
        }
        if spec.conv == b'%' {
            out.write_bytes(b"%");
            continue;
        }
        // Find the value slot. If `*` modifiers consumed slots, the value is
        // at next_slot + consumed - 1.
        let value_idx = next_slot + consumed - 1;
        let value = slots.get(value_idx).copied().unwrap_or(0);
        next_slot += consumed;

        emit_spec(out, &spec, value);
    }
    out.written()
}

/// Emit one specifier's value through `out`.
fn emit_spec<W: Writer>(out: &mut W, spec: &Spec, value: u64) {
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
            let prefix = if spec.alt && v != 0 { b"0".as_slice() } else { b"".as_slice() };
            emit_unsigned_int(out, spec, v, 8, false, prefix);
        }
        b'x' => {
            let v = if spec.is64 { value } else { value as u32 as u64 };
            let prefix = if spec.alt && v != 0 { b"0x".as_slice() } else { b"".as_slice() };
            emit_unsigned_int(out, spec, v, 16, false, prefix);
        }
        b'X' => {
            let v = if spec.is64 { value } else { value as u32 as u64 };
            let prefix = if spec.alt && v != 0 { b"0X".as_slice() } else { b"".as_slice() };
            emit_unsigned_int(out, spec, v, 16, true, prefix);
        }
        b'c' => {
            // %c emits one byte (low byte of int-promoted value).
            let buf = [value as u8];
            emit_padded(out, spec, &buf);
        }
        b'C' => {
            // %C / %lc — wide char. We render the low byte of the wchar.
            let wc = value as u16;
            let b = if wc <= 0x7F { wc as u8 } else { b'?' };
            emit_padded(out, spec, &[b]);
        }
        b's' => emit_ansi_string(out, spec, value as usize as *const u8),
        b'S' => emit_wide_string(out, spec, value as usize as *const u16),
        b'Z' => {
            if spec.is_wide_str {
                emit_unicode_string(out, spec, value as usize as *const UnicodeStringSlim);
            } else {
                // %Z — STRING* (ANSI counterpart, rare)
                emit_ansi_string_struct(out, spec, value as usize as *const AnsiStringSlim);
            }
        }
        b'p' => {
            // Pointer: 0x + zero-padded hex of native ptr width
            // (Width is fixed: 8 hex digits on 32-bit, 16 on 64-bit.)
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
            // Unknown conversion — emit '%' + the conv char literally to aid debug
            out.write_bytes(b"%");
            out.write_bytes(&[spec.conv]);
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
// Helpers: number formatting, padding, string emission.

fn emit_signed_int<W: Writer>(out: &mut W, spec: &Spec, v: i64) {
    let mut buf = [0u8; 24];
    let mut len = 0;
    let abs: u64 = if v < 0 { (v as i128).wrapping_neg() as u64 } else { v as u64 };
    if abs == 0 {
        // Precision 0 with value 0 emits nothing.
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

    // Apply precision (min digits — zero-pad)
    let digits_len = len;
    let prec = if spec.precision >= 0 { spec.precision as usize } else { 0 };
    let total_digits = digits_len.max(prec);
    let body_len = sign.len() + total_digits;

    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(body_len);

    let zero_pad = spec.zero_pad && spec.precision < 0 && !spec.left_align;

    if pad > 0 && !spec.left_align && !zero_pad {
        emit_repeat(out, b' ', pad);
    }
    out.write_bytes(sign);
    if zero_pad && pad > 0 {
        emit_repeat(out, b'0', pad);
    }
    if total_digits > digits_len {
        emit_repeat(out, b'0', total_digits - digits_len);
    }
    // Reverse buf[..digits_len]
    let mut i = digits_len;
    while i > 0 {
        i -= 1;
        out.write_bytes(&[buf[i]]);
    }
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ', pad);
    }
}

fn emit_unsigned_int<W: Writer>(out: &mut W, spec: &Spec, v: u64, base: u32, upper: bool, prefix: &[u8]) {
    let mut buf = [0u8; 24];
    let mut len = 0;
    let digits = if upper { b"0123456789ABCDEF" } else { b"0123456789abcdef" };

    if v == 0 {
        // Spec: precision 0 + value 0 → emit nothing for the digits, but
        // padding still applies.
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
        emit_repeat(out, b' ', pad);
    }
    out.write_bytes(prefix);
    if zero_pad && pad > 0 {
        emit_repeat(out, b'0', pad);
    }
    if total_digits > len {
        emit_repeat(out, b'0', total_digits - len);
    }
    let mut i = len;
    while i > 0 {
        i -= 1;
        out.write_bytes(&[buf[i]]);
    }
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ', pad);
    }
}

fn emit_padded<W: Writer>(out: &mut W, spec: &Spec, body: &[u8]) {
    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(body.len());
    if pad > 0 && !spec.left_align {
        emit_repeat(out, b' ', pad);
    }
    out.write_bytes(body);
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ', pad);
    }
}

fn emit_ansi_string<W: Writer>(out: &mut W, spec: &Spec, ptr: *const u8) {
    if ptr.is_null() {
        emit_padded(out, spec, b"(null)");
        return;
    }
    // Find string length, capped by precision if given.
    let max_scan = if spec.precision >= 0 { spec.precision as usize } else { 0x10000 };
    let mut n = 0usize;
    unsafe {
        while n < max_scan {
            if *ptr.add(n) == 0 { break; }
            n += 1;
        }
    }
    let body = unsafe { core::slice::from_raw_parts(ptr, n) };
    emit_padded(out, spec, body);
}

fn emit_wide_string<W: Writer>(out: &mut W, spec: &Spec, ptr: *const u16) {
    if ptr.is_null() {
        emit_padded(out, spec, b"(null)");
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
    emit_wide_padded(out, spec, ptr, n);
}

fn emit_unicode_string<W: Writer>(out: &mut W, spec: &Spec, p: *const UnicodeStringSlim) {
    if p.is_null() {
        emit_padded(out, spec, b"(null)");
        return;
    }
    let s = unsafe { &*p };
    let nchars = (s.length as usize) / 2;
    if s.buffer.is_null() || nchars == 0 {
        emit_padded(out, spec, b"");
        return;
    }
    emit_wide_padded(out, spec, s.buffer, nchars);
}

fn emit_ansi_string_struct<W: Writer>(out: &mut W, spec: &Spec, p: *const AnsiStringSlim) {
    if p.is_null() {
        emit_padded(out, spec, b"(null)");
        return;
    }
    let s = unsafe { &*p };
    if s.buffer.is_null() || s.length == 0 {
        emit_padded(out, spec, b"");
        return;
    }
    let body = unsafe { core::slice::from_raw_parts(s.buffer, s.length as usize) };
    emit_padded(out, spec, body);
}

fn emit_wide_padded<W: Writer>(out: &mut W, spec: &Spec, ptr: *const u16, nchars: usize) {
    let width = if spec.width > 0 { spec.width as usize } else { 0 };
    let pad = width.saturating_sub(nchars);
    if pad > 0 && !spec.left_align {
        emit_repeat(out, b' ', pad);
    }
    // Emit one byte per wide char — low byte if printable ASCII, else '?'
    let mut i = 0;
    while i < nchars {
        let wc = unsafe { *ptr.add(i) };
        let b = if wc <= 0x7F { wc as u8 } else { b'?' };
        out.write_bytes(&[b]);
        i += 1;
    }
    if pad > 0 && spec.left_align {
        emit_repeat(out, b' ', pad);
    }
}

fn emit_repeat<W: Writer>(out: &mut W, byte: u8, n: usize) {
    let chunk = [byte; 32];
    let mut left = n;
    while left >= 32 {
        out.write_bytes(&chunk);
        left -= 32;
    }
    if left > 0 {
        out.write_bytes(&chunk[..left]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(fmt: &[u8], slots: &[u64]) -> String {
        let mut buf: Vec<u8> = Vec::new();
        let mut w = VecBuf::new(&mut buf);
        format_into(&mut w, fmt, slots);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn literals() {
        assert_eq!(fmt(b"hello world", &[]), "hello world");
        assert_eq!(fmt(b"100%%", &[]), "100%");
        assert_eq!(fmt(b"", &[]), "");
    }

    #[test]
    fn decimal_int() {
        assert_eq!(fmt(b"%d", &[42]), "42");
        assert_eq!(fmt(b"%d", &[(-42i32) as u32 as u64]), "-42");
        assert_eq!(fmt(b"%d", &[0]), "0");
        assert_eq!(fmt(b"%i", &[0xFFFFFFFF]), "-1");
    }

    #[test]
    fn unsigned() {
        assert_eq!(fmt(b"%u", &[42]), "42");
        assert_eq!(fmt(b"%u", &[0xFFFFFFFF]), "4294967295");
    }

    #[test]
    fn hex() {
        assert_eq!(fmt(b"%x", &[0xdeadbeef]), "deadbeef");
        assert_eq!(fmt(b"%X", &[0xdeadbeef]), "DEADBEEF");
        assert_eq!(fmt(b"%#x", &[0x42]), "0x42");
        assert_eq!(fmt(b"%#X", &[0]), "0");  // no prefix when value is 0
    }

    #[test]
    fn octal() {
        assert_eq!(fmt(b"%o", &[8]), "10");
        assert_eq!(fmt(b"%#o", &[8]), "010");
    }

    #[test]
    fn width_padding() {
        assert_eq!(fmt(b"%5d", &[42]), "   42");
        assert_eq!(fmt(b"%-5d|", &[42]), "42   |");
        assert_eq!(fmt(b"%05d", &[42]), "00042");
        assert_eq!(fmt(b"%08x", &[0xab]), "000000ab");
    }

    #[test]
    fn precision_int() {
        assert_eq!(fmt(b"%.5d", &[42]), "00042");
        assert_eq!(fmt(b"%.0d", &[0]), "");          // precision 0 + value 0 → empty
        assert_eq!(fmt(b"%5.3d", &[42]), "  042");
    }

    #[test]
    fn signed_flags() {
        assert_eq!(fmt(b"%+d", &[42]), "+42");
        assert_eq!(fmt(b"% d", &[42]), " 42");
        assert_eq!(fmt(b"%+d", &[(-42i32) as u32 as u64]), "-42");
    }

    #[test]
    fn long_long() {
        assert_eq!(fmt(b"%lld", &[0x1_0000_0001]), "4294967297");
        assert_eq!(fmt(b"%I64x", &[0x12345678abcdef00]), "12345678abcdef00");
        assert_eq!(fmt(b"%016I64x", &[0x42]), "0000000000000042");
    }

    #[test]
    fn pointer() {
        // Pointer width depends on test target; on a 64-bit host:
        let s = fmt(b"%p", &[0xdeadbeef]);
        if cfg!(target_pointer_width = "64") {
            assert_eq!(s, "00000000deadbeef");
        } else {
            assert_eq!(s, "deadbeef");
        }
    }

    #[test]
    fn ansi_string() {
        let s = b"Hello\0";
        let slot = s.as_ptr() as usize as u64;
        assert_eq!(fmt(b"%s", &[slot]), "Hello");
        assert_eq!(fmt(b"%-10s|", &[slot]), "Hello     |");
        assert_eq!(fmt(b"%10s|", &[slot]), "     Hello|");
        // precision truncates
        assert_eq!(fmt(b"%.3s", &[slot]), "Hel");
        // null
        assert_eq!(fmt(b"%s", &[0]), "(null)");
    }

    #[test]
    fn star_width_and_precision() {
        // %*s with width=10, then "Hello"
        let s = b"Hello\0";
        let slot = s.as_ptr() as usize as u64;
        assert_eq!(fmt(b"%*s|", &[10, slot]), "     Hello|");
        // %.*s with prec=3
        assert_eq!(fmt(b"%.*s", &[3, slot]), "Hel");
    }

    #[test]
    fn char_emit() {
        assert_eq!(fmt(b"%c", &[b'A' as u64]), "A");
        assert_eq!(fmt(b"<%c>", &[b'\n' as u64]), "<\n>");
    }

    #[test]
    fn unicode_string() {
        // Build a UNICODE_STRING with "Hi" (4 bytes payload).
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
    fn complex_real_world_strings() {
        let s = b"WinDir\0";
        let slot = s.as_ptr() as usize as u64;

        // "Adding boot driver: '%wZ', '%wZ'\n"
        let buf: [u16; 4] = [b'A' as u16, b'C' as u16, b'P' as u16, b'I' as u16];
        let us = UnicodeStringSlim { length: 8, maximum_length: 8, buffer: buf.as_ptr() };
        let us_slot = (&us as *const _) as usize as u64;
        assert_eq!(
            fmt(b"Adding boot driver: '%wZ', '%wZ'\n", &[us_slot, us_slot]),
            "Adding boot driver: 'ACPI', 'ACPI'\n"
        );

        // Mixed: width + hex + pointer + string
        assert_eq!(
            fmt(b"%08x %s\n", &[0x42, slot]),
            "00000042 WinDir\n"
        );
    }

    #[test]
    fn bounded_buf_truncation() {
        let mut dst = [b'@'; 8];
        let mut w = BoundedBuf::new(dst.as_mut_ptr(), 8);
        let written = format_into(&mut w, b"Hello, World!", &[]);
        w.terminate();
        assert_eq!(written, 13);                         // total including dropped
        assert_eq!(&dst[..7], b"Hello, ");
        assert_eq!(dst[7], 0);                           // NUL at cap-1
    }

    #[test]
    fn bounded_buf_exact_fit() {
        let mut dst = [b'@'; 6];
        let mut w = BoundedBuf::new(dst.as_mut_ptr(), 6);
        format_into(&mut w, b"Hello", &[]);
        w.terminate();
        assert_eq!(&dst, b"Hello\0");
    }

    #[test]
    fn empty_buf_is_safe() {
        let mut w = BoundedBuf::new(core::ptr::null_mut(), 0);
        let n = format_into(&mut w, b"%d %s", &[42, 0]);
        assert_eq!(n, 9);  // "42 (null)"
    }
}
